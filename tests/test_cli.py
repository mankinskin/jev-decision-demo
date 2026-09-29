"""Offline contract tests for the agent-callable Jev CLI."""

import io
import json
import os
import unittest
from contextlib import redirect_stderr, redirect_stdout
from types import SimpleNamespace
from unittest.mock import patch

from jev_decision_demo.cli import main


class DecisionCliTests(unittest.TestCase):
    args = [
        "--state", "My invoice was charged twice",
        "--option", "billing:Invoice and payment issues",
        "--option", "support:Technical support issues",
    ]

    def test_missing_key_never_constructs_client(self):
        stderr = io.StringIO()
        with patch.dict(os.environ, {}, clear=True), patch(
            "jev_decision_demo.cli.TypeSafeClient"
        ) as client, redirect_stderr(stderr):
            status = main(self.args)

        self.assertEqual(status, 2)
        self.assertIn("TYPESAFE_API_KEY", stderr.getvalue())
        client.assert_not_called()

    def test_choice_returns_typed_decision_without_executing_it(self):
        stdout = io.StringIO()
        response = SimpleNamespace(
            model="jev-1.13.0",
            answers={"decision": SimpleNamespace(
                choice="billing",
                probabilities={"billing": 0.85, "support": 0.1, "none": 0.05},
                confidence=0.8,
            )},
        )
        with patch.dict(os.environ, {"TYPESAFE_API_KEY": "example-test-key"}, clear=True), patch(
            "jev_decision_demo.cli.TypeSafeClient"
        ) as client, redirect_stdout(stdout):
            client.return_value.__enter__.return_value.system_one.return_value = response
            status = main(self.args)

        self.assertEqual(status, 0)
        self.assertEqual(json.loads(stdout.getvalue()), {
            "choice": "billing",
            "probabilities": {"billing": 0.85, "support": 0.1, "none": 0.05},
            "confidence": 0.8,
            "model": "jev-1.13.0",
        })
        request = client.return_value.__enter__.return_value.system_one.call_args.kwargs
        self.assertEqual(request["model"], "jev-latest")
        self.assertEqual(request["state"], "My invoice was charged twice")
        self.assertIn("none", request["questions"]["decision"].criteria)

    def test_none_is_a_visible_decision(self):
        stdout = io.StringIO()
        response = SimpleNamespace(
            model="jev-1.13.0",
            answers={"decision": SimpleNamespace(
                choice="none",
                probabilities={"billing": 0.1, "support": 0.1, "none": 0.8},
                confidence=0.7,
            )},
        )
        with patch.dict(os.environ, {"TYPESAFE_API_KEY": "example-test-key"}, clear=True), patch(
            "jev_decision_demo.cli.TypeSafeClient"
        ) as client, redirect_stdout(stdout):
            client.return_value.__enter__.return_value.system_one.return_value = response
            status = main(self.args)

        self.assertEqual(status, 0)
        self.assertEqual(json.loads(stdout.getvalue())["choice"], "none")

    def test_invalid_options_fail_before_client_construction(self):
        invalid = [
            ["--state", "request", "--option", "billing:Invoices"],
            ["--state", "request", "--option", "billing", "--option", "support:Help"],
            ["--state", "request", "--option", "none:Reserved", "--option", "support:Help"],
            ["--state", "request", "--option", "support:Help", "--option", "support:Other"],
        ]
        with patch.dict(os.environ, {}, clear=True), patch(
            "jev_decision_demo.cli.TypeSafeClient"
        ) as client:
            for args in invalid:
                with self.subTest(args=args), redirect_stderr(io.StringIO()):
                    with self.assertRaises(SystemExit) as error:
                        main(args)
                    self.assertEqual(error.exception.code, 2)
            client.assert_not_called()


if __name__ == "__main__":
    unittest.main()