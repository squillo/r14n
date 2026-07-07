# Counsel brief — RLPS (r14n) pre-publication legal review

**Prepared:** 2026-07-06, by the RLPS maintainers (Squillo). **Audience:** licensed counsel
retained to review this project before the repository is made public.
**Design record:** Squillo OS an internal design memo; normative spec: `spec/RLPS-v0.1.md` in this repo.

**Status quo:** this repository is **PRIVATE** and stays private until the reviews requested
here are complete (an internal design memo §11 gate). Nothing has been published, announced, or distributed.

---

## 1. What RLPS is (plain language)

RLPS — the *Regulatory Localization Pack Specification* — is an open-source **file format plus a
software library**. It lets an application vendor express, in configuration files ("packs"),
*which technical compliance controls the application should enforce* for a given regulatory
profile, jurisdiction, and activity (for example: "before recording a meeting, require an
explicit user attestation and an on-screen recording indicator").

The analogy we use: what i18n does for user-facing strings (externalize them into per-language
files instead of hardcoding), RLPS does for compliance controls (externalize them into
per-profile files instead of hardcoding). The resolver library reads the packs and answers
"which controls are required right now?" — **failing closed** (requiring *everything* the app
can enforce) whenever a pack is missing, malformed, expired, or the jurisdiction is ambiguous.

What RLPS deliberately is **not**:

- It does **not** encode, interpret, or state the law (that is LegalRuleML / Akoma Ntoso
  territory, which we cite and avoid).
- It does **not** decide which jurisdiction's law applies — jurisdiction is an *input* supplied
  by the consuming application; when that input is ambiguous the resolver blocks.
- It does **not** certify or represent that any configuration is lawful. The spec normatively
  states "**aggressive ≠ compliant**" and that no posture is a compliance claim.

## 2. What we are asking counsel to review

1. **UPL (unauthorized practice of law) exposure** — §3 below.
2. **Liability allocation and disclaimer adequacy** — §4 below.
3. **Trademark / naming clearance** — §5 below (pre-flight data in
   `docs/preflight-name-check.md`).
4. **The jurisdiction-pack publication model** — §6 below: the conditions under which
   jurisdiction-specific packs (e.g. a US recording-consent matrix) may be published at all, and
   the required attorney-of-record attestation envelope.
5. Anything counsel identifies that we have not — this brief frames our questions; it does not
   bound the review.

## 3. UPL question

**The concern (an internal design memo §11):** a tool that appears to produce authoritative legal conclusions
("in Germany you must do X") could be characterized as practicing law, and a *jurisdiction pack*
authored by a non-lawyer could be characterized as legal advice to every downstream user.

**Posture we have taken (for counsel to validate or correct):**

- Positioning is "localization pack **specification**" — a configuration/interchange format —
  never "localizes law" or "tells you what the law is."
- Every artifact (spec, README, catalog, packs, resolver docs) carries a NOT-LEGAL-ADVICE
  disclaimer (`docs/not-legal-advice.md`), stating no attorney–client relationship is created.
- A pack is framed as "a template of controls an author chose to require," not a statement of law.
- Real-jurisdiction packs are **held out of the repo entirely** until this review; when
  published, each MUST name a `reviewing_attorney_of_record`, the reviewer's `jurisdiction`, and
  a `bar_credential` (spec §6 makes these non-optional for jurisdiction claims).
- Decisions resolved from *unattested or self-attested* packs are normatively **advisory-only**
  and carry an `unverified_provenance` taint in the machine-readable decision receipt.

**Questions for counsel:**

- (a) Is the specification + resolver + non-jurisdictional baseline packs (the current repo
  contents), with these disclaimers, publishable without UPL exposure — in the US, and are there
  material differences in the EU/UK?
- (b) Does *hosting a community registry* of third-party jurisdiction packs change the analysis
  (platform/republisher exposure)?
- (c) Does the attorney-of-record attestation model itself create issues for the attesting
  attorneys (advice to unknown third parties, malpractice-coverage scope, bar advertising
  rules), and how should the attestation text be worded to stay within opinion-practice norms?
- (d) Is the phrase "Regulatory Localization" itself, or the i18n analogy, likely to be read as
  a claim to translate law → if so, what wording changes does counsel direct?

## 4. Liability-allocation question

**The concern (an internal design memo §12.1, the "dial-down attack"):** a consuming application sets the
`minimal` posture (or authors a lax pack), under-restricts, and violates a recording/consent
law. Where does liability land — the pack author, the resolver implementer (us), the spec
publisher (us), or the deploying operator?

**Posture we have taken (for counsel to validate or correct):**

- The disclaimer states liability rests with the **consuming operator** who selected the
  posture/pack; the spec states it "cannot and does not allocate legal liability."
- Technical mitigations that narrow the negligence surface: fail-closed defaults everywhere;
  `minimal` normatively requires an attested legal sign-off to select; the decision receipt
  records the posture, pack SHA, and any fallback — so *who chose what* is always auditable.
- Licensing: code Apache-2.0 (§7–8 warranty disclaimer + liability exclusion), spec/schemas/packs
  CC BY 4.0 (§5 warranty disclaimer).

**Questions for counsel:**

- (a) Are Apache-2.0/CC-BY-4.0 warranty disclaimers adequate here, or do we need a
  project-specific terms-of-use / additional-disclaimer layer for packs specifically?
- (b) Should the pack format carry an *in-band* liability notice field (machine-readable
  disclaimer per pack), and does that help or hurt?
- (c) The attestation flow: when licensed counsel signs a jurisdiction pack, what engagement
  structure limits the attesting attorney's exposure to downstream consumers they never advised?
- (d) Contributor exposure: does a non-lawyer community contributor to a *draft* jurisdiction
  pack (pre-attestation) face UPL/liability risk, and what contribution-gating or CLA language
  is required?

## 5. Trademark / naming clearance

Engineering pre-flight completed 2026-07-06 (`docs/preflight-name-check.md`): package
registries clean; `github.com/squillo/r14n` secured via our org; web search shows *no dominant
prior claim* on "RLPS" / "r14n" / "Regulatory Localization" in software/compliance classes —
nearest brand users are RLPS Architects (architecture, since 1954) and RLPS Technology
(packaging). **We could not run formal TESS/EUIPO/WIPO searches; that is counsel's task** (word
marks: `RLPS`, `r14n`, `RegLoc`, "Regulatory Localization"; Nice classes 9, 42, 45). We also
want counsel's view on whether to file, or rely on use + the descriptive-mark weakness.

