//! The typed Snapp source and the published TOML say the same thing, or this fails.
//!
//! `recording_consent` is the only domain this repository ships BOTH ways: as normative
//! TOML under `/packs`, and as typed N Lang in the baselines Snapp. Everywhere else the
//! two representations cover disjoint domains, so they cannot disagree — and cannot be
//! checked either. Here they can, which makes this pair the convergence proof for the
//! whole migration.
//!
//! Why a test and not a one-off comparison: a script that passed once is exactly the
//! shape that let ten Snapp packs sit uncompiled and the shipped packs go unresolved. The
//! comparison has to run on every push or it is a claim, not a check.
//!
//! It fails in BOTH directions by construction — the subject maps are compared whole, so
//! a subject added to either side alone turns this red and names it. That matters more
//! than it sounds: the failure mode is not two files drifting loudly, it is someone
//! editing the TOML a reviewer reads while the typed source a build consumes keeps the
//! old answer.
//!
//! What this does NOT prove: that the committed bundle is what the current Snapp sources
//! would emit. That needs the compiler, which is proprietary and cannot run on public
//! CI. The drift gate in `.github/scripts/` covers the rest of that seam; here the bundle
//! is taken as given and compared against the TOML.
//!
//! Style note: FQ paths, no module-top `use` — the house style of this crate.
//!
//! Revision History
//! - 2026-10-15: authored when recording_consent landed in the baselines Snapp.

mod common;

/// The emitted baselines bundle, keyed by pack name.
fn snapp_packs() -> ::serde_json::Value {
  let p = common::repo_root().join("snapp").join("r14n_baselines@1.0.0.json");
  let raw = ::std::fs::read_to_string(&p)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()));
  let doc: ::serde_json::Value =
    ::serde_json::from_str(&raw).expect("baselines bundle is not valid JSON");
  doc["policies"].clone()
}

fn toml_pack(profile: &str) -> ::toml::Value {
  let p = common::repo_root()
    .join("packs")
    .join(profile)
    .join("recording_consent.r14n.toml");
  let raw = ::std::fs::read_to_string(&p)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()));
  raw.parse::<::toml::Value>().expect("published pack is not valid TOML")
}

/// A control list in AUTHORED ORDER. Deliberately does NOT sort.
///
/// It used to. Sorting made this a set comparison, which could not see two copies of one
/// baseline listing the same controls in different sequences — and as of the N Lang Book's
/// `--format toml` definition (authoring Book, Binary Format §Export: `toml` emits the same
/// document as `json`, "rendered as TOML 1.0 in the `json` order"), that sequence is the
/// published artifact's byte order. Once a Toml exporter exists, an order divergence here is
/// a byte divergence in a shipped pack. Measured 2026-10-26: both pairs already agree in
/// sequence, so pinning order costs nothing and buys the guarantee.
fn ordered_strings(v: ::std::option::Option<&::serde_json::Value>) -> ::std::vec::Vec<::std::string::String> {
  v.and_then(|x| x.as_array())
    .map(|a| a.iter().filter_map(|s| s.as_str().map(::std::string::String::from)).collect())
    .unwrap_or_default()
}

/// N Lang emits a list either as a JSON array or as a `~N`-indexed object, depending on the
/// authoring form. Both mean the same ordered list, so comparisons normalize through here
/// rather than asserting on an encoding neither side chose deliberately.
fn denumber_values(v: &::serde_json::Value) -> ::std::vec::Vec<::serde_json::Value> {
  if let ::std::option::Option::Some(arr) = v.as_array() {
    return arr.clone();
  }
  if let ::std::option::Option::Some(map) = v.as_object() {
    let mut keyed: ::std::vec::Vec<(usize, ::serde_json::Value)> = map
      .iter()
      .filter_map(|(k, val)| k.strip_prefix('~')?.parse::<usize>().ok().map(|i| (i, val.clone())))
      .collect();
    keyed.sort_by_key(|(i, _)| *i);
    return keyed.into_iter().map(|(_, val)| val).collect();
  }
  ::std::vec::Vec::new()
}

