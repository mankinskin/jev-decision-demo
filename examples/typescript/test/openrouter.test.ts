import assert from "node:assert/strict";
import test from "node:test";
import { complete, OPENROUTER_CHAT_URL, type Fetch } from "../src/openrouter.ts";

const response = (body: unknown, status = 200): Response => new Response(JSON.stringify(body), { status });

test("requires the credential before making a request", async () => {
  let called = false;
  const fetcher: Fetch = async () => {
    called = true;
    return response({});
  };

  await assert.rejects(complete("synthetic prompt", { OPENROUTER_MODEL: "test/model" }, fetcher), /OPENROUTER_API_KEY is required/);
  assert.equal(called, false);
});

test("requires an explicit model before making a request", async () => {
  let called = false;
  const fetcher: Fetch = async () => {
    called = true;
    return response({});
  };

  await assert.rejects(complete("synthetic prompt", { OPENROUTER_API_KEY: "test-only-key" }, fetcher), /OPENROUTER_MODEL is required/);
  assert.equal(called, false);
});

test("sends a fixed authenticated request and returns assistant text", async () => {
  let requestUrl: RequestInfo | URL | undefined;
  let requestInit: RequestInit | undefined;
  const fetcher: Fetch = async (input, init) => {
    requestUrl = input;
    requestInit = init;
    return response({ choices: [{ message: { content: "Use clear invoice line items." } }] });
  };

  const answer = await complete(
    "A synthetic invoice is unclear.",
    { OPENROUTER_API_KEY: "test-only-key", OPENROUTER_MODEL: "test/model" },
    fetcher,
  );

  assert.equal(answer, "Use clear invoice line items.");
  assert.equal(requestUrl, OPENROUTER_CHAT_URL);
  assert.equal(requestInit?.method, "POST");
  assert.equal(requestInit?.redirect, "manual");
  assert.equal(new Headers(requestInit?.headers).get("authorization"), "Bearer test-only-key");
  assert.deepEqual(JSON.parse(String(requestInit?.body)), {
    model: "test/model",
    messages: [{ role: "user", content: "A synthetic invoice is unclear." }],
  });
});

test("reports HTTP status without reading or exposing the provider body", async () => {
  const secretBody = "provider response containing test-only-key";
  const failingResponse = new Response(secretBody, { status: 503 });
  failingResponse.json = async () => {
    throw new Error("response body should not be parsed");
  };

  await assert.rejects(
    complete("synthetic prompt", { OPENROUTER_API_KEY: "test-only-key", OPENROUTER_MODEL: "test/model" }, async () => failingResponse),
    (error: Error) => error.message === "OpenRouter request failed (HTTP 503)." && !error.message.includes(secretBody),
  );
});

test("sanitizes transport errors", async () => {
  await assert.rejects(
    complete("synthetic prompt", { OPENROUTER_API_KEY: "test-only-key", OPENROUTER_MODEL: "test/model" }, async () => {
      throw new Error("socket failed with Bearer test-only-key");
    }),
    (error: Error) => error.message === "OpenRouter request failed (network error)." && !error.message.includes("test-only-key"),
  );
});