---
name: spec
description: >-
  RLPS (Regulatory Localization Pack Specification, shorthand r14n) crash course and
  reference. Use when working with r14n or RLPS, `.r14n.toml` policy packs, the Control
  Catalog, the fail-closed resolution algorithm, posture/strictness selection
  (aggressive / as_configured / minimal), jurisdiction negotiation and the
  RFC-4647-style profile lookup, decision receipts (ISO/IEC TS 27560 + W3C DPV JSON-LD,
  Kantara CR v1.1), legal-review provenance and the attorney-of-record attestation
  envelope, the reviewer-key directory and trust root, conformance levels 1–3 and the
  vector suite, or the `r14n` CLI (extract / merge / validate / keygen / sign / verify /
  publish). Also covers the JS/TS and Python bindings generated from the Rust resolver,
  the RLPS vocabulary namespace, and the counsel-gating rules that govern what may be
  published. NOT LEGAL ADVICE — RLPS never states what the law is.
---

# RLPS (r14n) — working crash course

**`r14n : compliance controls :: i18n : strings`**

RLPS externalizes *which compliance controls are required* the way i18n externalizes
strings. Where i18n maps `locales/<lang>/<feature>.toml` → localized strings, RLPS maps
`packs/<profile>/<domain>.r14n.toml` → **required controls**. An application asks a
**resolver** "given who is involved and where, what is required right now?" and gets back
a control set plus provenance proving which pack drove the decision.

> **⚠ NOT LEGAL ADVICE — the first thing to internalize.** RLPS is a data-interchange and
> configuration standard. A pack is a machine-readable *template of controls an author
> chose to require*; it is not a statement of law, and it never certifies that any
> configuration is lawful. **"aggressive" ≠ "compliant"** — the posture names are a
> strictness dial, NOT a legal ordering. Never write, generate, or imply otherwise; never
> answer a user's "is this legal in X?" from a pack. See `docs/not-legal-advice.md`.

## Repo map

- `spec/RLPS-v0.1.md` — the normative spec (RFC 2119 keywords). §1 Scope, §2 Data model,
  §3 Posture, §4 Resolution algorithm (§4.1 jurisdiction attribution is an INPUT, §4.2
  cross-jurisdiction merge, §4.3 inheritance/delta-merge), §5 Fail-closed, §6 Provenance
  and trust-root, §7 Temporal validity, §8 Interoperability, §9 Conformance levels,
  §10 Governance, §11 Related work.
- `catalog/` — the canonical **Control Catalog**: control keys + deontic kind
  (`permit` / `obligation` / `prohibition`) + facets. A pack may only reference keys
  declared here.
- `packs/` — `aggressive` and `minimal` posture baselines (NOT jurisdiction claims) plus a
  deliberately fictional `example/region` pack demonstrating the
  `<regime>/<jurisdiction>` profile grammar.
- `schema/` — JSON Schema for packs and for the decision receipt.
- `conformance/` — language-neutral JSON vectors for the three levels, `vector.schema.json`,
  and `VERSIONING.md` (suite MINOR is append-only; level claims do not carry across a MAJOR).
- `resolver/` — the Rust reference resolver (crate `r14n`). `src/lib.rs` is the port +
  adapters, `src/receipt.rs` the receipt serializers, `src/wire.rs` the JSON boundary the
  bindings are generated over.
- `tools/` — the `r14n` CLI (crate `r14n-tools`).
- `bindings/` — `bindings/wasm` (wasm-bindgen → `@squillo/r14n` on npm) and
  `bindings/python` (PyO3 abi3 → `r14n` on PyPI). Thin FFI shims over `resolver/src/wire.rs`.
- `registry/` — reviewer-key directory + pack-index schemas (the trust root).
- `ns/` — the published vocabulary (`context.jsonld`, `rlps.ttl`), served at
  <https://r14n.squillo.com/ns>.
- `GOVERNANCE.md` — staging, roles, change process, and the **hard gates** below.

