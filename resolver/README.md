# r14n — RLPS reference resolver

The embeddable Rust reference resolver for the **Regulatory Localization Pack Specification
(RLPS)**: it loads `.r14n.toml` packs and resolves `(profile × jurisdiction × subject) → required
compliance controls`, **fail-closed**, and serializes ISO/IEC TS 27560-structured + W3C DPV decision
receipts. `r14n : compliance controls :: i18n : strings`.

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange and configuration format, not legal counsel. A
> pack is a machine-readable template of controls; it does not tell you what the law is.
> **"aggressive" ≠ "compliant."** See the repo's `docs/not-legal-advice.md`.

## Use

```rust
let adapter = r14n::TomlRegulatoryPolicyAdapter::new(pack_root, None);
let decision = r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);
// decision.required — the controls the caller MUST satisfy (⊆ query.universe)
// decision.provenance — pack source, review status, fell_back, advisory-only taint
```

The port is **infallible**: a missing/malformed/expired/floor-less pack degrades to
aggressive-over-universe with a loud `fell_back` flag — never an empty set. See
[`receipt`](src/receipt.rs) for decision-receipt serialization.

## Design

- **Generic:** `serde` + `toml` + `serde_json` + `std` only, zero framework deps. It is a twin of
  the Squillo OS regulatory-policy engine; keep it generic so the two stay in sync.
- **Fail-closed everywhere** (spec §4/§5); receipts are `advisory_only` until provenance is
  cryptographically verified (the reference resolver verifies none — that is the tooling's job).

## Stability

Pre-1.0: the API and pack/receipt formats **may change** between `0.x` minor versions; breaking
changes are noted in the repo's `CHANGELOG.md`. From `1.0` the crate will follow semver.
**MSRV: Rust 1.85.**

## License

Apache-2.0 (`../LICENSE-APACHE`, `../NOTICE`). The spec, schemas, and vocabulary are CC-BY-4.0.
