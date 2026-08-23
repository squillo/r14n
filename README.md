# r14n — Regulatory Localization

[![gates](https://github.com/squillo/r14n/actions/workflows/gates.yml/badge.svg)](https://github.com/squillo/r14n/actions/workflows/gates.yml)
[![crates.io](https://img.shields.io/crates/v/r14n)](https://crates.io/crates/r14n)
[![License](https://img.shields.io/badge/license-Apache--2.0_%2F_CC--BY--4.0-blue)](LICENSE)
[![Status](https://img.shields.io/badge/status-v0.1_pre--1.0-orange)](#status)
[![NOT LEGAL ADVICE](https://img.shields.io/badge/⚠-NOT_LEGAL_ADVICE_·_counsel--gated-red)](docs/not-legal-advice.md)

> **`r14n : compliance controls :: i18n : strings`**

**r14n** is the home of the **Regulatory Localization Pack Specification (RLPS)** — an open,
human-readable, machine-readable format for expressing *which compliance controls are required*
for a given **(regulatory-profile × jurisdiction × subject)**, with a fail-closed default, a
posture selector, and legal-review provenance.

**▶ [Watch the explainer video](https://drive.google.com/file/d/1N9q9FhnVUT6jzMLTHg-trTCcxBmdNVVa/view?usp=sharing)** — the i18n analogy, the fail-closed resolver, and the counsel-gating model in plain language, before the spec makes them precise.

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

## The problem it solves

Whether you may record a meeting — and what you must do first (get consent, show an on-screen
indicator, announce an AI notetaker, …) — depends on **where** everyone is and **what kind** of
recording it is. Today that logic is usually buried in application code: hard to audit, easy to get
subtly wrong, and impossible for a compliance officer to review without reading source.

This is a measured problem, not a hypothetical. US recording-consent law alone splits three ways
(38 one-party states + DC, 11 all-party states, one with no wiretap statute — 12 Harv. L. & Pol'y
Rev. 177). A think-tank estimate puts the US state *privacy* patchwork at $98–112B/year in
projected out-of-state compliance costs (ITIF 2022), the FCA/Bank of England pilots found
**inconsistent interpretation of regulations is the single biggest cost driver** of manual
compliance, and the AI era is multiplying the surface: state AI laws grew from 1/year (2016) to
131/year (2024) (Stanford HAI AI Index). Full annotated bibliography — 28 verified sources — in
[`docs/secondary-sources.md`](docs/secondary-sources.md).

```text
# BEFORE — jurisdiction rules tangled into app code (pseudocode, any language)
if any(participant in a two-party-consent state):
    require(consent)
else if any(participant in the EU):
    if ai_notetaker: require(announcement)
    require(consent)
# …dozens more brittle branches — invisible to a compliance reviewer, easy to get subtly wrong
```

```toml
# AFTER — the same rules as a reviewable pack: recording_consent.r14n.toml
[subject.twin_attend]
controls = ["announcement", "aph_mandate"]
```

The app stops *deciding* and just *enforces* what the resolver returns.

RLPS pulls those rules out of the code into **packs** — small, human-diffable TOML files that map a
`(profile × jurisdiction × subject)` to the **set of compliance controls required**. An app asks a
**resolver** "given who's involved and where, what's required right now?" and gets back a control
set plus a signed, machine-readable **receipt** proving which pack drove the decision and who
attested it. The resolver **fails closed**: if a pack is missing, malformed, expired, or the
jurisdiction is ambiguous, it demands *everything* the app can enforce and loudly flags the
fallback — it never silently under-restricts.

**The seven pieces** (spec §2): a **control** (one required action, a stable key), a **domain**
(regulated activity, e.g. `recording_consent`), a **subject** (the discriminator within a domain,
e.g. a recording rail), a **profile** (`<regime>/<jurisdiction>` tag), a **pack** (one
`<domain>.r14n.toml` for one profile), a **decision** (the resolver's output), and **provenance**
(the pack's legal-review block + a signed decision receipt).

## Examples

### 1. A pack — `packs/aggressive/recording_consent.r14n.toml`

A pack a compliance officer and an engineer can both read and review in a PR. The `posture`
selector (`strictness`) chooses which table governs; `[legally_required]` is the fail-closed floor.

```toml
[meta]
strictness = "aggressive"      # posture — NOT a legal ordering; "aggressive" ≠ "compliant"

[meta.legal_review]
status = "not_required"        # this baseline over-restricts; it can never under-restrict

[subject.telepresence]
controls = ["signal_notice", "indicator_mount", "aph_mandate"]

[subject.twin_attend]
controls = ["announcement", "aph_mandate"]

[legally_required]             # the floor: required under the `minimal` posture / as a catch-all
controls = ["attestation", "signal_notice", "indicator_mount", "announcement", "aph_mandate"]
```

A pack may also declare `[prohibited]` (controls no posture may add — the data-minimization guard),
`inherits` (a parent profile it deltas from), and an `effective_from`/`effective_until` envelope.

### 2. Resolving — the reference resolver (Rust)

<details>
<summary><b>Rust — build a query and resolve the required controls</b> (click to expand)</summary>

```rust
use std::collections::BTreeSet;

// The caller declares EVERY control it can enforce — the aggressive/fallback ceiling.
let universe: BTreeSet<r14n::ControlKey> =
    ["attestation", "signal_notice", "indicator_mount", "aph_mandate"]
        .into_iter().map(|s| r14n::ControlKey(s.into())).collect();

let query = r14n::RegulatoryQuery {
    domain: "recording_consent".into(),
    profile: r14n::RegulatoryProfile("aggressive".into()),   // selects packs/aggressive/
    jurisdiction: "all_party".into(),
    subject: "telepresence".into(),
    universe,
    as_of: Some("2026-07-08".into()),                        // enables the effective-date check
};

// Load packs from a directory laid out like locales/: <root>/<profile>/<domain>.r14n.toml
let adapter = r14n::TomlRegulatoryPolicyAdapter::new("packs".into(), None);
let decision = r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);

// decision.required      — the controls the caller MUST satisfy (⊆ universe)
// decision.verdict       — Permit | Block
// decision.provenance    — pack source, review status, fell_back, advisory-only taint
assert!(decision.required.contains(&r14n::ControlKey("indicator_mount".into())));
```
</details>

The port is **infallible**: a missing/malformed/expired/floor-less pack returns
aggressive-over-universe with `provenance.fell_back = true`, never an empty set.

<details>
<summary><b>JS/TS and Python — the same resolver, generated from the Rust core</b> (click to expand)</summary>

Both packages are **generated** from `resolver/` — wasm-bindgen for JS/TS, a PyO3 abi3 wheel for
Python — so they are the same decision logic, not re-implementations. Neither has runtime
dependencies. There is no filesystem in wasm, so packs are passed in as a JSON object mapping
`"<profile>/<domain>.r14n.toml"` to the pack's TOML text (the same layout as on disk).

```console
$ npm install @squillo/r14n     # https://www.npmjs.com/package/@squillo/r14n
$ pip install r14n              # https://pypi.org/project/r14n/
```

```js
import { resolve } from "@squillo/r14n";
const decision = JSON.parse(resolve(JSON.stringify(packs), JSON.stringify(query)));
decision.provenance.fell_back; // true ⇒ NO reviewed pack governed this decision
```

```python
import json, r14n
decision = json.loads(r14n.resolve(json.dumps(packs), json.dumps(query)))
```

`resolve_with_receipts` additionally returns both serialized receipt forms (TS 27560 + DPV
JSON-LD, and the Kantara CR v1.1 shim). Fail-closed behavior is identical in every host; a
*caller's* malformed JSON throws/raises instead, so a bug is never mistaken for a policy
fallback.
</details>

### 3. The pack lifecycle — the `r14n` CLI

```console
$ r14n extract --catalog catalog/recording_consent.catalog.toml --profile gdpr/eu   # scaffold a pack
$ r14n validate packs/aggressive/recording_consent.r14n.toml \
        --catalog catalog/recording_consent.catalog.toml                            # lint it
$ r14n keygen --out reviewer                                                         # Ed25519 keypair
$ r14n sign packs/aggressive/recording_consent.r14n.toml --key reviewer.seed         # attest it
$ r14n verify packs/aggressive/recording_consent.r14n.toml --directory registry/directory.json \
        --as-of 2026-07-08 --jurisdiction US-CA        # + check the signer is a trusted reviewer
$ r14n publish packs/aggressive/recording_consent.r14n.toml --id aggressive/recording_consent
```

`merge` is the gettext-`msgmerge` analogue: when the catalog changes it flags **only** the changed
controls for legal re-review, preserving the reviewed pack byte-for-byte.

### 4. The decision receipt (ISO/IEC TS 27560 + W3C DPV JSON-LD)

Every decision can be serialized as a signed, machine-readable receipt — audit evidence of *what*
was decided, by *which* pack, and whether provenance was verified.

<details>
<summary><b>JSON — a decision receipt</b> (trimmed; click to expand — <a href="docs/examples/receipt-ai-act-50.json">full example</a>)</summary>

```jsonc
{
  "@type": "rlps:ControlDecisionReceipt",
  "processing": { "domain": "recording_consent", "subject": "twin_attend",
                  "operations": ["rlps:AudioRecording", "rlps:VoiceRecording"] },
  "jurisdiction": { "dpv:hasJurisdiction": ["DE"], "attribution_source": "operator_declared" },
  "decision": { "verdict": "permit", "posture": "aggressive",
                "required_controls": ["ai_disclosure", "announcement", "attestation", …] },
  "provenance": { "advisory_only": true, "provenance_verified": false, "fell_back": false },
  "ai_disclosure": { "disclosed_at": "2026-08-02T08:59:00Z", "method": "in_meeting_announcement" },
  "disclaimer": "NOT LEGAL ADVICE. …"
}
```
</details>

`advisory_only` stays `true` until a signature from a directory-listed, non-revoked reviewer is
verified — a self-declared `legal_review.status = "approved"` never clears it on its own.

## Agents acting on a human's behalf (APH)

When an AI agent — a "twin" — records or attends on a person's behalf, the question isn't only
*may this be recorded here* but *is this agent authorized to act for this human at all*. RLPS
already carries that as a first-class control: **`aph_mandate`** ("a valid standing mandate
authorizes a delegate/twin to act on the user's behalf"), required on the delegate/twin rails
(`twin_attend`, `telepresence`).

That control is the plug-in point for **APH — Agent-Per-Human notarization** (Squillo's protocol
extending Google's **A2A** agent-to-agent protocol and the **AP2** agent-payments protocol): the
APH mandate is the machine-verifiable proof that a specific agent may act for a specific human, and
RLPS consumes a **verified** mandate as a satisfied control. So the same fail-closed resolver that
decides *which recording controls apply* also gates *whether a delegated agent is permitted to act
in the first place* — a missing or unverified mandate simply leaves `aph_mandate` unmet, and the
action does not proceed. RLPS **interoperates with** APH (it treats the mandate as a control); it
does not define the mandate itself. Still NOT legal advice.

## Status

**v0.1 — draft. Reference-implementation-first.** This repository is the working existence proof;
a twin of the resolver already ships inside [Squillo OS](https://squillo.com) as its
regulatory-policy engine — the live first consumer.

## Repository layout (counsel-safe subset)

- [`spec/RLPS-v0.1.md`](spec/RLPS-v0.1.md) — the normative specification (conflict rules,
  trust-root, temporal split, interop mappings).
- [`schema/`](schema/) — JSON Schema for `.r14n.toml` packs + the decision-receipt schema
  ([`receipt.schema.json`](schema/receipt.schema.json); mapping to TS 27560 in
  [`docs/receipt-27560-mapping.md`](docs/receipt-27560-mapping.md)).
- [`ns/`](ns/) — the published RLPS vocabulary: the `@context`
  ([`context.jsonld`](ns/context.jsonld)) + SKOS/RDFS term definitions ([`rlps.ttl`](ns/rlps.ttl)),
  so the `https://r14n.squillo.com/ns#` namespace resolves.
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
- [`bindings/`](bindings/) — the JS/TS and Python packages, **generated from `resolver/`**:
  [`bindings/wasm`](bindings/wasm) (wasm-bindgen shim → [`@squillo/r14n`](https://www.npmjs.com/package/@squillo/r14n)
  on npm, built by [`bindings/npm/build.mjs`](bindings/npm/build.mjs)) and
  [`bindings/python`](bindings/python) (PyO3 abi3 shim → [`r14n`](https://pypi.org/project/r14n/)
  on PyPI, built by maturin). Both are thin FFI shims over one tested JSON boundary
  ([`resolver/src/wire.rs`](resolver/src/wire.rs)) — not ports, so there is nothing to drift.
- [`site/`](site/) — the [r14n.squillo.com](https://r14n.squillo.com) Worker: the splash page and
  the vocabulary namespace host (`/ns`), which serves the `ns/` artifacts byte-identical by
  importing them directly at bundle time.
- [`r14n Spec/`](r14n%20Spec/) + [`baselines/`](baselines/) + [`snapp/`](snapp/) — the RLPS
  specification expressed as an **N Lang Snapp** (typed, machine-checkable spec sources), a
  baseline-pack consumer Snapp, and the emitted ABI bundle other Snapps consume. The Snapp
  *sources* are Apache-2.0 like the code; **N Lang itself** (language, compiler, runtime, Snapp
  tooling) is proprietary to Squillo, Inc. and not required to use RLPS — the markdown spec in
  [`spec/`](spec/) is the normative text, and where the two disagree, `spec/` wins.
- [`GOVERNANCE.md`](GOVERNANCE.md) — reference-impl-first staging, federated-with-attestation
  ownership, SDO entry criteria. [`CONTRIBUTING.md`](CONTRIBUTING.md) (issue-first RFC process,
  DCO + pack gate), [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), [`CHANGELOG.md`](CHANGELOG.md),
  [`TRADEMARKS.md`](TRADEMARKS.md) (name + conformance-claim usage),
  [`SECURITY.md`](SECURITY.md) + [`docs/key-management.md`](docs/key-management.md) (crypto threat
  surface + key hygiene), [`docs/receipt-data-handling.md`](docs/receipt-data-handling.md) (receipts
  carry PII), [`docs/secondary-sources.md`](docs/secondary-sources.md) (the 28-source annotated
  evidence annex: fragmentation cost + AI-era amplification + policy-as-code lineage),
  [`docs/wiretap-us-dossier.md`](docs/wiretap-us-dossier.md) (secondary-source research memo for
  the counsel-gated `wiretap/us` engagement — no pack, no legal conclusions), and
  [`docs/audits/`](docs/audits/) (the standing council-audit reports).
- CI: [`.github/workflows/gates.yml`](.github/workflows/gates.yml) runs both test suites (which
  include the schema-vs-producers conformance tests in
  [`tools/tests/schema_conformance.rs`](tools/tests/schema_conformance.rs)), lints the shipped
  packs, and gates on sanitization (no internal references in tracked files), clippy
  (warnings-as-errors), the declared MSRV (1.85), and DCO sign-off on PRs. Rust only — no Python.

**Deliberately NOT here yet (held for licensed counsel):** real jurisdiction packs
(`wiretap/us` 50-state matrix, `gdpr/eu`, `ccpa-cpra/us/ca`, …). Any pack claiming a real
jurisdiction MUST carry a `reviewing_attorney_of_record` + bar number and be attested by licensed
counsel — see the spec §Governance and the counsel gate in [`GOVERNANCE.md`](GOVERNANCE.md).

## Agent pack

Coding agents get RLPS as a first-class skill rather than a re-read of the spec. One knowledge
source serves both ecosystems: [`skills/spec/SKILL.md`](skills/spec/SKILL.md) is written in the
open [Agent Skills](https://agents.md) format, [`AGENTS.md`](AGENTS.md) is the agents.md-convention
entry point (Codex and others) carrying orientation plus the invariants that must not break, and
[`.claude-plugin/`](.claude-plugin/) packages the same skill with slash commands
([`/validate`](commands/validate.md), [`/resolve`](commands/resolve.md),
[`/conformance`](commands/conformance.md)).

```console
/plugin marketplace add squillo/r14n
/plugin install r14n@r14n-rlps
```

The pack teaches the fail-closed rule, the advisory-only taint, and the governed
conformance-claim wording — and it refuses to author a real-jurisdiction pack, because those are
counsel-gated.

## Honest positioning

RLPS is **not** the first attempt to make compliance machine-readable — see the related work in the
spec (NIST OSCAL, W3C DPV / ISO 27560, Policy Cards, LegalRuleML, OPA/Cedar), and it sits in a
recognized *institutional* lineage: governments themselves publish and pilot machine-consumable
rules (OECD "Rules as Code"; the FCA/Bank of England machine-executable-regulation pilots; FSB
suptech; BIS embedded supervision — spec §11 and [`docs/secondary-sources.md`](docs/secondary-sources.md) §V).
RLPS occupies one specific, un-owned layer: the **control-prescription layer**, authored in
i18n-ergonomic TOML a compliance officer *and* an engineer can diff in a PR, with an
RFC-4647-style jurisdiction-negotiation algorithm and a fail-closed floor. It interoperates with
those standards rather than replacing them.

## License

Dual: the **spec text, catalog, and schemas** are **CC BY 4.0**; the **reference resolver and
tooling code** are **Apache-2.0**. See [`LICENSE`](LICENSE) (code) and [`LICENSE-SPEC`](LICENSE-SPEC) (docs).

## Authors

Squillo, Inc.
