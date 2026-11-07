// SPDX-License-Identifier: Apache-2.0
//! The committed Snapp json face is read through a TYPED contract, or this fails.
//!
//! ⚠ THIS FILE TYPES THE HUMAN-READABLE FACE, NOT THE RECORD. The json is the human-readable
//! form of an exported Snapp; the system of record is the Snapp's named `.nbin`, read through
//! the ABI reader. This test exists because r14n is a PUBLIC repo whose CI cannot link the
//! proprietary ABI, and because r14n pins and ships that human face.
//! The ABI-typed twin lives outside this public repo and holds the same types over the
//! decoded `.nbin`. Do not read a green run here as proof about the record.
//!
//! `snapp/r14n@<ver>.json` is a shipped artifact, and a consumer reads it as DATA with a
//! shape, never as text. This file is that shape as types: closed categoricals are enums
//! (`Strictness`, `LegalReviewStatus`), the N Lang ordered-list encoding is one reader
//! (`OrderedList`), a control key is a newtype that refuses the empty string, and every
//! struct refuses a field it does not know. Nothing here greps, hashes or substring-matches
//! the artifact.
//!
//! WHY A TYPED READ AND NOT A STRING CHECK: the committed bundle once carried
//! `"prohibited": {"~0": {}}` for a pack whose source forbids `full_library_access`. The
//! source line was a bare list where the schema types the field `ControlList`; the compiler
//! accepted it, and the json export normalised the mismatch to an EMPTY prohibition, which
//! reads as "forbids nothing" (the fail-open direction for a data-minimisation guard). The
//! byte-for-byte drift gate agreed with that bundle, because it compares bytes to a pin, not
//! meaning to a contract. A typed decode cannot agree with it: `prohibited` is optional,
//! but when it is PRESENT it must be a `ControlList`, and the shape the unfixed export
//! produced is refused by `the_unfixed_export_shape_is_refused_not_coerced` below.
//!
//! A control key is a REFERENCE into a closed, declared vocabulary: spec section 2 makes it
//! unique within, and namespaced by, its owning domain, and `catalog/<domain>.catalog.toml`
//! declares it under `schema/catalog.schema.json` (`kind` closed and REQUIRED, unknown
//! fields refused). So the catalogs are read through types here as well (`Catalog`,
//! `ControlKind`), and a key a pack names that its own domain's catalog does not declare is
//! a failure. Strictness, legal review status, the pack id grammar and the list encoding
//! are the other closed shapes below.
//!
//! What this does NOT prove: that the committed bundle is what the current Snapp sources
//! would emit (that needs the proprietary compiler, which cannot run on public CI), and
//! nothing about the `.n.json` or `.nbin` faces, which are gitignored and absent on CI.
//!
//! Style note: FQ paths, no module-top `use`, the house style of this crate. Fields exist to
//! be TYPE-CHECKED by the decode, so most are never read.
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-11-07: authored after the `prohibited` drop above, on the standing order that
//!   artifacts are read through data contracts, never string matching.
//! - 2026-11-07: added the typed catalog cross-check. It found `full_library_access` named by
//!   a pack and declared by no catalog, hidden until the pack's `prohibited` field was
//!   written in its typed shape; the header's earlier "control keys are opaque" was wrong.

#![allow(dead_code)]

mod common;

/// A list in the N Lang JSON wire encoding: an object keyed `~0`, `~1`, ... whose KEY ORDER
/// IS NOT THE LIST ORDER (a sorted map puts `~10` before `~2`), so the index is parsed and
/// the entries are laid out numerically. A bare JSON array is a different shape and is
/// refused; so is a gap, a foreign key, and a non-canonical index such as `~01`.
#[derive(Debug, PartialEq, Eq)]
struct OrderedList<T>(::std::vec::Vec<T>);

