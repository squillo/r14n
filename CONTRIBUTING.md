# Contributing to r14n / RLPS

Thank you for your interest. This document covers **how** to contribute and the **licensing +
gating** rules that keep the project counsel-safe. **NOT LEGAL ADVICE** — see
[`docs/not-legal-advice.md`](docs/not-legal-advice.md).

> This repository is **PRIVATE** pending legal review (spec §Governance). External contribution
> opens when it goes public; the rules below are the standing policy.

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
  must pass `python scripts/check-schemas.py`. CI (`.github/workflows/gates.yml`) enforces this.
- **Every test carries a `/// Why:` doc** stating the invariant / spec section / regression it
  guards — never a restatement of the body.
- The **resolver stays generic** (serde + toml + serde_json + std, zero Squillo deps): it is a
  twin of Squillo's `the Squillo OS policy engine`. Coordinate any breaking resolver API/format change
  (see `the maintainer notes`).
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
