"""Ask Jev for a bounded decision without executing the selected action."""

import argparse
import json
import os
import sys

from typesafe_sdk import Choice, TypeSafeClient, TypeSafeError


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Evaluate a bounded decision with Jev")
    parser.add_argument("--state", required=True, help="Text to evaluate")
    parser.add_argument(
        "--option", action="append", required=True, metavar="NAME:DESCRIPTION",
        help="Candidate decision and its meaning (repeat at least twice)",
    )
    args = parser.parse_args(argv)

    if not args.state.strip():
        parser.error("--state must contain text")

    criteria: dict[str, str] = {}
    for option in args.option:
        name, separator, description = option.partition(":")
        name, description = name.strip(), description.strip()
        if not separator or not name or not description or name == "none":
            parser.error("--option must be NAME:DESCRIPTION; 'none' is reserved")
        if name in criteria:
            parser.error(f"duplicate option: {name}")
        criteria[name] = description
    if len(criteria) < 2:
        parser.error("provide at least two distinct --option values")

    if not os.environ.get("TYPESAFE_API_KEY", "").strip():
        print("TYPESAFE_API_KEY is required; set it in your environment.", file=sys.stderr)
        return 2

    criteria["none"] = "None of the supplied options fits the state."
    try:
        with TypeSafeClient() as client:
            response = client.system_one(
                state=args.state,
                questions={
                    "decision": Choice(
                        instructions="Which supplied option best fits the state? Select none if no option fits.",
                        criteria=criteria,
                    )
                },
                model="jev-latest",
            )
    except TypeSafeError as error:
        print(
            f"TypeSafe request failed ({type(error).__name__}); check API access and connectivity.",
            file=sys.stderr,
        )
        return 1

    answer = response.answers["decision"]
    print(json.dumps({
        "choice": answer.choice,
        "probabilities": answer.probabilities,
        "confidence": answer.confidence,
        "model": response.model,
    }))
    return 0


if __name__ == "__main__":
    sys.exit(main())