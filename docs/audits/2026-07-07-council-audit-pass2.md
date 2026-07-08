# Council audit — pass 2 (independent re-run) — 2026-07-07 (repo @ `8f9789d`)

**Why a second pass:** pass 1 (`docs/audits/2026-07-06-council-audit.md`) ran mostly chair-inline
after a rate-limit outage killed its agent fleet. Pass 2 is a **blind independent re-run**: all six
auditors and verifiers were forbidden from reading `docs/audits/`, so nothing anchored on the prior
report; the chair diffs the two only at the end (this document's cross-validation section). Pass 2
got further — all six auditors completed (61 raw findings; 18 adversarially verified before the
limit again killed the verifier fleet + chair). The chair (me) then re-verified the material
net-new findings by hand with live commands. **NOT LEGAL ADVICE** — engineering + process audit.

## Council verdict

**Confirmed not counsel-ready — and pass 2 is materially harsher than pass 1.** The blocker from
pass 1 (minimal + missing floor → fail-open) was independently re-found, and pass 2 escalated two
pass-1 majors to **blocker** (the stale counsel brief and the disclaimer-less packs both violate
stated governing constraints, not just polish). More importantly, pass 2 caught **a cluster of
integrity defects in the sign/publish/receipt surfaces that pass 1 missed entirely** — `publish`
embeds signatures it never cryptographically verifies, `status="approved"` clears the advisory
taint with zero crypto, and the flagship interop receipt makes false conformance-lineage claims and
fabricates a Kantara `consentType: EXPLICIT`. Fix the blockers + the sign/publish integrity cluster
before counsel; the receipt-fidelity cluster before public.

---

## NET-NEW confirmed findings (pass 2 caught, pass 1 missed) — the headline

### NN1 [major] `status = "approved"` clears the advisory taint with no cryptographic check
The taint is `advisory_only = fell_back || status != "approved"` (`receipt.rs`), and the resolver
deserializes only `meta.legal_review.status` — never the attorney envelope, a signature, or the
reviewer-key directory. Writing the literal string `status = "approved"` **is** self-attestation
and silently upgrades a decision to trusted. `GOVERNANCE.md` promises the taint "stops your pack
being silently trusted"; today one line of TOML defeats it. The level-3 `approved_pack` vector even
pins `advisory_only=false` for a pack whose only credential is the word "approved".
*Verified: an `approved`-status pack with no envelope lints clean and produces an untainted receipt.*
**Fix:** the taint must require verified provenance (signature by a directory-listed, non-revoked
key), not a self-declared string — or the spec/GOVERNANCE claim must be softened to say so.

### NN2 [major] `publish` embeds signatures it never verifies (demonstrated)
`publish` compares only `sig.sha256` to the pack hash and embeds the whole sig block; it never calls
the working `verify_file`. *Reproduced: a `.sig` with a correct sha256 but garbage Ed25519 fields
(`AAAA…`/`BBBB…`) — `r14n verify` rejects it ("bad signature"), but `r14n publish` writes it into
the index as an apparently-attested entry with exit 0.* The index is the artifact receipts cite.
**Fix:** `publish` must call `verify_file` and refuse on failure; add a test.

### NN3 [major] index version-chain has zero read integrity (demonstrated)
`publish` computes the next version as `max(existing)+1`, coercing a missing/wrong-type version to
`0` (`as_u64().unwrap_or(0)`), takes `>=` over unsorted entries, and enforces no `(id,version)`
uniqueness (nor does `pack-index.schema.json`). *Demonstrated: a hand-edited entry with `version:"3"`
(string) is read as 0, so the next publish mints `v1` for an id already advertising v3 and sets
`supersedes` to the wrong entry's sha — a self-contradictory chain, exit 0.* A forged high-version
entry becomes the parent of the next legitimate publish.
**Fix:** parse versions strictly (error on wrong shape), enforce uniqueness + monotonicity, and add
a schema `uniqueItems`-style constraint.

### NN4 [major] The minted RLPS terms expand as string literals, not IRIs (corrects pass-1 N2)
Pass 1 reported "JSON-LD expansion is clean." That was only true for `@type` positions. *Verified
with pyld:* the receipt's `operations` values (`"rlps:AudioRecording"`, `"rlps:VoiceRecording"`) and
`event_type` values expand to `{"@value": "rlps:AudioRecording"}` — **plain literals containing a
colon**, not `https://rlps.squillo.com/ns#AudioRecording` — because the `@context` coerces no term
with `@type: @id`. The bare `"id"` on `pii_principal`/`pii_controller` likewise expands to a
literal `rlps:id` property on a blank node, not `@id` node identity. The RLPS-owned namespace's
entire purpose (linkable recording terms) is defeated in the flagship example.
**Fix:** coerce the term-bearing keys (`operations`, `event_type`, `@type`-position terms) with
`@type: @id` in the `@context`, and map `"id"` to `@id`; re-expand the example as a test.

### NN5 [major] "follows the dpv-27560 serialization profile" is a false lineage claim
`receipt.rs` module doc claims the receipt "follows the public DPV 'dpv-27560' serialization
profile." *Auditor WebFetch of the DPVCG 27560 guide:* that profile serializes
`@type: dpv:ConsentRecord` with `dct:conformsTo`, `dpv:hasDataSubject`, `dpv:hasDataController`,
`dpv:hasProcess` — the RLPS receipt uses **none** of them (flat `record`/`pii_principal`/… sections
under `@vocab`). The section *structure* is a defensible approximation of TS 27560; the "dpv-27560
profile" lineage is not.
**Fix:** reword to "structured after TS 27560's abstract record layout; NOT the DPVCG dpv-27560
JSON-LD profile" (or actually adopt the profile properties). Highest-value interop claim must be true.

### NN6 [major] The Kantara shim fabricates `consentType: "EXPLICIT"` + `thirdPartyDisclosure: false`
`kantara_cr_v1_1()` hardcodes `consentType: "EXPLICIT"` and `thirdPartyDisclosure: false` for
**every** decision — including advisory-only and fallen-back ones — though the resolver only computes
which controls are *required*, never that consent was *obtained*. The shim exists for "consumers we
don't control," who read the standard fields, not the `rlps` extension block where the taint lives.
The shipped example emits `consentType: EXPLICIT` for an `advisory_only=true`, null-review decision.
**Fix:** don't assert consent semantics the resolver can't know — drop these fields or derive them
honestly (e.g. `consentType` absent / `"NA"`); surface the taint in a standard field.

### NN7 [major] `legal_basis_hint: "eu_ai_act_article_50"` is stamped regardless of jurisdiction
`dpv_27560()` unconditionally stamps that named statute whenever `ai_disclosure` is present — even
for US-only or empty jurisdictions. Naming a statute as a disclosure's legal basis is the closest
thing in the repo to a machine-generated legal conclusion — exactly the UPL surface `counsel-brief.md`
§3 frets about — yet the brief never mentions receipts or this field.
**Fix:** gate the hint on EU jurisdiction (or drop it); add receipts + this field to the counsel brief.

### NN8 [major] Conformance runner writes vector `packs` at unchecked paths (traversal)
`run_suite` does `tmp.path().join(rel)` + `fs::write` for every key in a vector's `packs` object. A
`../`-prefixed key escapes the tempdir; an absolute key **replaces** the base — so a hostile or
sloppy vector file writes arbitrary files anywhere writable. All 22 shipped vectors are safe today,
but the kit is explicitly what third parties run against community/vendor vector files.
**Fix:** reject any pack key that is absolute or contains `..`; add a hostile-vector test.

### NN9 [major] Receipt determinism breaks under Cargo feature unification
The module doc sells byte-determinism (for content-addressing/signing) on serde_json being
BTreeMap-backed. That holds only when `serde_json/preserve_order` is **off**; feature unification
means any crate anywhere in a consumer's graph enabling it flips r14n's receipts to insertion order.
The shipped determinism test can't catch it (stable within one build). Since this crate's whole
selling point is signable receipts, a silent order flip breaks signature portability across builds.
**Fix:** don't rely on Map ordering — serialize through an explicitly sorted structure (or a
canonical-JSON/JCS pass), and state the canonical form normatively in spec §8.

