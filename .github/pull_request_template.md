<!-- The checklist below is CONTRIBUTING.md's and GOVERNANCE.md's review rules made visible at
the moment they apply. Delete lines that do not apply to this PR — an unchecked irrelevant box
reads the same as an unmet requirement. -->

## What this changes

<!-- One paragraph. Link the RFC / Erratum / Conformance issue this lands. -->

## Which surface

- [ ] **Normative** (spec text, resolution algorithm, conformance levels, catalog semantics) —
      needs **two maintainer approvals**, a version consequence per GOVERNANCE.md §Change
      process, and a dated `CHANGELOG.md` entry + spec Revision History banner
- [ ] **Published vectors / schemas** — bytes other implementations test against; appended per
      `conformance/VERSIONING.md` (suite MINOR is append-only), never silently mutated; the
      reference runner's pinned run/skip counts updated consciously
- [ ] **Packs** — lints clean (`r14n validate --catalog`), carries the NOT-LEGAL-ADVICE header,
      references only catalog controls, and makes **no jurisdiction claim** (those are
      counsel-gated — GOVERNANCE.md hard gates)
- [ ] **Non-normative docs / implementation only** — one approval

## The record

- [ ] Dated `CHANGELOG.md` entry (required for normative or vector/schema-touching changes)
- [ ] Every commit carries `Signed-off-by:` (DCO — `git commit -s`)
- [ ] Both crates green: `cargo test` in `resolver/` and `tools/`
- [ ] Every new test carries a `/// Why:` doc stating the invariant it guards
- [ ] Counts and enumerations this change moves (vector run/skip counts, test counts, source
      counts) were found by sweep and updated — a stale count is a defect
