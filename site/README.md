# site/ — r14n.squillo.com

The Cloudflare Worker behind `r14n.squillo.com`: the RLPS splash page at `/` and the vocabulary
namespace host at `/ns` (content-negotiated JSON-LD / Turtle, plus the direct artifacts
`/ns/context.jsonld` and `/ns/rlps.ttl`). **NOT LEGAL ADVICE** — see `/docs/not-legal-advice.md`.

## `worker/` — the Rust Worker (`r14n-ns`)

A pure routing table (`src/site.rs`: plain data in, plain data out, no host types) plus a thin
workers-rs shim (`src/lib.rs`). It is a standalone crate with its own `Cargo.lock`, so building the
resolver or the CLI never pulls in a WASM toolchain.

- The vocabulary, the splash page and the banner are embedded **directly from their source of
  record** (`ns/`, `site/src/index.html`, `assets/`), so the served bytes cannot drift from it.
- **ETags are derived, not hand-pinned.** `build.rs` computes each one (first 32 hex of the
  sha256) from the file it describes, and a test recomputes it independently. Changing `ns/`
  needs no other edit.
- Behaviour is pinned to a recorded matrix of the previous Worker (`tests/fixtures/live_parity.toml`,
  20 requests: methods, HEAD, conditional requests, Accept variants, 404/405). The only intentional
  differences are two response headers added to every response: `x-content-type-options: nosniff`
  and `x-squillo-impl: rust-wasm-workers-rs` (so a parity audit can tell the two Workers apart).
- `worker` is pinned exactly (`=0.8.5`) and the `wasm-bindgen` CLI must equal the `wasm-bindgen`
  crate in `Cargo.lock`. The Worker is built with plain `cargo` + `wasm-bindgen`, not `worker-build`.

```console
$ cargo test --locked --manifest-path site/worker/Cargo.toml
$ RUSTFLAGS="-C target-feature=+reference-types,+multivalue" \
    cargo build --locked --release --target wasm32-unknown-unknown --manifest-path site/worker/Cargo.toml
```

How the module is bundled and deployed is a deploy-side concern and lives with the deploy
descriptor, not in this repository.

## `src/index.js` + `wrangler.jsonc` — the previous Worker

Still what is deployed until the Rust Worker is cut over; it is retired (and these two files
deleted) once a canary of the Rust Worker has been audited against it. Until then, if `ns/`
changes, its two hand-pinned ETag constants must be updated (first 32 hex chars of each file's
sha256) — the Rust Worker has no such step.

```console
$ cd site && wrangler deploy
```
