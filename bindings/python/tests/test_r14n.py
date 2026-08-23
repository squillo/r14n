# SPDX-License-Identifier: Apache-2.0
"""Tests against the BUILT r14n wheel — the artifact a consumer pip-installs.

The boundary's logic is pinned by ``cargo test`` in the core crate (``r14n::wire``);
what these prove is that the compiled extension module, its ABI, and the exported
Python surface actually work under a real interpreter.

Usage (after ``maturin develop`` or ``pip install`` of the built wheel)::

    python -m unittest discover -s bindings/python/tests -v

NOT LEGAL ADVICE.
"""

import json
import unittest

import r14n

PACK = (
    '[meta]\n'
    'strictness = "minimal"\n'
    '[subject.telepresence]\n'
    'controls = ["user_attestation"]\n'
    '[legally_required]\n'
    'controls = ["user_attestation"]\n'
)
PACKS = json.dumps({"minimal/recording_consent.r14n.toml": PACK})
QUERY = json.dumps(
    {
        "domain": "recording_consent",
        "profile": "minimal",
        "jurisdiction": "all_party",
        "subject": "telepresence",
        "universe": ["user_attestation", "signal_notice"],
    }
)
CTX = json.dumps(
    {
        "record_id": "urn:test:1",
        "issued_at": "2026-08-23T12:00:00Z",
        "issued_at_unix": 1787832000,
        "language": "en",
        "pii_principal_id": "user-1",
        "pii_controller": "Example Operator",
        "domain": "recording_consent",
        "subject": "telepresence",
        "attribution_source": "operator_declared",
    }
)


class TestModuleSurface(unittest.TestCase):
    """Why: the module attributes are the cheapest sanity checks a consumer runs
    first, and RLPS_NS must match the live vocabulary host."""

    def test_module_attributes(self):
        self.assertTrue(r14n.__version__)
        self.assertEqual(r14n.RLPS_NS, "https://r14n.squillo.com/ns#")
        self.assertTrue(r14n.NOT_LEGAL_ADVICE.startswith("NOT LEGAL ADVICE"))


class TestResolve(unittest.TestCase):
    def test_happy_path_returns_the_core_decision(self):
        """Why: the happy path across the real extension ABI — verdict, the
        sorted required set, and the pack key as provenance source."""
        d = json.loads(r14n.resolve(PACKS, QUERY))
        self.assertEqual(d["verdict"], "permit")
        self.assertEqual(d["required"], ["user_attestation"])
        self.assertEqual(
            d["provenance"]["source"], "minimal/recording_consent.r14n.toml"
        )
        self.assertFalse(d["provenance"]["fell_back"])
        self.assertTrue(d["disclaimer"].startswith("NOT LEGAL ADVICE"))

    def test_missing_pack_fails_closed(self):
        """Why: fail-closed (spec §4/§5) is the property the standard rests on —
        a missing pack demands the caller's whole universe, loudly, and must not
        raise or resolve empty."""
        d = json.loads(r14n.resolve("{}", QUERY))
        self.assertTrue(d["provenance"]["fell_back"])
        self.assertEqual(d["required"], ["signal_notice", "user_attestation"])

    def test_malformed_input_raises_value_error(self):
        """Why: a caller bug must be distinguishable from a policy fallback —
        bad JSON is a ValueError, never a silent aggressive decision."""
        with self.assertRaises(ValueError) as ctx:
            r14n.resolve(PACKS, "{not json")
        self.assertIn("query_json", str(ctx.exception))

    def test_strictness_override_dial(self):
        """Why: the global dial must reach the core and be reported back."""
        d = json.loads(r14n.resolve(PACKS, QUERY, "aggressive"))
        self.assertEqual(d["provenance"]["strictness"], "aggressive")
        self.assertEqual(d["required"], ["signal_notice", "user_attestation"])


class TestReceipts(unittest.TestCase):
    def test_both_receipt_forms_advisory_only(self):
        """Why: receipts are the auditable artifact (spec §8); both forms must
        serialize and stay advisory-only, since the reference resolver verifies
        no provenance (spec §6)."""
        out = json.loads(r14n.resolve_with_receipts(PACKS, QUERY, CTX))
        self.assertEqual(out["decision"]["verdict"], "permit")
        self.assertEqual(out["receipt_dpv27560"]["decision"]["verdict"], "permit")
        self.assertTrue(out["receipt_dpv27560"]["provenance"]["advisory_only"])
        self.assertIsInstance(out["receipt_kantara_cr_v1_1"], dict)

    def test_malformed_context_names_ctx_json(self):
        """Why: three JSON inputs mean three distinct error surfaces; a bad
        context must not be blamed on the query."""
        with self.assertRaises(ValueError) as ctx:
            r14n.resolve_with_receipts(PACKS, QUERY, "{}")
        self.assertIn("ctx_json", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
