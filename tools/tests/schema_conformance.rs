// SPDX-License-Identifier: Apache-2.0
//! Schema-conformance tests — validate the repo's PRODUCERS against the JSON
//! Schemas, in Rust (this project is Rust + N Lang; this replaces the former
//! `scripts/check-schemas.py`). Run by `cargo test`, so CI needs no Python.
//!
//! Covers: every schema is a valid Draft 2020-12 document; the shipped packs
//! (TOML → JSON) conform to `schema/pack.schema.json`; each conformance
//! `level-*.json` conforms to `conformance/vector.schema.json`; reviewer-key
//! directory fixtures conform to `registry/reviewer-key.schema.json`; and the
//! worked example receipt conforms to `schema/receipt.schema.json`.
//!
//! Revision History
//! - 2026-07-08: authored — de-Python: port scripts/check-schemas.py to Rust.

/// Repo root (the tools crate's parent).
fn root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load_json(rel: &str) -> ::serde_json::Value {
  let path = root().join(rel);
  let raw = ::std::fs::read_to_string(&path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
  ::serde_json::from_str(&raw).unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()))
}

/// Build a Draft-2020-12 validator for a schema file (also asserts the schema
/// itself is well-formed — a broken schema fails the build).
fn validator(schema_rel: &str) -> ::jsonschema::Validator {
  let schema = load_json(schema_rel);
  ::jsonschema::draft202012::options()
    .build(&schema)
    .unwrap_or_else(|e| ::std::panic!("{schema_rel} is not a valid Draft 2020-12 schema: {e}"))
}

/// Parse a TOML file at `rel` into a JSON value (packs are authored in TOML but
/// validated against JSON Schema).
fn toml_as_json(rel: &str) -> ::serde_json::Value {
  let path = root().join(rel);
  let text = ::std::fs::read_to_string(&path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
  let toml_val: ::toml::Value =
    ::toml::from_str(&text).unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()));
  ::serde_json::to_value(toml_val)
    .unwrap_or_else(|e| ::std::panic!("{} toml→json: {e}", path.display()))
}

fn errors(v: &::jsonschema::Validator, instance: &::serde_json::Value) -> ::std::vec::Vec<::std::string::String> {
  v.iter_errors(instance).map(|e| e.to_string()).collect()
}

/// Every `packs/**/*.r14n.toml` in the repo (relative to root).
fn shipped_packs() -> ::std::vec::Vec<::std::string::String> {
  let mut out = ::std::vec::Vec::new();
  fn walk(dir: &::std::path::Path, out: &mut ::std::vec::Vec<::std::path::PathBuf>) {
    if let ::std::result::Result::Ok(entries) = ::std::fs::read_dir(dir) {
      for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
          walk(&p, out);
        } else if p.to_string_lossy().ends_with(".r14n.toml") {
          out.push(p);
        }
      }
    }
  }
  let mut abs = ::std::vec::Vec::new();
  walk(&root().join("packs"), &mut abs);
  let base = root();
  for p in abs {
    out.push(p.strip_prefix(&base).unwrap_or(&p).to_string_lossy().replace('\\', "/"));
  }
  out.sort();
  out
}

/// Why: the pack JSON Schema is documented as the pack contract — if the shipped
/// baseline/example packs stopped conforming (or the schema drifted), authors
/// copying them would inherit an invalid template. TOML→JSON then validate.
#[test]
fn shipped_packs_conform_to_pack_schema() {
  let v = validator("schema/pack.schema.json");
  let packs = shipped_packs();
  ::std::assert!(packs.len() >= 3, "expected the baseline + example packs, found {packs:?}");
  for rel in packs {
    let errs = errors(&v, &toml_as_json(&rel));
    ::std::assert!(errs.is_empty(), "{rel}: {errs:?}");
  }
}

/// Why: a malformed vector file or a typo'd capability would otherwise be caught
/// only by the runner's skip-count drift — the schema catches the shape at the
/// file level, and this pins that all three level files conform.
#[test]
fn conformance_vector_files_conform_to_vector_schema() {
  let v = validator("conformance/vector.schema.json");
  for level in ["conformance/level-1.json", "conformance/level-2.json", "conformance/level-3.json"] {
    let errs = errors(&v, &load_json(level));
    ::std::assert!(errs.is_empty(), "{level}: {errs:?}");
  }
}

/// Why: the reviewer-key directory is the trust root; a fixture that drifted from
/// its schema would let `verify --directory` behavior diverge from the documented
/// shape. Fixtures exercise key ENTRIES, so a missing `steward` block is tolerated
/// (matching the former Python check), but every other rule must hold.
#[test]
fn reviewer_directory_fixtures_conform_to_reviewer_key_schema() {
  let v = validator("registry/reviewer-key.schema.json");
  for level in ["conformance/level-1.json", "conformance/level-2.json", "conformance/level-3.json"] {
    let doc = load_json(level);
    for vector in doc["vectors"].as_array().expect("vectors") {
      let dir = &vector["reviewer_directory"];
      if dir.is_null() {
        continue;
      }
      let errs: ::std::vec::Vec<::std::string::String> =
        errors(&v, dir).into_iter().filter(|e| !e.contains("steward")).collect();
      let name = vector["name"].as_str().unwrap_or("?");
      ::std::assert!(errs.is_empty(), "{level}:{name}.reviewer_directory: {errs:?}");
    }
  }
}

/// Why: the shipped worked-example receipt is documentation integrators copy; it
/// must validate against the receipt schema (the golden-guard pins its bytes,
/// this pins its shape against the published contract).
#[test]
fn example_receipt_conforms_to_receipt_schema() {
  let v = validator("schema/receipt.schema.json");
  let example = load_json("docs/examples/receipt-ai-act-50.json");
  let errs = errors(&v, &example["dpv_27560"]);
  ::std::assert!(errs.is_empty(), "example dpv_27560: {errs:?}");
}
