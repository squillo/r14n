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

### Fixed (revision 2026-10-31 — three fail-opens in the resolver, found by audit)

All three fell open on input that should have resolved most-restrictive (spec §5). Each was
reproduced through the wasm boundary with a control, and re-run there after the fix.

- **A malformed date in a reviewer-key entry left a revoked key trusted.** Dates were compared as
  raw strings, which is only sound for zero-padded ISO `YYYY-MM-DD`. A `revoked_at` of `"2026-9-01"`,
  `""` or `"soon"`, or a `valid_until` of `"2026-9-1"`, resolved `trusted` with
  `provenance_verified = true` — a malformed revocation record RESTORED trust in a revoked key,
  against §6:188. `key_status` now validates every date (calendar-aware) and returns a new
  `KeyStatus::InvalidDate { field }` (wire tag `invalid_date`), which is never trusted. Introduced
  in the 2026-10-21 trust-root revision below.
- **A malformed effective-window bound kept an expired pack in force.** `effective_until` of
  `"2026-9-1"` or `"never"` was treated as an open window (`fell_back = false`). A malformed bound
  or a malformed `as_of` now fails closed, with `<invalid-effective-envelope:…>` or
  `<invalid-decision-date:…>` in provenance. Pre-existing.
- **Inheritance cycles and missing parents did not fail closed.** §4.3 says a cycle "MUST fail
  closed (§5)". The 2026-10-19 revision did the opposite — it stopped at the cycle and let the
  packs resolved so far govern, with a test asserting that — and a child whose parent did not exist
  resolved with `fell_back = false` and the parent's `[prohibited]` silently dropped. A cycle, an
  unresolved parent and an unreadable pack in the chain now all fail closed, named in provenance
  (`<inherits-cycle:…>`, `<inherits-unresolved:…>`, `<unreadable:…>`). An unreadable pack is
  deliberately NOT treated as absent: truncation would let a less specific pack govern in its
  place. The test that asserted the old behaviour is removed; another test's fixture had omitted
  the parent it declared and passed only because of this bug.

Also: the chain walk — including its cycle guard — was written twice, once per adapter, and both
copies had the same deviation. It is now one function.

The published schemas no longer rely on `"format": "date"`, which the validator does not assert
(measured: `deprecated = "not-a-date"` validated). Eight date fields across `pack.schema.json`,
`catalog.schema.json` and `registry/reviewer-key.schema.json` now carry a `pattern`. This
TIGHTENS published schemas: a pack or directory with a malformed date that used to validate is now
rejected. The pattern checks shape, not the calendar; the resolver checks the calendar.

### Fixed (revision 2026-10-31 — the inheritance chain's input, found by review)

A review of the fixes above found that the chain they made fail-closed was still BUILT from
unvalidated input. The quoted-key and inline-table forms were reproduced through the wasm
boundary before the fix (both resolved with `fell_back = false` and the parent's `[prohibited]`
gone); every item was then re-run there after it. Each fix has a test that failed before it and
was shown to fail again with the fix removed.

- **`[meta] inherits` was read by a line scan, not by the TOML parser.** Valid TOML that wrote
  the key quoted (`"inherits" = "x"`), wrote `meta` as an inline table or dotted keys, or put a
  decoy `inherits` line in another table made the resolver ignore the parent or follow the wrong
  one. It is now read from the parsed document, from `[meta]` only; a value that is not a string
  fails closed as `<inherits-malformed:…>`. Introduced by the 2026-10-19 revision.
- **A parent is looked up exactly.** Progressive truncation (§4 step 1) negotiates the caller's
  REQUEST; applied to `inherits` it let a misspelt parent resolve to the ROOT pack whenever one
  existed — the least specific pack, often the most permissive.
- **Profile and `inherits` tags are validated** (`<invalid-tag:…>`): non-empty `/`-separated
  segments of ASCII alphanumerics and `_-.`, none starting with `.`, at most 16 segments and
  256 bytes. Under the filesystem adapter a tag is a path component and `inherits` is text the pack
  controls, so `../x` could pull in a pack from outside the root. The same bound ends the
  quadratic candidate expansion on a very long profile.