impl<'de, T: ::serde::Deserialize<'de>> ::serde::Deserialize<'de> for OrderedList<T> {
  fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> ::std::result::Result<Self, D::Error> {
    let raw: ::std::collections::BTreeMap<::std::string::String, T> =
      ::serde::Deserialize::deserialize(d)?;
    let mut indexed: ::std::vec::Vec<(usize, T)> = ::std::vec::Vec::with_capacity(raw.len());
    for (key, value) in raw {
      let digits = key.strip_prefix('~').ok_or_else(|| {
        <D::Error as ::serde::de::Error>::custom("a list entry key must be `~<index>`")
      })?;
      let index: usize = digits.parse().map_err(|_| {
        <D::Error as ::serde::de::Error>::custom("a list entry index must be a number")
      })?;
      if index.to_string() != digits {
        return ::std::result::Result::Err(<D::Error as ::serde::de::Error>::custom(
          "a list entry index must be canonical (no leading zeros)",
        ));
      }
      indexed.push((index, value));
    }
    indexed.sort_by_key(|(index, _)| *index);
    for (expected, (index, _)) in indexed.iter().enumerate() {
      if *index != expected {
        return ::std::result::Result::Err(<D::Error as ::serde::de::Error>::custom(
          "list entry indices must run 0..n with no gap",
        ));
      }
    }
    ::std::result::Result::Ok(Self(indexed.into_iter().map(|(_, value)| value).collect()))
  }
}

/// A control key as a pack names it. Never empty. Whether it is DECLARED is the catalog
/// cross-check's question, below.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ::serde::Deserialize)]
#[serde(try_from = "::std::string::String")]
struct ControlKey(::std::string::String);

impl ::std::convert::TryFrom<::std::string::String> for ControlKey {
  type Error = &'static str;
  fn try_from(raw: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
    if raw.is_empty() {
      return ::std::result::Result::Err("a control key must not be empty");
    }
    ::std::result::Result::Ok(Self(raw))
  }
}

/// The pack id grammar the spec states: lowercase `^[a-z][a-z0-9_]{0,63}$`.
#[derive(Debug, Clone, PartialEq, Eq, ::serde::Deserialize)]
#[serde(try_from = "::std::string::String")]
struct PackId(::std::string::String);

impl ::std::convert::TryFrom<::std::string::String> for PackId {
  type Error = &'static str;
  fn try_from(raw: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
    let mut chars = raw.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if first_ok && rest_ok && raw.len() <= 64 {
      return ::std::result::Result::Ok(Self(raw));
    }
    ::std::result::Result::Err("a pack id must match ^[a-z][a-z0-9_]{0,63}$")
  }
}

/// Spec closed set: `aggressive`, `as_configured`, `minimal`. An unknown token is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Strictness {
  Aggressive,
  AsConfigured,
  Minimal,
}

/// Spec closed set: `draft`, `requires_signoff`, `not_required`. An unknown token is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegalReviewStatus {
  Draft,
  RequiresSignoff,
  NotRequired,
}

/// A named list of control keys. `controls` is REQUIRED: an empty list is meaningful and is
/// not the same as an absent one, and a missing field must never default to "empty".
#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlList {
  controls: OrderedList<ControlKey>,
}

#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LegalReview {
  status: LegalReviewStatus,
  counsel: ::std::option::Option<::std::string::String>,
}

#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PackMeta {
  strictness: Strictness,
  description: ::std::option::Option<::std::string::String>,
  legal_review: ::std::option::Option<LegalReview>,
}

#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectRow {
  key: ::std::string::String,
  controls: OrderedList<ControlKey>,
}

/// One pack as the json face carries it. `prohibited` is OPTIONAL (absent = never reviewed),
/// but a PRESENT value that is not a `ControlList` is an error: `serde(default)` fires only
/// when the field is missing, never to paper over a malformed one.
#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyPackFace {
  id: PackId,
  meta: PackMeta,
  subject: OrderedList<SubjectRow>,
  legally_required: ControlList,
  #[serde(default)]
  prohibited: ::std::option::Option<ControlList>,
}

/// The json face: every other top-level key is ignored, `packs` is read through the contract.
#[derive(::serde::Deserialize)]
struct FaceDoc {
  packs: ::std::collections::BTreeMap<::std::string::String, PolicyPackFace>,
}

