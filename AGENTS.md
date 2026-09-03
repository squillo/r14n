# AGENTS.md — orientation for coding agents

This file is the entry point for agents following the [agents.md](https://agents.md) convention
(Codex and others). It carries **orientation, commands, and the invariants you must not break**.
Deep protocol knowledge lives in [`skills/spec/SKILL.md`](skills/spec/SKILL.md) — the same file
the plugin in [`.claude-plugin/`](.claude-plugin/) loads, written in the open Agent Skills
format, so both ecosystems read **one** knowledge source. This file deliberately does not
duplicate it.

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange and configuration standard. Nothing you
> generate here may state what the law is or claim that a configuration is lawful.
> **"aggressive" ≠ "compliant."** See [`docs/not-legal-advice.md`](docs/not-legal-advice.md).

## Repo map

- `spec/RLPS-v0.1.md` — the normative specification (RFC 2119).
- `catalog/`, `packs/`, `schema/`, `conformance/`, `registry/`, `ns/` — the published artifacts.
- `resolver/` — the Rust reference resolver (crate `r14n`); `src/wire.rs` is the JSON boundary.
- `tools/` — the `r14n` CLI (crate `r14n-tools`).
- `bindings/` — generated JS/TS (`bindings/wasm` → `@squillo/r14n`) and Python
  (`bindings/python` → `r14n`) packages, plus `bindings/npm/build.mjs`.
- `site/` — the r14n.squillo.com Worker (splash + the `/ns` vocabulary host).
- `GOVERNANCE.md`, `CONTRIBUTING.md`, `TRADEMARKS.md`, `SECURITY.md` — the rules.

## Build and test (exactly what CI runs)

```console
cargo test --locked --manifest-path resolver/Cargo.toml
cargo test --locked --manifest-path tools/Cargo.toml
cargo clippy --manifest-path resolver/Cargo.toml --all-targets -- -D warnings
cargo run --quiet --manifest-path tools/Cargo.toml -- validate \
  packs/aggressive/recording_consent.r14n.toml \
  packs/minimal/recording_consent.r14n.toml \
  packs/example/region/recording_consent.r14n.toml \
  --catalog catalog/recording_consent.catalog.toml
node bindings/npm/build.mjs && node --test bindings/npm/smoke.test.mjs
maturin build --release --manifest-path bindings/python/Cargo.toml --out dist/wheels
```

There is **no rustfmt gate** — the tree follows a documented hand-rolled house style
(fully-qualified paths, two-space indent). Match the surrounding code; do not reformat it.

## Invariants — break these and CI (or a verifier somewhere) breaks you

1. **Fail-closed is the point.** A missing, malformed, out-of-envelope, or floor-less pack
   resolves to the caller's entire universe with `provenance.fell_back = true` — never an
   empty set, never an error from the port. Do not "fix" a fallback into a permissive default.
2. **Never author a real-jurisdiction pack.** `wiretap/us`, `gdpr/eu`, and every other real
   jurisdiction are counsel-gated and deliberately absent. A pack claiming a jurisdiction needs
   a named attorney of record. Do not generate one, even as an example — use
   `packs/example/region/` (explicitly fictional) to demonstrate profile grammar.
3. **The disclaimer travels with every artifact.** Packs, receipts, docs, and generated output
   all carry NOT-LEGAL-ADVICE. Do not remove it, weaken it, or move it below the fold.
4. **Bindings are generated, and their shims stay thin.** TS/JS and Python come from the Rust
   core (wasm-bindgen, PyO3). The JSON boundary lives in `resolver/src/wire.rs` and is tested
   there once. Logic added to a shim is logic the other language never gets — put it in the core.
5. **Every test carries a `/// Why:` doc** stating the invariant, spec section, or regression it
   guards — never a restatement of the body.
6. **Conformance claims are governed wording.** "conforms to RLPS Level N,
   rlps-conformance/0.1" only when every vector for that level passes; otherwise "implements
   Level N; Level N+1 in progress". Never a bare "RLPS conformant" (`TRADEMARKS.md`).
7. **Sanitization CI.** Every push greps tracked files for internal markers — internal
   design-doc ids, private component names, session ids, personal emails, local paths, and
   outside-vendor names. Do not introduce them, in files or in commit messages.
8. **No authorship trailers, ever.** A commit message describes what changed and why — nothing
   else. No co-authorship or generated-by trailer naming anyone or anything, and no tool or
   vendor named in the prose. The sign-off (DCO) is the sole exception and is required. A trailer
   is how a forge decides who appears on the project's public contributor list; one added out of
   habit attaches a third party to this project's provenance. CI checks every commit message.
9. **DCO on every commit** (`git commit -s`), and the `resolver` crate stays dependency-lean:
   `serde` + `serde_json` + `toml` + `std`, nothing else.
10. **Additive by default.** The Control Catalog grows; renames and removals are deprecations,
   because a control key is long-lived audit evidence. New terms enter `docs/namespace.md`
   before first shipped use.

## Change process

Substantive changes start as a GitHub issue, not a PR — **RFC** (normative text), **Erratum**
(published text is wrong), or **Conformance disagreement**. Normative changes need two
maintainer approvals and a dated `CHANGELOG.md` entry plus a spec Revision History banner. See
`CONTRIBUTING.md` and `GOVERNANCE.md`.
