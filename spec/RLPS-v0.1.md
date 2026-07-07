# Regulatory Localization Pack Specification (RLPS) v0.1

**Status:** Draft. **Shorthand:** `r14n`. **Requirement keywords** (MUST / SHOULD / MAY) per RFC 2119.

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange + configuration standard. It does not state
> what the law is. A conforming implementation makes no representation that any posture or pack is
> lawful in any jurisdiction. **"aggressive" ≠ "compliant."** See `/docs/not-legal-advice.md`.

## 1. Scope

RLPS specifies a portable format and resolution algorithm for the **control-prescription layer**:
a map from **(regulatory-profile × jurisdiction × subject) → the set of required compliance
CONTROLS**, authored in human-diffable TOML, resolved fail-closed, with legal-review provenance.

RLPS is **not** a legal-rule encoding (cf. LegalRuleML, Akoma Ntoso), a privacy vocabulary (cf.
W3C DPV), a security-control catalog (cf. NIST OSCAL), or a runtime policy engine (cf. OPA, Cedar).
It is the stable-key layer *between* legal text and application configuration, and it interoperates
with those standards (§8).

## 2. Data model (normative)

An implementation MUST model these seven entities:

1. **Control** — the atom. Declared once in a **Control Catalog** with a `kind` ∈
   {`permit`, `obligation`, `prohibition`} and OPTIONAL faceted fields (`trigger`, `mechanism`,
   `retention`, `subject_rights`, `legal_basis_hint`; an `obligation` SHOULD carry `deadline` +
   `consequence`). A control is **domain-scoped**: the control *key* is a bare identifier
   (e.g. `attestation`) that is unique within — and namespaced by — its owning `domain` (the
   catalog/pack it is declared in), so `recording_consent`'s `attestation` and another domain's
   `attestation` are distinct controls. The on-disk key is the bare form; the domain comes from
   context (catalog/pack file), never a literal `domain.control` string. A control is a stable
   identifier, NOT legal text.
2. **Domain** — partitions controls by regulated activity (e.g. `recording_consent`). On disk it
   is the pack filename: `<domain>.r14n.toml`.
3. **Subject** — a first-class key dimension within a domain (e.g. a recording rail). A pack
   expresses `[subject.<name>] controls = [...]`.
4. **Profile** — a `<regime>/<jurisdiction>/<subjurisdiction>` hierarchical tag (e.g.
   `gdpr/eu/de`). Resolved by **RFC-4647-style** progressive-subtag truncation (the truncation
   procedure of RFC 4647 §3.4, adapted here from BCP-47 `-`-delimited language tags to
   `/`-delimited profile tags; the adapted algorithm is defined normatively in §4) with
   delta-only inheritance.