- **`legal_review` no longer inherits.** The merge copied the parent's review, signature key
  included, onto a child that had none, so a child nobody reviewed read as `approved` and, with a
  verified signature, as `provenance_verified = true`. Spec §4.3 says it does NOT inherit. (Rule 1
  of the same section lists `legal_review` among the scalars a child inherits when unset; the
  closing sentence is the specific one and is the reading implemented.)
- **A minimal floor wholly cancelled by `[prohibited]` fails closed**
  (`<minimal-floor-prohibited:…>`). The floor guard only checked that the list was non-empty.
- **`controls` is required in a control list**, as `pack.schema.json` already said. It defaulted
  to empty, so a misspelt key read as "requires nothing" and, in a child delta, ERASED the parent's
  table for that subject. An explicit `controls = []` is still valid. This TIGHTENS the runtime: a
  pack that relied on the default now fails closed as `<malformed:…>`.
- **A key id listed twice can no longer hide a revocation.** `key_status` took the first entry;
  it now evaluates every entry for the id and the most restrictive wins. The CLI's `directory.rs`
  carried its own copy of the date rules with their own fail-opens (an empty `revoked_at` read as
  "not revoked"); it now locates the entry by public key and asks the resolver. It gains a
  `KeyStatus::InvalidDate` variant and an `r14n verify` against a directory with an unreadable
  date now exits as untrusted.
- **A profile directory the process cannot enter is no longer "absent".** `is_file()` was false
  for any metadata error, so truncation let a less specific pack govern in its place. Only
  not-found is absent; anything else is `<unreadable:…>`.

Process note: these are fixes against requirements the spec already states (§4.3, §5, §6), but
three are readings the spec does not pin down: exact-match parent lookup, the tag grammar, and
`legal_review` against rule 1. No conformance vector covers any of them, and adding one is a
normative act, so other implementations are not yet bound to them. An Erratum or RFC should settle
them; none has been filed.

The earlier date tests asserted only "not trusted", which is also true of an expired or unknown
key; they now pin the exact `invalid_date` status and the `<invalid-…>` provenance marker.

### Added (recorded retroactively — shipped 2026-10-20 to 2026-10-30 without a CHANGELOG entry)

These landed on `main` without an entry, an RFC issue, or a second maintainer approval, which
`GOVERNANCE.md` requires for catalog changes. They are listed so the omission is visible, and so
that a maintainer can ratify or revert each one deliberately.

- **Eleven Control Catalogs** (was two): `data_residency`, `data_retention`, `analytics_consent`,
  `data_protection`, `health`, `location`, `nfc`, `photo_library`, `smart_home` added. No real
  jurisdiction is named in any of them; none ships a pack (invariant 2). In `data_retention`,
  `usage_analytics_opt_in` and `external_agent_harvest_permitted` are DEPRECATED, not removed,
  with `superseded_by` into `analytics_consent` (invariant 10).
- **`schema/catalog.schema.json`** — the catalog was the only published artifact with no schema.
- **`schema/pack.schema.json`**: `subject` gained a `propertyNames` pattern. TIGHTENING.
- **A `catalog` keyword and `ControlDef` type** in the r14n framework Snapp, and a node-validation
  suite for the baselines Snapp.
- The Snapp/TOML parity test now pins authored ORDER rather than comparing sets.

### Added (revision 2026-10-21 — spec §6 trust-root resolution; Level 3 complete)

- **`trust_root_revocation` is implemented** (spec §6:188-192). **Every vector in all three
  level files now runs and passes — Level 1 7/7, Level 2 17/17, Level 3 7/7, and all three
  skip lists are empty.**

  The resolver now resolves a pack's signing key against a reviewer-key directory as of the
  decision date: listing, revocation and expiry, with revocation outranking expiry and the
  revocation date itself already revoked ("from `revoked_at` FORWARD"). §6:190's
  non-retroactivity holds — a decision dated before revocation keeps its attestation — and
  §6:192's "expiry MUST be handled identically to revocation" is enforced by the same path.

  All of it is lexical ISO-date comparison, so it needs no new dependency and invariant 9's
  cap (serde + serde_json + toml + std) is untouched. Signature verification is the half this
  crate cannot do and arrives as `TrustRoot::signatures_verified`, an explicit caller
  assertion. **Both halves must hold**: a verified signature by a revoked key is exactly the
  case §6 exists to catch, and a directory listing proves nothing about a pack that merely
  NAMES the key.

  The trust root crosses the JSON boundary, so the generated JS and Python packages can supply
  one and read `provenance_verified` / `trust_status` back. A field the bindings cannot send
  would be a capability only Rust callers have (invariant 4).

