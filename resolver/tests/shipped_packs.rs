//! The packs this repository actually SHIPS, resolved through the real adapter.
//!
//! Why this file exists: until 2026-10-15 nothing resolved `/packs`. CI validated them —
//! parse plus a catalog cross-check — and the conformance suite exercised the resolver
//! against inline fixtures in tempdirs. Both are necessary and neither asks the question a
//! consuming operator asks: *when I select this profile, what do I actually get?* A pack
//! can parse, satisfy its catalog, and still resolve to something nobody intended.
//!
//! That gap is the same one that let ten N Lang packs sit on disk compiled by nothing, and
//! the same one that left the data-minimization guard tested only on governed packs. The
//! pattern is a fixture standing in for the artifact, so this file points the adapter at
//! the committed `/packs` tree and asserts on what it returns.
//!
//! Style note: FQ paths, no module-top `use` — the house style of this crate.
//!
//! Revision History
//! - 2026-10-15: authored with the `credential_use` baselines.

/// `/packs` as the adapter root: `pack_path` joins `<root>/<profile>/<domain>.r14n.toml`,
/// which is exactly the shipped layout, so no fixture is copied anywhere.
fn packs_root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("packs")
}

fn query(
  profile: &str,
  domain: &str,
  subject: &str,
  universe: &[&str],
) -> ::r14n::RegulatoryQuery {
  ::r14n::RegulatoryQuery {
    domain: ::std::string::String::from(domain),
    profile: ::r14n::RegulatoryProfile(::std::string::String::from(profile)),
    jurisdiction: ::std::string::String::from("unspecified"),
    subject: ::std::string::String::from(subject),
    universe: universe
      .iter()
      .map(|k| ::r14n::ControlKey(::std::string::String::from(*k)))
      .collect(),
    as_of: ::std::option::Option::None,
    attribution_source: ::std::option::Option::None,
    per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
    gpc_signal: false,
    ieee7012_no_recording: false,
  }
}

fn resolve(q: &::r14n::RegulatoryQuery) -> ::r14n::ControlDecision {
  let adapter = ::r14n::TomlRegulatoryPolicyAdapter::new(packs_root(), ::std::option::Option::None);
  ::r14n::RegulatoryPolicyPort::required_controls(&adapter, q)
}

fn keys(d: &::r14n::ControlDecision) -> ::std::collections::BTreeSet<::std::string::String> {
  d.required.iter().map(|k| k.0.clone()).collect()
}

/// Why: the shipped `minimal/credential_use` floor is deliberately ONE control, and the
/// whole point of that choice is lost if the pack does not GOVERN — a `minimal` pack whose
/// floor is missing or empty fails closed to the caller's entire universe instead, which
/// would silently discard the pack's prohibitions too. This pins that it governs (
/// `fell_back = false`) AND that the floor is the single control intended, so a later edit
/// that empties the floor fails here rather than in a consuming gate.
#[test]
fn shipped_minimal_credential_use_governs_with_its_one_control_floor() {
  let d = resolve(&query(
    "minimal",
    "credential_use",
    "private_key",
    &["secret_by_reference_only", "secret_ttl_bounded", "secret_assurance_floor"],
  ));
  ::std::assert!(
    !d.provenance.fell_back,
    "the shipped minimal pack must GOVERN, not fall back — source was {:?}",
    d.provenance.source
  );
  ::std::assert_eq!(
    keys(&d),
    ::std::collections::BTreeSet::from([::std::string::String::from("secret_by_reference_only")]),
    "minimal resolves its floor ONLY; the subject table is documentary under this posture"
  );
}

/// Why: a prohibition is the only part of a pack that changes a resolved set under EVERY
/// posture, and the shipped `minimal` pack carries three. If a caller declares one of them
/// in its universe anyway — the mistake the catalog header warns against — the subtraction
/// must still remove it. This asserts the subtraction, not merely that the key is absent
/// for some other reason: the universe below contains it, so absence can only come from
/// `[prohibited]`.
#[test]
fn shipped_prohibitions_are_subtracted_even_when_the_caller_declares_them() {
  let d = resolve(&query(
    "minimal",
    "credential_use",
    "private_key",
    &["secret_by_reference_only", "secret_value_in_agent_context"],
  ));
  ::std::assert!(!d.provenance.fell_back, "must govern for the guard to run at all");
  ::std::assert!(
    !keys(&d).contains("secret_value_in_agent_context"),
    "a prohibited control the CALLER declared must still be subtracted; got {:?}",
    keys(&d)
  );
  ::std::assert!(
    keys(&d).contains("secret_by_reference_only"),
    "subtracting a prohibition must not disturb the floor"
  );
}

