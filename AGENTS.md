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
  `schema/` carries three contracts: `pack.schema.json`, `catalog.schema.json` (added
  2026-10-27 — the catalog was the last published artifact with none) and
  `receipt.schema.json`. Each is validated, and every shipped artifact is validated
  against its own, by `tools/tests/schema_conformance.rs`.
- `resolver/` — the Rust reference resolver (crate `r14n`); `src/wire.rs` is the JSON boundary.
- `tools/` — the `r14n` CLI (crate `r14n-tools`).
- `bindings/` — generated JS/TS (`bindings/wasm` → `@squillo/r14n`) and Python
  (`bindings/python` → `r14n`) packages, plus `bindings/npm/build.mjs`.
- `site/` — the r14n.squillo.com Worker (splash + the `/ns` vocabulary host).
- `r14n Spec/`, `baselines/`, `snapp/` — the **N Lang Snapp tree**: the typed spec sources
  (`policies` keyword + `PolicyPack`/`Subject` blocks), a baseline-pack consumer Snapp that
  `@deps` them under an integrity pin, and the emitted ABI bundles. Sources are Apache-2.0;
  **N Lang itself is proprietary**, so none of it builds on public CI — see below.
- `GOVERNANCE.md`, `CONTRIBUTING.md`, `TRADEMARKS.md`, `SECURITY.md` — the rules.

## Build and test (exactly what CI runs)

```console
cargo test --locked --manifest-path resolver/Cargo.toml
cargo test --locked --manifest-path tools/Cargo.toml
cargo clippy --manifest-path resolver/Cargo.toml --all-targets -- -D warnings
# Pack lint — CI globs every catalog and validates its domain's packs, in both
# directions (every pack reaches a catalog; no pack exists without one). One domain:
cargo run --quiet --manifest-path tools/Cargo.toml -- validate \
  --catalog catalog/recording_consent.catalog.toml \
  packs/aggressive/recording_consent.r14n.toml packs/minimal/recording_consent.r14n.toml
node bindings/npm/build.mjs && node --test bindings/npm/smoke.test.mjs
maturin build --release --manifest-path bindings/python/Cargo.toml --out dist/wheels
```

### The Snapp tree — local only, and never unguarded

N Lang is proprietary, so **no public CI job compiles the Snapp tree**. CI instead proves
that the committed sources and the committed artifacts AGREE (`tools/tests/snapp_drift_gate.rs`, and
`resolver/tests/snapp_toml_parity.rs` for the one domain shipped both ways). Re-emitting is
a local step, and it is on you to run it in the same change as the source edit.
```console
# Reproducible dates first — see rule 4. Without this, every export below rewrites
# `@snapp.date` and so invalidates the integrity pin even when nothing changed.
export SOURCE_DATE_EPOCH="$(git log -1 --format=%ct -- 'r14n Spec/1.0.0' baselines/1.0.0)"

# check -> build -> export, for BOTH Snapps, framework first.
nlang check  -r "r14n Spec/1.0.0"
nlang export -r "r14n Spec/1.0.0" -f json     # writes snapp/r14n@<ver>.json
# The export above changes the framework bundle's bytes WHEN ITS CONTENT CHANGED, which
# INVALIDATES the integrity pin in baselines/1.0.0/lib.n.md. Refresh it in the same change:
#   shasum -a 256 "snapp/r14n@0.1.0-alpha.1.json"   -> paste as sha256-<hex>
nlang check  -r baselines/1.0.0
nlang test   -r baselines/1.0.0     # 23-arm node-validation suite; see rule 5
nlang export -r baselines/1.0.0 -f json
```

Five rules, each one something that has already gone wrong here:

1. **Never run `nlang` unguarded.** An unmonitored build can consume unbounded memory.
   Run it under a memory/time watchdog.
2. **Isolate the cache every invocation** — `NLANG_CACHE_DIR=$(mktemp -d) nlang …`. The
   global ABI cache is name-keyed, and once an entry exists a later export under that name
   can silently collapse to the `@snapp` header **at `rc=0`**.
3. **Gate on artifact CONTENT, never on `rc`.** A green build is not evidence the bundle is
   readable; only a green export is, and only if you look at what it wrote. `nlang build
   --emit-snapp` also drops a `.n.json` sidecar carrying **no `@snapp` root** — gitignored,
   because a consumer that globbed it would load a bundle with no identity.