fn key(raw: &str) -> ControlKey {
  <ControlKey as ::std::convert::TryFrom<::std::string::String>>::try_from(
    ::std::string::String::from(raw),
  )
  .expect("a non-empty control key")
}

fn shipped_face() -> FaceDoc {
  let p = common::repo_root()
    .join("snapp")
    .join("r14n@0.1.0-alpha.1.json");
  let raw =
    ::std::fs::read_to_string(&p).unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()));
  ::serde_json::from_str(&raw)
    .unwrap_or_else(|e| ::std::panic!("the committed face does not meet the typed contract: {e}"))
}

/// A valid pack as `serde_json::Value`, with `prohibited` supplied by the caller (`None` omits it).
fn pack_with(prohibited: ::std::option::Option<::serde_json::Value>) -> ::serde_json::Value {
  let mut pack = ::serde_json::json!({
    "id": "photo_library",
    "meta": { "strictness": "minimal", "legal_review": { "status": "requires_signoff", "counsel": "TBD" } },
    "subject": { "~0": { "key": "read", "controls": { "~0": "consent_prompt" } } },
    "legally_required": { "controls": { "~0": "consent_prompt" } }
  });
  if let ::std::option::Option::Some(p) = prohibited {
    pack["prohibited"] = p;
  }
  pack
}

fn decode(pack: ::serde_json::Value) -> ::std::result::Result<PolicyPackFace, ::serde_json::Error> {
  ::serde_json::from_value(pack)
}

/// Why: the committed human-readable face is what r14n ships and pins, so its shape is a contract.
/// A pack that stops meeting it (an unknown field, a bare list where a `ControlList` belongs) must
/// fail here instead of being coerced by a consumer; the export once wrote `prohibited` as `{"~0": {}}`.
#[test]
fn every_shipped_pack_decodes_through_the_typed_contract() {
  let face = shipped_face();
  ::std::assert!(!face.packs.is_empty(), "the face carries no packs");
  for (name, pack) in &face.packs {
    ::std::assert!(
      !pack.subject.0.is_empty(),
      "{name}: a pack needs at least one subject row"
    );
    for row in &pack.subject.0 {
      ::std::assert!(
        !row.controls.0.is_empty(),
        "{name}: a subject row needs at least one control"
      );
    }
  }
}

/// Why: the one rule `prohibited` carries (spec): a pack that both requires and forbids a control is
/// not satisfiable. Checked over every shipped pack, across the floor and every subject row.
#[test]
fn no_pack_requires_a_control_it_also_forbids() {
  // The one rule `prohibited` carries (spec): MUST NOT intersect `legally_required`.
  let face = shipped_face();
  for (name, pack) in &face.packs {
    if let ::std::option::Option::Some(forbidden) = &pack.prohibited {
      let required: ::std::collections::BTreeSet<&ControlKey> =
        pack.legally_required.controls.0.iter().collect();
      for control in &forbidden.controls.0 {
        ::std::assert!(
          !required.contains(control),
          "{name}: requires and forbids {control:?}"
        );
      }
    }
  }
}

/// Why: regression pin for the export that dropped this pack's prohibition to empty. The pack states its
/// preference for scoped access as DATA, so the face must carry `full_library_access` under `prohibited`.
#[test]
fn the_photo_library_minimal_pack_forbids_the_full_library() {
  let face = shipped_face();
  let pack = face
    .packs
    .get("photo_library_minimal")
    .expect("the photo_library_minimal pack");
  ::std::assert_eq!(pack.meta.strictness, Strictness::Minimal);
  ::std::assert_eq!(
    pack.meta.legal_review.as_ref().map(|r| r.status),
    ::std::option::Option::Some(LegalReviewStatus::RequiresSignoff)
  );
  let forbidden = pack
    .prohibited
    .as_ref()
    .expect("the pack must carry its prohibition");
  ::std::assert_eq!(
    forbidden.controls.0,
    ::std::vec![key("full_library_access")]
  );
}

