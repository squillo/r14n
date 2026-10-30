// SPDX-License-Identifier: Apache-2.0
//! Framework-link tests — the OPTIONAL `frameworks` block a catalog control may carry,
//! validated against `schema/catalog.schema.json` with INLINE FICTIONAL fixtures.
//!
//! A control can name the compliance-framework EDITIONS it relates to: a closed set of
//! snake_case edition ids, each with a closed `relation` (`informs` | `contributes` |
//! `necessary` — deliberately never "satisfies"), optional `criteria`, and an optional
//! `review`. The block is a pointer for a human reviewer. It is not a determination, it never
//! says a control is sufficient for anything, and a resolver MUST NOT vary a decision on it.
//!
//! ⚠ The contract ships AHEAD of its content. No shipped catalog populates `frameworks`, and
//! a guard test in this file fails the build if one does before a maintainer opens the
//! licensing gate on purpose.
//!
//! Every fixture is fictional on purpose: the criterion placeholder is `EXAMPLE-1`, never a real
//! identifier of any framework (licensing for those is uncleared; see `docs/framework-links.md`
//! section 6).
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-10-30: authored — the typed catalog-to-framework link contract, schema side.
mod common;

/// The schema under test. `common::validator` builds it and fails the build if it is not itself a
/// valid Draft 2020-12 schema; `common::toml_text_as_json` panics on a fixture that is not valid
/// TOML, so a typo'd fixture fails its test instead of counting as a "rejection".
const SCHEMA: &str = "schema/catalog.schema.json";

/// A minimal VALID one-control catalog. Every fixture below appends its `frameworks` tables to
/// this, so the only thing that varies between a pass and a refusal is the block under test.
/// Fictional domain, fictional control.
const BASE_CATALOG: &str = r#"
[domain]
name = "example"
description = "A fictional catalog used only by schema tests."

[control.example_key]
kind = "obligation"
title = "Example control"
"#;

/// `BASE_CATALOG` plus `frameworks` (TOML tables under `[control.example_key.frameworks.*]`),
/// as JSON.
fn catalog_with(frameworks: &str) -> ::serde_json::Value {
  common::toml_text_as_json(&::std::format!("{}\n{}\n", BASE_CATALOG, frameworks))
}

/// The schema must ACCEPT this block. `what` names the arm in the failure message.
fn assert_accepted(v: &::jsonschema::Validator, what: &str, frameworks: &str) {
  let errs = common::errors(v, &catalog_with(frameworks));
  ::std::assert!(errs.is_empty(), "{what}: expected ACCEPTED but the schema refused it: {errs:?}");
}

/// The schema must REFUSE this block, and for the stated reason. Every call is paired in its test
/// with an accepted control that differs minimally, and `why` must appear in the schema's own
/// error text: "something was refused" is also true of a fixture broken some unrelated way (a
/// misspelt table, a wrong type), which would pass this arm for a reason that has nothing to do
/// with the rule it claims to pin.
fn assert_rejected(v: &::jsonschema::Validator, what: &str, frameworks: &str, why: &str) {
  let errs = common::errors(v, &catalog_with(frameworks));
  ::std::assert!(!errs.is_empty(), "{what}: expected REFUSED but the schema accepted it");
  ::std::assert!(
    errs.iter().any(|e| e.contains(why)),
    "{what}: refused, but not for the stated reason {why:?}; the schema said {errs:?}"
  );
}

