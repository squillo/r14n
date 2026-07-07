# Security Policy

> **⚠ NOT LEGAL ADVICE.** This policy covers software-security defects in the RLPS reference
> implementation. It is not a warranty and does not address legal compliance. See
> [`docs/not-legal-advice.md`](docs/not-legal-advice.md).

## Status

RLPS is **pre-1.0 and this repository is private** pending legal review. There is no public
release and no supported-version guarantee yet. Once published, this policy will name supported
versions.

## Reporting a vulnerability

Report suspected vulnerabilities privately to **security@squillo.com** — do **not** open a public
issue for a security defect. Please include a description, affected files/commit, and a
reproduction. We aim to acknowledge within a few business days.

## What is in scope

The reference implementation ships cryptography and trust-root machinery whose failure modes are
security-relevant:

- **Signing / verification** (`tools/src/keys.rs`): Ed25519 detached signatures over pack bytes;
  content-addressing (SHA-256). A signature binds the pack *bytes*, not (yet) the pack's
  profile/domain identity — see the known-limitations note below.
- **Publish integrity** (`tools/src/publish.rs`): `publish` cryptographically verifies an embedded
  signature and refuses corrupt/duplicate index versions; the local index is not authenticated as
  a whole.
- **Untrusted input parsing** (`resolver/src/lib.rs`, the `tools` crate): packs, catalogs,
  signature docs, and index files are parsed from untrusted input. The resolver's contract is that
  any malformed pack degrades fail-closed (aggressive-over-universe), never panics.
- **Key material**: `r14n keygen` writes the private seed with `0600` permissions; seeds are
  git-ignored (`*.seed`). Treat a seed as a private signing key.

## Known limitations (v0.1, tracked — not vulnerabilities to report)

- **No reviewer-key directory verification.** The reference resolver does not consult a
  reviewer-key directory or revocation list, so `provenance_verified` is always `false` and every
  decision receipt is `advisory_only`. A self-declared `legal_review.status = "approved"` does not
  establish trust.
- **A signature binds pack bytes, not profile identity.** Until pack identity is bound into the
  signed material, a signed pack could be relocated under a different profile directory with its
  signature still verifying. Do not rely on a signature to prove a pack's intended profile.
- **The local pack index is not authenticated end-to-end.** It is a local artifact; a hosted,
  signed registry is future work (counsel-gated).

These are documented in `docs/audits/` and the roadmap; reports that restate them are welcome but
already known.
