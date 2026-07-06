//! `r14n validate` — the pack LINTER (spec §2.5/§3/§6/§7).
//!
//! Enforces the cross-field rules `schema/pack.schema.json` can only document:
//! the floor MUST exist, approved packs MUST carry the attorney-of-record
//! envelope + the interpretation-currency date, `[prohibited]` MUST NOT
//! intersect the floor, and (with `--catalog`) every referenced control MUST
//! be declared. Errors fail the pack; warnings surface authoring smells.
//! NOT LEGAL ADVICE — passing validation is a FORMAT claim, never a
//! compliance claim.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI).

/// Validation outcome.
pub struct Findings {
  /// Rule violations — the pack MUST NOT ship.
  pub errors: ::std::vec::Vec<::std::string::String>,
  /// Authoring smells — review recommended.
  pub warnings: ::std::vec::Vec<::std::string::String>,
}

impl Findings {
  /// True when the pack passes (warnings allowed).
  pub fn ok(&self) -> bool {
    self.errors.is_empty()
  }
}

const KNOWN_POSTURES: &[&str] = &["aggressive", "as_configured", "minimal"];
const KNOWN_REVIEW_STATUS: &[&str] = &["not_required", "draft", "requires_signoff", "approved"];

fn list_of(pack: &::toml::Value, table: &str) -> ::std::collections::BTreeSet<::std::string::String> {
  pack
    .get(table)
    .and_then(|t| t.get("controls"))
    .and_then(|c| c.as_array())
    .map(|a| {
      a.iter()
        .filter_map(|v| v.as_str().map(::std::string::String::from))
        .collect()
    })
    .unwrap_or_default()
}

/// Validate pack text against the RLPS pack rules (+ optional catalog).
pub fn validate(
  pack_text: &str,
  catalog: ::std::option::Option<&crate::catalog::Catalog>,
) -> Findings {
  let mut errors: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  let mut warnings: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();

  let pack: ::toml::Value = match ::toml::from_str(pack_text) {
    ::std::result::Result::Ok(p) => p,
    ::std::result::Result::Err(e) => {
      errors.push(::std::format!("not valid TOML: {e}"));
      return Findings { errors, warnings };
    }
  };

  // [meta] + posture (wire field `strictness`, spec §3).
  let meta = pack.get("meta");
  match meta.and_then(|m| m.get("strictness")).and_then(|s| s.as_str()) {
    ::std::option::Option::None => {
      errors.push(::std::string::String::from("[meta] strictness is required (spec §3)"));
    }
    ::std::option::Option::Some(s) if KNOWN_POSTURES.contains(&s) => {}
    ::std::option::Option::Some(s) => warnings.push(::std::format!(
      "unknown posture \"{s}\" — spec §3 permits forward-additive values; conforming resolvers fail closed on it"
    )),
  }

  // Floor (spec §2.5: MUST provide [legally_required]).
  let floor = list_of(&pack, "legally_required");
  match pack.get("legally_required") {
    ::std::option::Option::None => {
      errors.push(::std::string::String::from(
        "[legally_required] floor is required (spec §2.5; a missing floor blocks, §4)",
      ));
    }
    ::std::option::Option::Some(_) if floor.is_empty() => warnings.push(
      ::std::string::String::from("[legally_required] floor is EMPTY — under `minimal` nothing is enforced"),
    ),
    _ => {}
  }

  // [meta.legal_review] (spec §6).
  let review = meta.and_then(|m| m.get("legal_review"));
  match review.and_then(|r| r.get("status")).and_then(|s| s.as_str()) {
    ::std::option::Option::None => errors.push(::std::string::String::from(
      "[meta.legal_review] status is required (spec §2.5/§6)",
    )),
    ::std::option::Option::Some(status) => {
      if !KNOWN_REVIEW_STATUS.contains(&status) {
        errors.push(::std::format!("legal_review.status \"{status}\" is not one of {KNOWN_REVIEW_STATUS:?}"));
      }
      if status == "approved" {
        for field in ["reviewing_attorney_of_record", "jurisdiction", "bar_credential", "review_date"] {
          if review.and_then(|r| r.get(field)).and_then(|v| v.as_str()).is_none() {
            errors.push(::std::format!(
              "approved pack missing legal_review.{field} (spec §6 — NOT optional for jurisdiction claims)"
            ));
          }
        }
        if meta
          .and_then(|m| m.get("last_reviewed_against_guidance"))
          .and_then(|v| v.as_str())
          .is_none()
        {
          errors.push(::std::string::String::from(
            "approved pack missing meta.last_reviewed_against_guidance (spec §7 temporal split)",
          ));
        }
      }
    }
  }

  // Effective-date envelope ordering (spec §7; ISO dates compare lexically).
  let from = meta.and_then(|m| m.get("effective_from")).and_then(|v| v.as_str());
  let until = meta.and_then(|m| m.get("effective_until")).and_then(|v| v.as_str());
  if let (::std::option::Option::Some(f), ::std::option::Option::Some(u)) = (from, until) {
    if f > u {
      errors.push(::std::format!("effective_from {f} is after effective_until {u}"));
    }
  }

  // Data-minimization consistency (spec §2.5/§3).
  let prohibited = list_of(&pack, "prohibited");
  let conflict: ::std::vec::Vec<&::std::string::String> = prohibited.intersection(&floor).collect();
  if !conflict.is_empty() {
    errors.push(::std::format!(
      "[prohibited] intersects [legally_required] {conflict:?} — a pack cannot require what it forbids (spec §2.5)"
    ));
  }
  if let ::std::option::Option::Some(subjects) = pack.get("subject").and_then(|s| s.as_table()) {
    for (name, table) in subjects {
      let controls: ::std::collections::BTreeSet<::std::string::String> = table
        .get("controls")
        .and_then(|c| c.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(::std::string::String::from)).collect())
        .unwrap_or_default();
      let overlap: ::std::vec::Vec<&::std::string::String> =
        controls.intersection(&prohibited).collect();
      if !overlap.is_empty() {
        warnings.push(::std::format!(
          "[subject.{name}] lists prohibited controls {overlap:?} — resolvers subtract them (guard), but the table should not list them"
        ));
      }
    }
  }

  // Catalog membership (the linter half of schema/pack.schema.json's $comment).
  if let ::std::option::Option::Some(cat) = catalog {
    let referenced = crate::merge::referenced_controls(&pack);
    for key in &referenced {
      if !cat.controls.contains_key(key) {
        errors.push(::std::format!(
          "control \"{key}\" is not declared in the {} catalog (packs reference keys, never define them — spec §2.1)",
          cat.domain
        ));
      }
    }
  }

  Findings { errors, warnings }
}

