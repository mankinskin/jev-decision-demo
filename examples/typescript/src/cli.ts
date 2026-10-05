import { complete } from "./openrouter.ts";

try {
  const answer = await complete("Write one concise sentence about why clear invoices matter.");
  console.log(answer);
} catch (error) {
  console.error(error instanceof Error ? error.message : "OpenRouter request failed.");
  process.exitCode = 1;
}