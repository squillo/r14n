# RLPS Governance

**Model: reference-implementation-first** (the OpenTelemetry trajectory) with
**federated-with-attestation** ownership. This document is the working governance policy for the
`r14n` project; it becomes a charter proposal when the project seeks a neutral home (Stage 3).

> **⚠ NOT LEGAL ADVICE.** Governance here covers the *specification and its artifacts*. Nothing
> in this process makes any pack a statement of law. See `docs/not-legal-advice.md`.

## Stages (from an internal design memo §10; do not skip gates)

| Stage | Window | Exit criteria |
|---|---|---|
| **1 — Existence proof** (now) | → ~6 mo | Spec v0.x + Rust reference resolver + Squillo's own posture baselines published (AFTER the counsel gate); shipping inside Squillo OS as the live consumer. |
| **2 — Corpus + second resolver** | 6–18 mo | 5–10 **counsel-attested** community packs (gdpr/eu, ccpa-cpra, pipeda, apra, lgpd); the `/conformance` suite adopted; at least one **independent** conforming resolver. |
| **3 — Neutral home** | 18–36 mo | With **3+ conforming resolvers and 20+ packs**: submit to a **W3C Community Group** (front-runner — DPV/GPC adjacency), with **OASIS Open Projects** as the alternative (LegalRuleML adjacency). NOT CNCF (the consent/legal framing is core, not cloud-native infra). |

Do **not** enter an SDO before the Stage-3 criteria hold — a standard with one implementer and no
corpus gets dismissed, and the "standard" label is earned, never asserted (critique verdict:
CONDITIONAL GO as a config-interop convention; NOT-YET a "standard for legal compliance").

## Roles

- **Steward** (currently Squillo): maintains the canonical Control Catalog, the reviewer-key
  directory (admission + revocation), the spec text, and the reference resolver. The steward
  signs the reviewer-key directory. Stewardship transfers to the neutral home at Stage 3.
- **Pack authors** (federated): **anyone MAY author and publish a pack** — but pack metadata MUST
  carry the legal-review attestation envelope (spec §6), and a pack claiming a real jurisdiction
  MUST name a `reviewing_attorney_of_record` + `jurisdiction` + `bar_credential`. Unattested or
  self-attested packs resolve **advisory-only** (the trust-root taint) — the format does not stop
  you publishing; it stops your pack being silently trusted.
- **Resolver implementers:** claim a conformance level ONLY by passing the corresponding
  `/conformance` suite (all capabilities of the level, all vectors).
- **Reviewers (licensed counsel):** appear in the reviewer-key directory with jurisdiction +
  credential; may attest packs within their jurisdiction. Directory admission standard is the
  hardest open governance question (an internal design memo §12.3) — until counsel defines it, admission is
  steward-manual and conservative.

## Change process

- **Spec:** versioned `spec/RLPS-vX.Y.md`; normative changes bump the version; breaking wire
  changes require a major bump + a migration note + coordinated twin-sync (the maintainer notes).
- **Control Catalog:** additive by default (new keys with deontic kind + facets). Renames and
  removals are DEPRECATIONS (a control key is long-lived audit evidence; `r14n merge` flags the
  delta for legal re-review). New terms enter `/docs/namespace.md` before first shipped use.
- **Conformance vectors:** adding a vector is a normative act — it pins behavior. Vectors ship
  with capability tags; the reference runner pins exact run/skip counts so drift is loud.
- **Packs:** content-addressed and immutable at a version; changes publish a new version whose
  `supersedes` chains the prior sha256 (`registry/pack-index.schema.json`).

## Hard gates (restated from the maintainer notes — these override everything above)

1. This repository stays **PRIVATE** until licensed counsel clears the UPL/liability posture
   (`docs/counsel-brief.md` is the gate document).
2. **Jurisdiction packs are held out** until counsel signs off; each requires an attorney of
   record. The counsel-safe subset is: spec, schemas, catalog, resolver, tools, conformance,
   and Squillo's own `aggressive`/`minimal` posture baselines.
3. Every artifact carries the NOT-LEGAL-ADVICE disclaimer. **"aggressive" ≠ "compliant"** —
   posture framing is never a legal ordering.