### Fixed (revision 2026-10-21 — two controls that were read but never armed)

- **`advisory_only` was unconditionally `true`.** It was computed from a hardcoded
  `const PROVENANCE_VERIFIED: bool = false`, so no input could ever clear the taint — and the
  revocation vector EXPECTS `advisory_only: true`. Enabling the capability without this fix
  would have turned Level 3 green while proving nothing, because the suite could not tell a
  working revocation check from a resolver that taints everything.

  Fixed by making it read `PolicyProvenance::provenance_verified` and deleting the constant.
  A **green control** vector (`directory_listed_unrevoked_key_clears_the_advisory_taint` —
  same pack, same directory, revocation block removed, expects `false`) now makes the revoked
  vector meaningful, and three receipt tests cover the direction that was previously
  unreachable. Verified by mutation, not by observing green: removing the revocation
  comparison turns the revoked vector red, and clearing `provenance_verified` turns the green
  control red.

- **The conformance runner silently ignored a vector field.** Two shipped vectors spelled the
  decision date `decision_date`; the runner reads `as_of`. Both therefore resolved with NO
  date, so their temporal intent had never been applied and nothing reported it. Renamed to
  the one spelling, and `vector.schema.json`'s `query` object — which listed 7 of the 12 keys
  actually in use and left `additionalProperties: true`, which is WHY this went unnoticed —
  now declares all 12 and is sealed. A vector key the runner cannot read is a schema failure
  rather than a silent no-op, matching the closed-set guard the runner already applies to
  `expect`.

### Added (revision 2026-10-19 — RFC-4647 negotiation and delta-merge; Level 2 complete)

- **`rfc4647_negotiation` and `delta_merge` are implemented** (spec §4 steps 1-2, §4.3).
  **All 17 Level-2 vectors now run and pass; the skip list is empty for the first time.**

  Profile negotiation truncates `/`-delimited subtags most-specific-first
  (`example/region/locality → example/region → example → root`) and the first pack that exists
  governs. Truncation is LOOKUP ONLY: provenance still reports the profile the caller ASKED
  for, and `source` names the pack that answered, so a reader can see that a request for a
  locality was served by its region. Rewriting the profile would erase the only evidence that
  negotiation happened.

  Delta-merge follows `[meta] inherits` and applies §4.3's three rules, each a REPLACE rather
  than a union — which is what "delta-only" means: a child that lists a subject states the
  complete table for it. Union would be wrong in both directions. On floors it would make a
  child unable to state its own floor at all, which the spec gives it authority to do; on
  prohibitions it would let a parent forbid a control the child's reviewing attorney
  permitted, silently.

  Both adapters resolve the chain identically and a malformed pack ANYWHERE in it fails
  closed, not only a malformed child — an unparseable parent means the effective pack is
  unknown, which is invariant 1's case. An inheritance cycle terminates and the packs resolved
  before it still govern, because a loop is an authoring mistake in files the caller may not
  own.

  ⚠ **The governed conformance claim is now ELIGIBLE and has deliberately NOT been turned.**
  `TRADEMARKS.md` permits "conforms to RLPS Level 2, rlps-conformance/0.1" only when every
  vector for the level passes, and that is now true. `conformance/README.md` states the
  measurement and keeps the conservative wording: making a public conformance claim on a
  standards project is a maintainer decision, not a consequence of a green test run.

### Fixed (revision 2026-10-17 — the conformance suite stops naming a real jurisdiction)

- **Two shipped Level-2 vectors carried inline `gdpr/eu` pack fixtures**, which invariant 2
  names by name as counsel-gated and deliberately absent: *"do not generate one, even as an
  example — use `packs/example/region/` (explicitly fictional)"*. They are now
  `example` → `example/region` → `example/region/locality`.

  Nothing was lost. What those vectors prove is RFC-4647 subtag truncation and delta-only
  inheritance, and fictional subtags prove both identically — the grammar does not depend on
  the subtag being real. The packs were inline TOML, so no file under `packs/` existed for the
  pack lint to notice, and this repository is PUBLIC: a published conformance vector naming
  `gdpr/eu` reads as a GDPR pack existing, which is exactly the misreading invariant 2 exists
  to prevent.