/// Why: negative control for the pin above. The shape the export once wrote must be an error, or that pin
/// could pass on a decoder that quietly coerces it.
#[test]
fn the_unfixed_export_shape_is_refused_not_coerced() {
  // NEGATIVE CONTROL: byte-for-byte what the unfixed export wrote for this field.
  let lossy = ::serde_json::json!({ "~0": {} });
  ::std::assert!(decode(pack_with(::std::option::Option::Some(lossy))).is_err());
}

/// Why: negative control. A bare list is the shape of the source line the compiler accepted; `prohibited`
/// is a `ControlList`, never a bare list.
#[test]
fn a_bare_list_prohibition_is_refused() {
  // NEGATIVE CONTROL: the shape the source line had, which the compiler let through.
  let bare = ::serde_json::json!(["full_library_access"]);
  ::std::assert!(decode(pack_with(::std::option::Option::Some(bare))).is_err());
}

/// Why: the `ControlList` schema text: empty is meaningful and is not the same as absent (an absent
/// `prohibited` was never reviewed). The decoder must keep the two apart.
#[test]
fn an_empty_prohibition_is_present_and_empty_not_absent() {
  let empty = decode(pack_with(::std::option::Option::Some(
    ::serde_json::json!({ "controls": {} }),
  )))
  .expect("an empty ControlList is a valid, meaningful value");
  ::std::assert_eq!(empty.prohibited.expect("present").controls.0.len(), 0);
  let absent =
    decode(pack_with(::std::option::Option::None)).expect("an absent prohibition is valid");
  ::std::assert!(absent.prohibited.is_none());
}

/// Why: a missing `controls` must not become an empty list, which would turn an omitted prohibition into
/// an explicit statement that nothing is forbidden.
#[test]
fn a_controllist_without_its_controls_field_does_not_default_to_empty() {
  ::std::assert!(
    decode(pack_with(::std::option::Option::Some(::serde_json::json!(
      {}
    ))))
    .is_err()
  );
}

/// Why: the ordered-list encoding keys entries `~0`, `~1`, ...; a sorted map puts `~10` before `~2`, so
/// reading in key order would reorder controls past ten entries.
#[test]
fn the_ordered_list_lays_entries_out_by_index_not_by_key_order() {
  // A sorted map puts `~10` before `~2`; the list order is numeric.
  let mut map = ::serde_json::Map::new();
  for i in 0..11 {
    map.insert(
      ::std::format!("~{i}"),
      ::serde_json::Value::String(::std::format!("c{i}")),
    );
  }
  let list: OrderedList<ControlKey> =
    ::serde_json::from_value(::serde_json::Value::Object(map)).expect("an eleven-entry list");
  let expected: ::std::vec::Vec<ControlKey> =
    (0..11).map(|i| key(&::std::format!("c{i}"))).collect();
  ::std::assert_eq!(list.0, expected);
}

/// Why: a malformed list is refused whole, never repaired: a gap would silently drop an entry, `~01` would
/// alias `~1`, and a bare array is a different shape.
#[test]
fn the_ordered_list_refuses_a_gap_a_foreign_key_and_a_noncanonical_index() {
  for refused in [
    ::serde_json::json!({ "~0": "a", "~2": "c" }),
    ::serde_json::json!({ "first": "a" }),
    ::serde_json::json!({ "~01": "a" }),
    ::serde_json::json!(["a"]),
  ] {
    ::std::assert!(::serde_json::from_value::<OrderedList<ControlKey>>(refused).is_err());
  }
}

/// Why: strictness is a closed set by the spec (aggressive, as_configured, minimal); an unknown token must
/// be a visible error, never a silent downgrade.
#[test]
fn a_closed_categorical_refuses_an_unknown_token() {
  let mut pack = pack_with(::std::option::Option::None);
  pack["meta"]["strictness"] = ::serde_json::json!("lax");
  ::std::assert!(decode(pack).is_err());
}

