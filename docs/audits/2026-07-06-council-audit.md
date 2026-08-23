# Council audit — 2026-07-06 (repo @ `d766b34`)

**Method:** six-lens adversarial council (spec-conformance · crypto/security · Rust correctness ·
standards-interop fidelity · legal posture · test/conformance coverage), one empirical protocol:
every finding below was verified against files at cited lines or demonstrated with a command run
against the prebuilt CLI / a JSON-LD processor. Gates at audit time: resolver 21/21, tools 22/22,
tree clean. **NOT LEGAL ADVICE** — this is an engineering + process audit.

## Council verdict

**Not ready for counsel handoff as-is — but every gap is cheap and local.** One genuine fail-open
bug in the resolver (B1) directly contradicts the "fail-closed defaults everywhere" representation
the counsel brief makes; the gate document itself is stale (M3); and two demonstrated crypto gaps
(M1, M2) undermine the §6 trust-root story counsel will be asked to opine on. Fix B1 + M1–M3 and
refresh the brief, and the handoff is honest. The deeper theme across lenses: **the spec makes
several MUST claims its own reference artifacts don't yet implement** (temporal envelope, receipt
currency field, verdict) — each needs either implementation or an explicit v0.1 downgrade to
SHOULD, never silence.

## Objections — BLOCKER

### B1. `minimal` + missing floor resolves fail-OPEN (empty required set)
Under `Strictness::Minimal`, an absent `[legally_required]` table yields `.unwrap_or_default()` —
an **empty** control set with `fell_back = false` and a `permit` receipt. Spec §4 says a missing
floor MUST block; §5 says absence resolves most-restrictively. Such a pack parses as valid TOML
(the field is `Option` + `serde(default)`), so the malformed-pack fallback never triggers. The
linter catches it at author time, but the resolver is the runtime guarantee against unlinted packs
— and `docs/counsel-brief.md` §4 tells counsel "fail-closed defaults everywhere," which is
currently false. No test or vector pins this path.
**Evidence:** `resolver/src/lib.rs` Minimal arm (`.unwrap_or_default()`); spec §4/§5; the only
minimal vector (level-2) has a floor. **Fix:** absent floor under Minimal ⇒
`aggressive_over_universe(query, source, true)`; decide empty-floor semantics explicitly; add a
lib test + level-1 vector; mirror to the Squillo twin. *(test-coverage lens; chair-verified.)*

## Objections — MAJOR

### M1. The signature does not bind the pack's PROFILE (demonstrated)
A pack's profile is its **directory name**, which is in neither the signed bytes nor the sig
document. Demonstrated: signed a pack under `one_party/`, copied pack+sig to `all_party/`,
`r14n verify` → "signature + content address OK". A counsel-attested permissive pack can be
redeployed under a stricter profile's name with "valid" attestation.
**Fix:** bind identity — either a required `[meta] profile`/`domain` field validated by the linter
against the path, or an `id` field inside the signed sig-document; verify checks both. *(crypto.)*

### M2. `keygen` writes the private seed world-readable (0644)
Demonstrated: `reviewer.seed` lands `-rw-r--r--`. Reviewer signing keys are the trust root.
**Fix:** create with 0600 (`OpenOptions` + `PermissionsExt`), test pins the mode. *(crypto.)*

### M3. The counsel brief — the gate document — is stale
`docs/counsel-brief.md` §7 lists the repo contents counsel will review: it omits `conformance/`,
`tools/` (the CLI!), `registry/` schemas, `GOVERNANCE.md`, and the receipt module, and claims
"6 unit tests" (actual: 43). A representations document with wrong numbers invites "what else is
stale." Same sweep: `README.md` "18 tests", `tools/README.md` "15 tests". **Fix:** refresh §7 +
the stale counts (or replace counts with the
gate command). *(legal + test-coverage lenses converged.)*

