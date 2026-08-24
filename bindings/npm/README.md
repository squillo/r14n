# @squillo/r14n

**RLPS for JS/TS — WebAssembly bindings generated from the Rust reference resolver.**
Not a port: the decision logic is the same `r14n` crate the Rust ecosystem links, compiled to
wasm. One behavior, one source of truth. **Zero runtime dependencies.**

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange and configuration standard. It does not state
> what the law is, and **"aggressive" ≠ "compliant"**. See
> <https://github.com/squillo/r14n/blob/main/docs/not-legal-advice.md>.

```console
$ npm install @squillo/r14n
```

## Resolve required controls

There is no filesystem in wasm, so packs are passed in as an object mapping
`"<profile>/<domain>.r14n.toml"` to the pack's TOML text — the same layout as on disk.

```js
import { resolve } from "@squillo/r14n";

const packs = {
  "aggressive/recording_consent.r14n.toml": await readFile("packs/aggressive/recording_consent.r14n.toml", "utf8"),
};

const decision = JSON.parse(resolve(
  JSON.stringify(packs),
  JSON.stringify({
    domain: "recording_consent",
    profile: "aggressive",
    jurisdiction: "all_party",
    subject: "telepresence",
    // EVERY control you can enforce — the fail-closed ceiling.
    universe: ["attestation", "signal_notice", "indicator_mount", "aph_mandate_principal_signed"],
    as_of: "2026-08-23", // optional; enables the effective-date check
  }),
  // optional global strictness dial: "aggressive" | "as_configured" | "minimal"
));

// decision.verdict            — "permit" | "block"
// decision.required           — controls you MUST satisfy (sorted, ⊆ universe)
// decision.provenance.source  — which pack governed
// decision.provenance.fell_back — true ⇒ NO reviewed pack governed this decision
```

**It fails closed.** A missing, malformed, or out-of-envelope pack — or a `minimal` pack with no
`[legally_required]` floor — resolves to your *entire* universe with `provenance.fell_back = true`.
It never returns an empty set and never silently under-restricts. Malformed *input* (bad JSON)
throws instead, so a caller bug is never mistaken for a policy fallback.

## Decision receipts

`resolve_with_receipts` returns the decision plus both serialized receipt forms — an ISO/IEC
TS 27560-structured W3C DPV JSON-LD document and a Kantara CR v1.1 shim:

```js
import { resolve_with_receipts } from "@squillo/r14n";

const { decision, receipt_dpv27560, receipt_kantara_cr_v1_1 } =
  JSON.parse(resolve_with_receipts(JSON.stringify(packs), query, JSON.stringify({
    record_id: "urn:uuid:…", issued_at: "2026-08-23T12:00:00Z", issued_at_unix: 1787832000,
    language: "en", pii_principal_id: "user-1", pii_controller: "Example Operator",
    domain: "recording_consent", subject: "telepresence",
    attribution_source: "operator_declared",
  })));
```

Receipts stay `advisory_only: true` until a signature from a directory-listed reviewer is
verified — which the reference resolver never does, by design. A pack's self-declared
`legal_review.status = "approved"` does not clear it.

## Also exported

- `version()` — the binding version
- `rlps_ns()` — the vocabulary namespace IRI (<https://r14n.squillo.com/ns>)

## Links

- Project + spec: <https://github.com/squillo/r14n>
- Rust crate: <https://crates.io/crates/r14n>
- Python: <https://pypi.org/project/r14n/>

Apache-2.0 (code) · CC-BY-4.0 (spec & vocabulary) · © Squillo, Inc.