/// `denumber_values` narrowed to strings, in authored order.
///
/// The ENCODING normalization in `denumber_values` stays — array vs `~N`-indexed object is a
/// representation neither side chose, and it restores order by index rather than discarding
/// it. What is gone is the alphabetical sort layered on top, which is an ORDER normalization
/// and a different claim entirely. See `ordered_strings`.
fn denumber(v: &::serde_json::Value) -> ::std::vec::Vec<::std::string::String> {
  denumber_values(v)
    .iter()
    .filter_map(|x| x.as_str().map(::std::string::String::from))
    .collect()
}

/// The published TOML side of `ordered_strings`. TOML arrays are ordered, so this reads the
/// sequence as written.
fn ordered_toml_strings(v: ::std::option::Option<&::toml::Value>) -> ::std::vec::Vec<::std::string::String> {
  v.and_then(|x| x.as_array())
    .map(|a| a.iter().filter_map(|s| s.as_str().map(::std::string::String::from)).collect())
    .unwrap_or_default()
}

/// Compare one (snapp pack, published profile) pair on everything that changes a
/// resolved set: posture, floor, and the per-subject table.
fn assert_parity(pack_key: &str, profile: &str) {
  let packs = snapp_packs();
  let s = &packs[pack_key];
  ::std::assert!(
    !s.is_null(),
    "the baselines bundle has no pack {pack_key:?} — it was removed from the Snapp, or the \
     bundle was not re-emitted after it was added"
  );
  let t = toml_pack(profile);

  // Posture — the dial that decides WHICH table governs. Both sides nest it under `meta`
  // since 2026-10-15: the typed source was flattened (`strictness`, `legal_review_status`)
  // and the published TOML was not, so every comparison here needed a projection rule in
  // its head. Nesting the typed side removed the rule rather than encoding it.
  ::std::assert_eq!(
    s["meta"]["strictness"].as_str(),
    t["meta"]["strictness"].as_str(),
    "{pack_key}: strictness disagrees with packs/{profile}/recording_consent.r14n.toml"
  );

  // Review provenance — comparable for the first time now that the typed side nests it the
  // same way. A pack whose published form says counsel approved it while the typed source
  // says `draft` is the disagreement with the most consequence, because a consuming gate
  // decides whether the pack governs AT ALL from this field.
  ::std::assert_eq!(
    s["meta"]["legal_review"]["status"].as_str(),
    t["meta"]["legal_review"]["status"].as_str(),
    "{pack_key}: legal_review.status disagrees with packs/{profile}/ — one side claims a \
     different level of counsel review than the other"
  );

  // The floor. Both sides now spell it the SAME WAY — `legally_required { controls = [...] }`
  // in the typed source and `[legally_required] controls = [...]` in TOML — since the
  // framework type was renamed on 2026-10-16. This comparison used to carry a note that the
  // shapes "differ by serialization, so the CONTENT is compared, not the encoding". That note
  // is gone because the difference is gone: the only normalization left is sort order, which
  // neither side chose.
  ::std::assert_eq!(
    ordered_strings(s.get("legally_required").and_then(|v| v.get("controls"))),
    ordered_toml_strings(t.get("legally_required").and_then(|v| v.get("controls"))),
    "{pack_key}: the legally_required floor disagrees with packs/{profile}/"
  );

  // The per-subject table, compared WHOLE: a subject present on one side only is a
  // failure naming that subject, not a silent pass on the subjects that do match.
  // Both sides read `subject` — singular — for the same reason as the floor above.
  let s_subjects = s["subject"].as_object().cloned().unwrap_or_default();
  let t_subjects = t
    .get("subject")
    .and_then(|v| v.as_table())
    .cloned()
    .unwrap_or_default();

  let mut s_names: ::std::vec::Vec<&::std::string::String> = s_subjects.keys().collect();
  let mut t_names: ::std::vec::Vec<&::std::string::String> = t_subjects.keys().collect();
  s_names.sort();
  t_names.sort();
  ::std::assert_eq!(
    s_names, t_names,
    "{pack_key}: the SET of subjects disagrees with packs/{profile}/ — a subject was added \
     or removed on one side only"
  );

  for name in s_names {
    ::std::assert_eq!(
      ordered_strings(s_subjects[name].get("controls")),
      ordered_toml_strings(t_subjects[name].get("controls")),
      "{pack_key}: subject `{name}` has different controls than packs/{profile}/"
    );
  }
}