## The seven pieces (spec §2)

| Piece | What it is |
|---|---|
| **control** | One required action, a stable key (e.g. `attestation`), declared once in a catalog with a deontic `kind`. Domain-scoped: the key is unique within its owning domain. |
| **domain** | A regulated activity — selects the pack file (e.g. `recording_consent`). |
| **subject** | The discriminator within a domain (e.g. a recording rail: `telepresence`, `twin_attend`). |
| **profile** | A `<regime>/<jurisdiction>` tag — selects the pack directory (e.g. `gdpr/eu`, or a bare posture like `aggressive`). |
| **pack** | One `<domain>.r14n.toml` for one profile. |
| **decision** | The resolver's output: verdict + required control set + provenance. |
| **provenance** | The pack's legal-review block plus the signed decision receipt. |

## Pack shape

```toml
[meta]
strictness = "aggressive"        # posture — NOT a legal ordering
inherits = "gdpr/eu"             # optional parent profile to delta from (§4.3)
effective_from = "2026-01-01"    # optional temporal envelope (§2.5/§7)
effective_until = "2026-12-31"
last_reviewed_against_guidance = "2026-06-01"   # interpretation currency (§7)

[meta.legal_review]
status = "not_required"          # not_required | requires_signoff | approved
# A pack claiming a REAL jurisdiction MUST also carry:
#   reviewing_attorney_of_record, jurisdiction, bar_credential  (§6)

[subject.telepresence]           # per-subject table, used under `as_configured`
controls = ["signal_notice", "indicator_mount"]

[legally_required]               # the FLOOR — what `minimal` resolves to
controls = ["attestation", "signal_notice"]

[prohibited]                     # data-minimization guard: no posture may add these
controls = ["biometric_retention"]
```

## APH-backed controls — pick the right one of the two