### M4. The effective-date envelope is decorative everywhere
Spec §2.5 MUSTs an envelope; §4 MUSTs blocking outside it; §7 calls text-in-effect "solved". But:
no conformance capability names it, zero vectors exercise it, `PackMeta` never deserializes the
fields (an expired pack governs forever), the linter checks only ordering-when-both-present, and
both shipped packs omit it — with a test pinning that they pass. Meanwhile
`pack-index.schema.json` tells resolvers to pick versions by envelope coverage.
**Fix:** implement (deserialize + block/fallback + `temporal_envelope` capability + vectors +
linter presence rule) or downgrade the §2.5/§4 envelope clauses to SHOULD for v0.1 explicitly.
*(test-coverage.)*

### M5. `verdict ∈ {permit, block}` is unrepresentable, hardcoded, unassertable
`ControlDecision` has no verdict field; `receipt.rs` hardcodes `"verdict": "permit"`; the runner
never reads `expect.verdict`, so the two vectors that assert verdicts (incl. the deontic-conflict
BLOCK) are vacuous the day their capabilities land. Deontic conflict and envelope expiry have no
permit-shaped alternative — they are inexpressible today.
**Fix:** add verdict to `ControlDecision` now (only `Permit` producible), thread through both
serializers, teach the runner to assert it. *(test-coverage.)*

### M6. Conformance runner is open-set — unknown `expect` keys silently assert nothing
The README contract says "assert every key present in expect"; the runner handles exactly three
keys and drops the rest (incl. `verdict`), and evaluates `expect_receipt` only when
`receipt_context` exists — the level-3 revocation vector has `expect_receipt` but no
`receipt_context`, so its only assertion will never run. A typo'd expect key passes forever.
**Fix:** panic on unrecognized expect keys and on `expect_receipt` without `receipt_context`;
give the revocation vector a context. *(test-coverage.)*

### M7. §4.1 ambiguous attribution + IEEE-7012 escalation have no capability, vector, or input
`RegulatoryQuery` has no jurisdictions/attribution fields (they exist only in `ReceiptContext`,
recorded but never resolution-affecting); no `ambiguous_attribution` or `ieee7012_escalation`
capability exists, so a resolver could claim Level 2/3 while silently permitting under ambiguous
attribution — the exact failure §4.1 exists to prevent.
**Fix:** mint both capabilities + authored-ahead vectors (the established pattern). *(test-coverage.)*

### M8. Spec §7's receipt MUST for `last_reviewed_against_guidance` is unimplemented
The linter requires the field on approved packs (so authors will write it) — and every conforming
receipt then violates the MUST because the resolver never deserializes it and neither serializer
emits it. **Fix:** thread pack → `PolicyProvenance` → both receipts + test + level-3 vector; or
downgrade to SHOULD. *(test-coverage.)*

### M9. The steward directory signature is required by schema but unimplementable
`reviewer-key.schema.json` REQUIRES `steward.signature_ed25519` over RFC-8785/JCS canonical JSON.
Zero JCS code exists in the repo; no tool signs or verifies a directory; `r14n verify` never
consults a directory at all (any embedded key verifies). The trust-root story is schema-only.
**Fix:** implement JCS + `r14n sign-directory` / `verify --directory` (see missing items), or
mark the field provisional in the schema description. *(crypto.)*

### M10. Both shipped packs lack the NOT-LEGAL-ADVICE disclaimer
Governing constraint 1 says every artifact. The two `.r14n.toml` packs — the files downstream
authors will copy first — carry SPDX + comments but not the disclaimer. (Code-file gaps in
`lib.rs`/`packtoml.rs`/`conformance.rs` module docs are lesser but worth the same sweep.)
**Fix:** one header line each. *(legal.)*

## Objections — MINOR

- **N1. Kantara shim under-populates `piiControllers`** — v1.1 entries carry `piiController,
  contact, address, email, phone`; the shim emits only the name. Extend `ReceiptContext` with
  controller contact fields. *(interop; verified against spec sources.)*
