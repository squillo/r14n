# Contributing to r14n / RLPS

Thank you for your interest. This document covers **how** to contribute and the **licensing +
gating** rules that keep the project counsel-safe. **NOT LEGAL ADVICE** — see
[`docs/not-legal-advice.md`](docs/not-legal-advice.md).

> External contributions are welcome. For substantive changes, open a GitHub issue **before** a
> PR — see [Issue-first change process](#issue-first-change-process) below. Jurisdiction-claiming
> packs remain counsel-gated regardless (see [Pack contributions](#pack-contributions--the-gate)).

## Issue-first change process

For substantive discussion, open a GitHub issue before opening a PR so the design conversation
lives in the issue tracker. Three issue forms carry the process:

- **RFC** — any change to normative text: the spec, the resolution algorithm, catalog semantics,
  conformance levels, or schema shape. Problem first, proposal second, compatibility and the
  legal boundary after that.
- **Erratum** — published text that is simply wrong (spec, catalog, or schema). Quote the bytes.
- **Conformance disagreement** — your implementation disagrees with a published vector, schema,
  or the reference resolver. The spec is normative; where a vector and the spec conflict, the
  vector is the defect; where the spec is silent, that silence is itself a spec defect.

Review routing is codified in `.github/CODEOWNERS`; the PR template restates the approval bar at
the moment it applies (see `GOVERNANCE.md` §Change process). Normative changes land with a dated
entry in `CHANGELOG.md` and a Revision History entry in the spec.

## Developer Certificate of Origin (DCO)

Every commit must be signed off under the [DCO 1.1](https://developercertificate.org/) — add a
`Signed-off-by: Your Name <you@example.com>` trailer (git `-s`). Signing off certifies you wrote
the change (or have the right to submit it) under the project's license (below). We use DCO rather
than a CLA — no copyright assignment; you keep your copyright.

## License in = license out

By contributing you agree your contribution is licensed under the same terms as the part of the
repo it touches:

- **Code** (`/resolver`, `/tools`, `/scripts`, fuzz targets) → **Apache-2.0**. New `.rs` files
  carry an `SPDX-License-Identifier: Apache-2.0` header.
- **Spec, schemas, catalog, conformance vectors, registry schemas, vocabulary, prose docs** →
  **CC-BY-4.0**.
- **Packs** carry their own SPDX header (CC0-1.0 or CC-BY-4.0 for the data).

## Engineering rules

- Both crates must stay green: `cd resolver && cargo test` and `cd tools && cargo test`; schemas
  the schema-conformance tests (in `tools/tests/schema_conformance.rs`) must pass. CI
  (`.github/workflows/gates.yml`) enforces `cargo test`.
- **Every test carries a `/// Why:` doc** stating the invariant / spec section / regression it
  guards — never a restatement of the body.
- The **resolver stays generic** (serde + toml + serde_json + std, zero Squillo deps): a twin of
  it ships inside Squillo OS as the live first consumer. Flag any breaking resolver API/format
  change prominently in the PR and in `CHANGELOG.md` so downstream consumers can coordinate.
- **Bindings are generated from the Rust resolver — never hand-written.** Any TypeScript/JS or
  Python package this repo ships must be produced directly from `resolver/` (wasm-bindgen for
  JS/TS, PyO3 abi3 wheels for Python) or run the compiled wasm/WASI module — one behavior, one
  source of truth, nothing to drift. A hand-ported implementation is not accepted here even if
  its tests pass. This is distinct from **independent resolvers** (GOVERNANCE.md Stage 2), which
  are welcome — in their own repositories, proving themselves against `/conformance`; an in-repo
  binding wraps the reference implementation and is NOT an independent implementation.
- Follow the house style already in the tree (fully-qualified paths, Revision History blocks).

## Pack contributions — the gate

Packs are **federated-with-attestation** (spec §Governance):

- Anyone may contribute a **posture** or **example** pack (non-jurisdictional). It must lint clean
  (`r14n validate --catalog`), reference only catalog controls, and carry the NOT-LEGAL-ADVICE
  header. Example/demo profiles must be clearly marked NOT a jurisdiction claim (see
  `packs/example/`).
- A pack that **claims a real jurisdiction** MUST NOT be submitted here until the counsel gate
  opens. When it does, it MUST carry a `reviewing_attorney_of_record` + `jurisdiction` +
  `bar_credential`, be authored/attested by **licensed counsel**, and be signed by a key listed in
  the reviewer-key directory. Non-lawyer *draft* contributions to a future jurisdiction pack are
  advisory-only and gated behind attorney attestation before they carry any weight.

## Reporting security issues

Do **not** open a public issue — see [`SECURITY.md`](SECURITY.md).

## Code of conduct

This project follows the [Contributor Covenant 2.1](CODE_OF_CONDUCT.md); conduct reports go to
**legal@squillo.com**.
