//! `r14n merge` — the gettext-msgmerge analogue (spec §9 lifecycle).
//!
//! Diffs a deployed pack against the CURRENT catalog and flags ONLY the changed
//! controls for legal re-review: controls newly added to the catalog since the
//! pack was reviewed (`NEEDS-LEGAL-REVIEW`) and controls the pack references
//! that no longer exist (`STALE`). The pack body is preserved BYTE-FOR-BYTE
//! (comments included) — the report rides in a prepended comment block, so a
//! reviewer sees exactly what changed and nothing is silently rewritten.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI).
//! - 2026-07-06: DRY pass — control extraction moved to the shared `packtoml`.

/// The delta between a pack's referenced controls and the current catalog.
pub struct MergeReport {
  /// Catalog controls the pack does not reference yet (need legal review).
  pub added: ::std::vec::Vec<::std::string::String>,
  /// Pack-referenced controls absent from the catalog (stale keys).
  pub stale: ::std::vec::Vec<::std::string::String>,
}

/// Merge: returns the updated pack text (report block + verbatim body) + the
/// structured report.
pub fn merge(
  pack_text: &str,
  catalog: &crate::catalog::Catalog,
) -> ::std::result::Result<(::std::string::String, MergeReport), ::std::string::String> {
  let pack: ::toml::Value =
    ::toml::from_str(pack_text).map_err(|e| ::std::format!("pack does not parse: {e}"))?;
  let referenced = crate::packtoml::referenced_controls(&pack);
  let catalog_keys: ::std::collections::BTreeSet<::std::string::String> =
    catalog.controls.keys().cloned().collect();
  let report = MergeReport {
    added: catalog_keys.difference(&referenced).cloned().collect(),
    stale: referenced.difference(&catalog_keys).cloned().collect(),
  };
  if report.added.is_empty() && report.stale.is_empty() {
    return ::std::result::Result::Ok((::std::string::String::from(pack_text), report));
  }
  let mut out = ::std::string::String::new();
  out.push_str("# ── r14n merge report — RE-REVIEW REQUIRED (NOT LEGAL ADVICE) ──\n");
  out.push_str(&::std::format!("# Catalog: domain {} — this pack drifted from it.\n", catalog.domain));
  for key in &report.added {
    out.push_str(&::std::format!(
      "# NEEDS-LEGAL-REVIEW: control `{key}` is in the catalog but absent from this pack —\n#   counsel must place it (subject tables / floor / prohibited) or record why not.\n"
    ));
  }
  for key in &report.stale {
    out.push_str(&::std::format!(
      "# STALE: control `{key}` is referenced here but no longer in the catalog — remove or re-add to the catalog.\n"
    ));
  }
  out.push_str("# After counsel review: set [meta.legal_review] status accordingly and update\n");
  out.push_str("# last_reviewed_against_guidance. Delete this block once resolved.\n");
  out.push_str("# ───────────────────────────────────────────────────────────────\n");
  out.push_str(pack_text);
  ::std::result::Result::Ok((out, report))
}

#[cfg(test)]
mod tests {
  /// Why: merge is the msgmerge analogue — its whole value is flagging EXACTLY
  /// the catalog delta for legal re-review while never rewriting counsel's
  /// reviewed text. A missed `added` under-flags legal review; a mangled body
  /// would silently alter a reviewed artifact. The output must also still parse
  /// (the report block is comments only).
  #[test]
  fn flags_added_and_stale_controls_and_preserves_body() {
    let pack = "# original comment\n[meta]\nstrictness = \"as_configured\"\n\n\
       [subject.telepresence]\ncontrols = [\"signal_notice\", \"retired_control\"]\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let (out, report) = super::merge(pack, &crate::catalog::test_catalog()).expect("merge");
    ::std::assert_eq!(report.added, ["aph_mandate"], "aph_mandate is new since review");
    ::std::assert_eq!(report.stale, ["retired_control"]);
    ::std::assert!(out.contains("NEEDS-LEGAL-REVIEW: control `aph_mandate`"));
    ::std::assert!(out.contains("STALE: control `retired_control`"));
    ::std::assert!(out.ends_with(pack), "body must be preserved verbatim");
    let parsed: ::std::result::Result<::toml::Value, _> = ::toml::from_str(&out);
    ::std::assert!(parsed.is_ok(), "merged output must remain valid TOML: {parsed:?}");
  }

  /// Why: a no-drift merge must be a byte-for-byte no-op — if it prepended an
  /// empty report block anyway, every CI re-run would churn reviewed packs and
  /// bury real deltas in noise.
  #[test]
  fn no_drift_returns_pack_unchanged() {
    let pack = "[meta]\nstrictness = \"as_configured\"\n\n\
       [subject.telepresence]\ncontrols = [\"signal_notice\", \"aph_mandate\"]\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let (out, report) = super::merge(pack, &crate::catalog::test_catalog()).expect("merge");
    ::std::assert!(report.added.is_empty() && report.stale.is_empty());
    ::std::assert_eq!(out, pack);
  }
}
