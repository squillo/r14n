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