/// Why: a misspelled field (`prohibted`) must fail instead of being ignored. Ignoring unknown fields is
/// how a prohibition vanishes without any error.
#[test]
fn a_field_the_contract_does_not_know_is_refused() {
  let mut pack = pack_with(::std::option::Option::None);
  pack["prohibted"] = ::serde_json::json!({ "controls": {} });
  ::std::assert!(decode(pack).is_err());
}

/// Why: a control key is an identifier a gate resolves against a catalog; the empty string names nothing
/// and must never be accepted as one.
#[test]
fn an_empty_control_key_is_refused() {
  ::std::assert!(::serde_json::from_value::<ControlKey>(::serde_json::json!("")).is_err());
}

/// Schema `catalog.schema.json`: `kind` is REQUIRED and closed. It is required, not
/// defaulted, on purpose: a loader that defaults a missing kind turns an omission into an
/// assertion nobody wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ControlKind {
  Permit,
  Obligation,
  Prohibition,
}

/// One declared control. Every field the schema allows is named, so an unknown one is
/// refused (the schema sets `additionalProperties: false`). The facets this file never
/// reads are carried as `toml::Value` so their PRESENCE is typed without restating them.
#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlDecl {
  kind: ControlKind,
  title: ::std::string::String,
  description: ::std::option::Option<::std::string::String>,
  trigger: ::std::option::Option<::std::string::String>,
  mechanism: ::std::option::Option<::std::string::String>,
  deprecated: ::std::option::Option<::toml::Value>,
  superseded_by: ::std::option::Option<::toml::Value>,
  attestation_mode: ::std::option::Option<::toml::Value>,
  frameworks: ::std::option::Option<::toml::Value>,
  legal_basis_hint: ::std::option::Option<::toml::Value>,
  subject_rights: ::std::option::Option<::toml::Value>,
  verified_by: ::std::option::Option<::toml::Value>,
}

#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainDecl {
  name: ::std::string::String,
  description: ::std::option::Option<::std::string::String>,
}

#[derive(Debug, ::serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
  domain: DomainDecl,
  control: ::std::collections::BTreeMap<::std::string::String, ControlDecl>,
}

fn catalog_of(domain: &PackId) -> Catalog {
  let p = common::repo_root()
    .join("catalog")
    .join(::std::format!("{}.catalog.toml", domain.0));
  let raw = ::std::fs::read_to_string(&p).unwrap_or_else(|e| {
    ::std::panic!("a pack's domain needs a catalog, read {}: {e}", p.display())
  });
  ::toml::from_str(&raw)
    .unwrap_or_else(|e| ::std::panic!("{} does not meet the catalog contract: {e}", p.display()))
}

/// Every control key a pack names, in any table, by the table that names it.
fn required_keys(pack: &PolicyPackFace) -> ::std::vec::Vec<&ControlKey> {
  pack
    .legally_required
    .controls
    .0
    .iter()
    .chain(pack.subject.0.iter().flat_map(|row| row.controls.0.iter()))
    .collect()
}

fn prohibited_keys(pack: &PolicyPackFace) -> ::std::vec::Vec<&ControlKey> {
  pack
    .prohibited
    .iter()
    .flat_map(|list| list.controls.0.iter())
    .collect()
}

/// Why: RLPS spec section 2: a control key is unique within, and namespaced by, its owning domain, so every
/// key a pack names must be declared by that domain's catalog. This is the check that found
/// `full_library_access` declared by no catalog.
#[test]
fn every_pack_names_only_controls_its_own_domain_catalog_declares() {
  let face = shipped_face();
  for (name, pack) in &face.packs {
    let catalog = catalog_of(&pack.id);
    ::std::assert_eq!(
      catalog.domain.name,
      pack.id.0,
      "{name}: the catalog's domain is the pack id"
    );
    for control in required_keys(pack).into_iter().chain(prohibited_keys(pack)) {
      ::std::assert!(
        catalog.control.contains_key(&control.0),
        "{name}: {control:?} is declared by no catalog of domain {}",
        pack.id.0
      );
    }
  }
}