/// Why: the whole contract is only worth shipping if the plain, intended use validates. This is
/// the positive control for every refusal arm below: if a minimal link were refused, each
/// "rejected" test in this file would pass for the wrong reason (the whole block refused, not
/// the defect under test). Covers the three shapes an author writes — relation only, every
/// optional facet populated, and two editions on one control.
#[test]
fn a_valid_link_is_accepted() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "minimal link, relation only",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_accepted(
    &v,
    "full link, relation + criteria + review with counsel",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "contributes"
      criteria = ["EXAMPLE-1", "EXAMPLE-2"]
      review = { status = "requires_signoff", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_accepted(
    &v,
    "two editions on one control",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"

      [control.example_key.frameworks.iso27001_2022_a1_2024]
      relation = "contributes"
    "#,
  );
}

/// Why: the edition set is CLOSED, so a typo in one of the four ids would silently refuse every
/// author who spells it correctly — and none of the other tests would notice, because they
/// mostly use one edition. Each id gets its own catalog so a bad one is named by the failure.
/// `hipaa_45cfr164_2013` carries the `review` it requires, so this arm tests the id, not the
/// review rule (that has its own tests).
#[test]
fn every_declared_edition_is_accepted_as_a_key() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "soc2_tsc_2017",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_accepted(
    &v,
    "iso27001_2022_a1_2024",
    r#"
      [control.example_key.frameworks.iso27001_2022_a1_2024]
      relation = "informs"
    "#,
  );
  assert_accepted(
    &v,
    "hipaa_45cfr164_2013 (with the review it requires)",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "informs"
      review = { status = "draft" }
    "#,
  );
  assert_accepted(
    &v,
    "hecvat_4_1_6",
    r#"
      [control.example_key.frameworks.hecvat_4_1_6]
      relation = "informs"
    "#,
  );
}

/// Why: the closed edition set and the single-spelling rule. An edition id is long-lived audit
/// evidence, so an unknown one must be refused rather than carried — and the wire spelling is
/// the snake_case id with NO alias table in code, so the hyphenated informal spelling and a
/// case variant must be refused too (accepting them would force every reader to normalize, and
/// a receipt would then carry two spellings of one edition). One bad key refuses the whole
/// block even beside a good one, so a catalog cannot smuggle an unknown edition in alongside a
/// valid link. Control: the same link under a declared edition is accepted.
#[test]
fn an_unknown_edition_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: the same link under a declared edition",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_rejected(
    &v,
    "an edition the vocabulary has not declared",
    r#"
      [control.example_key.frameworks.example_framework_2099]
      relation = "informs"
    "#,
    "is not one of \"soc2_tsc_2017\"",
  );
  assert_rejected(
    &v,
    "hyphenated informal alias of a declared edition",
    r#"
      [control.example_key.frameworks."soc2-tsc-2017"]
      relation = "informs"
    "#,
    "is not one of \"soc2_tsc_2017\"",
  );
  assert_rejected(
    &v,
    "case variant of a declared edition",
    r#"
      [control.example_key.frameworks.SOC2_TSC_2017]
      relation = "informs"
    "#,
    "is not one of \"soc2_tsc_2017\"",
  );
  assert_rejected(
    &v,
    "a declared edition beside an undeclared one",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"

      [control.example_key.frameworks.example_framework_2099]
      relation = "informs"
    "#,
    "is not one of \"soc2_tsc_2017\"",
  );
}

/// Why: "no control is sufficient for a framework criterion" is the load-bearing hedge of this
/// whole feature, and it is enforced by the relation set being CLOSED, not by a comment. A
/// `satisfies` relation (or its synonyms `meets` / `complies`) would turn a pointer for a human
/// into a compliance claim about someone else's posture. Control: both permitted non-gating
/// relations are accepted, so the refusal is about the value and not the shape.
#[test]
fn a_satisfies_relation_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: relation informs",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_accepted(
    &v,
    "control: relation contributes",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "contributes"
    "#,
  );
  assert_rejected(
    &v,
    "relation satisfies",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "satisfies"
    "#,
    "is not one of \"informs\"",
  );
  assert_rejected(
    &v,
    "relation meets",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "meets"
    "#,
    "is not one of \"informs\"",
  );
  assert_rejected(
    &v,
    "relation complies",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "complies"
    "#,
    "is not one of \"informs\"",
  );
}

/// Why: `relation` is the one required facet — without it a link says only "related somehow",
/// which is the weakest claim a reader can misread upward. Required rather than defaulted on
/// purpose, the same reasoning as a control's `kind`: a loader that defaulted it would turn an
/// omission into an assertion nobody wrote. Also pins that an approved review cannot stand in
/// for the missing relation. Control: the same link with a relation is accepted.
#[test]
fn a_link_without_a_relation_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: criteria with a relation",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1"]
    "#,
  );
  assert_rejected(
    &v,
    "criteria without a relation",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      criteria = ["EXAMPLE-1"]
    "#,
    "\"relation\" is a required property",
  );
  assert_rejected(
    &v,
    "an approved review without a relation",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
    "\"relation\" is a required property",
  );
  assert_rejected(
    &v,
    "an empty link",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
    "#,
    "\"relation\" is a required property",
  );
}