### NN10 [major] Control-key namespacing MUST is violated by every artifact
Spec §2.1: "A control key MUST be namespaced `domain.control`." *Verified:* the catalog declares
`[control.attestation]`, both packs list `["attestation", "aph_mandate"]`, and every test, vector,
and receipt uses bare keys. Either the wording is wrong (it means "scoped by the domain pack") or
every artifact is non-conformant. An SDO reviewer will read it literally.
**Fix:** reword §2.1 to match the on-disk reality (domain from the pack/catalog, bare key on the
line) — or namespace the keys. Also fix the adjacent facet-name drift (spec `legal_basis` vs catalog
`legal_basis_hint`).

### Net-new MINOR / NIT
- **NN11 [minor] Fixture epoch/ISO timestamps disagree by ~4 days** — *verified:* `receipt.rs` tests
  pair `2026-07-06T12:00:00Z` with `1783685600` (= 2026-07-10T12:13:20Z); four level-3 vectors pair
  `2026-07-06T00:00:00Z` with `1783641600` (= 2026-07-10; correct `1783296000`). `dpv_27560` emits
  only the ISO, Kantara only the epoch → the two receipts for one decision date 4 days apart. The
  AI-Act example is correct, showing intent. **Fix:** correct the fixtures; add an epoch==ISO assert.
