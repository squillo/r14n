# /registry — pack versioning + reviewer-key directory

Content-addressed pack SHAs + effective-date supersession + the reviewer public-key
directory (jurisdiction + credential_type + revocation). The spec mandates the
provenance envelope; the registry establishes trusted signers (code-signing model).

**Here now:**
- [`reviewer-key.schema.json`](reviewer-key.schema.json) — the normative trust-root
  directory schema (spec §6): steward-signed, per-key `jurisdiction` +
  `credential_type` + `credential_id`, expiry + revocation. A directory entry proves
  *who* may attest, never that a review was correct — NOT LEGAL ADVICE.
  Consumed by `r14n verify --directory` (the `directory` module in `/tools`): given a
  verified signing key + a decision date, it reports whether the key is a listed,
  non-revoked, non-expired, jurisdiction-scoped reviewer — the tooling half of the
  trust root (the generic resolver stays directory-agnostic).
- [`pack-index.schema.json`](pack-index.schema.json) — content-addressed pack
  versioning + supersession (monotonic version per `<profile>/<domain>` id,
  `supersedes` sha256 chain, optional embedded `rlps-sig/0.1` signature). Written
  locally by `r14n publish` (see `/tools`).

**Planned:** the hosted directory + index instances (counsel-gated, like everything
public-facing here). `index.json` is intentionally absent until real packs publish.
