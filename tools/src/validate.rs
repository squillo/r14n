// SPDX-License-Identifier: Apache-2.0
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
//! - 2026-07-06: DRY pass — control extraction moved to the shared `packtoml`.

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

/// True iff `s` is an ISO calendar date `YYYY-MM-DD` (shape check only — the
/// linter is std-only, so it validates the format, not calendar validity).
fn is_iso_date(s: &str) -> bool {
  let b = s.as_bytes();
  b.len() == 10
    && b[4] == b'-'
    && b[7] == b'-'
    && b[..4].iter().all(u8::is_ascii_digit)
    && b[5..7].iter().all(u8::is_ascii_digit)
    && b[8..10].iter().all(u8::is_ascii_digit)
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
  let floor = crate::packtoml::control_list(&pack, "legally_required");
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
        if meta.and_then(|m| m.get("effective_from")).and_then(|v| v.as_str()).is_none() {
          errors.push(::std::string::String::from(
            "approved pack missing meta.effective_from (spec §2.5/§7 text-in-effect envelope)",
          ));
        }
      }
    }
  }

  // Date-typed fields must be ISO YYYY-MM-DD (council-audit N5: the extract
  // template's `TODO-YYYY-MM-DD` placeholders previously lint-passed and flowed
  // into the content-addressed index). A malformed date is a warning on a draft
  // (templates carry placeholders) but an error on any non-draft pack.
  let is_draft = review
    .and_then(|r| r.get("status"))
    .and_then(|s| s.as_str())
    == ::std::option::Option::Some("draft");
  for field in ["effective_from", "effective_until", "last_reviewed_against_guidance"] {
    if let ::std::option::Option::Some(v) = meta.and_then(|m| m.get(field)).and_then(|v| v.as_str()) {
      if !is_iso_date(v) {
        let msg = ::std::format!("{field} = \"{v}\" is not an ISO date (YYYY-MM-DD)");
        if is_draft {
          warnings.push(msg);
        } else {
          errors.push(msg);
        }
      }
    }
  }
  if let ::std::option::Option::Some(v) = review.and_then(|r| r.get("review_date")).and_then(|v| v.as_str()) {
    if !is_iso_date(v) {
      errors.push(::std::format!("legal_review.review_date = \"{v}\" is not an ISO date"));
    }
  }

  // Effective-date envelope ordering (spec §7; ISO dates compare lexically).
  let from = meta.and_then(|m| m.get("effective_from")).and_then(|v| v.as_str());
  let until = meta.and_then(|m| m.get("effective_until")).and_then(|v| v.as_str());
  if let (::std::option::Option::Some(f), ::std::option::Option::Some(u)) = (from, until) {
    if is_iso_date(f) && is_iso_date(u) && f > u {
      errors.push(::std::format!("effective_from {f} is after effective_until {u}"));
    }
  }

  // Data-minimization consistency (spec §2.5/§3).
  let prohibited = crate::packtoml::control_list(&pack, "prohibited");
  let conflict: ::std::vec::Vec<&::std::string::String> = prohibited.intersection(&floor).collect();
  if !conflict.is_empty() {
    errors.push(::std::format!(
      "[prohibited] intersects [legally_required] {conflict:?} — a pack cannot require what it forbids (spec §2.5)"
    ));
  }
  for (name, controls) in crate::packtoml::subject_tables(&pack) {
    let overlap: ::std::vec::Vec<&::std::string::String> =
      controls.intersection(&prohibited).collect();
    if !overlap.is_empty() {
      warnings.push(::std::format!(
        "[subject.{name}] lists prohibited controls {overlap:?} — resolvers subtract them (guard), but the table should not list them"
      ));
    }
  }

  // Catalog membership (the linter half of schema/pack.schema.json's $comment).
  if let ::std::option::Option::Some(cat) = catalog {
    let referenced = crate::packtoml::referenced_controls(&pack);
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
  const GOOD: &str = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
     [subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";

  /// Why: the linter must not cry wolf — a well-formed draft pack passing is
  /// the baseline that keeps every error assertion below meaningful (if this
  /// fails, the other tests' errors are noise, not signal).
  #[test]
  fn good_pack_passes_with_catalog() {
    let f = super::validate(GOOD, ::std::option::Option::Some(&crate::catalog::test_catalog()));
    ::std::assert!(f.ok(), "errors: {:?}", f.errors);
  }

  /// Why: the two packs we actually ship are the counsel-safe public subset —
  /// if the linter ever rejects our own artifacts, either the packs drifted or
  /// a lint rule broke; both need to fail the build, not be found at publish.
  #[test]
  fn shipped_baseline_packs_pass_without_catalog() {
    for rel in ["../packs/aggressive/recording_consent.r14n.toml", "../packs/minimal/recording_consent.r14n.toml"] {
      let path = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join(rel);
      let text = ::std::fs::read_to_string(&path).expect("read baseline pack");
      let f = super::validate(&text, ::std::option::Option::None);
      ::std::assert!(f.ok(), "{rel} errors: {:?}", f.errors);
    }
  }

  /// Why: spec §2.5/§4 — a pack without a floor BLOCKS at resolve time and a
  /// pack without a review status has no provenance; the linter must catch both
  /// at author time so they never reach a resolver.
  #[test]
  fn missing_floor_and_review_status_are_errors() {
    let f = super::validate("[meta]\nstrictness = \"minimal\"\n", ::std::option::Option::None);
    ::std::assert!(!f.ok());
    ::std::assert!(f.errors.iter().any(|e| e.contains("legally_required")));
    ::std::assert!(f.errors.iter().any(|e| e.contains("legal_review")));
  }

  /// Why: spec §6 makes the attorney-of-record envelope NON-OPTIONAL for
  /// approved (jurisdiction-claiming) packs, §7 requires the
  /// interpretation-currency date, and §2.5/§7 the effective-date envelope —
  /// this is the UPL/counsel gate enforced in code, exactly 6 findings so a
  /// dropped rule is caught.
  #[test]
  fn approved_pack_requires_attorney_envelope_currency_and_effective_dates() {
    let pack = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"approved\"\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert_eq!(
      f.errors.iter().filter(|e| e.contains("approved pack missing")).count(),
      6,
      "attorney_of_record + jurisdiction + bar_credential + review_date + currency + effective_from: {:?}",
      f.errors
    );
  }

  /// Why: spec §2.5 — a pack that both requires and forbids a control is
  /// self-contradictory; resolvers fail toward not-adding (guard), but the
  /// authoring error must be rejected before counsel ever signs it.
  #[test]
  fn prohibited_intersecting_floor_is_an_error() {
    let pack = "[meta]\nstrictness = \"aggressive\"\n\n[meta.legal_review]\nstatus = \"not_required\"\n\n\
       [legally_required]\ncontrols = [\"attestation\", \"aph_mandate\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert!(f.errors.iter().any(|e| e.contains("cannot require what it forbids")), "{:?}", f.errors);
  }

  /// Why: spec §2.1 — packs reference catalog keys, never define them; a typo'd
  /// control would silently never be enforced (the resolver intersects with the
  /// caller's universe), so the ONLY place a typo is catchable is this lint.
  #[test]
  fn unknown_control_vs_catalog_is_an_error() {
    let pack = "[meta]\nstrictness = \"as_configured\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
       [subject.telepresence]\ncontrols = [\"made_up_control\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::Some(&crate::catalog::test_catalog()));
    ::std::assert!(f.errors.iter().any(|e| e.contains("made_up_control")), "{:?}", f.errors);
  }

  /// Why: spec §3 keeps posture forward-additive (`Other`), and resolvers fail
  /// CLOSED on unknown values — so an unknown posture is safe (warning), and
  /// hard-erroring would break packs written for future spec versions.
  #[test]
  fn unknown_posture_is_a_warning_not_error() {
    let pack = "[meta]\nstrictness = \"turbo\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
       [legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(pack, ::std::option::Option::None);
    ::std::assert!(f.ok(), "{:?}", f.errors);
    ::std::assert!(f.warnings.iter().any(|w| w.contains("turbo")));
  }

  /// Why: council-audit N5 — a placeholder/garbage date is a warning while a
  /// pack is a draft (templates carry `TODO-YYYY-MM-DD`), but on any non-draft
  /// pack it must be an ERROR so a malformed date can never reach the
  /// content-addressed index via publish.
  #[test]
  fn malformed_date_warns_on_draft_but_errors_on_non_draft() {
    let draft = "[meta]\nstrictness = \"minimal\"\neffective_from = \"TODO-YYYY-MM-DD\"\n\n\
       [meta.legal_review]\nstatus = \"draft\"\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(draft, ::std::option::Option::None);
    ::std::assert!(f.ok(), "draft placeholder date is a warning, not an error: {:?}", f.errors);
    ::std::assert!(f.warnings.iter().any(|w| w.contains("effective_from")));

    let signed = "[meta]\nstrictness = \"minimal\"\neffective_from = \"TODO-YYYY-MM-DD\"\n\n\
       [meta.legal_review]\nstatus = \"requires_signoff\"\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
    let f = super::validate(signed, ::std::option::Option::None);
    ::std::assert!(!f.ok(), "non-draft malformed date must error");
    ::std::assert!(f.errors.iter().any(|e| e.contains("effective_from")));
  }

  /// Why: council-audit E1 — the linter parses untrusted pack text; it must
  /// NEVER panic on garbage (a panic is a DoS on any CI/hook that lints
  /// contributed packs). Malformed input yields a Findings with errors, not a
  /// crash.
  #[test]
  fn validate_never_panics_on_arbitrary_input() {
    let cases = [
      "",
      "\u{0}\u{1}not toml",
      "[[[[",
      "[meta]\nstrictness = 5",
      "[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = 7",
      "[subject.x]\ncontrols = [1,2]\n[legally_required]\ncontrols=[\"a\"]",
      "[prohibited]\ncontrols = [\"a\"]\n[legally_required]\ncontrols=[\"a\"]",
      "[meta]\nstrictness=\"turbo\"\ndescription=\"\u{202e}\u{1f4a3}\"",
    ];
    for case in cases {
      let f = super::validate(case, ::std::option::Option::None);
      // No panic reaching here IS the test; a broken pack simply has errors.
      let _ = (f.ok(), f.errors.len(), f.warnings.len());
    }
  }
}