/// Why: `necessary` is the one relation that reads as a claim about someone else's compliance
/// posture, so the schema makes it UNSAYABLE without a reviewed sign-off — the if/then on
/// `framework_link`. A bare `necessary` is the exact shape an unreviewed bulk import would
/// produce. The `if` carries its own `required: [relation]`, and this test is also the proof the
/// `then` actually fires: the approved control below is accepted, so a refusal here can only be
/// the missing review.
#[test]
fn necessary_without_a_review_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: necessary with an approved review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_rejected(
    &v,
    "necessary with no review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
    "#,
    "\"review\" is a required property",
  );
  assert_rejected(
    &v,
    "necessary with criteria but no review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      criteria = ["EXAMPLE-1"]
    "#,
    "\"review\" is a required property",
  );
}

/// Why: a review block that EXISTS is not enough for `necessary` — `approved` is the only status
/// that records a sign-off. `draft` and `requires_signoff` are exactly the states in which
/// someone has NOT yet looked, and letting either unlock the strongest relation would defeat the
/// gate while appearing to satisfy it. Control: the identical link with `approved` is accepted.
#[test]
fn necessary_with_a_non_approved_review_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: necessary with status approved",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_rejected(
    &v,
    "necessary with status draft",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "draft" }
    "#,
    "\"approved\" was expected",
  );
  assert_rejected(
    &v,
    "necessary with status requires_signoff",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "requires_signoff" }
    "#,
    "\"approved\" was expected",
  );
}

/// Why: the positive half of the `necessary` gate. A rule that refused every `necessary` link
/// would also pass the two refusal tests above, so this pins that a properly approved link is
/// accepted in each shape an author writes — the minimum (an approval naming its reviewer) and
/// with criteria. An approval WITHOUT a named reviewer is refused; that is pinned separately
/// by `an_approved_review_must_name_its_reviewer`.
#[test]
fn necessary_with_an_approved_review_is_accepted() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "necessary, approved, counsel named (the minimum)",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_accepted(
    &v,
    "necessary, approved, with criteria",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      criteria = ["EXAMPLE-1"]
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
}

/// Why: guards the if/then against OVER-reaching. The review gate must bind `necessary` and
/// nothing else — if `informs` or `contributes` needed an approved review, authors would
/// either stop linking or approve reviews that never happened, and the sign-off would stop
/// meaning anything. Both weaker relations are accepted with no review and with a draft one.
#[test]
fn informs_and_contributes_need_no_review() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "informs, no review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_accepted(
    &v,
    "contributes, no review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "contributes"
    "#,
  );
  assert_accepted(
    &v,
    "informs, draft review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft" }
    "#,
  );
  assert_accepted(
    &v,
    "contributes, requires_signoff review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "contributes"
      review = { status = "requires_signoff" }
    "#,
  );
}

/// Why: the regulatory edition `hipaa_45cfr164_2013` makes a link read as a claim about the law,
/// and an unreviewed claim of that kind must not ship — so ANY link to it, whatever its relation,
/// must carry a review block. The rule is per ENTRY: a review on a different edition's link must
/// not satisfy it. It is also edition-specific: the identical review-less link under a
/// non-regulatory edition is accepted (the control), so this is not a blanket review requirement.
#[test]
fn a_hipaa_link_without_a_review_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: the same review-less link under a non-regulatory edition",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_rejected(
    &v,
    "hipaa, informs, no review",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "informs"
    "#,
    "\"review\" is a required property",
  );
  assert_rejected(
    &v,
    "hipaa, contributes with criteria, no review",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "contributes"
      criteria = ["EXAMPLE-1"]
    "#,
    "\"review\" is a required property",
  );
  assert_rejected(
    &v,
    "a review on the soc2 link does not cover the hipaa link",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }

      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "informs"
    "#,
    "\"review\" is a required property",
  );
}

