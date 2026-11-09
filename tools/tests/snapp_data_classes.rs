// SPDX-License-Identifier: Apache-2.0
//! Snapp-declaration tests, Snapp-tree side — the closed data-class vocabulary (`DataClass`,
//! `DataClassSet`) and the closed-set prop `data_class_set` on `SnappCompliance`. Asserted against the
//! committed SOURCES and the committed bundle, never against a compiler: N Lang is proprietary and this
//! repository's CI runs on public runners, so a gate here cannot invoke `nlang` (see
//! `snapp_drift_gate.rs`).
//!
//! WHY THIS IS A SEPARATE FILE FROM `snapp_declarations.rs`. Every test here reads a file that exists
//! only once the Snapp-tree change has been applied and the bundle re-emitted (the data-class keyword
//! document, the keyword index entry, the closed-set prop, the re-exported bundle). A test that reads an
//! absent file is red, and a red test is never ignored or softened to make a change land. So the catalog
//! and section 7.3 tests (`snapp_declarations.rs`) landed with the catalogs, and these land with the
//! Snapp-tree change, in the same change as the files they read.
//!
//! WHAT THIS CAN AND CANNOT PROVE. It proves the vocabulary is the one that was designed (the nine
//! classes, in order, each an OPTIONAL `DataClass` prop of a closed `DataClassSet` with a description,
//! none a profile and none carrying a criterion identifier), that the closed-set prop sits LAST and
//! OPTIONAL beside the unchanged required pair, and that the types stay declared in `*`, where a
//! dependent Snapp's `use { $::* }` reaches them. It CANNOT prove the N Lang forms compile, or that an
//! undeclared class is refused: those are the arms in `r14n Spec/1.0.0/smoke_tests/type_constraint_tests.n`
//! (spec root) and `baselines/1.0.0/data_class_consumer_tests.n` (across the dependency), run by
//! `nlang test`, and their measured results are tabulated in the data-class keyword document
//! (`DATA_CLASS_DOC` below).
//!
//! Style note: FQ paths, no module-top `use` — the house style of this tree.
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-11-03: authored — split out of the first proposal of `snapp_declarations.rs`: the data-class
//!   vocabulary tests, the typed-path prop tests and their helpers.
//! - 2026-11-08: rewritten for the closed-set form, the measured fallback. The typed-path design did not
//!   compile (a keyed instance cannot be written `pub`), so the vocabulary is the nine OPTIONAL props of
//!   `DataClassSet` and the prop is `data_class_set`. The member helpers and
//!   `every_data_class_member_is_declared_pub` are withdrawn with the members. Two tests are new:
//!   `the_data_class_types_are_closed_and_the_set_has_no_min_props`, and
//!   `the_compliance_types_are_declared_in_star_so_a_dependent_snapp_reaches_them`, which pins a
//!   measurement: a type declared at the root (`pub blocks`) was not reached by a dependent Snapp's
//!   `use { $::* }`.

mod common;

const DATA_CLASS_DOC: &str =
  "r14n Spec/1.0.0/R14N_SPEC/R14N_SPEC_KEYWORDS/R14N_SPEC_KEYWORD_DATA_CLASSES_1_0_0.n.md";
const SNAPP_COMPLIANCE_DOC: &str =
  "r14n Spec/1.0.0/R14N_SPEC/R14N_SPEC_KEYWORDS/R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0.n.md";
const KEYWORD_INDEX: &str = "r14n Spec/1.0.0/R14N_SPEC/R14N_SPEC_KEYWORDS/mod.n.md";

/// The closed first list, in contract order. LAST-position adds only: a new class is appended here
/// AND in the document, and an existing one is never reordered, renamed or removed.
const DATA_CLASSES: [&str; 9] = [
  "phi",
  "pii",
  "financial",
  "biometric",
  "student_record",
  "credentials",
  "location",
  "communications",
  "telemetry",
];

/// The declaration line of each type, in `*`. A dependent Snapp imports this Snapp's types with
/// `use { $::* }`, and a type declared at the root (`pub blocks "X" {`) was measured NOT to be reached
/// that way (the data-class keyword document, "What the types enforce").
const DATA_CLASS_DECL: &str = "pub mod * blocks \"DataClass\" {";
const DATA_CLASS_SET_DECL: &str = "pub mod * blocks \"DataClassSet\" {";
const SNAPP_COMPLIANCE_DECL: &str = "pub mod * blocks \"SnappCompliance\" {";

