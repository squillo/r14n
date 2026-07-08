# r14n — Regulatory Localization

> **`r14n : compliance controls :: i18n : strings`**

**r14n** is the home of the **Regulatory Localization Pack Specification (RLPS)** — an open,
human-readable, machine-readable format for expressing *which compliance controls are required*
for a given **(regulatory-profile × jurisdiction × subject)**, with a fail-closed default, a
posture selector, and legal-review provenance.

Where **i18n** maps `locales/<lang>/<feature>.toml` → localized strings, **RLPS** maps
`packs/<profile>/<domain>.r14n.toml` → **required controls**. A translation key is a stable id
for a user-facing string; an RLPS **control key** is a stable id for a required compliance action.

```
i18n:  translation key  → localized string      locales/<lang>/<feature>.toml
r14n:  control key       → required / prohibited  packs/<profile>/<domain>.r14n.toml
```

## ⚠ This is NOT legal advice

RLPS is a **data-interchange and configuration standard**, not legal counsel. A pack is a
machine-readable *template of controls*; it does not tell you what the law is, and selecting a
posture that under-restricts a jurisdiction's law is the **consuming operator's** responsibility.
**"aggressive" ≠ "compliant."** See [`docs/not-legal-advice.md`](docs/not-legal-advice.md).

## Status

**v0.1 — draft. Reference-implementation-first.** This repository is the working existence proof;
the resolver already ships inside [Squillo OS](https://squillo.com) as `the Squillo OS policy engine`.

**What's here now (counsel-safe subset):**
- [`spec/RLPS-v0.1.md`](spec/RLPS-v0.1.md) — the normative specification (conflict rules,
  trust-root, temporal split, interop mappings).
- [`schema/`](schema/) — JSON Schema for `.r14n.toml` packs + the decision-receipt schema
  ([`receipt.schema.json`](schema/receipt.schema.json); mapping to TS 27560 in
  [`docs/receipt-27560-mapping.md`](docs/receipt-27560-mapping.md)).
- [`ns/`](ns/) — the published RLPS vocabulary: the `@context`
  ([`context.jsonld`](ns/context.jsonld)) + SKOS/RDFS term definitions ([`rlps.ttl`](ns/rlps.ttl)),
  so the `https://rlps.squillo.com/ns#` namespace resolves.
- [`catalog/`](catalog/) — the canonical CONTROL CATALOG (control keys + deontic kind + facets).
- [`resolver/`](resolver/) — the Rust reference resolver (embeddable crate; run `cargo test`)
  including the [`receipt`](resolver/src/receipt.rs) module: ISO/IEC TS 27560-structured + W3C DPV
  JSON-LD decision receipts + a Kantara CR v1.1 shim ([namespace](docs/namespace.md); worked
  [AI-Act §50 example](docs/examples/receipt-ai-act-50.json)). Fail-closed everywhere; receipts are
  `advisory_only` until provenance is cryptographically verified.
- [`packs/`](packs/) — Squillo's own `aggressive` + `minimal` posture baselines (NOT jurisdiction
  claims) + a fictional `example/region` pack that demonstrates the `<regime>/<jurisdiction>`
  profile grammar (also NOT a jurisdiction claim).
- [`conformance/`](conformance/) — language-neutral JSON test vectors for the 3 conformance levels.
- [`tools/`](tools/) — the `r14n` pack-lifecycle CLI: `extract` / `merge` / `validate` /
  `keygen` / `sign` / `verify` / `publish`. Signatures bind the pack's `<profile>/<domain>`
  identity (not just its bytes); `verify --directory` checks the signer against a reviewer-key
  directory (revocation + expiry + jurisdiction); `publish` writes a local content-addressed index.
- [`registry/`](registry/) — reviewer-key directory + pack-index schemas (trust root, versioning,
  supersession).
- [`GOVERNANCE.md`](GOVERNANCE.md) — reference-impl-first staging, federated-with-attestation
  ownership, SDO entry criteria. [`CONTRIBUTING.md`](CONTRIBUTING.md) (DCO + pack gate),
  [`TRADEMARKS.md`](TRADEMARKS.md) (name + conformance-claim usage),
  [`SECURITY.md`](SECURITY.md) + [`docs/key-management.md`](docs/key-management.md) (crypto threat
  surface + key hygiene), [`docs/receipt-data-handling.md`](docs/receipt-data-handling.md) (receipts
  carry PII), and [`docs/audits/`](docs/audits/) (the standing council-audit reports).
- CI: [`.github/workflows/gates.yml`](.github/workflows/gates.yml) runs both test suites, lints the
  shipped packs, and validates every producer against its JSON Schema
  ([`scripts/check-schemas.py`](scripts/check-schemas.py)).

**Deliberately NOT here yet (held for licensed counsel):** real jurisdiction packs
(`wiretap/us` 50-state matrix, `gdpr/eu`, `ccpa-cpra/us/ca`, …). Any pack claiming a real
jurisdiction MUST carry a `reviewing_attorney_of_record` + bar number and be attested by licensed
counsel — see the spec §Governance. **This repo is not public until that legal review is complete.**

## Honest positioning

RLPS is **not** the first attempt to make compliance machine-readable — see the related work in the
spec (NIST OSCAL, W3C DPV / ISO 27560, Policy Cards, LegalRuleML, OPA/Cedar). RLPS occupies one
specific, un-owned layer: the **control-prescription layer**, authored in i18n-ergonomic TOML a
compliance officer *and* an engineer can diff in a PR, with an RFC-4647-style jurisdiction-negotiation
algorithm and a fail-closed floor. It interoperates with those standards rather than replacing them.

## License

Dual: the **spec text, catalog, and schemas** are **CC BY 4.0**; the **reference resolver and
tooling code** are **Apache-2.0**. See [`LICENSE`](LICENSE) (code) and [`LICENSE-SPEC`](LICENSE-SPEC) (docs).

---
*Reference: Squillo OS an internal design memo. Research provenance: adversarially-verified workflow (5 web sweeps + synthesis + critique).*