/// Why: the positive half of the hipaa rule. The requirement is that a review block be PRESENT,
/// not that it be approved — a `draft` review on an `informs` link is the honest state of work
/// in progress and must validate, otherwise the rule would force authors to fake an approval.
/// (An approved review is still required where the relation is `necessary`; covered there.)
/// Also pins the combination of both rules: hipaa + necessary + approved is accepted.
#[test]
fn a_hipaa_link_with_a_review_is_accepted() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "hipaa, informs, draft review",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "informs"
      review = { status = "draft" }
    "#,
  );
  assert_accepted(
    &v,
    "hipaa, contributes, requires_signoff review",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "contributes"
      review = { status = "requires_signoff", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_accepted(
    &v,
    "hipaa, necessary, approved review",
    r#"
      [control.example_key.frameworks.hipaa_45cfr164_2013]
      relation = "necessary"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
}

/// Why: an EMPTY `criteria` list reads as "answered: no criteria apply", which is worse than
/// absent — the same rule `superseded_by` states for an empty successor list, enforced by
/// `minItems`. Control: a one-element list and an absent list are both accepted, so the refusal
/// is specifically the empty list.
#[test]
fn an_empty_criteria_list_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: one criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1"]
    "#,
  );
  assert_accepted(
    &v,
    "control: criteria absent",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_rejected(
    &v,
    "criteria = []",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = []
    "#,
    "less than 1 item",
  );
}

/// Why: criteria identifiers are free-form strings because the identifier space belongs to each
/// framework's publisher, and licensing for the real ones is uncleared. The shipped contract
/// therefore has to be exercisable with an obviously fictional placeholder — `EXAMPLE-1` — and
/// that is the only kind of identifier any fixture or doc in this repository may carry.
#[test]
fn placeholder_criteria_are_accepted() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "one placeholder criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1"]
    "#,
  );
  assert_accepted(
    &v,
    "several placeholder criteria",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "contributes"
      criteria = ["EXAMPLE-1", "EXAMPLE-2", "EXAMPLE-3"]
    "#,
  );
}

/// Why: the remaining malformed-`criteria` shapes, each of which would otherwise ship into a
/// receipt reader's hands: an empty-string element (a blank cell exported from a spreadsheet),
/// a duplicate (the same criterion twice reads as two claims), a non-string element, and a bare
/// string where a list belongs. `uniqueItems` and `items` carry these, the same facets
/// `superseded_by` and `subject_rights` already use. Control: the well-formed list is accepted.
#[test]
fn malformed_criteria_are_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: two distinct non-empty criteria",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1", "EXAMPLE-2"]
    "#,
  );
  assert_rejected(
    &v,
    "an empty-string criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1", ""]
    "#,
    "is shorter than 1 character",
  );
  assert_rejected(
    &v,
    "a duplicated criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1", "EXAMPLE-1"]
    "#,
    "non-unique elements",
  );
  assert_rejected(
    &v,
    "a non-string criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = [1]
    "#,
    "is not of type \"string\"",
  );
  assert_rejected(
    &v,
    "a bare string where a list belongs",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = "EXAMPLE-1"
    "#,
    "is not of type \"array\"",
  );
}

/// Why: a link is a CLOSED object (`additionalProperties: false`), exactly as `control_def` is.
/// The likeliest stray keys are the ones that would assert more than the contract allows — a
/// `satisfied = true` flag, or a flattened `status = "approved"` that an author hopes unlocks
/// `necessary` without a review block. Either would be silently carried if the object were
/// open, and a reader would see a compliance claim the schema never granted. Control: the same
/// link without the stray key is accepted.
#[test]
fn an_unknown_property_inside_a_link_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: the link without a stray key",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_rejected(
    &v,
    "an invented satisfied flag",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      satisfied = true
    "#,
    "Additional properties are not allowed ('satisfied'",
  );
  assert_rejected(
    &v,
    "a status flattened out of review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      status = "approved"
    "#,
    "Additional properties are not allowed ('status'",
  );
  assert_rejected(
    &v,
    "a free-text notes key",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      notes = "anything"
    "#,
    "Additional properties are not allowed ('notes'",
  );
}