/// A subject token has the grammar of `Subject.key`: `^[a-z][a-z0-9_]{0,63}$`. A `/` can never
/// appear in one, which is what keeps a data class from being spelled as a profile.
fn is_subject_token(s: &str) -> bool {
  let mut chars = s.chars();
  match chars.next() {
    ::std::option::Option::Some(c) if c.is_ascii_lowercase() => {}
    _ => return false,
  }
  s.len() <= 64 && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The text of one type declaration: from the line that equals `decl` once trimmed, to the first line
/// that is just `}` at the left margin.
fn block_source(doc: &str, decl: &str) -> ::std::string::String {
  let mut out = ::std::string::String::new();
  let mut inside = false;
  for line in doc.lines() {
    if !inside {
      if line.trim() == decl {
        inside = true;
        out.push_str(line);
        out.push('\n');
      }
      continue;
    }
    out.push_str(line);
    out.push('\n');
    if line == "}" {
      break;
    }
  }
  ::std::assert!(inside, "no declaration `{decl}` found");
  out
}

/// The props of a declaration, in source order: each `<name>? {` or `<name> {` line at the props
/// indent (four spaces), with the text up to its closing `    }`. The name keeps its `?`.
fn props_in_order(block: &str) -> ::std::vec::Vec<(::std::string::String, ::std::string::String)> {
  let mut out: ::std::vec::Vec<(::std::string::String, ::std::string::String)> = ::std::vec::Vec::new();
  let mut current: ::std::option::Option<(::std::string::String, ::std::string::String)> =
    ::std::option::Option::None;
  for line in block.lines() {
    if let ::std::option::Option::Some((_, body)) = current.as_mut() {
      body.push_str(line);
      body.push('\n');
      if line == "    }" {
        out.push(current.take().expect("an open prop"));
      }
      continue;
    }
    let is_prop_head = line.starts_with("    ")
      && !line.starts_with("     ")
      && line.ends_with(" {")
      && !line.trim_start().starts_with("//");
    if is_prop_head {
      let name = line.trim().trim_end_matches('{').trim();
      current = ::std::option::Option::Some((
        ::std::string::String::from(name),
        ::std::format!("{line}\n"),
      ));
    }
  }
  out
}

/// `text` with every whitespace character removed, so `priority    = 3` reads `priority=3`.
fn squeeze(text: &str) -> ::std::string::String {
  text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The one committed `r14n@*.json` bundle that carries an `@snapp` root, parsed. The build-door
/// sidecar carries none, and `snapp_drift_gate` already refuses two `@`-bearing files for one Snapp.
fn r14n_bundle() -> ::serde_json::Value {
  let dir = common::root().join("snapp");
  let mut found: ::std::vec::Vec<::serde_json::Value> = ::std::vec::Vec::new();
  for entry in ::std::fs::read_dir(&dir).expect("read snapp/") {
    let path = entry.expect("dir entry").path();
    let is_candidate = path
      .file_name()
      .and_then(|n| n.to_str())
      .map(|n| n.starts_with("r14n@") && n.ends_with(".json"))
      .unwrap_or(false);
    if !is_candidate {
      continue;
    }
    let raw = ::std::fs::read_to_string(&path)
      .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
    let value: ::serde_json::Value = ::serde_json::from_str(&raw)
      .unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()));
    if value.get("@snapp").is_some() {
      found.push(value);
    }
  }
  ::std::assert_eq!(found.len(), 1, "expected exactly one @-bearing r14n bundle under snapp/");
  found.remove(0)
}

/// The first object, anywhere in the value, that holds `name` as a key whose value has a `props`
/// table. Searches rather than assuming the nesting, which belongs to the emitter.
fn find_block<'a>(
  v: &'a ::serde_json::Value,
  name: &str,
) -> ::std::option::Option<&'a ::serde_json::Value> {
  match v {
    ::serde_json::Value::Object(map) => {
      if let ::std::option::Option::Some(hit) = map.get(name) {
        if hit.get("props").is_some() {
          return ::std::option::Option::Some(hit);
        }
      }
      map.values().find_map(|child| find_block(child, name))
    }
    ::serde_json::Value::Array(items) => items.iter().find_map(|child| find_block(child, name)),
    _ => ::std::option::Option::None,
  }
}

