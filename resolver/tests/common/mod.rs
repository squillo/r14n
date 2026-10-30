// SPDX-License-Identifier: Apache-2.0
//! Helpers shared by the integration tests in this directory.
//!
//! Cargo builds every `tests/*.rs` as its OWN crate, so a helper defined in two test files is
//! two copies that can drift apart. Anything here was found defined, with identical
//! semantics, in two or more of them. A helper that merely LOOKS similar but behaves
//! differently stays in its file: merging those would be a behaviour change smuggled into a
//! refactor.
//!
//! Use it as `mod common;` and call `common::repo_root()` — fully-qualified, no module-top
//! `use`, the house style of this crate. Not a test file: Cargo does not discover
//! `tests/*/mod.rs` as a test target, so nothing here runs on its own.
//!
//! NOT LEGAL ADVICE.

// Each test crate compiles this module separately and calls only a subset of it, so a helper
// that is unused in one crate is expected; without this, `clippy -D warnings` rejects it.
#![allow(dead_code)]

/// Repo root (the resolver crate's parent). Four test files used to compute this path
/// themselves, each with a different tail (`packs`, `conformance`, `docs/examples`, `snapp`).
pub fn repo_root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("..")
}