/// Why: the `review` block is also closed, and its `status` is both required and a closed set.
/// The pack contract's `legal_review` has a fourth status (`not_required`) that does NOT exist
/// here — a link either has a review in one of three states or has no review — and accepting the
/// pack vocabulary by analogy would let an author write a status the `necessary` gate was never
/// designed to read. An empty `counsel` is refused like every other empty string in this schema.
/// Control: a minimal draft review is accepted, as is one with counsel named.
#[test]
fn the_review_block_is_closed_and_needs_a_known_status() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: a minimal draft review",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft" }
    "#,
  );
  assert_accepted(
    &v,
    "control: a review with counsel named",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_rejected(
    &v,
    "a review with an unknown property",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft", reviewer = "someone" }
    "#,
    "Additional properties are not allowed ('reviewer'",
  );
  assert_rejected(
    &v,
    "a review with no status",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { counsel = "Example Counsel (fictional)" }
    "#,
    "\"status\" is a required property",
  );
  assert_rejected(
    &v,
    "an empty review table",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = {}
    "#,
    "\"status\" is a required property",
  );
  assert_rejected(
    &v,
    "an unknown status",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "signed" }
    "#,
    "is not one of \"draft\"",
  );
  assert_rejected(
    &v,
    "the pack-level not_required status is not part of this vocabulary",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "not_required" }
    "#,
    "is not one of \"draft\"",
  );
  assert_rejected(
    &v,
    "an empty counsel string",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft", counsel = "" }
    "#,
    "is shorter than 1 character",
  );
}

/// Why: an empty `frameworks` table says nothing while looking like an answer ("this control
/// was reviewed against frameworks and none relate"). `minProperties: 1` refuses it, so the
/// property is either absent (no link has been made) or carries at least one link. Control: one
/// link is accepted.
#[test]
fn an_empty_frameworks_table_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: one link",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
    "#,
  );
  assert_rejected(
    &v,
    "an empty frameworks table",
    r#"
      [control.example_key.frameworks]
    "#,
    "has less than 1 property",
  );
}

/// Why: invariant 10 — the catalog is additive by default. Adding `frameworks` to the schema must
/// not invalidate a single catalog written before it existed, and every shipped catalog is one
/// (none populates it). The minimal catalog with NO frameworks must still validate. The second
/// half is the anti-vacuity probe: the same catalog with one field broken is REFUSED, and the
/// mutation is asserted to have changed the text, so "it validated" cannot mean "the validator
/// accepts anything".
#[test]
fn a_catalog_with_no_frameworks_still_validates() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "the base catalog, no frameworks property at all",
    r#""#,
  );

  let broken = BASE_CATALOG.replace("kind = \"obligation\"", "kind = \"bogus\"");
  ::std::assert_ne!(broken, BASE_CATALOG, "the probe mutated nothing, so it proves nothing");
  let errs = common::errors(&v, &common::toml_text_as_json(&broken));
  ::std::assert!(
    !errs.is_empty(),
    "a catalog with kind = \"bogus\" validated — the validator is not discriminating, so every \
     acceptance in this file is suspect"
  );
}

/// The edition ids the design decided, in the order it decided them. The schema's list may grow
/// AT THE END; it may never reorder, rename or drop one of these.
const DECIDED_EDITIONS: [&str; 4] =
  ["soc2_tsc_2017", "iso27001_2022_a1_2024", "hipaa_45cfr164_2013", "hecvat_4_1_6"];

