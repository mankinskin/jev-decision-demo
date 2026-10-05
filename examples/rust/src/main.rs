use jev_client::validate::{self, Options};
use jev_client::{ApiKey, BaseUrl, Choice, HttpTransport, Request, Response, Transport};
use serde_json::{Value, json};
use std::env;
use std::fmt;

const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";
const MODEL: &str = "jev-latest";

#[tokio::main]
async fn main() {
    match complete_from_env().await {
        Ok(answer) => println!("{answer}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

async fn complete_from_env() -> Result<Value, DemoError> {
    let api_key = env::var("TYPESAFE_API_KEY").map_err(|_| DemoError::MissingApiKey)?;
    let api_key = ApiKey::new(api_key).map_err(|_| DemoError::InvalidApiKey)?;
    let base_url = env::var("TYPESAFE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
    let base_url = BaseUrl::parse(&base_url).map_err(|_| DemoError::InvalidBaseUrl)?;
    let transport = HttpTransport::builder(api_key)
        .base_url(base_url)
        .build()
        .map_err(DemoError::Transport)?;

    let request = decision_request();
    if !validate::check(&request, &Options::default()).is_valid() {
        return Err(DemoError::InvalidRequest);
    }

    let response = transport
        .evaluate(&request)
        .await
        .map_err(DemoError::Transport)?;
    decision_json(&response.body)
}

fn decision_request() -> Request {
    Request::new("The customer was charged twice for an invoice.", MODEL).question(
        "decision",
        Choice::new("Which supplied option best fits the state? Select none if no option fits.")
            .option("billing", "Payments, invoicing, refunds")
            .option("support", "Technical troubleshooting")
            .bare_option("none"),
    )
}

fn decision_json(response: &Response) -> Result<Value, DemoError> {
    let answer = response
        .answers
        .get("decision")
        .and_then(|answer| answer.as_choice())
        .ok_or(DemoError::MissingChoiceAnswer)?;

    Ok(json!({
        "choice": answer.choice,
        "probabilities": answer.probabilities,
        "confidence": answer.confidence,
        "model": response.model,
    }))
}

#[derive(Debug)]
enum DemoError {
    MissingApiKey,
    InvalidApiKey,
    InvalidBaseUrl,
    InvalidRequest,
    Transport(jev_client::Error),
    MissingChoiceAnswer,
}

impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => formatter.write_str("TYPESAFE_API_KEY is required."),
            Self::InvalidApiKey => formatter.write_str("TYPESAFE_API_KEY is invalid."),
            Self::InvalidBaseUrl => formatter.write_str("TYPESAFE_BASE_URL is invalid."),
            Self::InvalidRequest => {
                formatter.write_str("The Jev request failed offline validation.")
            }
            Self::Transport(error) => write!(formatter, "TypeSafe Jev request failed: {error}"),
            Self::MissingChoiceAnswer => {
                formatter.write_str("Jev response did not contain a typed choice answer.")
            }
        }
    }
}

impl std::error::Error for DemoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transport(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DemoError, MODEL, decision_json, decision_request};
    use jev_client::Response;
    use jev_client::validate::{self, Options};
    use serde_json::Value;

    #[test]
    fn decision_request_is_a_valid_typed_choice() {
        let request = decision_request();
        let report = validate::check(&request, &Options::default());
        let serialized = serde_json::to_value(request).expect("serialize Jev request");

        assert!(report.is_valid());
        assert_eq!(serialized["model"], MODEL);
        assert_eq!(serialized["questions"]["decision"]["type"], "choice");
        assert_eq!(
            serialized["questions"]["decision"]["criteria"]["none"],
            Value::Null
        );
    }

    #[test]
    fn typed_choice_response_is_emitted_as_decision_json() {
        let response: Response = serde_json::from_str(
            r#"{"model":"jev-1.13.0","answers":{"decision":{"type":"choice","choice":"billing","probabilities":{"billing":0.91,"support":0.06,"none":0.03},"confidence":0.91}},"usage":{"input_tokens":80,"output_tokens":10}}"#,
        )
        .expect("parse Jev response");
        let output = decision_json(&response).expect("format typed choice response");

        assert_eq!(output["choice"], "billing");
        assert_eq!(output["probabilities"]["support"], 0.06);
        assert_eq!(output["confidence"], 0.91);
        assert_eq!(output["model"], "jev-1.13.0");
    }

    #[test]
    fn missing_typed_choice_is_rejected() {
        let response: Response = serde_json::from_str(
            r#"{"model":"jev-1.13.0","answers":{"decision":{"type":"noul","noul":0.91}},"usage":{"input_tokens":80,"output_tokens":10}}"#,
        )
        .expect("parse Jev response with another answer type");

        assert!(matches!(
            decision_json(&response),
            Err(DemoError::MissingChoiceAnswer)
        ));
    }
}