#[cfg(test)]
mod tests {
  fn catalog() -> crate::catalog::Catalog {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let path = tmp.path().join("c.toml");
    ::std::fs::write(&path, crate::catalog::TEST_CATALOG).expect("write");
    crate::catalog::load(&path).expect("load")
  }

  const GOOD: &str = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
     [subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";

  #[test]
  fn good_pack_passes_with_catalog() {
    let f = super::validate(GOOD, ::std::option::Option::Some(&catalog()));
    ::std::assert!(f.ok(), "errors: {:?}", f.errors);
  }

  #[test]
  fn shipped_baseline_packs_pass_without_catalog() {
    for rel in ["../packs/aggressive/recording_consent.r14n.toml", "../packs/minimal/recording_consent.r14n.toml"] {
      let path = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join(rel);
      let text = ::std::fs::read_to_string(&path).expect("read baseline pack");
      let f = super::validate(&text, ::std::option::Option::None);
      ::std::assert!(f.ok(), "{rel} errors: {:?}", f.errors);
    }
  }

  #[test]
  fn missing_floor_and_review_status_are_errors() {
    let f = super::validate("[meta]\nstrictness = \"minimal\"\n", ::std::option::Option::None);
    ::std::assert!(!f.ok());
    ::std::assert!(f.errors.iter().any(|e| e.contains("legally_required")));
    ::std::assert!(f.errors.iter().any(|e| e.contains("legal_review")));
  }

  #[test]
  fn approved_pack_requires_attorney_envelope_and_currency_date() {
    let pack = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"approved\"\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert_eq!(
      f.errors.iter().filter(|e| e.contains("approved pack missing")).count(),
      5,
      "attorney_of_record + jurisdiction + bar_credential + review_date + currency date: {:?}",
      f.errors
    );
  }

  #[test]
  fn prohibited_intersecting_floor_is_an_error() {
    let pack = "[meta]\nstrictness = \"aggressive\"\n\n[meta.legal_review]\nstatus = \"not_required\"\n\n\
       [legally_required]\ncontrols = [\"attestation\", \"aph_mandate\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert!(f.errors.iter().any(|e| e.contains("cannot require what it forbids")), "{:?}", f.errors);
  }

  #[test]
  fn unknown_control_vs_catalog_is_an_error() {
    let pack = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
       [subject.telepresence]\ncontrols = [\"made_up_control\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::Some(&catalog()));
    ::std::assert!(f.errors.iter().any(|e| e.contains("made_up_control")), "{:?}", f.errors);
  }

  #[test]
  fn unknown_posture_is_a_warning_not_error() {
    let pack = "[meta]\nstrictness = \"turbo\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert!(f.ok(), "{:?}", f.errors);
    ::std::assert!(f.warnings.iter().any(|w| w.contains("turbo")));
  }
}