- **NN12 [minor] Flagship example: `VoiceRecording` but `sensitive:false`/`spiCat:[]`** — contradicts
  `namespace.md`'s own "VoiceRecording = GDPR Art. 9, SEPARATE from audio" guidance. **Fix:** set
  `sensitive:true` + a spiCat in the example, or drop VoiceRecording from it.
- **NN13 [minor] CLI arg-parsing traps** — *verified pattern:* a flag missing its value swallows the
  next flag as the value; a repeated flag leaves the second occurrence to be misread as a positional;
  `sign`/`verify`/`publish` silently drop trailing args. **Fix:** error on unconsumed/duplicate args.
- **NN14 [minor] LICENSE names the pre-rename project + coverage gaps** — *verified:* `LICENSE:1`
  reads "Squillo and the **regloc** contributors"; the CC-BY enumeration omits `/conformance` and
  `/registry` (normative artifacts) — they fall under no stated license; full license texts unvendored.
- **NN15 [minor] Profiles named after postures** — the only two profile dirs are `aggressive/` and
  `minimal/`; the resolver's doc examples (`aggressive`, `us_all_party`) never show a §2.4
  grammar-conformant `<regime>/<jurisdiction>` tag, inviting the aggressive-as-a-tier misreading the
  project forbids. **Fix:** ship one grammar-shaped example profile.
- **NN16 [minor] `*.seed` is not gitignored** — *verified:* `.gitignore` has no `seed` entry; private
  signing seeds written next to packs can be committed. **Fix:** add `*.seed` (pairs with the 0644 perms
  finding). **Nit:** `tools/README.md` still says "15 tests" (actual 22).

---

## Convergent with pass 1 (independently re-found — strengthens both)

Same defect, found blind by both passes: **minimal-missing-floor fail-open** (BLOCKER, both);
**verdict unrepresentable / resolver-cannot-emit-block** (pass-1 M5); **runner drops unknown `expect`
keys + unreachable revocation-vector assertion** (pass-1 M6); **temporal envelope is not a capability
and never deserialized** (pass-1 M4); **§4.1 attribution + IEEE-7012 escalation uncovered** (pass-1
M7); **`last_reviewed_against_guidance` receipt MUST unimplemented** (pass-1 M8); **template TODO
dates pass validate + publish** (pass-1 N5 — pass 2 rates it major); **`main()` dispatch untested**
(pass-1 N7); **publish stores absolute path verbatim** (pass-1 N6); **signature doesn't bind profile**
(pass-1 M1); **keygen seed 0644** (pass-1 M2); **steward JCS/directory signing unimplemented**
(pass-1 M9); **packs lack disclaimer** (pass-1 M10 — pass 2 rates it BLOCKER); **counsel brief stale
scope + counts** (pass-1 M3 — pass 2 rates it BLOCKER); **Kantara `piiControllers` under-populated**
(pass-1 N1); **`not_required` advisory taint on own baselines unexplained** (pass-1 N9); **RFC-4647
cited normatively for non-language tags** (pass-1 N3).

