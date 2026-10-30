# Security Policy

> **⚠ NOT LEGAL ADVICE.** This policy covers software-security defects in the RLPS reference
> implementation. It is not a warranty and does not address legal compliance. See
> [`docs/not-legal-advice.md`](docs/not-legal-advice.md).

## Status and supported versions

RLPS is **pre-1.0**: the wire format, schemas, and APIs may change between 0.x releases, and
security fixes land only on the current draft line.

| Version | Supported |
|---|---|
| 0.1.x (current draft) | ✅ |
| < 0.1 | ❌ |

## Reporting a vulnerability

Report suspected vulnerabilities privately to **security@squillo.com** — do **not** open a public
issue for a security defect. Please include a description, affected files/commit, and a
reproduction. We aim to acknowledge within a few business days.

## What is in scope

The reference implementation ships cryptography and trust-root machinery whose failure modes are
security-relevant:

- **Signing / verification** (`tools/src/keys.rs`): Ed25519 detached signatures; content-addressing
  (SHA-256). A signature (`rlps-sig/1`) binds the pack bytes AND its `<profile>/<domain>` identity,
  derived from the on-disk path, so a signed pack relocated to another profile fails verification.
  It covers ONE pack: see "inherited parents" below.
- **Publish integrity** (`tools/src/publish.rs`): `publish` cryptographically verifies an embedded
  signature and refuses corrupt/duplicate index versions; the local index is not authenticated as
  a whole.
- **Untrusted input parsing** (`resolver/src/lib.rs`, the `tools` crate): packs, catalogs,
  signature docs, and index files are parsed from untrusted input. The resolver's contract is that
  any malformed pack degrades fail-closed (aggressive-over-universe), never panics. That includes
  the inheritance chain: `[meta] inherits` is read by the TOML parser, must be a well-formed
  profile tag (no `..`, absolute path, empty or dot-leading segment), and names its parent
  EXACTLY — a missing or unreadable parent fails closed rather than resolving to a less specific
  pack.
- **Key material**: `r14n keygen` writes the private seed with `0600` permissions; seeds are
  git-ignored (`*.seed`). Treat a seed as a private signing key.

## Known limitations (v0.1, tracked — not vulnerabilities to report)

- **Trust-root resolution is opt-in, and the signature check is the caller's.** The reference
  resolver consults a reviewer-key directory, revocation and expiry (spec §6) only when a caller
  supplies one. With none supplied, `provenance_verified` is `false` and every decision receipt is
  `advisory_only`. The resolver contains no cryptography (it stays dependency-lean), so
  `signatures_verified` is the CALLER's assertion: a caller that sets it wrongly produces a receipt
  that overstates its own provenance. Signature verification lives in `r14n-tools` (`r14n verify`).
  A self-declared `legal_review.status = "approved"` does not establish trust, and a pack's review
  is its own: an inheriting pack that carries none is not attested by its parent (spec §4.3).
  When a decision date is supplied, a date the resolver cannot read — in a directory entry, a
  pack's effective window, or the decision date itself — resolves most-restrictive, never trusted
  and never in force. **With no decision date the effective-date envelope is not checked** (the
  key resolves `unknown`, so the decision stays advisory-only, but the pack is treated as in
  force): supply `as_of` in any deployment that relies on the envelope.
- **Inherited parents are not covered by the child's attestation.** `provenance_verified` speaks
  to the pack that carries the review. The effective pack of an inheriting profile is merged from
  its parents, whose bytes that signature does not cover, and `signatures_verified` is one boolean
  for the whole decision. Verify every pack the decision's `source` names, not only the first.
- **A fix reaches you only when you take it.** A deployment pinned to an earlier build of the
  resolver, the npm package or the Python wheel keeps that build's behaviour; see `CHANGELOG.md`
  for what changed and when.
- **The local pack index is not authenticated end-to-end.** It is a local artifact; a hosted,
  signed registry is future work (counsel-gated).

These are documented in `docs/audits/` and the roadmap; reports that restate them are welcome but
already known.
