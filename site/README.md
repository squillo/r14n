# site/ — r14n.squillo.com

`r14n.squillo.com` is two tiers, kept apart on purpose: a **static tier** (the RLPS splash page at
`/` and its banner) and a **Worker** (the vocabulary namespace host at `/ns`, content-negotiated
JSON-LD / Turtle, plus the direct artifacts `/ns/context.jsonld` and `/ns/rlps.ttl`).
**NOT LEGAL ADVICE** — see `/docs/not-legal-advice.md`.

## `public/` — the static tier (frontend)

Plain files, served by the platform in front of the Worker, so a request for them never runs Rust:
`index.html`, `assets/r14n-banner.webp`, and `_headers`.

- `_headers` is the response policy the previous Worker sent by hand, now declared beside the files:
  ten minutes of cache for `/`, immutable for `/assets/*` (a file there is renamed when its content
  changes), and `X-Content-Type-Options: nosniff` plus `Allow: GET, HEAD` on everything static.
  `worker/tests/parity.rs` pins all three rules and that the page still points at the banner path.
- The page has **no framework and no build step**. It does load one third-party analytics script,
  so a strict Content-Security-Policy cannot be shipped while it does.
- Accessibility: landmarks, a skip link, labelled navigation, visible focus, 4.5:1 contrast (both
  colour schemes), reflow at 320 CSS px, and a `prefers-reduced-motion` rule. **Conformance is not
  claimed**: no keyboard walk or screen-reader pass has been done. Re-run a contrast and reflow
  check whenever the CSS changes.

## `worker/` — the Rust Worker (`r14n-ns`, backend)

A pure routing table (`src/site.rs`: plain data in, plain data out, no host types) plus a thin
workers-rs shim (`src/lib.rs`). A standalone crate with its own `Cargo.lock`, so building the
resolver or the CLI never pulls in a WASM toolchain.

- The vocabulary is embedded **directly from its source of record** (`ns/`), so the served bytes
  cannot drift from it. The page and the banner are **not** embedded: the Worker is 312 KB of wasm
  (114 KB gzipped), down from 527 KB when it carried them.
- **ETags are derived, not hand-pinned.** `build.rs` computes each one (first 32 hex of the sha256)
  from the file it describes, and a test recomputes it independently. Changing `ns/` needs no other
  edit.
- Behaviour is pinned to a recorded matrix of the previous Worker (`tests/fixtures/live_parity.toml`:
  16 of the 20 recorded requests — methods, HEAD, conditional requests, Accept variants, 404/405; the
  other 4 are the static tier's). Two response headers are new, on every response this Worker sends:
  `x-content-type-options: nosniff` and `x-squillo-impl: rust-wasm-workers-rs` (so a parity audit
  can tell the two Workers apart).
- `worker` is pinned exactly (`=0.8.5`) and the `wasm-bindgen` CLI must equal the `wasm-bindgen`
  crate in `Cargo.lock`. The Worker is built with plain `cargo` + `wasm-bindgen`, not `worker-build`.

```console
$ cargo test --locked --manifest-path site/worker/Cargo.toml
$ RUSTFLAGS="-C target-feature=+reference-types,+multivalue" \
    cargo build --locked --release --target wasm32-unknown-unknown --manifest-path site/worker/Cargo.toml
```

### What a deployment must provide

The Worker alone does not serve the page: **`/` and `/assets/*` are a 404 unless the static tier is
configured as the Worker's static-assets directory** (`site/public`). That is deliberate — a missing
tier is a visible failure, not a guess. How the module is bundled and deployed, and where the assets
directory is declared, is a deploy-side concern and lives with the deploy descriptor, not here.

Measured in a local workerd with `site/public` as the assets directory (all 20 recorded live
requests replayed): 14 are byte-for-byte identical to live. The 6 that differ, and why:

| Cases | Difference | Why |
|---|---|---|
| `GET`/`HEAD` `/` and the banner | gain `ETag`, `Allow`, `nosniff` | platform ETag enables revalidation; the others come from `_headers` |
| `GET /` | different body | the accessibility edits to the page |
| non-GET/HEAD on `/` and the banner | still 405 and `Allow: GET, HEAD`, but an empty body, no content-type, and the path's `Cache-Control` | the platform answers these itself; `_headers` cannot scope a rule by method |

## `src/index.js` + `wrangler.jsonc` — the previous Worker

Still what is deployed until the Rust Worker is cut over; it is retired (and these two files
deleted) once a canary of the Rust Worker has been audited against it. It imports the page and the
banner from `public/`, so it stays buildable. Until then, if `ns/` changes, its two hand-pinned ETag
constants must be updated (first 32 hex chars of each file's sha256) — the Rust Worker has no such
step.

```console
$ cd site && wrangler deploy
```