4. **Set `SOURCE_DATE_EPOCH`, or every re-export churns the pin.** Both bundles embed
   `@snapp.date`, and `@snapp.integrity` is a content address over the whole artifact — `date`
   included. That is correct for a content address, but it means re-exporting with IDENTICAL
   sources changes the bytes, which invalidates every downstream `integrity` pin for a reason
   that has nothing to do with content. "The pin went stale" then cannot distinguish *the
   dependency changed* from *somebody re-ran export*.

   The toolchain derives `date` from `SOURCE_DATE_EPOCH` when it is set, which makes the
   emitted artifact byte-reproducible. Measured on `nlang 0.1.0-dev (c87e92fd6)`: two exports
   at the same epoch produce identical bytes; without it, a no-op re-export differs in exactly
   `date` and `integrity` with all pack and type content unchanged.

   The convention here is the commit time of the Snapp sources, so the date means *when this
   content was last authored* rather than *when someone happened to run a build*:

   ```console
   export SOURCE_DATE_EPOCH="$(git log -1 --format=%ct -- 'r14n Spec/1.0.0' baselines/1.0.0)"
   ```

   ⚠ One wrinkle, stated rather than hidden: that value moves when you COMMIT the source
   change, so an emit-then-commit-then-emit sequence will move the date once without any
   content change. Re-emit before committing, and record the epoch in the commit message when
   it matters — the property this buys is that a NO-OP re-export is free, not that the date is
   derivable from nothing.

5. **Run `nlang test`, and read what it tells you about the TYPES.** `baselines/1.0.0`
   carries a node-validation suite (`policy_pack_tests.n`, 23 arms) in which the compiler is
   the assertion engine. Two things about it are load-bearing:

   **Every asserted type carries BOTH a positive and a negative arm.** An unresolvable
   `<Type>` makes a positive node test pass UNCONDITIONALLY, so a suite of positives alone
   cannot distinguish "this shape is valid" from "this type no longer resolves and nothing is
   being checked". The negative arm is the only thing that makes the positives mean anything.
   Keep the pairing when you add a type.

   **Three declared constraints are enforced by NOTHING, measured 2026-10-26**, and the suite
   pins each as a tripwire rather than pretending otherwise:

   - `Subject.key`'s `format` regex — still not applied across `@deps`, so the TYPE does not
     hold it. ✅ **The published schema now does**: `schema/pack.schema.json`'s `subject`
     gained a `propertyNames` pattern on 2026-10-27, so a malformed subject key no longer
     validates in every layer. The tripwire stays because the type-level gap is unchanged; it
     records that the enforcement moved, not that it arrived. A key is matched verbatim by a
     resolver, so a case mismatch is a row no query finds — and under `as_configured` that
     fails closed to the caller's whole universe, surfacing as silent over-requiring rather
     than an error.
   - `PolicyPack.subject`'s `min_items = 1` — unenforced, and whether it SHOULD be enforced
     is genuinely open: it is right under `as_configured` and wrong under `minimal`, where
     subject tables are ignored and a floor-only pack is coherent. The constraint is declared
     per-type but its correctness depends on the posture, which the type cannot see.
   - `Subject.controls`'s `min_items = 1` — ⚠ here the DECLARATION is the defect, not the
     missing enforcement. Four shipped packs require an empty subject control list:
     `minimal/skill_egress` (approved 2026-09-30) and the three opt-out `analytics_consent`
     packs. Under `as_configured` an ABSENT subject fails closed to the caller's whole
     universe, so an empty control list is the only way to say "contemplated, requires
     nothing" — omitting the subject says the opposite. If a compiler ever enforces this,
     remove `min_items = 1` from the type; do not change those packs.

   Scalar atom kind is likewise enforced where a type is DECLARED and not across `@deps`; two
   further tripwires pin that. When a tripwire reddens, the compiler got stricter: delete it
   and restore the real assertion named in its comment.

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
8. **Co-authorship names Squillo, never an outside vendor.** Machine-assisted commits carry
   `Co-Authored-By: Squillo Code <320728527+squillo-code@users.noreply.github.com>` — and no
   outside tool or vendor name anywhere, in the trailer or the prose. The sign-off (DCO) is
   separate and required. A trailer is how a forge decides who appears on this project's public
   contributor list, so a vendor credited there becomes a contributor to a standards project.
   CI checks every commit message over full history.
9. **DCO on every commit** (`git commit -s`), and the `resolver` crate stays dependency-lean:
   `serde` + `serde_json` + `toml` + `std`, nothing else.
10. **Additive by default.** The Control Catalog grows; renames and removals are deprecations,
   because a control key is long-lived audit evidence. New terms enter `docs/namespace.md`
   before first shipped use.
11. **Nothing ships that nothing checks.** A file this repository publishes must be reachable
   by something that validates it, and adding one must not require editing a list somewhere
   else — a hand-maintained list is how a pack ships unvalidated and a Snapp source sits
   compiled by nothing. Concretely: a new pack needs a catalog for its domain (CI globs both
   directions); a new Snapp source needs a `mod` declaration in its index, because discovery
   is index-only with **no filesystem-walk fallback**; a re-emitted bundle needs every
   dependent's integrity pin refreshed in the same change. This repository has shipped each
   of those three broken, and each was silent.

## Change process

Substantive changes start as a GitHub issue, not a PR — **RFC** (normative text), **Erratum**
(published text is wrong), or **Conformance disagreement**. Normative changes need two
maintainer approvals and a dated `CHANGELOG.md` entry plus a spec Revision History banner. See
`CONTRIBUTING.md` and `GOVERNANCE.md`.