**Severity escalations pass 2 asserts (adopt these):** the stale **counsel brief** and the
**disclaimer-less packs** are BLOCKERs, not majors — both violate a stated governing constraint the
counsel handoff is predicated on. The **template-TODO-dates** path is major (it flows unblocked into
the content-addressed index), not minor.

## Corrections / disputes vs pass 1

- **Pass-1 N2 was incomplete and is superseded by NN4.** Pass 1 concluded "pyld expansion is clean";
  that was verified only against `@type` positions. Value-position minted terms expand as literals —
  the more important half. Treat NN4 as the correct finding.
- No pass-1 finding was contradicted on the merits by pass 2; the sign/publish integrity cluster
  (NN1–NN3) sits *behind* pass-1 M1/M2 (pass 1 found the signature doesn't bind the profile and the
  seed is world-readable; pass 2 found the publish path doesn't verify signatures at all and the
  index has no read integrity — deeper failures in the same subsystem).

## Missing items (deduped; supersedes pass 1's list where overlapping)

**before_counsel:** refresh `counsel-brief.md` scope+counts (blocker overlap) and ADD a section on
the receipt serializations (27560/DPV + Kantara) and the `legal_basis_hint` UPL surface; a
threat-model / "what a signature binds, what `verify` checks, what it does NOT" document;
`CONTRIBUTING.md` with the pack-contribution CLA/DCO + gating stance.

**before_public:** CI gates workflow (both cargo suites + `r14n validate` on shipped packs + schema
validation of every fixture) — nothing runs automatically today; **executable schema conformance**
(packs↔pack.schema, publish output↔pack-index.schema, level-3 directory fixture↔reviewer-key.schema
— all validated only ad-hoc, externally); a **conformance vector-format schema** + strict runner
that rejects unknown capabilities/expect keys; **CLI integration tests** (`tools/tests/cli.rs`
driving the binary through extract→validate→sign→verify→publish incl. the publish lint gate);
**reviewer-key directory tooling** (build + JCS-canonicalize + steward-sign + `verify --directory`
+ revocation — without it the level-3 revocation vector has no implementer); a **receipt JSON Schema**
+ RLPS-field→TS-27560 clause mapping table + a byte-exact golden-receipt vector; a **doc-example
regeneration guard** (the checked-in receipt drifts silently from the generator); **SECURITY.md**;
**vendored license texts** + per-file SPDX + a `NOTICE`; **key-management guidance** (0600, rotation,
offline steward key); a normative spec definition of the `inherits` wire field + delta-merge; a
**machine-readable RLPS vocabulary artifact** (published `@context` + SKOS/RDF term defs) so the
namespace actually resolves.

**later:** cargo-fuzz targets for the untrusted TOML/JSON parse surfaces; `cargo llvm-cov` in CI with
a baseline; MSRV / `rust-version` + semver policy for the resolver crate; a trademark/conformance-
claim usage policy for the RLPS and r14n names; conformance-suite versioning policy.

---

*Process note: pass 2 ran 67 agents / 2.14M tokens; all six auditors completed, then a session rate
limit killed the 42-agent verifier fleet + the chair. 18 findings carry agent-verification; the
chair re-verified every material net-new finding by hand (namespacing, epoch/ISO, forged-sig publish,
JSON-LD literal expansion, LICENSE name, `*.seed` gitignore) with live commands recorded above.
Findings whose agent-verifier died and which the chair did not personally re-run are still evidence-
cited by their auditor and should be treated as high-confidence-unverified, not refuted.*

---

## Remediation status (2026-07-07, commits `5899e01`…`5d728af`)

Fixed across six batches (each gated + committed; resolver 27+3-conformance tests, tools 27+4):

- **Blockers — all fixed.** B1 minimal-missing/empty-floor now fails closed (`5899e01`, lib tests +
  level-1 vector); counsel brief refreshed to the real tree + counts + a receipts §8 (`e769cdd`);
  both packs carry the disclaimer (`5899e01`).
