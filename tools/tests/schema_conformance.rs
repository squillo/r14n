// SPDX-License-Identifier: Apache-2.0
//! Schema-conformance tests — validate the repo's PRODUCERS against the JSON
//! Schemas, in Rust (this project is Rust + N Lang; this replaces the former
//! `scripts/check-schemas.py`). Run by `cargo test`, so CI needs no Python.
//!
//! Covers: every schema is a valid Draft 2020-12 document; the shipped packs
//! (TOML → JSON) conform to `schema/pack.schema.json`; each conformance
//! `level-*.json` conforms to `conformance/vector.schema.json`; reviewer-key
//! directory fixtures conform to `registry/reviewer-key.schema.json`; and the
//! worked example receipt conforms to `schema/receipt.schema.json`; and every shipped
//! Control Catalog conforms to `schema/catalog.schema.json`.
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

/// Every shipped Control Catalog, repo-relative. Globbed, never listed: invariant 11 is
/// explicit that adding a published file must not require editing a list somewhere else.
fn shipped_catalogs() -> ::std::vec::Vec<::std::string::String> {
  let mut out: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for entry in ::std::fs::read_dir(root().join("catalog")).expect("read catalog/").flatten() {
    let p = entry.path();
    if let ::std::option::Option::Some(name) = p.file_name().and_then(|n| n.to_str()) {
      if name.ends_with(".catalog.toml") {
        out.push(::std::format!("catalog/{name}"));
      }
    }
  }
  out.sort();
  out
}

/// Why: the catalog is the only published r14n artifact that had no schema. Packs,
/// conformance vectors, reviewer directories and receipts all had one and were all
/// validated; the catalog — the jurisdiction-neutral vocabulary EVERY pack references —
/// had neither. That asymmetry matters more than it looks on a published standard: a
/// third party authoring a catalog for its own domain had no machine-readable contract to
/// check it against, while this repository enforced eight rules on its own catalogs in a
/// Rust test nobody outside can run.
///
/// The schema also carries two rules no layer above it could. `propertyNames` enforces the
/// control-key shape, which neither a `format` facet nor a regex prop pattern does
/// (measured 2026-10-26, both pinned as tripwires in the Snapp suites); and
/// `dependentRequired` makes `superseded_by` mandatory once `deprecated` is present, which
/// is invariant 10's deprecation contract stated declaratively rather than asserted in
/// Rust.
#[test]
fn shipped_catalogs_conform_to_catalog_schema() {
  let v = validator("schema/catalog.schema.json");
  let catalogs = shipped_catalogs();
  ::std::assert!(
    catalogs.len() >= 2,
    "expected the shipped catalogs, found {catalogs:?} — a broken glob would pass vacuously"
  );
  for rel in catalogs {
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

/// Counsel-gated jurisdiction tokens. Invariant 2 names the first two outright; the rest are
/// the regimes most likely to be reached for when someone wants a "realistic" example.
///
/// Deliberately NOT a general "looks like a law" matcher. A closed list of the names actually
/// at risk refuses the ones that matter and stays readable; a clever pattern would flag
/// `example/region` on its next revision and get deleted by whoever it blocked.
const COUNSEL_GATED: [&str; 8] =
  ["gdpr", "wiretap", "ccpa", "cpra", "hipaa", "coppa", "lgpd", "pipeda"];

/// Why: invariant 2 — "never author a real-jurisdiction pack … do not generate one, EVEN AS AN
/// EXAMPLE" — was prose only, and the conformance suite had been violating it. Two shipped
/// Level-2 vectors carried inline `gdpr/eu` pack fixtures, which is the exact string the
/// invariant names as counsel-gated and deliberately absent.
///
/// A vector fixture is the easiest place for this to happen and the worst place for it to
/// land: the packs are inline TOML, so no file under `packs/` exists to notice, and this
/// repository is PUBLIC — a published conformance vector naming `gdpr/eu` reads as a GDPR pack
/// existing, which is precisely the misreading invariant 2 exists to prevent. The grammar
/// these vectors prove is subtag truncation and delta-merge, and fictional subtags prove it
/// identically, so nothing was lost by renaming them to `example/region`.
///
/// Scans the whole serialized file rather than the pack keys alone: the jurisdiction can hide
/// in a profile string, an `inherits` value inside the inline TOML, or a description.
#[test]
fn no_conformance_vector_names_a_counsel_gated_jurisdiction() {
  let mut scanned = 0usize;
  for level in ["conformance/level-1.json", "conformance/level-2.json", "conformance/level-3.json"] {
    let raw = ::std::fs::read_to_string(root().join(level))
      .unwrap_or_else(|e| ::std::panic!("read {level}: {e}"));
    let lower = raw.to_lowercase();
    for token in COUNSEL_GATED {
      ::std::assert!(
        !lower.contains(token),
        "{level} names the counsel-gated jurisdiction {token:?}. Invariant 2 forbids authoring \
         a real-jurisdiction pack even as an example — use an explicitly fictional profile \
         (`example`, `example/region`) instead. The grammar a vector proves does not depend on \
         the subtag being real."
      );
    }
    scanned += 1;
  }
  ::std::assert_eq!(scanned, 3, "a level file was not scanned — the list or the paths moved");
}
