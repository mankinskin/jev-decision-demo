# Jev Decision Demo

An agent can invoke this Python CLI to ask TypeSafe's hosted Jev model to choose among named options. Jev supplies a typed decision with probabilities; the agent or calling program remains responsible for deciding what to do next. Jev is not a replacement for the language model that runs a coding agent. This pilot never executes the selected action.

## Setup

Install Python 3.10 or newer. From this repository's root, create an isolated environment and install the package. On Windows Git Bash:

```sh
py -3 -m venv .venv
.venv/Scripts/python.exe -m pip install -e .
```

On macOS/Linux:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -e .
```

On Windows PowerShell, use `.venv\Scripts\python.exe -m pip install -e .` instead. The install brings in `typesafe-sdk`; Jev itself is hosted and is not downloaded locally. The TypeSafe skill for this project's GitHub Copilot agent was installed with `npx skills add typesafe-ai/skills --skill typesafe-ai` and lives under `.agents/skills/typesafe-ai/`.

## OpenRouter credential bridge

Install the standalone credential CLI from the sibling `workflow-tools` checkout with `cargo install --path ../workflow-tools/openrouter-auth`, then run `openrouter-auth login` once. The CLI uses browser PKCE and stores the resulting OpenRouter key in the OS credential store. `openrouter-auth token` writes the secret to stdout; never run it by itself, paste its output, or put a key in source files, arguments, Git, or chat.

The Python CLI remains a TypeSafe Jev decision consumer. To route its existing `system_one` call through OpenRouter, pass the stored key and base URL only to the Python process. From macOS/Linux or Windows Git Bash:

```sh
TYPESAFE_API_KEY="$(openrouter-auth token)" \
TYPESAFE_BASE_URL="https://openrouter.ai/api" \
.venv/bin/python -m jev_decision_demo.cli \
	--state "The customer was charged twice for an invoice" \
	--option "billing:Questions about payments and invoices" \
	--option "support:Technical troubleshooting"
```

On Windows Git Bash, use `.venv/Scripts/python.exe` instead of `.venv/bin/python`. In PowerShell, capture the command directly into the child-process environment, run the same CLI, then remove both variables:

```powershell
$env:TYPESAFE_API_KEY = (& openrouter-auth token)
$env:TYPESAFE_BASE_URL = "https://openrouter.ai/api"
.venv\Scripts\python.exe -m jev_decision_demo.cli --state "The customer was charged twice for an invoice" --option "billing:Questions about payments and invoices" --option "support:Technical troubleshooting"
Remove-Item Env:TYPESAFE_API_KEY, Env:TYPESAFE_BASE_URL
```

The Python example preserves Jev's typed choice, probabilities, and confidence. The TypeScript and Rust examples under `examples/` call OpenRouter's `/api/v1/chat/completions` endpoint directly; they are not Jev implementations. Both require `OPENROUTER_MODEL` explicitly and use the same credential CLI:

```sh
OPENROUTER_API_KEY="$(openrouter-auth token)" OPENROUTER_MODEL="provider/model" node --experimental-strip-types examples/typescript/src/cli.ts
OPENROUTER_API_KEY="$(openrouter-auth token)" OPENROUTER_MODEL="provider/model" cargo run --manifest-path examples/rust/Cargo.toml
```

These demo commands make live provider requests and may incur usage charges. Tests are offline and do not retrieve a credential or contact a provider. The examples do not create accounts, purchase credits, store credentials, or execute a decision as an action.

## TypeSafe credentials (alternative)

The Python Jev CLI can also use a TypeSafe API key directly. Sign in at the [TypeSafe console](https://console.typesafe.ai/playground), review account billing requirements, and create a key in the [keys dashboard](https://console.typesafe.ai/keys). Set the key privately as `TYPESAFE_API_KEY`; do not put it in source files, command arguments, Git, or chat. Account registration, purchases, and key provisioning are user actions. Public [model pricing](https://docs.typesafe.ai/models.md) listed $0.042 per million input tokens and free output tokens on 2026-09-29; check current pricing before buying credit.

## Use

With `TYPESAFE_API_KEY` already set in your environment, run from this repository root (on macOS/Linux, replace `Scripts/python.exe` with `bin/python`):

```sh
.venv/Scripts/python.exe -m jev_decision_demo.cli \
	--state "The customer was charged twice for an invoice" \
	--option "billing:Questions about payments and invoices" \
	--option "support:Technical troubleshooting"
```

The command prints one JSON object with `choice`, `probabilities`, `confidence`, and `model`. The `choice` can be `billing`, `support`, or `none` when neither option fits. `probabilities` includes every option, including `none`; `confidence` exposes uncertainty but is not permission to act. The caller must review the decision before any side effect. The `model` field is the resolved version: `jev-latest` is an alias whose target may change. The CLI rejects duplicate or malformed options and fails with an actionable message when `TYPESAFE_API_KEY` is absent. It does not log or print the key.

Run the offline tests without an API key:

```sh
.venv/Scripts/python.exe -m unittest discover -s tests -p "test_*.py"
```

Offline tests stub the SDK and make no network request. A real invocation requires a valid key and available TypeSafe service; authentication, rate-limit, and connection failures are not simulated by a successful offline test. The CLI does not create an account, purchase credits, make a coding agent use Jev as its base model, run an MCP server, or dispatch the selected action.

References: [TypeSafe quick start](https://docs.typesafe.ai/introduction/quickstart.md), [Python SDK](https://docs.typesafe.ai/sdk/python.md), [Choice](https://docs.typesafe.ai/primitives/choice.md), [Jev with coding agents](https://docs.typesafe.ai/introduction/coding-agents.md).