5. **Pack** — one `<domain>.r14n.toml` for one profile. MUST contain `[meta]` (with `posture`
   and an effective-date envelope), MAY declare an inheritance parent, MUST provide a
   `[legally_required]` floor, MAY declare a `[prohibited]` table (controls the profile forbids —
   the §3 data-minimization guard's wire input; it MUST NOT intersect the floor), and MUST carry
   a `[meta.legal_review]` block. A pack is content-addressed and temporally bounded.
6. **Decision** — the resolver output for a query `(profile, jurisdiction, domain, subject,
   posture)` → `{ required_controls, verdict ∈ {permit, block} }`.
7. **Provenance** — (a) pack-provenance `[meta.legal_review]`; (b) decision-provenance, a receipt
   (§8) that MUST record the posture value, the pack SHA, and any escalation signal.

## 3. Posture (normative — NOT a legal ordering)

A pack MUST declare the **posture selector** — wire field `[meta].strictness` ∈ {`aggressive`,
`as_configured`, `minimal`} (the field is named `strictness` in v0.1; the *concept* is "posture",
and it is deliberately NOT a legal ordering). A resolver MAY accept a global override.

- `aggressive` — the resolver MUST require the caller's full declared control universe. It MUST
  NOT add a control a resolved jurisdiction pack marks `prohibition` (data-minimization guard;
  wire input: the pack's `[prohibited]` table, subtracted from EVERY posture's resolved set).
  Implementations MUST document that `aggressive` MAY over-collect and is not a compliance claim.
- `as_configured` — the resolver MUST apply exactly the resolved `[subject.<name>].controls` set.
  A subject absent from the pack MUST fail closed (require the full universe).
- `minimal` — the resolver MUST require only `[legally_required]`. Selecting `minimal` for a real
  jurisdiction SHOULD require an attested legal sign-off.

**Per-domain posture.** A single global posture lever is insufficient: one session may span
domains with different appropriate postures (e.g. `recording_consent` at `as_configured` while
`voice_biometric` stays `aggressive`). A resolver MUST support a per-domain posture override and
MUST apply precedence *most-specific-wins*: per-domain override → global override → the pack's
declared posture. Each domain MUST be resolved independently; a resolver MUST NOT merge control
sets across domains (a `recording_consent` decision never satisfies a `voice_biometric` control).
The data-minimization guard (§3, `aggressive`) applies per domain after posture selection.

## 4. Resolution algorithm (normative)

When the caller supplies a decision date (`as_of`), a pack whose effective-date envelope
(`effective_from`/`effective_until`) does not cover it MUST fail closed (no in-effect pack governs);
`as_of` absent opts out of the temporal check. A conforming resolver MUST, in order: (1) negotiate the profile by RFC-4647-style progressive
truncation over `/`-delimited subtags (`gdpr/eu/de → gdpr/eu → gdpr → root`); (2) delta-merge down the inheritance chain; (3) apply the
posture to select the governing table (per-domain precedence per §3); (4) on a session spanning
jurisdictions, apply the **most-restrictive merge** (§4.2); (5) escalate a subject to
`aggressive` on a `Sec-GPC: 1` signal or an IEEE-7012 `NoRecording` term. It MUST return
`verdict = block` when: no pack matches, the pack is outside its effective-date envelope, the
floor is missing, OR **jurisdiction attribution is ambiguous** (§4.1).

### 4.1 Jurisdiction attribution is an INPUT (normative)

Jurisdiction attribution is an explicit, provenance-carrying **input** to the resolver — never
an inference the resolver makes. The caller MUST supply the set of applicable jurisdictions
together with an attribution source (e.g. `operator_declared`, `participant_geo`,
`network_transit`, `unknown`); the decision receipt MUST record both. Attribution is
**ambiguous** when the caller supplies no jurisdictions, marks the source `unknown`, or supplies
claims it flags as conflicting. Ambiguous attribution MUST resolve fail-closed: `verdict =
block` — or, at an infallible interface, `aggressive` over the caller's declared universe with
the fallen-back flag set (§5). Genuinely contested attribution (VoIP transit states, a remote
employee's location) is acknowledged as unsolved; RLPS standardizes the *selection* given the
input, not the attribution itself.

### 4.2 Cross-jurisdiction merge + conflict rule (normative)

On a session spanning jurisdictions `J1..Jn`, a resolver MUST resolve the query independently
against each applicable jurisdiction's pack chain, then merge **most-restrictively**:

1. The merged `required` set is the **union** of the per-jurisdiction required sets.
2. The merged verdict is `block` if **any** per-jurisdiction resolution blocks.
3. **Deontic conflict:** if a control is required by one applicable jurisdiction and marked
   `prohibition` by another, the query is not simultaneously satisfiable — the verdict MUST be
   `block`, and the receipt MUST record the conflicting control key and both jurisdictions. A
   resolver MUST NOT resolve such a conflict by silently dropping either side.
4. The merge operates within one domain only; §3's per-domain independence still holds.

## 5. Fail-closed (normative)

Absence, ambiguity, or error MUST resolve to the most-restrictive outcome. A resolver MUST NOT
silently fall back to a looser previously-configured state. A missing/malformed pack MUST degrade
to `aggressive` over the caller's declared universe and MUST flag the decision as fallen-back.

## 6. Provenance & trust-root (normative)

- The pack `[meta.legal_review]` MUST carry `status`; for any pack claiming a real jurisdiction it
  MUST carry `reviewing_attorney_of_record`, `jurisdiction`, and a bar/credential identifier, and
  SHOULD be Ed25519-signed over the pack.
- A resolver MUST treat an unattested or self-attested pack's decisions as **advisory-only**: the
  verdict MUST carry an `unverified_provenance` (or `fell_back`) flag that propagates into the
  decision receipt.
- A registry MUST publish a reviewer-key directory with `jurisdiction` + `credential_type` fields
  and a revocation mechanism. The spec mandates the provenance *envelope*, not who is a valid
  signer (code-signing trust model).
- **Directory schema:** the normative reviewer-key directory schema is
  `/registry/reviewer-key.schema.json`. Each entry MUST carry `key_id`, `public_key_ed25519`,
  `reviewer_identity`, `jurisdiction`, `credential_type`, `credential_id`, and `valid_from`;
  MAY carry `valid_until`. The directory document MUST itself be signed by the steward key and
  SHOULD be content-addressed. A directory entry proves *who* may attest — it is NOT a warranty
  of review quality (see `/docs/not-legal-advice.md`).
- **Revocation semantics:** an entry is revoked by adding a `revocation` block (`revoked_at`,
  `reason`). From `revoked_at` forward a resolver MUST treat packs signed by that key as
  unattested (advisory-only, `unverified_provenance` taint). Decisions issued before
  `revoked_at` are not retroactively invalidated, but a receipt verifier SHOULD surface that
  the signing key has since been revoked. Key expiry (`valid_until` passed) MUST be handled
  identically to revocation for new decisions.

## 7. Temporal validity (normative split)

- **Text-in-effect** (solved): `[meta] effective_from` / `effective_until` + content-addressed
  supersession.
- **Interpretation-current** (unsolved — MUST NOT be claimed solved): a pack claiming a real
  jurisdiction MUST carry `last_reviewed_against_guidance` (other packs SHOULD); a resolver
  SHOULD surface staleness rather than silently trust an in-window pack. When the field is
  present the decision receipt MUST carry it; a resolver MAY accept a caller-supplied staleness
  threshold, and packs older than the threshold MUST be flagged in provenance (staleness alone
  does not force `block` — it is surfaced, not adjudicated). Interpretive drift (case-law, DPA
  guidance) is out of scope for v0.1 beyond surfacing it.

## 8. Interoperability (normative mappings)

- **ISO/IEC TS 27560 + W3C DPV** — the decision receipt SHOULD serialize natively as a 27560 +
  DPV Consent Record; a flat Kantara CR v1.1 shim MAY be emitted. RLPS mints recording controls in
  an **RLPS-owned namespace** that MUST degrade gracefully if DPV does not adopt them (v0.1
  namespace: `https://rlps.squillo.com/ns#`; term registry `/docs/namespace.md`; reference
  serializer: the resolver's `receipt` module; worked AI-Act §50 example:
  `/docs/examples/receipt-ai-act-50.json`).
- **NIST OSCAL** — the pack structure mirrors OSCAL Catalog→Profile→Baseline; the Control Catalog
  MAY be expressed as an OSCAL catalog.
- **EU AI Act Art. 50** — `ai_disclosure` is an `obligation` control for AI-notetaker sessions in
  EU profiles; the receipt MUST log disclosure timestamp + method.
- **OPA/Rego** — the standardized artifact is the resolver JSON input/output contract, NOT Rego.

## 9. Conformance levels

1. **Minimal-viable** — parse packs, resolve `aggressive`, fail closed.
2. **Configured** — full RFC-4647 negotiation + delta-merge + all three postures + most-restrictive.
3. **Comprehensive-with-provenance** — + trust-root taint + 27560/DPV receipt emission.

A resolver claiming a level MUST pass the corresponding `/conformance` suite.

## 10. Governance

Reference-implementation-first. Ownership is **federated-with-attestation**: anyone MAY publish a
pack, but pack metadata MUST carry the legal-review attestation envelope; a neutral steward
maintains the canonical Control Catalog + the reviewer-key directory. Target neutral home: a W3C
Community Group (DPV/GPC adjacency), OASIS as the alternative.

## 11. Related work (honest positioning)

RLPS is adjacent to, and interoperates with: NIST OSCAL, W3C DPV, ISO/IEC TS 27560, Kantara CR,
ISO/IEC 29184, IEEE 7012, GPC, EU AI Act §50, OPA/Cedar/XACML, LegalRuleML/Akoma Ntoso, and the
2025 "Policy Cards" proposal (arXiv 2510.24383). None of these makes the **control-prescription
layer** practitioner-diffable, jurisdiction-negotiated, and fail-closed in one portable format —
that is RLPS's contribution.
