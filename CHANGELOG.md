# Changelog

All notable changes to RLPS — the specification, the Control Catalog, the conformance suite, the
schemas, and the reference implementation — are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project's version rules are in
`GOVERNANCE.md` §Change process and `conformance/VERSIONING.md`. **NOT LEGAL ADVICE** — see
`docs/not-legal-advice.md`.

While RLPS is pre-1.0 with no external adopters, corrections land in place in the current draft
and are recorded as dated revision entries under the version heading below (plus a Revision
History banner in the spec for normative changes), rather than forking a version per fix.

## [0.1.0-draft]

### Changed (revision 2026-08-23 — public-repo preparation)

- **The vocabulary namespace moved** from its previous host to `https://r14n.squillo.com/ns#` —
  the `RLPS_NS` constant in `resolver/src/receipt.rs`, the JSON-LD `@context` (`ns/context.jsonld`),
  the SKOS/RDFS vocabulary (`ns/rlps.ttl`), and the worked receipt example. The URI is an
  identifier first (receipts are valid whether or not it dereferences), but receipt consumers
  comparing term IRIs byte-for-byte will see the new host in receipts minted after this revision.
  No shipped conformance vector asserts the namespace host, so no conformance claim moves. The
  spec carries the matching Revision History banner.
- **The change process is now issue-form-based**: three GitHub issue forms (RFC / Erratum /
  Conformance disagreement) carry proposals, `.github/CODEOWNERS` codifies review routing with
  the normative surface listed separately, and the PR template restates the approval bar at the
  moment it applies. `GOVERNANCE.md` §Change process and `CONTRIBUTING.md` were wired to the
  forms; this `CHANGELOG.md` and the spec's Revision History banner are the proposal→landed
  record.
- **Licensing for the N Lang Snapp tree clarified**: `r14n Spec/`, `baselines/`, and `snapp/`
  are Apache-2.0 (Snapp sources + emitted bundle) — N Lang itself remains proprietary to
  Squillo, Inc.; `/ns` was added explicitly to the CC-BY-4.0 bucket. `LICENSE` now enumerates
  all four buckets.