/// Why: the aggressive baseline is the fail-closed posture every fallback degrades to, so
/// a disagreement between its typed source and its published form is a disagreement about
/// what the safest answer IS. Compared on posture, floor, and all four subject rails.
#[test]
fn aggressive_recording_consent_agrees_across_both_representations() {
  assert_parity("RECORDING_CONSENT_AGGRESSIVE", "aggressive");
}

/// Why: the minimal baseline is the one an operator selects to relax, so its floor is the
/// last line of defence and the published TOML is what counsel reviews. If the typed
/// source carried a different floor, a build would enforce something counsel never read.
#[test]
fn minimal_recording_consent_agrees_across_both_representations() {
  assert_parity("RECORDING_CONSENT_MINIMAL", "minimal");
}

/// Why: `health_aggressive` is authored TWICE inside this repository — as a core-book
/// instance in the framework Snapp's smoke tests, and in the keyword form in baselines.
/// They had already drifted when this test was written: the baselines copy was missing the
/// `background` subject entirely. Under `aggressive` that changed no resolution, because
/// the posture requires the caller's whole universe regardless, which is exactly why it
/// survived unnoticed — a second source of truth whose disagreement is invisible at the
/// posture it is normally read at.
///
/// This compares the two emitted forms on the content that would differ under any other
/// posture. The serializations differ by construction (a row list with `~N` indices versus
/// a keyed map), so the shapes are normalized and the CONTROLS are compared.
#[test]
fn the_two_in_repo_copies_of_health_aggressive_agree() {
  let fw_raw = ::std::fs::read_to_string(common::repo_root().join("snapp").join("r14n@0.1.0-alpha.1.json"))
    .expect("framework bundle");
  let fw: ::serde_json::Value = ::serde_json::from_str(&fw_raw).expect("framework bundle JSON");
  let a = &fw["packs"]["health_aggressive"];
  let packs = snapp_packs();
  let b = &packs["HEALTH_AGGRESSIVE"];
  ::std::assert!(!a.is_null() && !b.is_null(), "both copies must be present to compare");

  ::std::assert_eq!(
    a["meta"]["strictness"].as_str(),
    b["meta"]["strictness"].as_str(),
    "strictness disagrees"
  );
  ::std::assert_eq!(
    denumber(&a["legally_required"]),
    denumber(&b["legally_required"]),
    "the floor disagrees between the smoke-test copy and the baselines copy"
  );

  // Instance form: `subjects` is a list of `{key, controls}` rows. Keyword form: a map.
  let mut a_subjects: ::std::vec::Vec<(::std::string::String, ::std::vec::Vec<::std::string::String>)> =
    ::std::vec::Vec::new();
  for row in denumber_values(&a["subject"]) {
    let key = row["key"].as_str().unwrap_or_default();
    a_subjects.push((::std::string::String::from(key), denumber(&row["controls"])));
  }
  a_subjects.sort();

  let mut b_subjects: ::std::vec::Vec<(::std::string::String, ::std::vec::Vec<::std::string::String>)> =
    b["subject"]
      .as_object()
      .map(|m| {
        m.iter()
          .map(|(k, v)| (k.clone(), denumber(&v["controls"])))
          .collect()
      })
      .unwrap_or_default();
  b_subjects.sort();

  ::std::assert_eq!(
    a_subjects, b_subjects,
    "the two in-repo copies of health_aggressive disagree about their subject table — one \
     was edited without the other"
  );
}

/// Why: parity between two representations is worth nothing if the floor itself is empty
/// — both sides would agree on a pack that fails closed and governs nothing. This pins
/// the property the council audit filed as blocker-class, on the typed side specifically,
/// since the TOML side is already covered by the shipped-packs suite.
#[test]
fn the_typed_minimal_floor_is_not_empty() {
  let packs = snapp_packs();
  let floor = ordered_strings(packs["RECORDING_CONSENT_MINIMAL"].get("legally_required").and_then(|v| v.get("controls")));
  ::std::assert!(
    !floor.is_empty(),
    "a minimal pack with an empty floor does not require nothing — it fails closed to the \
     caller's entire universe and discards the pack whole"
  );
}