/// Why: the edition list is long-lived audit evidence and every other language binding, the N
/// Lang prop set and any downstream table keyed by position follow its order, so a reorder or a
/// rename is a silent break and a removal strands every receipt that cites the id. Adding an
/// edition is a LAST-POSITION add; this test accepts that (the schema list only has to START WITH
/// the decided prefix) and fails the others. An edit to `DECIDED_EDITIONS` is therefore always a
/// deliberate act in review, never a side effect of editing the schema.
#[test]
fn the_edition_list_only_ever_grows_at_the_end() {
  let schema = common::load_json(SCHEMA);
  let list = schema["$defs"]["framework_edition"]["enum"]
    .as_array()
    .expect("$defs/framework_edition has an enum list");
  let editions: ::std::vec::Vec<&str> =
    list.iter().map(|e| e.as_str().expect("edition ids are strings")).collect();
  ::std::assert!(
    editions.starts_with(&DECIDED_EDITIONS),
    "the edition list {:?} no longer begins with {:?}. Adding an edition is a \
     last-position add; a reorder, rename or removal breaks audit evidence already issued.",
    editions,
    DECIDED_EDITIONS
  );
}

/// Why: the relation set is the mechanism that makes "no control is sufficient" unsayable, so it
/// is pinned EXACTLY rather than by the absence of one word. A future `meets`, `fulfils` or
/// `covers` is just `satisfies` by another name, and the per-word refusal test would not catch it.
/// Widening this set is a normative change (RFC), so it must fail here and be edited on purpose.
#[test]
fn the_relation_set_is_exactly_informs_contributes_necessary() {
  let schema = common::load_json(SCHEMA);
  let list = schema["$defs"]["framework_link"]["properties"]["relation"]["enum"]
    .as_array()
    .expect("$defs/framework_link has a relation enum");
  let relations: ::std::vec::Vec<&str> =
    list.iter().map(|e| e.as_str().expect("relations are strings")).collect();
  ::std::assert_eq!(
    relations,
    ::std::vec!["informs", "contributes", "necessary"],
    "the relation set changed — widening it is a normative change and needs an RFC"
  );
}

/// Flipped to `true` by a maintainer, deliberately, in the same change that populates a real
/// mapping — never as a side effect of anything else. While `false`, no shipped catalog may
/// carry a `frameworks` block.
const FRAMEWORK_LINK_LICENSING_CLEARED: bool = false;

/// Every shipped Control Catalog, repo-relative. Globbed, never listed: invariant 11 is explicit
/// that adding a published file must not require editing a list somewhere else.
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

/// The keys of the controls in a parsed catalog that carry a `frameworks` property.
fn controls_carrying_frameworks(cat: &::toml::Value) -> ::std::vec::Vec<::std::string::String> {
  let mut out: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  if let ::std::option::Option::Some(table) = cat.get("control").and_then(|c| c.as_table()) {
    for (key, def) in table {
      if def.get("frameworks").is_some() {
        out.push(key.clone());
      }
    }
  }
  out
}

/// Why: populating a real mapping is a PUBLIC CLAIM about someone else's certification scope —
/// "this control is related to that framework's criteria" published in an Apache-2.0 standard
/// reads as an assertion about what an auditor will accept. Licensing for the framework
/// criterion identifiers is uncleared, and no counsel has reviewed any mapping, so this build
/// delivers the CONTRACT only. A catalog that gains a `frameworks` block before then ships a
/// claim nobody was authorised to make; this test fails the build and names the file and the
/// control. A maintainer opens the gate on purpose by flipping
/// `FRAMEWORK_LINK_LICENSING_CLEARED`, in the change that populates it. Globbed (invariant 11);
/// the catalog count and the control count are both asserted so a broken glob or a parse that
/// found no controls cannot pass vacuously.
#[test]
fn no_shipped_catalog_populates_framework_links_before_the_licensing_gate_clears() {
  let catalogs = shipped_catalogs();
  ::std::assert!(
    catalogs.len() >= 2,
    "expected the shipped catalogs, found {catalogs:?} — a broken glob would pass vacuously"
  );

  let mut controls_scanned = 0usize;
  let mut populated: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for rel in &catalogs {
    let path = common::root().join(rel);
    let text = ::std::fs::read_to_string(&path)
      .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
    let cat: ::toml::Value =
      ::toml::from_str(&text).unwrap_or_else(|e| ::std::panic!("parse {rel}: {e}"));
    controls_scanned +=
      cat.get("control").and_then(|c| c.as_table()).map(|t| t.len()).unwrap_or(0);
    for key in controls_carrying_frameworks(&cat) {
      populated.push(::std::format!("{rel}: control {key:?}"));
    }
  }
  ::std::assert!(
    controls_scanned >= catalogs.len(),
    "scanned only {controls_scanned} control(s) across {} catalog(s) — every catalog declares at \
     least one, so the scan is not reading what it thinks it is reading",
    catalogs.len()
  );

  if !FRAMEWORK_LINK_LICENSING_CLEARED {
    ::std::assert!(
      populated.is_empty(),
      "{} control(s) carry a `frameworks` block while the licensing gate is closed:\n  {}\n\
       Populating a real mapping is a public claim about someone else's certification scope, and \
       the criterion identifiers it would cite are not cleared for publication. A maintainer opens \
       the gate deliberately, by flipping FRAMEWORK_LINK_LICENSING_CLEARED in \
       tools/tests/framework_links.rs in the same change - not by editing around this check.",
      populated.len(),
      populated.join("\n  ")
    );
  }
}