/// A bundle block's props as `(priority, name, optional)` rows, sorted by priority. serde_json maps
/// are sorted by key, so declaration order is read from `priority`.
fn prop_rows(block: &::serde_json::Value, label: &str) -> ::std::vec::Vec<(i64, ::std::string::String, bool)> {
  let props = block
    .get("props")
    .and_then(|p| p.as_object())
    .unwrap_or_else(|| ::std::panic!("{label}.props"));
  let mut rows: ::std::vec::Vec<(i64, ::std::string::String, bool)> = props
    .iter()
    .map(|(name, v)| {
      let priority = v
        .get("priority")
        .and_then(|p| p.as_i64())
        .unwrap_or_else(|| ::std::panic!("{label} prop {name:?} has no integer priority"));
      let optional = v.get("optional").and_then(|o| o.as_bool()).unwrap_or(false);
      (priority, name.clone(), optional)
    })
    .collect();
  rows.sort();
  rows
}

// ── The data-class vocabulary ───────────────────────────────────────────────────────────────

/// Why: the set is CLOSED and ORDERED, and the order is part of the contract (it is the `priority`
/// order). A class added in the middle, renamed or dropped changes what a declared key means.
#[test]
fn the_data_class_vocabulary_is_the_closed_first_nine_in_order() {
  let doc = common::read(DATA_CLASS_DOC);
  let set = block_source(&doc, DATA_CLASS_SET_DECL);
  let keys: ::std::vec::Vec<::std::string::String> = props_in_order(&set)
    .into_iter()
    .map(|(name, _)| name)
    .collect();
  let expected: ::std::vec::Vec<::std::string::String> =
    DATA_CLASSES.iter().map(|s| ::std::format!("{s}?")).collect();
  ::std::assert_eq!(
    keys, expected,
    "the props of DataClassSet must be exactly the closed first list, in contract order, each OPTIONAL"
  );
}

/// Why: a data class is a SUBJECT token, never a profile. A profile names WHOSE law (a `/`-delimited
/// tag); a data class names WHAT KIND OF DATA IS HANDLED. The grammar cannot contain a `/`.
#[test]
fn every_data_class_key_is_a_subject_token_and_never_a_profile() {
  for key in DATA_CLASSES.iter() {
    ::std::assert!(is_subject_token(key), "data class {key:?} is not a subject token");
    ::std::assert!(!key.contains('/'), "data class {key:?} is spelled like a profile");
  }
  // The checker itself must refuse what it claims to refuse: a profile-shaped tag and a capital.
  ::std::assert!(!is_subject_token("example/region"));
  ::std::assert!(!is_subject_token("Phi"));
  ::std::assert!(!is_subject_token(""));
}

/// Why: each class is one OPTIONAL `DataClass` prop at its contract priority, with a description a
/// human reviewer can read. A prop of another type, at another priority, or without a description is
/// a vocabulary that no longer reads the same as the table.
#[test]
fn every_data_class_set_prop_is_an_optional_data_class_at_its_priority_with_a_description() {
  let doc = common::read(DATA_CLASS_DOC);
  let set = block_source(&doc, DATA_CLASS_SET_DECL);
  let props = props_in_order(&set);
  ::std::assert_eq!(props.len(), DATA_CLASSES.len(), "DataClassSet props: {props:?}");
  for (i, (name, body)) in props.iter().enumerate() {
    let squeezed = squeeze(body);
    ::std::assert!(
      squeezed.contains("type=<DataClass>"),
      "{name} must be typed <DataClass>: {body}"
    );
    ::std::assert!(
      squeezed.contains(&::std::format!("priority={}", i + 1)),
      "{name} must carry priority = {}",
      i + 1
    );
    let description = body
      .lines()
      .map(str::trim)
      .find_map(|l| l.strip_prefix("description"))
      .map(|rest| rest.trim_start().trim_start_matches('=').trim().trim_matches('"').trim())
      .unwrap_or_else(|| ::std::panic!("{name} has no description"));
    ::std::assert!(!description.is_empty(), "{name} has an empty description");
  }
}

