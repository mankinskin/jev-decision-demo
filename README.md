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

To make live requests, sign in at the [TypeSafe console](https://console.typesafe.ai/playground), review any account billing requirements there, and create an API key in the [keys dashboard](https://console.typesafe.ai/keys). Set the key privately as `TYPESAFE_API_KEY` in the shell that launches the CLI. Do not put a key in source files, command arguments, Git, or chat. Account registration, purchases, and key provisioning are user actions; this tool does none of them. Public [model pricing](https://docs.typesafe.ai/models.md) listed $0.042 per million input tokens and free output tokens on 2026-09-29; check current pricing before buying credit.

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