/// Why: control for the guard above. That test passes when the glob finds NO populated control,
/// which is exactly what a detector that could never detect would also report. This feeds the
/// detector a catalog that DOES populate `frameworks` (in the TOML dotted-table form an author
/// would write) beside one that does not, and requires it to name the right control and only
/// that one — so a green guard means "looked and found none", not "could not have found any".
#[test]
fn the_populated_detector_finds_a_frameworks_block() {
  let populated: ::toml::Value = ::toml::from_str(
    r#"
      [domain]
      name = "example"
      description = "A fictional catalog used only by schema tests."

      [control.linked_key]
      kind = "obligation"
      title = "Linked"

      [control.linked_key.frameworks.soc2_tsc_2017]
      relation = "informs"

      [control.plain_key]
      kind = "obligation"
      title = "Plain"
    "#,
  )
  .expect("fixture is valid TOML");
  ::std::assert_eq!(
    controls_carrying_frameworks(&populated),
    ::std::vec![::std::string::String::from("linked_key")],
    "the detector must name the linked control and not the plain one"
  );

  let unpopulated: ::toml::Value = ::toml::from_str(BASE_CATALOG).expect("base catalog is valid TOML");
  ::std::assert!(
    controls_carrying_frameworks(&unpopulated).is_empty(),
    "the base catalog carries no frameworks block, so the detector must report nothing"
  );
}

/// Why: an `approved` review that names nobody is an approval with no one standing behind it —
/// the same hazard a pack's `legal_review` already refuses by requiring an attorney of record
/// before `approved`. Without this rule, `necessary` (which needs `approved`) could be
/// satisfied by writing one word. Controls: the identical link WITH counsel is accepted, and a
/// `draft` review with no counsel is accepted (the rule binds `approved` only, so unfinished
/// work stays expressible).
#[test]
fn an_approved_review_must_name_its_reviewer() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: approved, counsel named",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_accepted(
    &v,
    "control: draft, no counsel",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft" }
    "#,
  );
  assert_rejected(
    &v,
    "informs, approved, no counsel",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "approved" }
    "#,
    "\"counsel\" is a required property",
  );
  assert_rejected(
    &v,
    "necessary, approved, no counsel",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "necessary"
      review = { status = "approved" }
    "#,
    "\"counsel\" is a required property",
  );
}