/// Why: the catalog schema says a prohibition is subtracted under every posture and must never appear in a
/// requirement table; a key listed under `prohibited` must be declared a prohibition by its catalog.
#[test]
fn a_prohibited_key_is_a_prohibition_and_a_required_key_never_is() {
  // Schema: a prohibition is subtracted under every posture, so it must never appear in a
  // requirement table; and a key listed under `prohibited` that is not declared a
  // prohibition is a pack forbidding something its catalog calls an obligation.
  let face = shipped_face();
  for (name, pack) in &face.packs {
    let catalog = catalog_of(&pack.id);
    for control in prohibited_keys(pack) {
      let kind = catalog.control.get(&control.0).map(|c| c.kind);
      ::std::assert_eq!(
        kind,
        ::std::option::Option::Some(ControlKind::Prohibition),
        "{name}: {control:?}"
      );
    }
    for control in required_keys(pack) {
      let kind = catalog.control.get(&control.0).map(|c| c.kind);
      ::std::assert_ne!(
        kind,
        ::std::option::Option::Some(ControlKind::Prohibition),
        "{name}: {control:?}"
      );
    }
  }
}

/// Why: pins the declaration that closed the finding, so removing the catalog entry fails here by name
/// and not only through the generic cross-check.
#[test]
fn the_photo_library_catalog_declares_the_full_library_prohibition() {
  let face = shipped_face();
  let pack = face
    .packs
    .get("photo_library_minimal")
    .expect("the photo_library_minimal pack");
  let declared = catalog_of(&pack.id);
  let control = declared
    .control
    .get("full_library_access")
    .expect("declared by its own catalog");
  ::std::assert_eq!(control.kind, ControlKind::Prohibition);
}

/// Why: the catalog schema requires `kind` and says it is required rather than defaulted on purpose: a
/// defaulted kind turns an omission into an assertion nobody wrote.
#[test]
fn a_catalog_control_without_a_kind_is_refused_not_defaulted() {
  let raw = "[domain]\nname = \"d\"\n[control.k]\ntitle = \"t\"\n";
  ::std::assert!(::toml::from_str::<Catalog>(raw).is_err());
}

/// Why: `kind` is the closed set permit, obligation, prohibition; any other token is an error.
#[test]
fn a_catalog_control_with_an_unknown_kind_is_refused() {
  let raw = "[domain]\nname = \"d\"\n[control.k]\nkind = \"mandate\"\ntitle = \"t\"\n";
  ::std::assert!(::toml::from_str::<Catalog>(raw).is_err());
}

/// Why: the catalog schema sets additional properties to false; a misspelled facet must fail, not vanish.
#[test]
fn a_catalog_refuses_a_field_the_schema_does_not_allow() {
  let raw =
    "[domain]\nname = \"d\"\n[control.k]\nkind = \"obligation\"\ntitle = \"t\"\nkinds = \"x\"\n";
  ::std::assert!(::toml::from_str::<Catalog>(raw).is_err());
}

/// Why: catalogs are data other code trusts. Every shipped one must meet the schema's required and
/// closed fields, so a strict loader cannot break on a shipped file.
#[test]
fn every_shipped_catalog_meets_the_catalog_contract() {
  let dir = common::repo_root().join("catalog");
  let mut seen = 0;
  for entry in ::std::fs::read_dir(&dir).expect("the catalog directory") {
    let path = entry.expect("a directory entry").path();
    let is_catalog = path
      .file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.ends_with(".catalog.toml"));
    if !is_catalog {
      continue;
    }
    let raw = ::std::fs::read_to_string(&path).expect("a readable catalog");
    let catalog: Catalog = ::toml::from_str(&raw).unwrap_or_else(|e| {
      ::std::panic!("{} does not meet the catalog contract: {e}", path.display())
    });
    ::std::assert!(
      !catalog.control.is_empty(),
      "{}: a catalog declares at least one control",
      path.display()
    );
    seen += 1;
  }
  ::std::assert!(seen > 0, "no catalog was read");
}
