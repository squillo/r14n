// SPDX-License-Identifier: Apache-2.0
//! Control-catalog loading shared by `extract`, `merge`, and `validate`.
//!
//! A catalog (`/catalog/<domain>.catalog.toml`) declares the control-key
//! universe a domain's packs may reference: `[domain]` + one `[control.<key>]`
//! table per control with a deontic `kind` and a `title`. NOT LEGAL ADVICE —
//! a control is a stable identifier, not a statement of law.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI).
//! - 2026-07-06: DRY pass — `parse` split out of `load`; shared `test_catalog`
//!   fixture replaces the per-module tempdir dance.
//! - 2026-11-07: `kind` and `title` are now REQUIRED and `kind` is a closed set, as
//!   `schema/catalog.schema.json` states ("required rather than defaulted on purpose: a
//!   loader that defaults a missing kind turns an omission into an assertion nobody
//!   wrote"). `parse` used to default a missing `kind` to `obligation` and a missing
//!   `title` to the key, and accepted any `kind` string. New cells only; the existing
//!   cell and its expected value are unchanged.

/// The closed deontic set `schema/catalog.schema.json` allows for `kind`.
#[derive(Clone, Copy)]
enum ControlKind {
  Permit,
  Obligation,
  Prohibition,
}

impl ControlKind {
  fn from_token(token: &str) -> ::std::option::Option<Self> {
    match token {
      "permit" => ::std::option::Option::Some(Self::Permit),
      "obligation" => ::std::option::Option::Some(Self::Obligation),
      "prohibition" => ::std::option::Option::Some(Self::Prohibition),
      _ => ::std::option::Option::None,
    }
  }

  fn as_str(self) -> &'static str {
    match self {
      Self::Permit => "permit",
      Self::Obligation => "obligation",
      Self::Prohibition => "prohibition",
    }
  }
}

/// One catalog control declaration (the fields the CLI needs).
pub struct ControlDef {
  /// Deontic kind: `permit` | `obligation` | `prohibition`.
  pub kind: ::std::string::String,
  /// Human-readable title.
  pub title: ::std::string::String,
}

/// A parsed control catalog.
pub struct Catalog {
  /// The domain the catalog declares (e.g. `recording_consent`).
  pub domain: ::std::string::String,
  /// Control key → definition, in stable key order.
  pub controls: ::std::collections::BTreeMap<::std::string::String, ControlDef>,
}

/// Load a catalog TOML file.
pub fn load(path: &::std::path::Path) -> ::std::result::Result<Catalog, ::std::string::String> {
  let raw = ::std::fs::read_to_string(path)
    .map_err(|e| ::std::format!("read {}: {e}", path.display()))?;
  parse(&raw, &path.display().to_string())
}

/// Parse catalog TOML text (`origin` labels error messages).
pub fn parse(
  raw: &str,
  origin: &str,
) -> ::std::result::Result<Catalog, ::std::string::String> {
  let doc: ::toml::Value =
    ::toml::from_str(raw).map_err(|e| ::std::format!("parse {origin}: {e}"))?;
  let domain = doc
    .get("domain")
    .and_then(|d| d.get("name"))
    .and_then(|n| n.as_str())
    .ok_or_else(|| ::std::format!("{origin}: missing [domain] name"))?;
  let mut controls: ::std::collections::BTreeMap<::std::string::String, ControlDef> =
    ::std::collections::BTreeMap::new();
  let table = doc
    .get("control")
    .and_then(|c| c.as_table())
    .ok_or_else(|| ::std::format!("{origin}: missing [control.<key>] tables"))?;
  for (key, def) in table {
    let token = def.get("kind").and_then(|k| k.as_str()).ok_or_else(|| {
      ::std::format!("{origin}: [control.{key}] has no `kind` (required, never defaulted)")
    })?;
    let kind = ControlKind::from_token(token).ok_or_else(|| {
      ::std::format!(
        "{origin}: [control.{key}] kind {token:?} is not one of permit | obligation | prohibition"
      )
    })?;
    let title = def
      .get("title")
      .and_then(|t| t.as_str())
      .ok_or_else(|| ::std::format!("{origin}: [control.{key}] has no `title` (required)"))?;
    controls.insert(
      key.clone(),
      ControlDef {
        kind: ::std::string::String::from(kind.as_str()),
        title: ::std::string::String::from(title),
      },
    );
  }
  ::std::result::Result::Ok(Catalog { domain: ::std::string::String::from(domain), controls })
}