/// Why: `aggressive` requires the caller's ENTIRE universe rather than the pack's tables,
/// so the only thing the shipped aggressive pack changes is its `[prohibited]` list. A
/// reader who assumes the long `[legally_required]` list is doing the work would not notice
/// if the prohibitions stopped applying. This pins both halves: everything the caller
/// declared is required, EXCEPT what the pack forbids.
#[test]
fn shipped_aggressive_credential_use_is_universe_minus_prohibitions() {
  let d = resolve(&query(
    "aggressive",
    "credential_use",
    "crypto_wallet_seed",
    &[
      "secret_by_reference_only",
      "secret_ttl_bounded",
      "secret_remote_agent_redeem",
      "secret_in_durable_shared_store",
    ],
  ));
  ::std::assert!(!d.provenance.fell_back, "the shipped aggressive pack must govern");
  ::std::assert_eq!(
    keys(&d),
    ::std::collections::BTreeSet::from([
      ::std::string::String::from("secret_by_reference_only"),
      ::std::string::String::from("secret_ttl_bounded"),
    ]),
    "aggressive = the caller's universe MINUS the pack's prohibitions"
  );
}

/// Why: the `recording_consent` baselines predate this file and are the packs every
/// published example and conformance fixture is modelled on, so they deserve the same
/// end-to-end assertion rather than only the inline-fixture coverage they had. `minimal`
/// resolving to its floor is also the regression the council audit filed as blocker-class:
/// an empty or missing floor must never resolve to an empty required set.
#[test]
fn shipped_recording_consent_baselines_resolve_as_published() {
  let min = resolve(&query(
    "minimal",
    "recording_consent",
    "telepresence",
    &["attestation", "signal_notice", "announcement"],
  ));
  ::std::assert!(!min.provenance.fell_back, "shipped minimal recording_consent must govern");
  ::std::assert_eq!(
    keys(&min),
    ::std::collections::BTreeSet::from([::std::string::String::from("attestation")]),
    "the published minimal floor is `attestation`"
  );
  ::std::assert!(!min.required.is_empty(), "a floor must never resolve to nothing (audit blocker)");

  let agg = resolve(&query(
    "aggressive",
    "recording_consent",
    "telepresence",
    &["attestation", "signal_notice"],
  ));
  ::std::assert!(!agg.provenance.fell_back, "shipped aggressive recording_consent must govern");
  ::std::assert_eq!(
    keys(&agg),
    ::std::collections::BTreeSet::from([
      ::std::string::String::from("attestation"),
      ::std::string::String::from("signal_notice"),
    ]),
    "aggressive requires the caller's full declared universe"
  );
}

/// Why: a domain with no pack under a profile must fail closed to the caller's entire
/// universe, loudly — never an empty set and never an error from the port. This is
/// invariant 1, asserted against the REAL packs tree rather than a tempdir, because the
/// failure mode it guards is an operator selecting a profile/domain pair this repository
/// does not ship and silently receiving no obligations at all.
#[test]
fn an_unshipped_domain_fails_closed_against_the_real_packs_tree() {
  let universe = ["secret_by_reference_only", "attestation"];
  let d = resolve(&query("minimal", "no_such_domain", "any", &universe));
  ::std::assert!(d.provenance.fell_back, "a missing pack must announce its fallback");
  ::std::assert_eq!(
    keys(&d),
    universe.iter().map(|s| ::std::string::String::from(*s)).collect(),
    "fail-closed means the caller's ENTIRE universe, not a subset and not nothing"
  );
  ::std::assert!(
    d.provenance.source.contains("<missing:"),
    "the provenance must name WHY it fell back; got {:?}",
    d.provenance.source
  );
}