Delegate/agent authorization comes in two control keys, not one, because
[APH](https://github.com/squillo/aph) distinguishes two trust models:

- **`aph_mandate_principal_signed`** — the human's own key signed the act. Consent,
  cryptographically. **This is what a consent-gating pack requires.**
- **`aph_mandate_notary_attested`** — a notary *asserts* the human authorized it. Provenance,
  not consent.
- **`aph_mandate`** — **deprecated and ambiguous**; never use it in a new pack.

Two rules that are easy to get wrong: an absent `attestationMode` on an APH envelope means the
**weak** mode, never "probably the strong one"; and neither key supplies any *other*
participant's consent — a mandate authorizes an agent to act for **its own** principal, which is
what `attestation` / `announcement` / `signal_notice` are for. Full contract, with citations:
`docs/aph-integration.md`. Do not restate APH's verification rules in r14n docs — cite them.

## Resolution — and the fail-closed rule that matters most

The caller supplies a **universe**: every control it can actually enforce. That is the
ceiling; a pack can never demand something outside it.

1. **Aggressive** → the whole universe.
2. **AsConfigured** → the `[subject.<subject>]` table, intersected with the universe. A
   subject *absent* from the pack fails closed to the whole universe.
3. **Minimal** → `[legally_required]`, intersected with the universe.
4. `[prohibited]` is subtracted from **every** posture's result, including aggressive.

**Fail-closed (spec §4/§5) — the property the standard rests on.** If the pack is missing,
malformed, outside its effective-date envelope, or is `minimal` with a missing/empty
`[legally_required]` floor, the resolver returns the caller's **entire universe** with
`provenance.fell_back = true`. It never returns an empty set and never silently
under-restricts. The reference port is **infallible** — it returns a decision, never an
error. (A *caller's* malformed JSON at the binding boundary is a different thing: that
raises/throws, so a bug is never mistaken for a policy fallback.)

## Provenance and the advisory-only taint (spec §6)

Every decision receipt is `advisory_only: true` unless provenance was **cryptographically
verified** — which the reference resolver never does (`PROVENANCE_VERIFIED = false`). A
pack's self-declared `legal_review.status = "approved"` does **not** clear the taint; it
does not prove *who* attested. Clearing it requires a signature from a key listed,
non-revoked, and non-expired in the reviewer-key directory, with a matching jurisdiction.

## Using the resolver

```rust
let universe: BTreeSet<r14n::ControlKey> = ["attestation", "signal_notice"]
    .into_iter().map(|s| r14n::ControlKey(s.into())).collect();
let query = r14n::RegulatoryQuery {
    domain: "recording_consent".into(),
    profile: r14n::RegulatoryProfile("aggressive".into()),
    jurisdiction: "all_party".into(),
    subject: "telepresence".into(),
    universe,
    as_of: Some("2026-08-23".into()),   // None disables the temporal check
};
let adapter = r14n::TomlRegulatoryPolicyAdapter::new("packs".into(), None);
let decision = r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);
```

`InMemoryRegulatoryPolicyAdapter` is the same pipeline keyed by
`"<profile>/<domain>.r14n.toml"` for hosts with no filesystem. JS/TS and Python get the
same decisions through generated bindings (`resolve` / `resolve_with_receipts`, JSON in and
JSON out) — never re-implement the logic in another language.

## The CLI

```console
r14n extract --catalog <catalog> --profile <profile>   # scaffold a fail-closed pack template
r14n merge <pack> --catalog <catalog>                  # msgmerge analogue: flag ONLY changed controls for re-review
r14n validate <pack>... --catalog <catalog>            # the LINTER (cross-file rules a schema cannot express)
r14n keygen --out <prefix>                             # Ed25519 keypair; <prefix>.seed is PRIVATE (0600, gitignored)
r14n sign <pack> --key <seed>                          # binds <profile>/<domain> identity, not just bytes
r14n verify <pack> --directory <dir> --as-of <date> --jurisdiction <j>
r14n publish <pack> --id <profile>/<domain>            # LOCAL content-addressed index only; never touches a network
```

## Conformance (spec §9)

Three levels — 1 minimal-viable, 2 configured, 3 comprehensive-with-provenance — defined by
the JSON vectors in `conformance/`, tagged by capability. Some vectors are authored *ahead*
of the reference implementation and are skipped by it; that is deliberate and the runner
pins exact run/skip counts so drift is loud. **Claim wording is governed** (`TRADEMARKS.md`):
say "conforms to RLPS Level N, rlps-conformance/0.1" only after passing every vector for
that level, and "implements Level N; Level N+1 in progress" otherwise — never a bare
"RLPS conformant".

Precedence when artifacts disagree: **the spec is normative**; vectors and schemas are not.
A vector that conflicts with the spec is a defect in the vector. If the spec is silent, that
silence is itself a spec defect — file an Erratum.

## Hard gates — do not cross these

1. **Publication of jurisdiction claims is counsel-gated.** Real-jurisdiction packs
   (`wiretap/us`, `gdpr/eu`, …) are deliberately held out of the repository. Do NOT author,
   generate, or commit one. A pack claiming a real jurisdiction requires a named
   `reviewing_attorney_of_record` + `jurisdiction` + `bar_credential`, authored or attested
   by licensed counsel.
2. **Every artifact carries the NOT-LEGAL-ADVICE disclaimer**, and no output may present a
   pack or decision as a statement of law.
3. **"aggressive" ≠ "compliant."** Never describe a posture as making anything compliant.

## Contributing (see `CONTRIBUTING.md`, `GOVERNANCE.md`)

Substantive changes start as a GitHub issue, not a PR — three forms carry the process: **RFC**
(normative text), **Erratum** (published text is wrong), **Conformance disagreement**
(an implementation disagrees with a published artifact). Every commit needs a DCO
`Signed-off-by` trailer. Every test carries a `/// Why:` doc stating the invariant it guards.
Bindings are **generated** from the Rust resolver, never hand-written, and their shims stay
thin — logic belongs in the core where one test suite covers every language.
