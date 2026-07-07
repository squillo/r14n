# /tools — the RLPS pack-lifecycle CLI (`r14n`)

Rust binary crate (`cargo build` here; binary name `r14n`). **NOT LEGAL ADVICE** — every
subcommand operates on configuration artifacts; none states what any jurisdiction's law is.

| Subcommand | i18n analogue | What it does |
|---|---|---|
| `extract --catalog <f> --profile <p>` | `xgettext` | Catalog → **fail-closed pack template** (floor = every catalog control; `status = "draft"`). |
| `merge --pack <f> --catalog <f>` | `msgmerge` | Diff a deployed pack against the current catalog; flag ONLY changed controls (`NEEDS-LEGAL-REVIEW` / `STALE`) in a prepended comment block — the pack body is preserved byte-for-byte. |
| `validate <pack>... [--catalog <f>]` | lint | The cross-file linter behind `schema/pack.schema.json`: floor present, approved ⇒ attorney-of-record envelope + `last_reviewed_against_guidance`, `[prohibited]` ∩ floor = ∅, catalog membership. |
| `keygen --out <prefix>` | — | Ed25519 keypair: `<prefix>.seed` (hex, PRIVATE) + `<prefix>.pub` (unpadded base64, the reviewer-directory encoding). |
| `sign <pack> --key <seed> [--key-id <id>]` | — | Detached signature `<pack>.sig` (JSON: sha256 + Ed25519 over the exact bytes). |
| `verify <pack> [--sig <f>]` | — | Verify content address + signature. |
| `publish <pack> --id <profile>/<domain>` | — | Validate, then append a content-addressed version entry (monotonic version, `supersedes` sha chain, signature embed) to the **LOCAL** `registry/index.json`. |

`publish` never touches a network. Making anything public — pushing this repo, crates.io, a
hosted registry — is a human decision behind the counsel gate (`the maintainer notes` constraint 3).

Gate: `cargo test` here (unit + `tests/cli.rs` integration) alongside the resolver's suite.