- **Invariant 2 is machine-checked for the conformance suite now**
  (`tools/tests/schema_conformance.rs`). It was prose, and the suite had been violating it. The
  guard scans every serialized level file — not just pack keys — because the jurisdiction can
  hide in a profile string, an `inherits` value inside inline TOML, or a description. Verified
  by running it against the pre-change file, where it refuses and names the token.

  It matches a closed list of counsel-gated names rather than trying to detect "looks like a
  law". A clever pattern would flag `example/region` on its next revision and be deleted by
  whoever it blocked.

### Added (revision 2026-10-17 — GPC and IEEE-7012 escalation, spec §4 step 5)

- **`gpc_escalation` and `ieee7012_escalation` are implemented.** A query may carry
  `gpc_signal` (a `Sec-GPC: 1` signal) or `ieee7012_no_recording` (an IEEE-7012 `NoRecording`
  term); either escalates the subject to `aggressive` regardless of configured posture.
  Level-2 vectors run 15 of 17 — only `rfc4647_negotiation` and `delta_merge` remain
  authored-ahead.

  **Escalate-only, by construction.** `aggressive` is the maximum posture — the caller's whole
  declared universe — so forcing it can only ever require MORE. There is deliberately no
  inverse: no signal here can relax a posture, because a relaxation arriving as a request
  header is precisely what a fail-closed standard must not honour.

  **It sits ABOVE the posture ladder, not inside it.** "Regardless of configured posture" means
  it beats the pack's declared posture, the global dial AND a per-domain override. Both
  conformance vectors supply no override at all, so a resolver that folded escalation into the
  ladder would pass them and still lose to the most specific rung — pinned by a test that
  supplies one.

  **It does not override a prohibition.** Spec §3 says no posture may ADD a prohibited
  control, "including Aggressive", so an escalated decision is the universe MINUS the
  prohibition. A resolver that short-circuited to `query.universe` on a signal would hand back
  a control the pack forbids — the under-forbidding direction, and the worse one.

  **It is not a fallback.** A pack governed the decision; escalation changed which table was
  read. `fell_back` stays false, or every gate filtering for unreviewed decisions would flag an
  escalated one and the signal that means "no pack governed" would lose its meaning. The
  provenance `source` carries an `<escalated:…>` marker naming the signals — both when both
  fired — because `strictness = aggressive` against a pack declaring `minimal` otherwise looks
  to an auditor like the pack and the receipt disagree.

### Changed (revision 2026-10-17 — the consumer declares the version it folds)

- **`baselines/1.0.0` now pins its dependency by `name@version`**, not just by hash:
  `@deps "r14n@0.1.0-alpha.1" { … }` in place of the versionless
  `@deps { r14n { … } }`. The integrity hash was already catching byte changes; what was
  missing is that a versionless declaration means the version-agreement check never runs at
  all, so the consumer folded whatever artifact the name resolved to and verified only its
  bytes. Now it states the version it expects and the compiler enforces it.

  This also removes a latent trap the toolchain lane traced to its root cause: a versionless
  `@deps` makes the resolver FABRICATE version `0.0.0` and write `.n/deps/local/r14n@0.0.0.json`,
  and a later scan trusts that filename over the artifact's own `@snapp.version`. That is why
  every build here reported `dep:r14n@0.0.0` while the bundle declared `0.1.0-alpha.1`, and why
  anyone who later wrote the versioned form hit a confusing pin failure against a cache entry
  the versionless form had minted. The dep cache now holds exactly
  `r14n@0.1.0-alpha.1.json` and builds report that identity.

- **The drift gate's `@deps` parser is whitespace-insensitive.** It matched the exact strings
  `from =` / `integrity =`, so padding a key for column alignment (`from      = "…"`) made it
  stop seeing the entry. It caught this itself on the change above — the right failure, because
  an unparsed `from` leaves nothing for the `integrity` line to be checked against, and
  silently skipping a pin is the one thing this gate exists to prevent.