- **N2. JSON-LD modeling nits** — pyld expansion is clean (10 rlps-ns predicates, DPV terms
  survive at depth, nothing lost), but bare `"id"` expands to `rlps:id` (literal) rather than
  `@id` (node identifier), and `dpv:hasJurisdiction` takes a string literal where DPV models
  jurisdictions as concepts/IRIs. *(interop; demonstrated.)*
- **N3. RFC-4647 cited normatively** (spec §2.4/§4 "MUST … by RFC-4647 lookup") — RFC 4647
  defines matching for BCP-47 *language* tags with `-` subtags; ours are `/` profile tags. An SDO
  reviewer will flag it. Reword to "RFC-4647-**style** progressive truncation (normatively defined
  here)". *(spec.)*
- **N4. Internal Squillo references leaked in the aggressive pack** (an internal design-doc id
  and an internal component name) — scrubbed/genericized before anything leaves the repo. *(legal.)*
- **N5. TODO placeholder dates lint clean and publish** — demonstrated: the extract template's
  `TODO-YYYY-MM-DD` passes `validate` (0 warnings) and `publish` indexes it. Add a date-shape
  rule (warning for drafts, error otherwise); update the template test to expect the warning.
  *(test-coverage; demonstrated.)*
- **N6. `publish` leaks absolute local paths into the index** — `path` stored verbatim;
  demonstrated with a scratchpad absolute path. Store index-relative; constrain in the schema.
  *(test-coverage; demonstrated.)*
- **N7. `main()` dispatch layer has zero coverage** — incl. the publish lint-before-index gate,
  which only exists in `main`; `publish()` the library fn validates nothing. ~5 CLI integration
  tests close it. *(test-coverage.)*
- **N8. CLI silently ignores leftover args** — demonstrated (`extract … utterly-ignored-junk-arg`
  exits 0). Error on unconsumed args. *(rust.)*
- **N9. Baselines are advisory-only by design — undocumented** — `not_required`/
  `requires_signoff` statuses mean every decision from Squillo's own shipped packs carries
  `advisory_only: true`. Correct behavior; surprising to deployers; document it in README/packs.
  *(legal.)*
- **Nits:** stale test counts (folded into M3); `validate` reports "strictness is required" when
  the field is present but non-string.

## Missing items

**before_counsel:** refresh of `docs/counsel-brief.md` (M3) + a counsel-brief question on the
receipt's `legal_basis_hint` field (naming a legal basis in machine output — UPL surface?).

**before_public:**
1. CI gates workflow (both cargo suites + `r14n validate` on shipped packs + schema checks) — all
   43 tests currently run only when remembered.
2. In-repo machine validation of the three JSON schemas against their producers (packs ↔
   pack.schema, publish output ↔ pack-index.schema, the level-3 directory fixture ↔
   reviewer-key.schema) — currently validated only ad-hoc, externally.
3. Conformance-vector meta-validation: a vector-file schema + a test that every capability tag is
   in the README vocabulary (a typo'd tag is skipped forever today).
4. Reviewer-key directory verification tooling (`r14n verify --directory`, revocation + expiry
   semantics) — until then the level-3 revocation vector has no possible implementer (pairs with M9).
5. `SECURITY.md` / disclosure policy (the repo ships trust-root cryptography).
6. One-time confirmation of the three DPV term IRIs against the published DPV 2.x spec (the spec
   site wasn't fetchable during audit; expansion-level usage verified).

**later:** fuzz targets for the TOML/JSON parsing surfaces (turn "malformed input degrades
safely" into a fuzzed property); llvm-cov wired to the gate; a conformance-suite
versioning/evolution policy (what "rlps-conformance/0.1" means across spec versions).

## Refuted / process record

No findings were refuted on the merits. Where independent verification was interrupted, the
chair re-verified every salvaged finding against the cited files and re-ran the affected lenses
inline with empirical demonstrations. Findings the chair could not fully verify externally are
marked as such (DPV term existence → missing item 6).
