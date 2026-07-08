# Conformance-suite versioning & evolution policy

How the RLPS conformance suite versions against the spec, and what a conformance claim means across
revisions. **NOT LEGAL ADVICE** — a conformance claim is an engineering claim about resolver
behavior, never a compliance claim ("aggressive" ≠ "compliant").

## Suite identifier

Every vector file carries `"suite": "rlps-conformance/MAJOR.MINOR"` (currently
`rlps-conformance/0.1`). A resolver states the suite version it passed:
**"conforms to RLPS Level N, rlps-conformance/0.1"**.

## What the version tracks

The suite version tracks the **spec version** it validates. `rlps-conformance/0.1` validates
`spec/RLPS-v0.1.md`. When the spec bumps, the suite bumps in step.

- **MINOR** bump (e.g. 0.1 → 0.2): **append-only**. New vectors and new capability tags may be
  added; existing vectors' `name`, `capabilities`, and `expect` MUST NOT change meaning (fix a
  genuinely-wrong vector only via a MAJOR bump + a changelog entry). A resolver that passed 0.1
  still passes the 0.1 subset of 0.2.
- **MAJOR** bump (e.g. 0.x → 1.0, or any breaking change to an existing vector): existing vectors
  may change or be removed; conformance claims do NOT carry across a MAJOR bump — re-test.

## Capability vocabulary

The set of capability tags is closed and lives in three synchronized places (a test enforces the
sync): `conformance/vector.schema.json` (the enum), `conformance/README.md` (the level tables), and
`resolver/tests/conformance.rs` (`KNOWN_CAPABILITIES` + the resolver's `SUPPORTED` subset). Adding a
capability is a MINOR change and MUST update all three. A vector may only use tags from the
vocabulary; `scripts/check-schemas.py` + the meta-validation test reject typos.

## Level-claim semantics

- To claim **Level N**, a resolver MUST pass **every** vector for **all** capabilities that level
  requires (`conformance/README.md`). Partial support is stated as "implements Level N; Level N+1 in
  progress", never as bare "RLPS conformant" (see `TRADEMARKS.md`).
- Vectors authored **ahead** of the reference implementation (skipped by it today) are normative:
  they define required behavior for any resolver claiming that capability, and the reference
  runner's pinned run/skip counts force a conscious update when a capability lands.

## Changelog

Suite-version changes are recorded in the spec's revision history and in the commit that bumps the
`suite` field; there is no separate changelog file until the first MINOR bump.
