# /registry — pack versioning + reviewer-key directory

Content-addressed pack SHAs + effective-date supersession + the reviewer public-key
directory (jurisdiction + credential_type + revocation). The spec mandates the
provenance envelope; the registry establishes trusted signers (code-signing model).

**Here now:**
- [`reviewer-key.schema.json`](reviewer-key.schema.json) — the normative trust-root
  directory schema (spec §6): steward-signed, per-key `jurisdiction` +
  `credential_type` + `credential_id`, expiry + revocation. A directory entry proves
  *who* may attest, never that a review was correct — NOT LEGAL ADVICE.

**Planned:** the pack version index (content-addressed supersession) + the hosted
directory instance itself (counsel-gated, like everything public-facing here).