### Changed (revision 2026-10-17 — re-exporting a Snapp is idempotent now)

- **The Snapp pipeline sets `SOURCE_DATE_EPOCH`**, and `AGENTS.md` carries it as a fourth
  rule beside the guard, the cache isolation and the gate-on-content rule.

  Both bundles embed `@snapp.date`, and `@snapp.integrity` is a content address over the whole
  artifact — `date` included. That is correct behaviour for a content address, but it meant a
  no-op re-export changed the bytes and so invalidated the downstream `integrity` pin for a
  reason unrelated to content. "The pin went stale" could not distinguish *the dependency
  changed* from *somebody re-ran export*, which made the one gate guarding invariant 11 noisy
  in exactly the way that teaches people to refresh pins without reading them.

  Measured on `nlang 0.1.0-dev (c87e92fd6)`: without the variable, a re-export with identical
  sources differs in exactly `date` and `integrity`, all pack and type content byte-identical.
  With it, two exports at the same epoch are byte-identical, and both bundles here now re-emit
  with zero diff. The epoch convention is the commit time of the Snapp sources, so the date
  records when the content was last authored rather than when a build happened to run.

  Both bundles were re-emitted under the convention and the framework pin refreshed in the
  same change. The wrinkle is documented rather than hidden: the source commit time moves when
  the change is committed, so emit before committing — the property this buys is that a NO-OP
  re-export is free, not that the date derives from nothing.

### Added (revision 2026-10-17 — per-domain posture precedence, spec §3)

- **`per_domain_posture` is implemented.** A query may carry
  `per_domain_posture_overrides`, keyed by domain, and the MOST SPECIFIC setting wins: a
  per-domain entry beats the global strictness dial, which beats the pack's own declared
  posture, which falls back to `aggressive`. Level-2 vectors run 13 of 17 (up from 12); four
  remain authored-ahead.

  Carried on the QUERY rather than the adapter, because it is per-domain and an adapter
  serves every domain — the alternative was one adapter per domain or a mutable dial, and a
  mutable dial is a posture that can change between two queries a caller believes are
  comparable. Kept as a map rather than a resolved value so the precedence rule has one home
  instead of one copy per call site.

  Tests pin the whole ladder in one place, because the failure mode is a resolution that
  SUCCEEDS at the wrong posture: the fixture pack declares `aggressive`, has a one-control
  subject table and a different one-control floor, so every rung returns a distinct
  well-formed set and an off-by-one in the precedence looks like an answer rather than an
  error. §3's per-domain independence is pinned separately — an override for another domain
  must not reach this one, and a lookup that ignored the key or took the first entry would
  pass every single-entry test.

### Changed (revision 2026-10-17 — the domain lives in the path, not the pack)

- **`domain` was removed from the typed `PolicyPack`.** The emitted keyword-form node is now
  byte-for-byte a publishable pack: it validates against `schema/pack.schema.json` **as
  emitted**, with nothing stripped first. Spec §2.2 already placed the domain in the PATH —
  on disk a pack is `<domain>.r14n.toml` under its profile directory — and an emitted bundle
  keys each pack by its declaration name, so carrying the domain in the body as well was
  duplicate data that cost the node its validity against a schema with
  `additionalProperties: false`.

  Removing it, rather than permitting it, was chosen over two alternatives. Adding `domain`
  as an optional schema property would have argued against §2.2's stated design and needed an
  RFC with two maintainer approvals. Having a writer drop the key named `domain` would have
  been a **by-name string rule** — the thing `Data Contracts Over String-Matching` forbids,
  and the same defect class as the legacy tool-id literals retired elsewhere this week. The
  measurement that settled it: every consumer that read the field already held the same value
  as the bundle key it was iterating (verified across all shipped downstream profiles), so the
  field was redundant rather than load-bearing.

  Both bundles were re-emitted and the framework integrity pin refreshed in this same change.
  `nlang check` + `test` (11/11) + `export` green for both Snapps under the memory/time guard
  with an isolated cache, gated on emitted CONTENT rather than exit code.

  ⚠ A downstream consumer that reads a pack's `domain` from an emitted bundle must switch to
  the bundle key. No shipped consumer breaks today, because the vendored copy of the framework
  bundle still carries the pre-change bytes with its matching pin and is internally
  consistent — the change surfaces when that copy is refreshed.

