# /conformance — RLPS conformance test vectors

Language-neutral JSON vectors for the three conformance levels (spec §9). A resolver claiming a
level MUST pass every vector in that level's file whose `capabilities` it implements — and to
*claim* the level, it must implement **all** capabilities the level requires (see table below).

> **⚠ NOT LEGAL ADVICE.** Vectors verify resolver BEHAVIOR (fail-closed, posture selection,
> provenance taint) — passing them is an engineering conformance claim, never a compliance claim.

## Files

| File | Level (spec §9) | Requires capabilities |
|---|---|---|
| `level-1.json` | **Minimal-viable** — parse packs, resolve `aggressive`, fail closed | `pack_parse`, `posture_selection`, `fail_closed_fallback` |
| `level-2.json` | **Configured** — negotiation + delta-merge + all postures + most-restrictive | + `fail_closed_unknown_subject`, `posture_override`, `data_minimization_guard`, `rfc4647_negotiation`, `delta_merge`, `most_restrictive_merge`, `deontic_conflict`, `gpc_escalation`, `per_domain_posture` |
| `level-3.json` | **Comprehensive-with-provenance** — trust-root taint + receipt emission | + `trust_root_taint`, `receipt_27560`, `receipt_kantara`, `trust_root_revocation` |

## Vector format

```json
{
  "suite": "rlps-conformance/0.1",
  "level": 1,
  "vectors": [
    {
      "name": "snake_case_unique_name",
      "description": "What normative behavior this pins (spec § reference).",
      "capabilities": ["pack_parse"],
      "packs": { "relative/path/<domain>.r14n.toml": "<full TOML text>" },
      "query": {
        "domain": "recording_consent",
        "profile": "aggressive",
        "jurisdiction": "all_party",
        "subject": "telepresence",
        "universe": ["attestation", "signal_notice"],
        "posture_override": null
      },
      "expect": {
        "required_controls": ["..."],
        "fell_back": false,
        "legal_review_status": "draft",
        "verdict": "permit"
      }
    }
  ]
}
```

**Harness contract:** materialize `packs` under a fresh temp root; construct the query (with the
optional global `posture_override`); resolve; assert every key present in `expect`
(`required_controls` compares as a sorted string set; `legal_review_status: null` means "none").
Level-3 vectors add `receipt_context` (the full `ReceiptContext` shape — see
`/resolver/src/receipt.rs`) plus `expect_receipt` / `expect_kantara`: maps of **RFC 6901 JSON
pointer → expected value** evaluated against the serialized receipt documents.

**Capability tags:** a harness runs a vector iff it implements every listed capability, and MUST
report (not silently drop) the vectors it skips. The reference resolver's runner
(`/resolver/tests/conformance.rs`) pins its supported set and asserts exact run counts, so a
newly-added vector fails the build until the runner acknowledges it.

Reference-resolver status (2026-07-06): implements all Level-1 capabilities; of Level 2:
`fail_closed_unknown_subject`, `posture_override`, `data_minimization_guard` (not yet:
`rfc4647_negotiation`, `delta_merge`, `most_restrictive_merge`, `deontic_conflict`,
`gpc_escalation`, `per_domain_posture` — normative per spec §4, vectors authored ahead of
implementation); of Level 3: all except `trust_root_revocation` (needs registry integration).