## 6. The counsel-gated publication sequence (for context)

Per an internal design memo §9–11 the publication order is: (1) this review; (2) publish spec + resolver +
Squillo's own two non-jurisdictional baseline packs; (3) only thereafter, jurisdiction packs
authored/attested by licensed counsel with attorney-of-record metadata — the first two planned
are a US recording-consent matrix (`wiretap/us`) and `gdpr/eu`. Counsel review of *those* packs
is a separate, later engagement; this brief covers steps (1)–(2).

## 7. Repo contents at review time (the proposed public subset)

*(Refreshed 2026-07-07 to the actual tree — the repo grew a tooling CLI, conformance suite,
registry schemas, decision-receipt serializer, and governance doc since the first draft.)*

- `spec/RLPS-v0.1.md` — the normative spec (RFC-2119).
- `schema/pack.schema.json` — pack JSON Schema.
- `catalog/recording_consent.catalog.toml` — control catalog (identifiers + descriptions only).
- `resolver/` — Rust library, Apache-2.0 (control resolution + the `receipt` module, §8 below);
  **54 tests** at review time (resolver + tools).
- `conformance/` — language-neutral JSON test vectors for the three conformance levels.
- `tools/` — the `r14n` pack-lifecycle CLI (extract / merge / validate / keygen / sign / verify /
  publish-to-local-index), Apache-2.0.
- `registry/` — JSON Schemas for the reviewer-key directory and the pack index (no populated
  instances — those are counsel-gated).
- `packs/aggressive/`, `packs/minimal/` — Squillo's own posture baselines; **no jurisdiction
  claims** (their `legal_review.status` is `not_required` / `requires_signoff` respectively).
- `docs/` — not-legal-advice, this brief, the name-check record, the namespace registry, a worked
  receipt example, and the audit reports under `docs/audits/`.
- `GOVERNANCE.md`, `README.md`, `the maintainer notes`, `LICENSE` (Apache-2.0), `LICENSE-SPEC` (CC BY 4.0).

## 8. Decision receipts — an added review surface (please opine)

The resolver emits a machine-readable **decision receipt** (an ISO/IEC TS 27560-structured
JSON-LD document + a flat Kantara Consent Receipt v1.1 compatibility shim). Two points we want
counsel's view on, because they touch the UPL surface of §3:

- (a) **We removed a machine-generated legal conclusion.** Earlier receipts stamped
  `legal_basis_hint: "eu_ai_act_article_50"` onto AI-disclosure events; on review that reads as
  the tool asserting a governing statute, so the resolver now records only the disclosure *facts*
  (timestamp + method) and leaves the legal basis to the pack author. **Does even the factual
  disclosure record, or the receipt's naming of controls, carry UPL risk we should further hedge?**
- (b) **The receipt never asserts that consent was obtained** — it records which controls a pack
  *requires*. The Kantara shim was changed to drop a fabricated `consentType: "EXPLICIT"` and now
  carries a top-level notice to that effect. **Is that notice sufficient to prevent a downstream
  party from treating an RLPS receipt as consent evidence?**
- (c) **Provenance is not cryptographically verified in v0.1**, so every receipt is marked
  `advisory_only: true` regardless of a pack's self-declared `legal_review.status`. We believe
  this is the honest posture; **please confirm it does not itself create a representation problem.**

---

*This brief was prepared by the project maintainers and is a request for legal review, not a
legal analysis. Nothing in this repository is legal advice.*
