export const OPENROUTER_CHAT_URL = "https://openrouter.ai/api/v1/chat/completions";
const REQUEST_TIMEOUT_MS = 15_000;

export type Environment = Readonly<Record<string, string | undefined>>;
export type Fetch = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

class SafeRequestError extends Error {}

export async function complete(prompt: string, environment: Environment = process.env, fetcher: Fetch = fetch): Promise<string> {
  const apiKey = environment.OPENROUTER_API_KEY?.trim();
  if (!apiKey) throw new SafeRequestError("OPENROUTER_API_KEY is required.");

  const model = environment.OPENROUTER_MODEL?.trim();
  if (!model) throw new SafeRequestError("OPENROUTER_MODEL is required.");

  try {
    const response = await fetcher(OPENROUTER_CHAT_URL, {
      method: "POST",
      headers: {
        authorization: `Bearer ${apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: prompt }],
      }),
      redirect: "manual",
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });

    if (!response.ok) {
      throw new SafeRequestError(`OpenRouter request failed (HTTP ${response.status}).`);
    }

    let payload: unknown;
    try {
      payload = await response.json();
    } catch {
      throw new SafeRequestError("OpenRouter returned invalid JSON.");
    }

    const content = assistantContent(payload);
    if (content === undefined) {
      throw new SafeRequestError("OpenRouter response did not contain assistant text.");
    }
    return content;
  } catch (error) {
    if (error instanceof SafeRequestError) throw error;
    if (error instanceof Error && error.name === "TimeoutError") {
      throw new SafeRequestError("OpenRouter request timed out.");
    }
    throw new SafeRequestError("OpenRouter request failed (network error).");
  }
}

function assistantContent(payload: unknown): string | undefined {
  if (typeof payload !== "object" || payload === null || !("choices" in payload)) return;
  const choices = payload.choices;
  if (!Array.isArray(choices) || choices.length === 0) return;
  const first = choices[0];
  if (typeof first !== "object" || first === null || !("message" in first)) return;
  const message = first.message;
  if (typeof message !== "object" || message === null || !("content" in message)) return;
  return typeof message.content === "string" ? message.content : undefined;
}