- **Community layer added**: `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, contact
  legal@squillo.com), a supported-versions table in `SECURITY.md`, and CI gained a sanitization
  gate, `permissions:` hardening, clippy (warnings-as-errors), an MSRV check, and a PR-scoped
  DCO check. The declared MSRV moved from 1.74 to **1.85** — the lockfile is v4 format and a
  locked dependency uses edition2024, so 1.85 is the first toolchain that actually builds the
  tree; the old claim was unverifiable.
- **Internal-reference scrub**: internal design-doc ids, private-crate names, agent-session
  identifiers, and local paths were removed from all published files; the internal maintainer
  handoff brief was withdrawn from the tree. No normative behavior changed.

### Changed (revision 2026-09-03 — co-authorship names Squillo, and only Squillo)

- **Machine-assisted commits carry
  `Co-Authored-By: Squillo Code <320728527+squillo-code@users.noreply.github.com>`.** Squillo's
  own identity belongs on its own work; what never appears is an outside tool or vendor, in a
  trailer or in message prose. The sign-off (DCO) is separate and names the human who certifies
  the contribution.
- **A stale `v0.1.0-draft` tag was why a removed vendor trailer kept surfacing.** The tag still
  pointed at a pre-rewrite commit, which kept that entire orphaned lineage — and its trailers —
  reachable on the forge long after the branch itself was corrected. Rewriting a branch does not
  move a tag; anything still pinning an old commit resurrects everything behind it. The tag now
  points into current history, and verification runs over every ref rather than the branch tip.
- **CI enforces the rule over full history.** The commit-message scan rejects a co-author line
  naming anyone other than Squillo Code, any other attribution-trailer form, and any outside
  vendor named anywhere in a message. `CONTRIBUTING.md` and `AGENTS.md` state it.

### Changed (revision 2026-08-24b — the APH exchange closes; one RFC filed upstream)

- **`docs/aph-dependency-report.md` cites APH's corrected trigger.** APH sharpened its
  pre-production exception in response to our report (`5932f2a`): the test is now whether a wire
  change would *break* a consumer or merely *cost them a documentation edit*, with an enumerated
  list of wire-asserting artifacts. Our report quoted the superseded phrase; it now cites the
  corrected one and checks r14n against that list. Status unchanged — **documentation-only, no
  wire dependency**, recorded on both sides.
- **Filed [squillo/aph#2](https://github.com/squillo/aph/issues/2)** — APH's §6.3.3.4 names three
  status outcomes, but *two* of them pass and only one is named: an envelope with no status claim
  and an envelope whose status was resolved and affirmatively clear both verify, and an audit
  record cannot tell them apart. The RFC asks only that the passing dispositions be named and
  that a recorded live result carry its instant and the list consulted; it explicitly does **not**
  ask for the evidence-record design, which stays deferred. Rationale for filing now rather than
  when an enforcement gate exists: `aph-integration.md` already had to invent enforcement-side
  vocabulary locally, which is the drift the naming prevents.
- Recorded that this project's prose is CC-BY-4.0, so APH may cite it with attribution.

### Changed (revision 2026-08-24 — `aph_mandate` splits in two; the APH contract is written down)

- **`aph_mandate` is DEPRECATED and superseded by two controls**:
  `aph_mandate_principal_signed` (the human's own key signed the act — consent,
  cryptographically) and `aph_mandate_notary_attested` (a notary asserts it — provenance, not
  consent). APH distinguishes these trust models and its security considerations §2.6 forbids
  collapsing them into one badge; **in RLPS a control key is that badge**, so one key satisfiable
  by either would record that *something* authorized an act while concealing whether a human ever
  signed anything — and a control key is long-lived audit evidence. The deprecated key is
  retained, not removed, because shipped receipts reference it; it MUST NOT be used in new packs.
  Per `GOVERNANCE.md` §Change process this is a deprecation, so `r14n merge` flags the delta for
  legal re-review on any pack that carries the old key.
- **Squillo's `aggressive` baseline and the fictional `example/region` pack now require
  `aph_mandate_principal_signed`** on every rail that previously carried `aph_mandate` — the
  aggressive posture takes the strong mode deliberately, and it can only ever over-restrict.
- **`docs/aph-integration.md`** — the contract: which mode satisfies which control, evaluation
  at decision time with a live revocation-status check (a revoked mandate and an unreachable
  status surface are both refusals, matching RLPS fail-closed from the other side), the APH error
  codes an enforcement gate must preserve rather than flatten to a boolean, body-digest binding,
  and separate protocol/crate version pinning. It **cites** APH rather than restating them, so
  the two documents cannot silently drift.
- **The reference resolver deliberately links no APH code.** RLPS prescribes which controls are
  required; it never adjudicates whether one is satisfied — that is the consuming enforcement
  gate's job. Recorded as a considered divergence from APH's integration guidance.
- **`docs/aph-dependency-report.md`** — a dated, standing answer to whether r14n depends on the
  APH wire format (today: no — documentation-only), so APH's pre-production exception has a
  visible expiry trigger instead of an assumption.
- Conformance vectors are **deliberately unchanged**: they use `aph_mandate` as an opaque
  set-algebra token, and renaming it would bump the suite version for no semantic gain
  (`conformance/VERSIONING.md`).

### Added (revision 2026-08-23g — the agent pack: one knowledge source, both ecosystems)

- **`skills/spec/SKILL.md`** — RLPS as a loadable skill in the open Agent Skills format: the
  seven pieces, pack shape, the resolution algorithm and the fail-closed rule, the
  advisory-only taint, the CLI, conformance levels with the governed claim wording, and the
  hard gates. **`AGENTS.md`** is the agents.md-convention entry point (Codex and others) —
  orientation, the exact commands CI runs, and its invariants — and it points *into* the
  skill rather than duplicating it, so there is one source to keep true.
- **`.claude-plugin/`** (plugin + marketplace manifests) and **`commands/`** — `/validate`
  (lint a pack and diagnose against the spec), `/resolve` (resolve a query and explain the
  decision, leading with any fallback), `/conformance` (run the vectors and report level
  claims in the wording `TRADEMARKS.md` requires). Each command carries the NOT-LEGAL-ADVICE
  framing and stops rather than authoring a counsel-gated jurisdiction pack.

### Added (revision 2026-08-23f — the JS/TS and Python bindings, generated from the Rust core)

- **`resolver/src/wire.rs`** — the JSON boundary every binding is generated over: JSON text in,
  JSON text out, packs as a `"<profile>/<domain>.r14n.toml" → TOML text` object. Defined and
  tested once, in the core, for every host. A missing or malformed *pack* still resolves
  fail-closed (spec §4/§5); an error from this layer always means the *caller's* JSON was
  unusable, so a consumer can tell a bug from a policy fallback.
- **`resolver`: `InMemoryRegulatoryPolicyAdapter`** — the pack transport for hosts with no
  filesystem (wasm, embedded callers). The whole decision pipeline moved into a shared
  `decide_from_pack_text`, so the filesystem and in-memory adapters cannot drift; an
  fs-vs-memory equivalence test pins it. Additive — no existing behavior changed.
- **`bindings/wasm`** → **`@squillo/r14n`** on npm: a wasm-bindgen shim, built by
  `bindings/npm/build.mjs`, with zero runtime dependencies. **`bindings/python`** →
  **`r14n`** on PyPI: a PyO3 abi3 shim (one wheel serves CPython 3.9+), built by maturin.
  Both are thin — FFI attribute and error mapping only — because the boundary lives in the core.
- **CI**: `bindings-js` builds the npm package and smoke-tests it under Node 20 and 22;
  `bindings-python` builds the wheel and runs its tests against the installed artifact. Both
  gate on what a consumer actually installs, not on source. Clippy now covers both shims.
  `CONTRIBUTING.md` gained the thin-shim rule that keeps logic out of bindings.

### Added (revision 2026-08-23e — the r14n.squillo.com site lives in-repo)

- `site/` — the Cloudflare Worker behind `r14n.squillo.com`: a single-file splash page at `/`
  (NOT-LEGAL-ADVICE banner, before/after pack contrast, packs/resolver/receipts overview, the
  measured-problem numbers, standards positioning) and the existing `/ns` vocabulary routes. The
  vocabulary is imported directly from `ns/` at bundle time — no copies, byte-identical by
  construction. Apache-2.0, added to the LICENSE code bucket.

### Added (revision 2026-08-23d — bindings policy: generated from Rust, or wasm/WASI)

- `CONTRIBUTING.md` engineering rule: any TS/JS or Python package shipped from this repo must be
  generated from the Rust reference resolver (wasm-bindgen / PyO3 abi3) or run the compiled
  wasm/WASI module — hand-written ports are not accepted in-repo. Independent resolvers remain
  welcome in their own repositories via the conformance suite (GOVERNANCE.md Stage 2); a binding
  is not an independent implementation.

### Changed (revision 2026-08-23c — the repository is public, so counsel can review in the open)

- The steward re-scoped GOVERNANCE.md hard gate 1: the repository was made public to enable
  outside counsel review, rather than waiting for that review to complete in private. What the
  gate protects is unchanged — jurisdiction packs, hosted registries, and any assertion of legal
  standing remain counsel-gated, and nothing in the repository claims legal clearance.

### Fixed (revision 2026-08-23b — the vocabulary Turtle was unparseable)

- `ns/rlps.ttl` used `rdf:Property` without declaring the `rdf:` prefix, so every conformant
  Turtle parser rejected the whole file. Added the missing
  `@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>` declaration; the file now parses
  (38 triples). Caught while standing up the `r14n.squillo.com/ns` host, before the artifact was
  ever served.

### Added (initial draft — 2026-07-06 onward)

- `spec/RLPS-v0.1.md` (data model, posture, fail-closed resolution, provenance/trust-root,
  temporal validity, interop, conformance levels), the recording-consent Control Catalog, pack +
  receipt JSON Schemas, the three-level conformance suite with pinned run/skip counts
  (`conformance/VERSIONING.md`), the Rust reference resolver + `r14n` pack-lifecycle CLI
  (extract / merge / validate / keygen / sign / verify / publish), registry schemas, the
  `aggressive`/`minimal` posture baselines, the decision-receipt module (ISO/IEC TS 27560 + W3C
  DPV JSON-LD + Kantara CR v1.1 shim), the JSON-LD/SKOS vocabulary under `ns/`, and the RLPS
  spec expressed as N Lang Snapp sources (`r14n Spec/1.0.0/`) with the emitted bundle under
  `snapp/`. Hardening history through the two council audits is recorded in `docs/audits/`.
