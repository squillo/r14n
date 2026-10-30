// SPDX-License-Identifier: Apache-2.0
//! Helpers shared by the integration tests in this directory.
//!
//! Cargo builds every `tests/*.rs` as its OWN crate, so a helper defined in two test files is
//! two copies that can drift apart. Anything here was found defined, with identical
//! semantics, in two or more of them. A helper that merely LOOKS similar but behaves
//! differently (a different panic message, error handling or filter) stays in its file: merging
//! those would be a behaviour change smuggled into a refactor.
//!
//! Use it as `mod common;` and call `common::root()` — fully-qualified, no module-top `use`,
//! the house style of this tree. Not a test file: Cargo does not discover `tests/*/mod.rs`
//! as a test target, so nothing here runs on its own.
//!
//! NOT LEGAL ADVICE.

// Each test crate compiles this module separately and calls only a subset of it, so a helper
// that is unused in one crate is expected; without this, `clippy -D warnings` rejects it.
#![allow(dead_code)]

/// Repo root (the tools crate's parent).
pub fn root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A repo-relative file as text; panics with the path on failure so a moved fixture names itself.
pub fn read(rel: &str) -> ::std::string::String {
  let p = root().join(rel);
  ::std::fs::read_to_string(&p).unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()))
}

/// A repo-relative JSON file, parsed; names the path on a read or parse failure.
pub fn load_json(rel: &str) -> ::serde_json::Value {
  let path = root().join(rel);
  let raw = ::std::fs::read_to_string(&path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
  ::serde_json::from_str(&raw).unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()))
}

/// A compiled Draft 2020-12 validator for a repo-relative schema file.
pub fn validator(schema_rel: &str) -> ::jsonschema::Validator {
  let schema = load_json(schema_rel);
  ::jsonschema::draft202012::options()
    .build(&schema)
    .unwrap_or_else(|e| ::std::panic!("{schema_rel} is not a valid Draft 2020-12 schema: {e}"))
}

/// Every validation error for an instance, as text.
pub fn errors(
  v: &::jsonschema::Validator,
  instance: &::serde_json::Value,
) -> ::std::vec::Vec<::std::string::String> {
  v.iter_errors(instance).map(|e| e.to_string()).collect()
}

/// TOML text as a JSON value, for validating a TOML artifact against a JSON Schema.
pub fn toml_text_as_json(text: &str) -> ::serde_json::Value {
  let toml_val: ::toml::Value =
    ::toml::from_str(text).unwrap_or_else(|e| ::std::panic!("not valid TOML: {e}\n{text}"));
  ::serde_json::to_value(toml_val).unwrap_or_else(|e| ::std::panic!("toml→json: {e}"))
}
