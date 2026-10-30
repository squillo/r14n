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

mod common;

/// Parse a TOML file at `rel` into a JSON value (packs are authored in TOML but
/// validated against JSON Schema).
fn toml_as_json(rel: &str) -> ::serde_json::Value {
  let path = common::root().join(rel);
  let text = ::std::fs::read_to_string(&path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
  let toml_val: ::toml::Value =
    ::toml::from_str(&text).unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()));
  ::serde_json::to_value(toml_val)
    .unwrap_or_else(|e| ::std::panic!("{} toml→json: {e}", path.display()))
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
  walk(&common::root().join("packs"), &mut abs);
  let base = common::root();
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
  let v = common::validator("schema/pack.schema.json");
  let packs = shipped_packs();
  ::std::assert!(packs.len() >= 3, "expected the baseline + example packs, found {packs:?}");
  for rel in packs {
    let errs = common::errors(&v, &toml_as_json(&rel));
    ::std::assert!(errs.is_empty(), "{rel}: {errs:?}");
  }
}

/// Every shipped Control Catalog, repo-relative. Globbed, never listed: invariant 11 is
/// explicit that adding a published file must not require editing a list somewhere else.
fn shipped_catalogs() -> ::std::vec::Vec<::std::string::String> {
  let mut out: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for entry in ::std::fs::read_dir(common::root().join("catalog")).expect("read catalog/").flatten() {
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
  let v = common::validator("schema/catalog.schema.json");
  let catalogs = shipped_catalogs();
  ::std::assert!(
    catalogs.len() >= 2,
    "expected the shipped catalogs, found {catalogs:?} — a broken glob would pass vacuously"
  );
  for rel in catalogs {
    let errs = common::errors(&v, &toml_as_json(&rel));
    ::std::assert!(errs.is_empty(), "{rel}: {errs:?}");
  }
}

/// One valid catalog with EVERY control facet populated, so each negative below changes one thing
/// against a base that is known to validate. Fictional domain and keys.
const FULL_CATALOG: &str = r#"
[domain]
name = "example"
description = "A fictional catalog used only by schema tests."

[control.example_key]
kind = "obligation"
title = "Example control"
trigger = "when the example happens"
mechanism = "the example mechanism"
verified_by = "an example check"
attestation_mode = "self"
subject_rights = ["access"]
legal_basis_hint = "an example hint"

[control.example_old]
kind = "permit"
title = "Retired example"
deprecated = "2026-01-01"
superseded_by = ["example_key"]
"#;

/// Why: `shipped_catalogs_conform_to_catalog_schema` proves the schema ACCEPTS what ships; nothing
/// proved it REFUSES anything. Each keyword in `catalog.schema.json` is a rule a test elsewhere
/// used to restate in Rust and no longer does, so a keyword deleted from the schema would turn
/// nothing red: the shipped catalogs would still validate. One row per rule: `edit` is applied to
/// the valid base, and the schema must refuse the result FOR the stated reason (a fixture that is
/// refused for an unrelated defect would pass for the wrong reason). The control is the base itself.
#[test]
fn the_catalog_schema_refuses_each_malformed_shape() {
  let v = common::validator("schema/catalog.schema.json");
  let base = common::errors(&v, &common::toml_text_as_json(FULL_CATALOG));
  ::std::assert!(base.is_empty(), "control: the full base catalog must validate: {base:?}");

  // (what, text to find in the base, replacement, why — a substring of the schema's own error)
  let rows: [(&str, &str, &str, &str); 25] = [
    ("domain table missing", "[domain]\nname = \"example\"\ndescription = \"A fictional catalog used only by schema tests.\"\n", "", "\"domain\" is a required property"),
    ("domain.name is not a control-key shape", "name = \"example\"", "name = \"Example\"", "does not match"),
    ("domain.description missing", "description = \"A fictional catalog used only by schema tests.\"\n", "", "\"description\" is a required property"),
    ("unknown key inside domain", "name = \"example\"", "name = \"example\"\nowner = \"x\"", "Additional properties are not allowed"),
    ("control key starts with a digit", "[control.example_key]", "[control.\"1example\"]", "does not match"),
    ("control key has an uppercase letter", "[control.example_key]", "[control.Example_key]", "does not match"),
    ("control key has a hyphen", "[control.example_key]", "[control.\"example-key\"]", "does not match"),
    ("control key is 65 characters", "[control.example_key]", "[control.a2345678901234567890123456789012345678901234567890123456789012345]", "does not match"),
    ("kind outside the closed set", "kind = \"obligation\"", "kind = \"requirement\"", "is not one of"),
    ("kind missing", "kind = \"obligation\"\n", "", "\"kind\" is a required property"),
    ("title empty", "title = \"Example control\"", "title = \"\"", "is shorter than 1 character"),
    ("title missing", "title = \"Example control\"\n", "", "\"title\" is a required property"),
    ("unknown control property", "kind = \"obligation\"", "kind = \"obligation\"\nseverity = \"high\"", "Additional properties are not allowed"),
    ("trigger empty", "trigger = \"when the example happens\"", "trigger = \"\"", "is shorter than 1 character"),
    ("attestation_mode empty", "attestation_mode = \"self\"", "attestation_mode = \"\"", "is shorter than 1 character"),
    ("mechanism empty", "mechanism = \"the example mechanism\"", "mechanism = \"\"", "is shorter than 1 character"),
    ("verified_by empty", "verified_by = \"an example check\"", "verified_by = \"\"", "is shorter than 1 character"),
    ("legal_basis_hint empty", "legal_basis_hint = \"an example hint\"", "legal_basis_hint = \"\"", "is shorter than 1 character"),
    ("subject_rights empty list", "subject_rights = [\"access\"]", "subject_rights = []", "has less than 1 item"),
    ("subject_rights duplicate", "subject_rights = [\"access\"]", "subject_rights = [\"access\", \"access\"]", "non-unique elements"),
    ("deprecated is not a zero-padded date", "deprecated = \"2026-01-01\"", "deprecated = \"2026-1-1\"", "does not match"),
    ("deprecated is not a date at all", "deprecated = \"2026-01-01\"", "deprecated = \"soon\"", "does not match"),
    ("deprecated without superseded_by", "superseded_by = [\"example_key\"]\n", "", "superseded_by"),
    ("superseded_by empty list", "superseded_by = [\"example_key\"]", "superseded_by = []", "has less than 1 item"),
    ("superseded_by names an empty key", "superseded_by = [\"example_key\"]", "superseded_by = [\"\"]", "is shorter than 1 character"),
  ];
  for (what, find, replace, why) in rows {
    ::std::assert_eq!(
      FULL_CATALOG.matches(find).count(),
      1,
      "{what}: the text to edit must occur exactly once in the base, or this row mutates nothing"
    );
    let edited = FULL_CATALOG.replacen(find, replace, 1);
    let errs = common::errors(&v, &common::toml_text_as_json(&edited));
    ::std::assert!(!errs.is_empty(), "{what}: expected REFUSED but the schema accepted it");
    ::std::assert!(
      errs.iter().any(|e| e.contains(why)),
      "{what}: refused, but not for the stated reason {why:?}; the schema said {errs:?}"
    );
  }

  // The two catalog-level rules that are not a one-line edit of the base.
  let no_controls = "[domain]\nname = \"example\"\ndescription = \"d\"\n";
  let errs = common::errors(&v, &common::toml_text_as_json(no_controls));
  ::std::assert!(errs.iter().any(|e| e.contains("\"control\" is a required property")), "{errs:?}");
  let empty_controls = "[domain]\nname = \"example\"\ndescription = \"d\"\n\n[control]\n";
  let errs = common::errors(&v, &common::toml_text_as_json(empty_controls));
  ::std::assert!(errs.iter().any(|e| e.contains("has less than 1 property")), "{errs:?}");
  let stray = ::std::format!("{FULL_CATALOG}\n[extra]\nx = 1\n");
  let errs = common::errors(&v, &common::toml_text_as_json(&stray));
  ::std::assert!(errs.iter().any(|e| e.contains("Additional properties are not allowed")), "{errs:?}");
}

/// Why: a malformed vector file or a typo'd capability would otherwise be caught
/// only by the runner's skip-count drift — the schema catches the shape at the
/// file level, and this pins that all three level files conform.
#[test]
fn conformance_vector_files_conform_to_vector_schema() {
  let v = common::validator("conformance/vector.schema.json");
  for level in ["conformance/level-1.json", "conformance/level-2.json", "conformance/level-3.json"] {
    let errs = common::errors(&v, &common::load_json(level));
    ::std::assert!(errs.is_empty(), "{level}: {errs:?}");
  }
}

/// Why: the reviewer-key directory is the trust root; a fixture that drifted from
/// its schema would let `verify --directory` behavior diverge from the documented
/// shape. Fixtures exercise key ENTRIES, so a missing `steward` block is tolerated
/// (matching the former Python check), but every other rule must hold.
#[test]
fn reviewer_directory_fixtures_conform_to_reviewer_key_schema() {
  let v = common::validator("registry/reviewer-key.schema.json");
  for level in ["conformance/level-1.json", "conformance/level-2.json", "conformance/level-3.json"] {
    let doc = common::load_json(level);
    for vector in doc["vectors"].as_array().expect("vectors") {
      let dir = &vector["reviewer_directory"];
      if dir.is_null() {
        continue;
      }
      let errs: ::std::vec::Vec<::std::string::String> =
        common::errors(&v, dir).into_iter().filter(|e| !e.contains("steward")).collect();
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
  let v = common::validator("schema/receipt.schema.json");
  let example = common::load_json("docs/examples/receipt-ai-act-50.json");
  let errs = common::errors(&v, &example["dpv_27560"]);
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
    let raw = ::std::fs::read_to_string(common::root().join(level))
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