- **Sign/publish integrity (NN2/NN3, N6) — fixed** (`38c559d`): `publish` cryptographically verifies
  embedded signatures (forged-sig test), refuses corrupt/duplicate versions, stores relative paths;
  conformance runner rejects `..`/absolute vector keys (NN8).
- **Receipt fidelity (NN1/NN4/NN5/NN6/NN7/NN9, NN11/NN12) — fixed** (`d9c52f7`): advisory taint no
  longer cleared by a self-declared status (`PROVENANCE_VERIFIED=false`); `@context` coerces minted
  terms to IRIs (pyld-verified); dropped the false dpv-27560 lineage, fabricated Kantara
  `consentType`, and hardcoded `legal_basis_hint`; explicit sort canonicalizer; fixture epochs
  corrected; example marks VoiceRecording sensitive.
- **Spec/CLI/linter (NN10, N3, NN13, N5) — fixed** (`1f890eb`): §2.1 namespacing reworded to the
  on-disk reality; RFC-4647 softened to "-style"; CLI errors on dangling/duplicate/leftover args;
  linter enforces ISO date shape (warn-draft / error-non-draft); `tools/tests/cli.rs` covers the
  `main()` dispatch layer.
- **Structural (M5/M8/M6) — fixed** (`80939d7`): `ControlDecision.verdict` (receipts read it, not a
  literal); `last_reviewed_against_guidance` deserialized + carried into the receipt (spec §7 MUST);
  conformance runner is closed-set (panics on unknown `expect` keys, requires `receipt_context`).
- **Temporal envelope (M4) — fixed** (`1e0c5a3`): `PackMeta` deserializes the envelope; a supplied
  `as_of` outside it fails closed; linter requires `effective_from` on approved packs; new
  `temporal_envelope` capability + level-2 vectors. *(Breaking `RegulatoryQuery` change — flagged
  for the twin in maintainer-notes sync note 7g.)*
- **Coverage + infra (M7, missing-items) — fixed** (`5d728af`): authored-ahead `ieee7012_escalation`
  + `ambiguous_attribution` vectors; capability meta-validation test; `scripts/check-schemas.py`
  (packs/fixtures ↔ schemas); `.github/workflows/gates.yml`; `SECURITY.md`.

**Previously-deferred, now FIXED (2026-07-08, `95d5a47`…`be3ca99`):**

- **M1 (signature binds bytes, not profile) — FIXED** (`95d5a47`): `sign`/`verify` now sign a
  domain-separated `rlps-sig/1` payload binding `<profile>/<domain>` (derived from the pack path) +
  sha256; `verify` re-derives the id from the pack's current location and rejects on mismatch. Test:
  a signed pack copied byte-identically (sig too) into another profile dir fails with "identity
  mismatch." Sig schema + pack-index schema + spec §6 updated.
- **M9 (reviewer-key directory verification) — FIXED** (`3f1b245`): new `tools/directory.rs` +
  `r14n verify --directory <d> [--as-of] [--jurisdiction]` — a listed, non-revoked, non-expired,
  jurisdiction-matched signer is `Trusted` (exit 0, may escape advisory-only); otherwise exit 3 and
  the decision stays advisory-only. Revocation is forward-dated, expiry treated as revocation
  (spec §6). 4 unit tests + an end-to-end CLI test. The generic resolver stays directory-agnostic.
- **NN15 (grammar-conformant profile example) — FIXED** (`be3ca99`):
  `packs/example/region/recording_consent.r14n.toml` demonstrates the `<regime>/<jurisdiction>`
  profile grammar as a deliberately-fictional, heavily-marked non-jurisdiction draft; a resolver test
  proves the nested profile dir loads.

**Still deferred (later priority):** cargo-fuzz, `llvm-cov` baseline, `CONTRIBUTING.md`/CLA, vendored
license texts, a published machine-readable `@context`/SKOS vocabulary, conformance-suite versioning;
and the resolver-level `trust_root_revocation` conformance capability remains authored-ahead (the M9
directory check lives at the tools layer, so `PROVENANCE_VERIFIED` stays `false` in the generic
resolver by design).
