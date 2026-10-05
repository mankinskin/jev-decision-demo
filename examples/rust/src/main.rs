use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::{Value, json};
use std::env;
use std::fmt;
use std::time::Duration;

const OPENROUTER_CHAT_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

fn main() {
    match complete_from_env("Write one concise sentence about why clear invoices matter.") {
        Ok(answer) => println!("{answer}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn complete_from_env(prompt: &str) -> Result<String, DemoError> {
    let api_key = env::var("OPENROUTER_API_KEY").ok();
    let model = env::var("OPENROUTER_MODEL").ok();
    complete_at(
        api_key.as_deref(),
        model.as_deref(),
        prompt,
        OPENROUTER_CHAT_URL,
    )
}

fn complete_at(
    api_key: Option<&str>,
    model: Option<&str>,
    prompt: &str,
    endpoint: &str,
) -> Result<String, DemoError> {
    let api_key = api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(DemoError::MissingApiKey)?;
    let model = model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(DemoError::MissingModel)?;

    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .build()
        .map_err(|_| DemoError::ClientConfiguration)?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [{ "role": "user", "content": prompt }]
        }))
        .send()
        .map_err(|error| {
            if error.is_timeout() {
                DemoError::Timeout
            } else {
                DemoError::Network
            }
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(DemoError::HttpStatus(status.as_u16()));
    }

    let payload: Value = response.json().map_err(|_| DemoError::InvalidResponse)?;
    payload["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_owned)
        .ok_or(DemoError::MissingAssistantText)
}

#[derive(Debug)]
enum DemoError {
    MissingApiKey,
    MissingModel,
    ClientConfiguration,
    Timeout,
    Network,
    HttpStatus(u16),
    InvalidResponse,
    MissingAssistantText,
}

impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => formatter.write_str("OPENROUTER_API_KEY is required."),
            Self::MissingModel => formatter.write_str("OPENROUTER_MODEL is required."),
            Self::ClientConfiguration => formatter.write_str("OpenRouter client setup failed."),
            Self::Timeout => formatter.write_str("OpenRouter request timed out."),
            Self::Network => formatter.write_str("OpenRouter request failed (network error)."),
            Self::HttpStatus(status) => {
                write!(formatter, "OpenRouter request failed (HTTP {status}).")
            }
            Self::InvalidResponse => {
                formatter.write_str("OpenRouter returned an invalid response.")
            }
            Self::MissingAssistantText => {
                formatter.write_str("OpenRouter response did not contain assistant text.")
            }
        }
    }
}

impl std::error::Error for DemoError {}

#[cfg(test)]
mod tests {
    use super::{DemoError, complete_at};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    fn start_server(status: u16, headers: &str, body: &str) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("read local address");
        let body = body.to_owned();
        let headers = headers.to_owned();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept local request");
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .expect("set local read timeout");
            let request = read_request(&mut stream);
            let reason = if status == 200 {
                "OK"
            } else if status == 302 {
                "Found"
            } else {
                "Error"
            };
            write!(
                stream,
                "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{headers}Connection: close\r\n\r\n{body}",
                body.len()
            )
            .expect("write local response");
            request
        });
        (format!("http://{address}/api/v1/chat/completions"), server)
    }

    fn read_request(stream: &mut TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0; 1024];
        loop {
            let count = stream.read(&mut chunk).expect("read local request");
            if count == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..count]);
            let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
            else {
                continue;
            };
            let header_end = header_end + 4;
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if request.len() >= header_end + content_length {
                break;
            }
        }
        String::from_utf8(request).expect("request should be UTF-8")
    }

    #[test]
    fn missing_api_key_is_rejected_before_endpoint_access() {
        let result = complete_at(None, Some("test/model"), "synthetic prompt", "not a URL");
        assert!(matches!(result, Err(DemoError::MissingApiKey)));
    }

    #[test]
    fn missing_model_is_rejected_before_endpoint_access() {
        let result = complete_at(Some("test-only-key"), None, "synthetic prompt", "not a URL");
        assert!(matches!(result, Err(DemoError::MissingModel)));
    }

    #[test]
    fn successful_response_sends_explicit_model_and_bearer_key() {
        let (endpoint, server) = start_server(
            200,
            "",
            r#"{"choices":[{"message":{"content":"Use clear invoice line items."}}]}"#,
        );
        let answer = complete_at(
            Some("test-only-key"),
            Some("test/model"),
            "A synthetic invoice is unclear.",
            &endpoint,
        )
        .expect("parse local success response");
        let request = server.join().expect("join local test server");

        assert_eq!(answer, "Use clear invoice line items.");
        assert!(request.starts_with("POST /api/v1/chat/completions HTTP/1.1"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer test-only-key")
        );
        assert!(request.contains(r#""model":"test/model""#));
        assert!(request.contains("A synthetic invoice is unclear."));
    }

    #[test]
    fn http_failure_does_not_expose_provider_body() {
        let secret_body = "provider response containing test-only-key";
        let (endpoint, server) = start_server(503, "", secret_body);
        let error = complete_at(
            Some("test-only-key"),
            Some("test/model"),
            "synthetic prompt",
            &endpoint,
        )
        .expect_err("surface only the HTTP status");
        let _request = server.join().expect("join local test server");

        assert_eq!(error.to_string(), "OpenRouter request failed (HTTP 503).");
        assert!(!error.to_string().contains(secret_body));
    }

    #[test]
    fn redirect_is_not_followed() {
        let redirect_target = TcpListener::bind("127.0.0.1:0").expect("bind redirect target");
        redirect_target
            .set_nonblocking(true)
            .expect("make redirect target nonblocking");
        let target = redirect_target.local_addr().expect("read redirect target");
        let (endpoint, server) = start_server(
            302,
            &format!("Location: http://{target}/redirected\r\n"),
            "{}",
        );
        let error = complete_at(
            Some("test-only-key"),
            Some("test/model"),
            "synthetic prompt",
            &endpoint,
        )
        .expect_err("return the redirect status rather than following it");
        let _request = server.join().expect("join local test server");

        assert_eq!(error.to_string(), "OpenRouter request failed (HTTP 302).");
        assert!(redirect_target.accept().is_err());
    }
}
