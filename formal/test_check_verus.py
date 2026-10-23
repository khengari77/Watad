#!/usr/bin/env python3
"""Focused tests for the Verus mutation-control diagnostic classifier."""
import importlib.util
from pathlib import Path
import unittest

DRIVER = Path(__file__).with_name("check-verus.py")
SPEC = importlib.util.spec_from_file_location("check_verus", DRIVER)
check_verus = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(check_verus)


def block(kind, location, body):
    return f"error: {kind}\n  --> {location}\n   |\n   | {body}\n\nerror: aborting due to 1 previous error\n"


class DiagnosticClassifierTest(unittest.TestCase):
    def test_accepts_intended_postcondition(self):
        output = block("postcondition not satisfied",
                       "/tmp/broken-verse-sadr-replay/verse.rs:167:13",
                       "ensures yes == legal(p, scope, sadr@, ajuz@, c)\n"
                       "choice::check(&f.sadr, &[], sadr, &c.sadr)")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)",
            "choice::check(&f.sadr, &[], sadr, &c.sadr)")
        self.assertTrue(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_accepts_contract_without_fragment(self):
        output = block("constructed value may fail to meet its declared type invariant",
                       "/tmp/broken-circle-basis/profile.rs:140:12",
                       "Ok(Self { source,lowered,requirements,origin:Some(Origin { circle,rot,reps,feet }) })")
        expected = check_verus.control_expect(
            "constructed value may fail to meet its declared type invariant", "profile.rs",
            "origin:Some(Origin")
        self.assertTrue(check_verus.control_failure_matches(
            output, "circle-basis", expected))

    def test_rejects_wrong_error_kind(self):
        output = block("invariant not satisfied at end of loop body",
                       "/tmp/broken-verse-sadr-replay/verse.rs:167:13",
                       "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_rejects_error_at_another_file(self):
        output = block("postcondition not satisfied",
                       "/tmp/broken-verse-sadr-replay/language.rs:26:13",
                       "ensures yes == prefix(input@, at as int, word@)")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_rejects_fragment_found_only_in_following_warning(self):
        output = (
            "error: postcondition not satisfied\n"
            "  --> /tmp/broken-verse-sadr-replay/verse.rs:167:13\n"
            "   |\n"
            "   | ensures answer.is_some() == form_derives(p, scope, sadr@, ajuz@, f)\n"
            "\n"
            "warning: unused variable: `ensures yes == legal(p, scope, sadr@, ajuz@, c)`\n"
            "  --> /tmp/broken-verse-sadr-replay/verse.rs:168:1\n"
            "\n"
            "error: aborting due to 1 previous error\n")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_rejects_warning_only(self):
        output = ("warning: unused variable: `x`\n"
                  "  --> /tmp/broken-verse-sadr-replay/verse.rs:1:1\n")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_rejects_parse_error(self):
        output = block("expected one of `!`, `(`, `+`, `,`, `::`, or `{`",
                       "/tmp/broken-verse-sadr-replay/verse.rs:167:13",
                       "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))

    def test_rejects_right_kind_but_unrelated_failure(self):
        output = block("postcondition not satisfied",
                       "/tmp/broken-verse-sadr-replay/verse.rs:167:13",
                       "ensures answer.is_some() == form_derives(p, scope, sadr@, ajuz@, f)")
        expected = check_verus.control_expect(
            "postcondition not satisfied", "verse.rs",
            "ensures yes == legal(p, scope, sadr@, ajuz@, c)")
        self.assertFalse(check_verus.control_failure_matches(
            output, "verse-sadr-replay", expected))


if __name__ == "__main__":
    unittest.main()