### Added (revision 2026-10-17 — §4.1 ambiguous attribution is enforced)

- **The reference resolver implements `ambiguous_attribution`** (spec §4.1, normative). A
  query may now carry `attribution_source` — `declared` | `inferred` | `unknown`, with a
  forward-additive escape hatch — and an ambiguous attribution resolves to
  `verdict = block`. Level-2 vectors run 12 of 17 (up from 11); five remain
  authored-ahead. This was the gap `conformance/README.md` had flagged for months as the one
  a reader was most likely to assume closed.

  **Checked before any pack is read.** Attribution answers *which* law applies, and no amount
  of correctly resolving the wrong jurisdiction's pack repairs a wrong answer to that — a
  resolver that resolved first and blocked second would still have chosen a pack it had no
  basis to choose.

  **Typed, never a bare string.** An unrecognised spelling decides as `unknown` rather than as
  a verified source, because ranking an unknown word as "attributed" is how one caller's typo
  would turn a fail-closed condition into a permit. The escape hatch round-trips verbatim so
  it stays auditable.

  **A block keeps the fail-closed SHAPE.** `required` is the caller's whole universe, not an
  empty set: a blocked decision reading empty would, to anything that ignored the verdict, be
  indistinguishable from "nothing is required" — the permissive reading of a refusal.
  `provenance.fell_back` is true and `source` names the ambiguity, because no pack was
  consulted and a gate filtering for unreviewed decisions has to see it.

  ⚠ **Absent is not assent.** Omitting `attribution_source` OPTS OUT of the §4.1 check and
  resolves normally — the same contract `as_of: None` already has for the temporal envelope,
  and for the same reason: a field every pre-§4.1 caller omits cannot start refusing their
  queries without notice. A gate that must enforce §4.1 has to SUPPLY the field; a permit
  from an opted-out query is not evidence that attribution succeeded.

  Reaches the JS/TS and Python packages through `resolver/src/wire.rs` with no shim logic
  (bindings policy). Verified through the wasm boundary: `unknown` and an unrecognised
  spelling both block with the universe intact, `declared` resolves, and an omitted field
  resolves exactly as before.

### Changed (revision 2026-10-16 — the typed pack shape IS the published pack shape)

- **The framework Snapp's `PolicyPack` was renamed to match the published schema**, so a
  straight serialization of the emitted node is already a valid pack. `subjects` → `subject`
  (the published `subject` is an object of `control_list`, which is what the keyword form was
  already emitting — only the name was wrong), and `legally_required` / `prohibited` became a
  new `ControlList` block carrying `controls`, matching `[legally_required] controls = [...]`.

  This is why it was done HERE and not in a serializer: the field name is a name this
  repository mints, while the subject keys (`telepresence`, …) are author-declared and were
  already preserved — a name we mint is ours to correct, an author's key is not. The
  projection a consumer was performing existed only because the producer shipped a spelling
  and no reader, so the fix belonged at the mint rather than at every consumer. The TOML
  exporter's projection half is consequently **not** needed and was not commissioned.

  `tools/tests/toml_export_acceptance.rs` now validates **every emitted pack against
  `schema/pack.schema.json` itself**, replacing an inverted tripwire that asserted the old
  mismatch and went red on purpose when this landed. `resolver/tests/snapp_toml_parity.rs`
  lost its "the shapes differ by serialization" normalization note, because the difference is
  gone — the only normalization left is sort order, which neither side chose.

  `domain` is the one top-level key the published schema does not allow and it stays in the
  typed source deliberately: a bundle holds many packs and `domain` names the file each one
  splits into. A writer drops one key, which needs no knowledge of what a pack is. Making
  `domain` an optional schema property would be additive but changes a PUBLISHED schema, so it
  is left as a governance question.

  Both bundles were re-emitted and the integrity pin on the framework bundle refreshed in this
  same change. Verified `nlang check` + `test` (11/11) + `export` for both Snapps under the
  memory/time guard with an isolated cache, gating on emitted CONTENT rather than exit code.

### Added (revision 2026-10-16 — cross-jurisdiction merge, spec §4.2)

