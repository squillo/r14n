//! Shared `.r14n.toml` pack accessors — the ONE place `controls = [...]` lists
//! are pulled out of a parsed pack. `merge` (delta detection) and `validate`
//! (lint rules) both consume these, so the definition of "what a pack
//! references" cannot drift between the two subcommands.
//!
//! Revision History
//! - 2026-07-06: authored — DRY pass (extracted from merge.rs + validate.rs).

/// The `controls` list of `pack[table]` as a key set (empty when absent).
pub fn control_list(
  pack: &::toml::Value,
  table: &str,
) -> ::std::collections::BTreeSet<::std::string::String> {
  list_of(pack.get(table))
}

/// Every `[subject.<name>]` table as `(name, controls)`.
pub fn subject_tables(
  pack: &::toml::Value,
) -> ::std::vec::Vec<(::std::string::String, ::std::collections::BTreeSet<::std::string::String>)> {
  pack
    .get("subject")
    .and_then(|s| s.as_table())
    .map(|subjects| {
      subjects
        .iter()
        .map(|(name, table)| (name.clone(), list_of(::std::option::Option::Some(table))))
        .collect()
    })
    .unwrap_or_default()
}

/// Every control key a pack references (subject tables ∪ floor ∪ prohibited).
pub fn referenced_controls(
  pack: &::toml::Value,
) -> ::std::collections::BTreeSet<::std::string::String> {
  let mut refs: ::std::collections::BTreeSet<::std::string::String> =
    ::std::collections::BTreeSet::new();
  for (_, controls) in subject_tables(pack) {
    refs.extend(controls);
  }
  refs.extend(control_list(pack, "legally_required"));
  refs.extend(control_list(pack, "prohibited"));
  refs
}

fn list_of(
  table: ::std::option::Option<&::toml::Value>,
) -> ::std::collections::BTreeSet<::std::string::String> {
  table
    .and_then(|t| t.get("controls"))
    .and_then(|c| c.as_array())
    .map(|a| {
      a.iter()
        .filter_map(|v| v.as_str().map(::std::string::String::from))
        .collect()
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  const PACK: &str = "[meta]\nstrictness = \"as_configured\"\n\n\
     [subject.telepresence]\ncontrols = [\"signal_notice\", \"aph_mandate\"]\n\n\
     [subject.import]\ncontrols = [\"attestation\"]\n\n\
     [legally_required]\ncontrols = [\"attestation\"]\n\n\
     [prohibited]\ncontrols = [\"voice_biometric\"]\n";

  /// Why: `referenced_controls` is the shared definition of "what a pack uses"
  /// for BOTH merge's delta detection and validate's catalog-membership rule —
  /// if it missed a source table (subjects, floor, or prohibited), merge would
  /// under-flag legal re-review and validate would under-lint, silently.
  #[test]
  fn referenced_controls_unions_all_three_sources() {
    let pack: ::toml::Value = ::toml::from_str(PACK).expect("pack parses");
    let refs = super::referenced_controls(&pack);
    let want: ::std::collections::BTreeSet<::std::string::String> =
      ["signal_notice", "aph_mandate", "attestation", "voice_biometric"]
        .into_iter()
        .map(::std::string::String::from)
        .collect();
    ::std::assert_eq!(refs, want);
  }

  /// Why: absent tables must read as EMPTY, not error — packs legitimately omit
  /// `[prohibited]` (most do) and templates omit subjects; an accessor that
  /// panicked or errored on absence would break validate on every minimal pack.
  #[test]
  fn absent_tables_read_as_empty() {
    let pack: ::toml::Value = ::toml::from_str("[meta]\nstrictness = \"aggressive\"\n")
      .expect("pack parses");
    ::std::assert!(super::control_list(&pack, "legally_required").is_empty());
    ::std::assert!(super::subject_tables(&pack).is_empty());
    ::std::assert!(super::referenced_controls(&pack).is_empty());
  }
}