/// Why: the closure is `additional_props = false` on both types, and `DataClassSet` deliberately has
/// NO `min_props`: an empty set declares none, as an empty `data_classes` list does, and this
/// repository measured `min_props` unenforced on a nested empty table anyway.
#[test]
fn the_data_class_types_are_closed_and_the_set_has_no_min_props() {
  let doc = common::read(DATA_CLASS_DOC);
  for decl in [DATA_CLASS_DECL, DATA_CLASS_SET_DECL] {
    let block = squeeze(&block_source(&doc, decl));
    ::std::assert!(block.contains("additional_props=false"), "`{decl}` must be closed");
  }
  let set = squeeze(&block_source(&doc, DATA_CLASS_SET_DECL));
  ::std::assert!(!set.contains("min_props"), "DataClassSet must carry no min_props");
}

/// Why: a data class is a KIND, never a statute and never a determination, and the identifier spaces
/// of the compliance frameworks are licensed by their publishers and not cleared. The document must
/// carry the disclaimer and none of the identifier shapes below. This is a denylist of SHAPES, not a
/// proof of absence of every identifier; `framework_links.rs` holds the licensing gate.
#[test]
fn the_data_class_doc_carries_the_disclaimer_and_names_no_framework_criterion_identifier() {
  let doc = common::read(DATA_CLASS_DOC);
  ::std::assert!(doc.contains("NOT LEGAL ADVICE"), "the data-class doc must carry the disclaimer");
  for shape in ["45 CFR 164.", "45 CFR §164", "§ 164.", "34 CFR 99", "CC6.", "CC7.", "CC8.", "CC9."] {
    ::std::assert!(
      !doc.contains(shape),
      "the data-class doc names what looks like a framework criterion identifier: {shape:?}"
    );
  }
}

/// Why: a document that is declared but not re-exported is invisible to a dependent Snapp, and one
/// re-exported but not declared does not compile. `snapp_drift_gate` checks the two lists agree; this
/// pins that THIS module is in both, so the vocabulary reaches a consumer's `use { $::* }`.
#[test]
fn the_data_class_doc_is_declared_and_reexported_in_the_keyword_index() {
  let index = common::read(KEYWORD_INDEX);
  ::std::assert!(
    index.lines().any(|l| l.trim() == "mod R14N_SPEC_KEYWORD_DATA_CLASSES_1_0_0 {};"),
    "mod.n.md must declare R14N_SPEC_KEYWORD_DATA_CLASSES_1_0_0"
  );
  ::std::assert!(
    index.lines().any(|l| l.trim() == "pub @use R14N_SPEC_KEYWORD_DATA_CLASSES_1_0_0 {}"),
    "mod.n.md must re-export R14N_SPEC_KEYWORD_DATA_CLASSES_1_0_0"
  );
}

/// Why: MEASURED 2026-11-08. A dependent Snapp imports these types with `use { $::* }`, and a type
/// declared at the root (`pub blocks "X" {`) was not reached that way: a `SnappCompliance` missing a
/// REQUIRED prop compiled in the dependent Snapp. Declared in `*`, the same type refused it. Moving one
/// of these to the root would silently switch its checks off for every Snapp that declares compliance,
/// and every such Snapp is a dependent one. The source and the bundle are both pinned.
#[test]
fn the_compliance_types_are_declared_in_star_so_a_dependent_snapp_reaches_them() {
  let data_class_doc = common::read(DATA_CLASS_DOC);
  let compliance_doc = common::read(SNAPP_COMPLIANCE_DOC);
  for (doc, decl) in [
    (&data_class_doc, DATA_CLASS_DECL),
    (&data_class_doc, DATA_CLASS_SET_DECL),
    (&compliance_doc, SNAPP_COMPLIANCE_DECL),
  ] {
    ::std::assert!(
      doc.lines().any(|l| l.trim() == decl),
      "expected `{decl}` (declared in `*`, where a dependent Snapp's `use {{ $::* }}` reaches it)"
    );
  }
  let bundle = r14n_bundle();
  let star_blocks = bundle
    .get("mod")
    .and_then(|m| m.get("*"))
    .and_then(|s| s.get("blocks"))
    .and_then(|b| b.as_object())
    .expect("the bundle must carry mod.*.blocks");
  for name in ["DataClass", "DataClassSet", "SnappCompliance"] {
    ::std::assert!(star_blocks.contains_key(name), "the bundle must carry {name} under mod.*.blocks");
  }
}