/// Shared test fixture: a minimal three-control catalog.
#[cfg(test)]
pub(crate) const TEST_CATALOG: &str = "[domain]\nname = \"recording_consent\"\ndescription = \"d\"\n\n\
   [control.attestation]\nkind = \"obligation\"\ntitle = \"Recording-consent attestation\"\n\n\
   [control.signal_notice]\nkind = \"obligation\"\ntitle = \"Recording signal active\"\n\n\
   [control.aph_mandate]\nkind = \"permit\"\ntitle = \"Delegate mandate\"\n";

/// Shared test fixture: [`TEST_CATALOG`] parsed — the ONE way test modules get
/// a `Catalog` (no per-module tempdir dance).
#[cfg(test)]
pub(crate) fn test_catalog() -> Catalog {
  parse(TEST_CATALOG, "TEST_CATALOG").expect("fixture catalog parses")
}

#[cfg(test)]
mod tests {
  /// Why: `extract`/`merge`/`validate` all key off catalog membership and kind —
  /// if loading dropped controls, reordered keys unstably, or lost `kind`, every
  /// downstream lint/template would silently go wrong. Also pins the load(path)
  /// → parse(text) split doing identical work.
  #[test]
  fn loads_domain_and_controls_in_key_order() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let path = tmp.path().join("recording_consent.catalog.toml");
    ::std::fs::write(&path, super::TEST_CATALOG).expect("write");
    let cat = super::load(&path).expect("load");
    ::std::assert_eq!(cat.domain, "recording_consent");
    let keys: ::std::vec::Vec<&str> = cat.controls.keys().map(|k| k.as_str()).collect();
    ::std::assert_eq!(keys, ["aph_mandate", "attestation", "signal_notice"]);
    ::std::assert_eq!(cat.controls["aph_mandate"].kind, "permit");
    ::std::assert_eq!(super::test_catalog().domain, cat.domain, "parse path matches load path");
  }

  /// Why: the schema says `kind` is required, not defaulted. A defaulted kind turns an
  /// omitted prohibition into an obligation nobody wrote.
  #[test]
  fn a_control_without_a_kind_is_refused_not_defaulted() {
    let raw = "[domain]\nname = \"d\"\n[control.k]\ntitle = \"t\"\n";
    let err = super::parse(raw, "fixture")
      .err()
      .expect("a missing kind is an error");
    ::std::assert!(err.contains("no `kind`"), "{err}");
  }

  /// Why: the schema closes `kind` to permit, obligation, prohibition. A loader that accepted any string
  /// would let a misspelled `prohibtion` pass as a kind nobody defined.
  #[test]
  fn a_control_with_an_unknown_kind_is_refused() {
    let raw = "[domain]\nname = \"d\"\n[control.k]\nkind = \"mandate\"\ntitle = \"t\"\n";
    let err = super::parse(raw, "fixture")
      .err()
      .expect("an unknown kind is an error");
    ::std::assert!(err.contains("not one of"), "{err}");
  }

  /// Why: the schema requires `title`. Naming a control after its own key would put an identifier
  /// where a human-readable title belongs, and every template and report would show it as one.
  #[test]
  fn a_control_without_a_title_is_refused_not_named_after_its_key() {
    let raw = "[domain]\nname = \"d\"\n[control.k]\nkind = \"obligation\"\n";
    let err = super::parse(raw, "fixture")
      .err()
      .expect("a missing title is an error");
    ::std::assert!(err.contains("no `title`"), "{err}");
  }
}
