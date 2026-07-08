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
    let kind = def.get("kind").and_then(|k| k.as_str()).unwrap_or("obligation");
    let title = def.get("title").and_then(|t| t.as_str()).unwrap_or(key);
    controls.insert(
      key.clone(),
      ControlDef {
        kind: ::std::string::String::from(kind),
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
}
