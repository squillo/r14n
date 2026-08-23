# r14n

**RLPS for Python — bindings generated from the Rust reference resolver.**
Not a port: the decision logic is the same `r14n` crate the Rust ecosystem links, compiled into a
native extension module. One behavior, one source of truth. **No Python dependencies.**

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange and configuration standard. It does not state
> what the law is, and **"aggressive" ≠ "compliant"**. See
> <https://github.com/squillo/r14n/blob/main/docs/not-legal-advice.md>.

```console
$ pip install r14n
```

One wheel serves CPython 3.9 and every later version (built against the stable ABI).

## Resolve required controls

Packs are passed in as a JSON object mapping `"<profile>/<domain>.r14n.toml"` to the pack's TOML
text — the same layout as on disk — so the resolver needs no filesystem access of its own.

```python
import json, pathlib, r14n

packs = {
    "aggressive/recording_consent.r14n.toml":
        pathlib.Path("packs/aggressive/recording_consent.r14n.toml").read_text(),
}

decision = json.loads(r14n.resolve(
    json.dumps(packs),
    json.dumps({
        "domain": "recording_consent",
        "profile": "aggressive",
        "jurisdiction": "all_party",
        "subject": "telepresence",
        # EVERY control you can enforce — the fail-closed ceiling.
        "universe": ["attestation", "signal_notice", "indicator_mount", "aph_mandate"],
        "as_of": "2026-08-23",   # optional; enables the effective-date check
    }),
    # optional global strictness dial: "aggressive" | "as_configured" | "minimal"
))

decision["verdict"]                  # "permit" | "block"
decision["required"]                 # controls you MUST satisfy (sorted, ⊆ universe)
decision["provenance"]["source"]     # which pack governed
decision["provenance"]["fell_back"]  # True ⇒ NO reviewed pack governed this decision
```

**It fails closed.** A missing, malformed, or out-of-envelope pack — or a `minimal` pack with no
`[legally_required]` floor — resolves to your *entire* universe with `provenance["fell_back"]`
true. It never returns an empty set and never silently under-restricts. Malformed *input* raises
`ValueError` instead, so a caller bug is never mistaken for a policy fallback.

## Decision receipts

`resolve_with_receipts` returns the decision plus both serialized receipt forms — an ISO/IEC
TS 27560-structured W3C DPV JSON-LD document and a Kantara CR v1.1 shim:

```python
out = json.loads(r14n.resolve_with_receipts(json.dumps(packs), query, json.dumps({
    "record_id": "urn:uuid:…", "issued_at": "2026-08-23T12:00:00Z",
    "issued_at_unix": 1787832000, "language": "en",
    "pii_principal_id": "user-1", "pii_controller": "Example Operator",
    "domain": "recording_consent", "subject": "telepresence",
    "attribution_source": "operator_declared",
})))

out["decision"], out["receipt_dpv27560"], out["receipt_kantara_cr_v1_1"]
```

Receipts stay `advisory_only: True` until a signature from a directory-listed reviewer is
verified — which the reference resolver never does, by design. A pack's self-declared
`legal_review.status = "approved"` does not clear it.

## Also exported

- `r14n.__version__` — the binding version
- `r14n.RLPS_NS` — the vocabulary namespace IRI (<https://r14n.squillo.com/ns>)
- `r14n.NOT_LEGAL_ADVICE` — the standing disclaimer string carried in decisions

## Links

- Project + spec: <https://github.com/squillo/r14n>
- Rust crate: <https://crates.io/crates/r14n>
- JS/TS: <https://www.npmjs.com/package/@squillo/r14n>

Apache-2.0 (code) · CC-BY-4.0 (spec & vocabulary) · © Squillo, Inc.
