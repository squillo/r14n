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

fn repo_root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The emitted baselines bundle, keyed by pack name.
fn snapp_packs() -> ::serde_json::Value {
  let p = repo_root().join("snapp").join("r14n_baselines@1.0.0.json");
  let raw = ::std::fs::read_to_string(&p)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()));
  let doc: ::serde_json::Value =
    ::serde_json::from_str(&raw).expect("baselines bundle is not valid JSON");
  doc["policies"].clone()
}

fn toml_pack(profile: &str) -> ::toml::Value {
  let p = repo_root()
    .join("packs")
    .join(profile)
    .join("recording_consent.r14n.toml");
  let raw = ::std::fs::read_to_string(&p)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", p.display()));
  raw.parse::<::toml::Value>().expect("published pack is not valid TOML")
}

fn sorted_strings(v: ::std::option::Option<&::serde_json::Value>) -> ::std::vec::Vec<::std::string::String> {
  let mut out: ::std::vec::Vec<::std::string::String> = v
    .and_then(|x| x.as_array())
    .map(|a| a.iter().filter_map(|s| s.as_str().map(::std::string::String::from)).collect())
    .unwrap_or_default();
  out.sort();
  out
}

fn sorted_toml_strings(v: ::std::option::Option<&::toml::Value>) -> ::std::vec::Vec<::std::string::String> {
  let mut out: ::std::vec::Vec<::std::string::String> = v
    .and_then(|x| x.as_array())
    .map(|a| a.iter().filter_map(|s| s.as_str().map(::std::string::String::from)).collect())
    .unwrap_or_default();
  out.sort();
  out
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

  // Posture — the dial that decides WHICH table governs.
  ::std::assert_eq!(
    s["strictness"].as_str(),
    t["meta"]["strictness"].as_str(),
    "{pack_key}: strictness disagrees with packs/{profile}/recording_consent.r14n.toml"
  );

  // The floor. Shapes differ by serialization — a bare list in the Snapp, a `controls`
  // table in TOML — so the CONTENT is compared, not the encoding.
  ::std::assert_eq!(
    sorted_strings(s.get("legally_required")),
    sorted_toml_strings(t.get("legally_required").and_then(|v| v.get("controls"))),
    "{pack_key}: the legally_required floor disagrees with packs/{profile}/"
  );

  // The per-subject table, compared WHOLE: a subject present on one side only is a
  // failure naming that subject, not a silent pass on the subjects that do match.
  let s_subjects = s["subjects"].as_object().cloned().unwrap_or_default();
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
      sorted_strings(s_subjects[name].get("controls")),
      sorted_toml_strings(t_subjects[name].get("controls")),
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

/// Why: parity between two representations is worth nothing if the floor itself is empty
/// — both sides would agree on a pack that fails closed and governs nothing. This pins
/// the property the council audit filed as blocker-class, on the typed side specifically,
/// since the TOML side is already covered by the shipped-packs suite.
#[test]
fn the_typed_minimal_floor_is_not_empty() {
  let packs = snapp_packs();
  let floor = sorted_strings(packs["RECORDING_CONSENT_MINIMAL"].get("legally_required"));
  ::std::assert!(
    !floor.is_empty(),
    "a minimal pack with an empty floor does not require nothing — it fails closed to the \
     caller's entire universe and discards the pack whole"
  );
}