/// Why: `docs/framework-links.md` carries a worked example that authors will copy. If the schema
/// tightens and the page does not follow, the documentation teaches a catalog the repository
/// itself rejects — a published example that fails its own contract is the cheapest way to lose
/// an author's trust. This extracts the complete example (the fenced `toml` block containing a
/// `[domain]` table) and validates it. Controls: the extraction found exactly one block, and that
/// block really does carry framework links, so the test cannot pass by validating an empty or
/// unrelated snippet.
#[test]
fn the_worked_example_in_the_docs_validates_against_the_schema() {
  let doc = common::read("docs/framework-links.md");
  let mut blocks: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  let mut cur: ::std::option::Option<::std::string::String> = ::std::option::Option::None;
  for line in doc.lines() {
    match (&mut cur, line.trim_end()) {
      (::std::option::Option::None, "```toml") => cur = ::std::option::Option::Some(::std::string::String::new()),
      (::std::option::Option::Some(b), "```") => {
        blocks.push(::std::mem::take(b));
        cur = ::std::option::Option::None;
      }
      (::std::option::Option::Some(b), l) => {
        b.push_str(l);
        b.push('\n');
      }
      _ => {}
    }
  }
  let complete: ::std::vec::Vec<&::std::string::String> =
    blocks.iter().filter(|b| b.contains("[domain]")).collect();
  ::std::assert_eq!(
    complete.len(),
    1,
    "expected exactly one complete catalog example in docs/framework-links.md, found {}",
    complete.len()
  );
  ::std::assert!(
    complete[0].contains(".frameworks."),
    "the example no longer shows a framework link — it would validate trivially"
  );
  let v = common::validator(SCHEMA);
  let errs = common::errors(&v, &common::toml_text_as_json(complete[0]));
  ::std::assert!(errs.is_empty(), "the documented example is rejected by the schema: {errs:?}");
}

/// Why: `minLength: 1` alone accepts a single space, so `counsel = " "` satisfied the rule that an
/// approval needs a named reviewer, and `criteria = [" "]` read as an identifier. The schema now
/// also requires one non-space character. Controls: a real-looking value is accepted in both
/// positions, so a refusal here is the blank-value rule and not the field.
#[test]
fn a_blank_counsel_or_criterion_is_rejected() {
  let v = common::validator(SCHEMA);
  assert_accepted(
    &v,
    "control: a named reviewer and a placeholder criterion",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1"]
      review = { status = "approved", counsel = "Example Counsel (fictional)" }
    "#,
  );
  assert_rejected(
    &v,
    "an approved review whose counsel is only spaces",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "approved", counsel = "   " }
    "#,
    "does not match",
  );
  assert_rejected(
    &v,
    "a draft review whose counsel is only spaces (blank is refused whatever the status)",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      review = { status = "draft", counsel = " " }
    "#,
    "does not match",
  );
  assert_rejected(
    &v,
    "a criterion that is only spaces",
    r#"
      [control.example_key.frameworks.soc2_tsc_2017]
      relation = "informs"
      criteria = ["EXAMPLE-1", "  "]
    "#,
    "does not match",
  );
}

/// Why: the licensing gate keeps real criterion identifiers out of the repository, and the guard
/// test above reads `catalog/*.catalog.toml` only. The worked example on the documentation page is
/// the one place a real identifier could be pasted past it, and it would still validate, so
/// validation cannot catch it. This pins that every criterion in the page's TOML blocks is an
/// `EXAMPLE-` placeholder and every `counsel` says it is fictional. Controls: the scan must find
/// at least one of each, so an extractor that matched nothing cannot pass.
#[test]
fn the_docs_example_contains_only_placeholder_criteria_and_fictional_counsel() {
  let doc = common::read("docs/framework-links.md");
  let mut in_toml = false;
  let (mut criteria_seen, mut counsel_seen) = (0usize, 0usize);
  for (i, line) in doc.lines().enumerate() {
    match line.trim_end() {
      "```toml" => in_toml = true,
      "```" => in_toml = false,
      l if in_toml && l.trim_start().starts_with("criteria") => {
        for item in l.split('"').skip(1).step_by(2) {
          criteria_seen += 1;
          ::std::assert!(
            item.starts_with("EXAMPLE-"),
            "docs/framework-links.md:{}: criterion {item:?} is not an EXAMPLE- placeholder",
            i + 1
          );
        }
      }
      l if in_toml && l.trim_start().starts_with("counsel") => {
        counsel_seen += 1;
        ::std::assert!(
          l.contains("(fictional)"),
          "docs/framework-links.md:{}: counsel is not marked fictional: {l}",
          i + 1
        );
      }
      _ => {}
    }
  }
  ::std::assert!(criteria_seen >= 1, "found no `criteria` in the page's TOML — the scan read nothing");
  ::std::assert!(counsel_seen >= 1, "found no `counsel` in the page's TOML — the scan read nothing");
}