// ── The closed-set prop on SnappCompliance ──────────────────────────────────────────────────

/// Why (source side): LAST-position additive. The closed-set prop must come AFTER `data_classes` in the
/// source, be OPTIONAL (a `?` after the name), carry `priority = 3`, and be typed `<DataClassSet>`, so
/// nothing already declared stops compiling.
#[test]
fn snapp_compliance_source_declares_the_closed_set_prop_last_and_optional() {
  let doc = common::read(SNAPP_COMPLIANCE_DOC);
  let block = block_source(&doc, SNAPP_COMPLIANCE_DECL);
  let props = props_in_order(&block);
  let names: ::std::vec::Vec<&str> = props.iter().map(|(n, _)| n.as_str()).collect();
  ::std::assert_eq!(
    names,
    ["exercises", "data_classes", "data_class_set?"],
    "SnappCompliance props must be [exercises, data_classes, data_class_set?] in that order"
  );
  let squeezed = squeeze(&props[2].1);
  ::std::assert!(squeezed.contains("priority=3"), "`data_class_set?` must carry priority = 3");
  ::std::assert!(
    squeezed.contains("type=<DataClassSet>"),
    "`data_class_set?` must be typed <DataClassSet>"
  );
}

/// Why (bundle side): the committed bundle is what a consumer pins. It must carry the closed-set prop
/// LAST (priority 3) and OPTIONAL, leave the required pair (`exercises` 1, `data_classes` 2) REQUIRED
/// and in place, and carry `DataClassSet` closed with the nine classes, each OPTIONAL, in contract order.
/// Passes only AFTER the bundle is re-emitted (`nlang export -r "r14n Spec/1.0.0" -f json`).
#[test]
fn the_emitted_bundle_carries_the_closed_set_prop_last_and_the_nine_classes_in_order() {
  let bundle = r14n_bundle();
  let compliance =
    find_block(&bundle, "SnappCompliance").expect("the bundle must carry the SnappCompliance block");
  ::std::assert_eq!(
    compliance.get("additional_props").and_then(|b| b.as_bool()),
    ::std::option::Option::Some(false),
    "SnappCompliance must stay CLOSED"
  );
  let rows = prop_rows(compliance, "SnappCompliance");
  let expected = [
    (1_i64, "exercises", false),
    (2_i64, "data_classes", false),
    (3_i64, "data_class_set", true),
  ];
  ::std::assert_eq!(rows.len(), expected.len(), "SnappCompliance props: {rows:?}");
  for (row, want) in rows.iter().zip(expected.iter()) {
    ::std::assert_eq!(
      (row.0, row.1.as_str(), row.2),
      *want,
      "SnappCompliance props must be [exercises 1 required, data_classes 2 required, \
       data_class_set 3 optional]; got {rows:?}"
    );
  }

  let set = find_block(&bundle, "DataClassSet").expect("the bundle must carry the DataClassSet block");
  ::std::assert_eq!(
    set.get("additional_props").and_then(|b| b.as_bool()),
    ::std::option::Option::Some(false),
    "DataClassSet must be CLOSED"
  );
  let set_rows = prop_rows(set, "DataClassSet");
  let got: ::std::vec::Vec<(i64, &str, bool)> =
    set_rows.iter().map(|(p, n, o)| (*p, n.as_str(), *o)).collect();
  let want: ::std::vec::Vec<(i64, &str, bool)> = DATA_CLASSES
    .iter()
    .enumerate()
    .map(|(i, k)| (i as i64 + 1, *k, true))
    .collect();
  ::std::assert_eq!(got, want, "DataClassSet must carry the nine classes, OPTIONAL, in contract order");
}