- **The reference resolver implements the most-restrictive merge and the deontic-conflict
  rule** (spec §4.2, normative). A session spanning jurisdictions `J1..Jn` now resolves
  against each jurisdiction's pack independently and merges: `required` is the UNION of the
  per-jurisdiction sets, the verdict is `block` if any per-jurisdiction resolution blocks, and
  a control required by one jurisdiction while marked `prohibition` by another is unsatisfiable
  — `verdict = block`, with the conflicting control key and BOTH jurisdictions recorded.
  `conformance/level-2.json`'s two vectors for these capabilities now run (11 of 17 level-2
  vectors, up from 9; 6 remain authored-ahead).

  Two capabilities is **not** a Level-2 conformance claim. Per `TRADEMARKS.md` the governed
  wording stays "implements Level 2; Level 2 in progress" until every vector for the level
  passes — `rfc4647_negotiation`, `delta_merge`, `gpc_escalation`, `per_domain_posture`,
  `ieee7012_escalation` and `ambiguous_attribution` are still open.

- **`ControlDecision` now surfaces `prohibited` and `conflicts`** (additive; the `prohibited`
  set was previously computed, used to subtract from `required`, and discarded). Two reasons,
  one of which is a fix rather than a feature:

  1. §4.2.3 cannot be implemented without the forbidden set — a merge has to compare what one
     jurisdiction requires against what another forbids.
  2. A caller could not previously tell "not required" from "forbidden" at all. That matters
     because a fail-closed fallback **over-requires** (the whole universe) while
     **under-forbidding** (nothing, since no pack was read), so `prohibited` is empty on a
     `fell_back` decision and is NOT evidence that nothing is prohibited. The asymmetry is now
     visible to a caller instead of being implicit in the resolver; it remains the reason a
     prohibition key must never appear in a caller's universe.

  Both fields are additive and reach the JS/TS and Python packages through
  `resolver/src/wire.rs` — the merge lives in the core and the shims gained no logic
  (bindings policy). Verified end-to-end through the wasm boundary: a cross-jurisdiction query
  returns `block` with the conflict naming both jurisdictions, and a query that omits
  `jurisdictions` resolves exactly as before.

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

### Added (revision 2026-09-22 — the w3id permanent identifier is live)

- **`https://w3id.org/rlps` resolves.** The alias PR merged 2026-08-24 and was verified serving
  byte-identical content through to `r14n.squillo.com/ns`. `docs/namespace.md` had still called it
  "requested"; it now records the alias as live and notes *why* it is the form to cite — a w3id
  identifier survives this project changing hosts, which is the entire reason to mint one.

### Changed (revision 2026-09-19 — aph#2 granted: this repo now cites APH's terms instead of its own)

- **APH accepted [squillo/aph#2](https://github.com/squillo/aph/issues/2) as RFC 0009 and closed
  it.** `StatusAbsent` and `StatusLive` are now a normative closed set there, with `StatusLive`
  required to carry the instant and the status-list identifier. `docs/aph-integration.md` cites
  those terms and **retires the local evidence vocabulary** this repository had been forced to
  invent to define its two APH-backed control keys — which was the whole argument for filing, and
  which RFC 0009 quotes as such.
- `docs/aph-dependency-report.md` records the grant. **Status unchanged: (a)**, documentation-only
  — citing a normative term is not asserting a wire fact, so APH's pre-production exception still
  holds and the notify-before-crossing commitment stands.
- **Correction of record, twice in one day — THE ENFORCEMENT GATE NOW EXISTS.** This project
  reported no evidence of a gate being built; one was then *ruled*; it has since **landed** and
  was verified here in the tree rather than taken on relay. `AphMandateVerifierService` links
  `aph-core`, fails closed on an unestablishable status, and emits RFC 0009 evidence
  (`StatusAbsent` / `StatusLive` carrying the instant and the status-list id). The satisfaction
  layer of `docs/aph-integration.md`'s three-layer model is real for the first time.
- **r14n's own status is unchanged: (a), documentation-only.** The gate lives in another
  repository. r14n still links no APH code and asserts no wire fact, so nothing here crosses
  APH's trigger — a separate consumer reaching (b) is APH's to record, not ours to claim.

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
