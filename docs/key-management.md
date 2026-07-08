# Key-management guidance

Operational guidance for the Ed25519 keys RLPS uses (reviewer signing keys and the steward
directory key). **NOT LEGAL ADVICE.** See [`SECURITY.md`](../SECURITY.md) for the threat model and
disclosure policy.

## Key types

- **Reviewer signing key** — a licensed reviewer signs packs with it (`r14n sign`). Its public key
  is listed in the reviewer-key directory.
- **Steward directory key** — the neutral steward signs the reviewer-key *directory* itself with it
  (`r14n sign-directory`, over the RFC-8785/JCS canonical form). It is the root of trust.

## Storage

- `r14n keygen` writes the private **seed** to `<prefix>.seed` (64 hex chars) with `0600`
  permissions and the public key to `<prefix>.pub` (unpadded base64). Seeds are git-ignored
  (`*.seed`) — **never commit a seed**.
- Keep seeds out of shared drives, backups that sync to third parties, and CI logs. Treat a seed
  exactly like an SSH private key.
- The **steward key** is the highest-value key: keep it **offline** (an air-gapped machine or an
  HSM/hardware token). It only needs to be online to re-sign the directory when it changes.

## Rotation & revocation

- **Reviewer keys:** to retire a key, add a `revocation` block (`revoked_at`, `reason`) to its
  directory entry, or let `valid_until` pass. From `revoked_at` forward the key is untrusted for new
  decisions (`r14n verify --directory` returns non-zero); prior decisions are not retroactively
  invalidated. Re-sign the directory with the steward key after any change.
- **Steward key:** rotating it re-roots trust — publish the new steward public key out-of-band and
  re-sign the directory. Plan for this before it is needed.
- On suspected **compromise**, revoke immediately (`reason: key_compromise`), re-sign the directory,
  and follow `SECURITY.md`.

## Verification hygiene

- Always run `r14n verify --directory <dir> --as-of <decision-date> [--jurisdiction <j>]` — a bare
  `verify` proves only WHO signed, not that they may attest. Supply the real decision date so
  expiry/revocation are evaluated correctly.
- A valid pack signature from a signer not listed (or revoked/expired) in the directory keeps the
  decision **advisory-only** — that is the intended, honest outcome, not an error to work around.
