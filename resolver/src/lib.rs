//! # r14n — the RLPS reference resolver (`r14n : controls :: i18n : strings`)
//!
//! Compliance rules should be no more hardcoded than user-facing strings are.
//! Where i18n maps `locales/<lang>/<feature>.toml` → localized strings, RLPS maps
//! `packs/<profile>/<domain>.r14n.toml` → the **set of compliance CONTROLS required**
//! for a given `(profile, jurisdiction, subject)`. A control is one stable-key
//! required compliance action — a domain declares its own control-key universe
//! exactly as an i18n feature declares its own message keys.
//!
//! This crate is the **reference resolver** for the Regulatory Localization Pack
//! Specification (RLPS). It was extracted verbatim from the live Squillo OS substrate
//! (`the Squillo OS policy engine`) and is fully generic — `serde` + `toml` + `std` only.
//!
//! ## The posture selector ([`Strictness`] — spec calls this the *posture*)
//!
//! A single dial modulates WHICH table governs the resolved control set. NOTE (RLPS
//! spec §Posture): this is a *fail-closed posture*, **NOT** a claim of legal ordering —
//! **"aggressive" ≠ "compliant"**. A control required in one jurisdiction may be
//! *prohibited* in another; posture selects among the pack's tables, it does not rank law.
//! - [`Strictness::Aggressive`] — require the caller's FULL declared universe (over-
//!   restrict). The correct dev/staging default + the mandatory fallback when
//!   jurisdiction is unknown.
//! - [`Strictness::AsConfigured`] — honor the pack's per-`subject` table (the
//!   counsel-reviewed production posture).
//! - [`Strictness::Minimal`] — require only the pack's `legally_required` floor.
//!
//! ## Fail-closed + provenance
//!
//! The port is INFALLIBLE: a missing/malformed pack degrades to `Aggressive` over the
//! caller-declared universe (never fewer controls), and the [`ControlDecision`] carries
//! [`PolicyProvenance`] (profile, pack path, legal-review status, whether it `fell_back`)
//! so every decision is auditable — the consumer surfaces it in the decision receipt.
//! Per RLPS §Trust-root: an unattested/self-attested pack SHOULD render the decision
//! advisory-only (carry the fall-back / unverified-provenance flag into the receipt).
//!
//! Style note: FQ paths (no module-top `use`) are inherited from the Squillo source;
//! they are not required by RLPS and may be relaxed in future resolver revisions.
//!
//! Revision History
//! - 2026-07-06: extracted as the RLPS reference resolver (r14n) from Squillo OS
//!   `the Squillo OS policy engine` (an internal design memo §PS.R / an internal design memo).

// ── Value types ──────────────────────────────────────────────────────────────

/// A named regulatory profile — the "which policy pack" selector (e.g.
/// `"aggressive"`, `"us_all_party"`, `"eu_gdpr"`, `"minimal"`). Mirrors an i18n
/// language tag.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::hash::Hash,
)]
pub struct RegulatoryProfile(pub ::std::string::String);

/// One compliance control ("bullet point") — a domain-defined key.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::cmp::PartialOrd,
  ::std::cmp::Ord,
  ::std::hash::Hash,
)]
pub struct ControlKey(pub ::std::string::String);

impl ControlKey {
  /// Borrow the underlying key.
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// The global strictness dial (see module doc). `Other(String)` LAST per §23.5.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::default::Default,
)]
pub enum Strictness {
  /// Require every control in the caller's universe (fail-closed default).
  #[default]
  Aggressive,
  /// Honor the pack's per-subject control table.
  AsConfigured,
  /// Require only the pack's `legally_required` controls.
  Minimal,
  /// Forward-additive escape hatch (§23.5).
  Other(::std::string::String),
}

impl Strictness {
  /// Parse the pack's declared strictness string; unknown ⇒ `Aggressive`
  /// (fail-closed).
  pub fn parse(s: &str) -> Self {
    match s {
      "aggressive" => Self::Aggressive,
      "as_configured" => Self::AsConfigured,
      "minimal" => Self::Minimal,
      other => Self::Other(::std::string::String::from(other)),
    }
  }
}

/// A query into the regulatory policy: the caller declares the FULL control
/// universe it could enforce; the port returns the required subset.
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct RegulatoryQuery {
  /// The compliance domain (selects the `<domain>.toml` pack file), e.g.
  /// `"recording_consent"`.
  pub domain: ::std::string::String,
  /// The active profile (selects the `<profile>/` pack directory).
  pub profile: RegulatoryProfile,
  /// The jurisdiction posture key (e.g. `"all_party"` / `"one_party"` /
  /// `"unknown"`), used by packs that vary controls by jurisdiction.
  pub jurisdiction: ::std::string::String,
  /// The subject within the domain (e.g. a recording rail: `"telepresence"`).
  pub subject: ::std::string::String,
  /// EVERY control the caller could enforce. The Aggressive/fallback ceiling.
  pub universe: ::std::collections::BTreeSet<ControlKey>,
}

/// Provenance of a policy decision — surfaced in consent receipts (§21.3
/// transparency). Records exactly which pack/profile/legal-review bound it.
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
pub struct PolicyProvenance {
  /// The profile that was resolved.
  pub profile: RegulatoryProfile,
  /// The strictness actually applied.
  pub strictness: Strictness,
  /// The pack source (path or `"<default>"` / `"<fallback>"`).
  pub source: ::std::string::String,
  /// The pack's declared legal-review status (`None` if unreviewed / fallback).
  pub legal_review_status: ::std::option::Option<::std::string::String>,
  /// True when the port degraded to the aggressive fallback (pack missing /
  /// malformed) — a loud signal that no reviewed pack governed this decision.
  pub fell_back: bool,
}

/// The resolved decision: the set of controls required for the query, plus
/// provenance.
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
pub struct ControlDecision {
  /// The controls the caller MUST satisfy (a subset of the query universe).
  pub required: ::std::collections::BTreeSet<ControlKey>,
  /// How this decision was reached.
  pub provenance: PolicyProvenance,
}

impl ControlDecision {
  /// Is `control` required by this decision?
  pub fn requires(&self, control: &ControlKey) -> bool {
    self.required.contains(control)
  }
}

// ── Sealed port ──────────────────────────────────────────────────────────────

/// Sealed-pair marker (§11.14a). External crates CANNOT implement the port.
pub mod __private_seal {
  /// Seal trait — private supertrait bound.
  pub trait RegulatoryPolicyPortSeal {}
}

/// The regulatory policy lookup (§3.1 port-gateway). INFALLIBLE: always returns a
/// [`ControlDecision`], degrading fail-closed to Aggressive-over-universe with
/// `fell_back = true` provenance if no reviewed pack governs the query.
pub trait RegulatoryPolicyPort:
  __private_seal::RegulatoryPolicyPortSeal + ::std::marker::Send + ::std::marker::Sync + 'static
{
  /// Resolve the required controls for `query`.
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision;
}

/// Composition alias (§11.21).
pub type DynRegulatoryPolicyPort = ::std::sync::Arc<dyn RegulatoryPolicyPort>;

// ── Aggressive default adapter (fail-closed baseline) ────────────────────────

/// Requires EVERY control in the query universe, always. The safe default when
/// no packs are configured (identical to the pre-config hardcoded behavior).
pub struct AggressiveDefaultPolicyAdapter;

impl __private_seal::RegulatoryPolicyPortSeal for AggressiveDefaultPolicyAdapter {}

impl RegulatoryPolicyPort for AggressiveDefaultPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    ControlDecision {
      required: query.universe.clone(),
      provenance: PolicyProvenance {
        profile: query.profile.clone(),
        strictness: Strictness::Aggressive,
        source: ::std::string::String::from("<aggressive-default>"),
        legal_review_status: ::std::option::Option::None,
        fell_back: false,
      },
    }
  }
}

// ── TOML pack adapter ────────────────────────────────────────────────────────

/// Deserialized policy pack (`policies/<profile>/<domain>.toml`).
#[derive(::serde::Deserialize)]
struct PolicyPackToml {
  meta: PackMeta,
  #[serde(default)]
  subject: ::std::collections::BTreeMap<::std::string::String, ControlList>,
  #[serde(default)]
  legally_required: ::std::option::Option<ControlList>,
}

#[derive(::serde::Deserialize)]
struct PackMeta {
  #[serde(default)]
  strictness: ::std::option::Option<::std::string::String>,
  #[serde(default)]
  legal_review: ::std::option::Option<LegalReview>,
}

#[derive(::serde::Deserialize)]
struct LegalReview {
  #[serde(default)]
  status: ::std::option::Option<::std::string::String>,
}

#[derive(::serde::Deserialize)]
struct ControlList {
  #[serde(default)]
  controls: ::std::vec::Vec<::std::string::String>,
}

/// Loads policy packs from a root dir mirroring `locales/`:
/// `<root>/<profile>/<domain>.toml`. A global `strictness_override` (the DIAL)
/// wins over each pack's declared strictness when set.
pub struct TomlRegulatoryPolicyAdapter {
  root: ::std::path::PathBuf,
  strictness_override: ::std::option::Option<Strictness>,
}

impl TomlRegulatoryPolicyAdapter {
  /// Construct with a pack root and an optional global strictness override.
  /// `strictness_override = Some(Aggressive)` forces maximum strictness system-wide
  /// regardless of packs (the "turn it all the way up" dial).
  pub fn new(
    root: ::std::path::PathBuf,
    strictness_override: ::std::option::Option<Strictness>,
  ) -> Self {
    Self { root, strictness_override }
  }

  fn pack_path(&self, query: &RegulatoryQuery) -> ::std::path::PathBuf {
    self
      .root
      .join(&query.profile.0)
      .join(::std::format!("{}.toml", query.domain))
  }

  /// Aggressive fallback decision (fail-closed) with loud provenance.
  fn fallback(query: &RegulatoryQuery, source: ::std::string::String) -> ControlDecision {
    ControlDecision {
      required: query.universe.clone(),
      provenance: PolicyProvenance {
        profile: query.profile.clone(),
        strictness: Strictness::Aggressive,
        source,
        legal_review_status: ::std::option::Option::None,
        fell_back: true,
      },
    }
  }
}

impl __private_seal::RegulatoryPolicyPortSeal for TomlRegulatoryPolicyAdapter {}

impl RegulatoryPolicyPort for TomlRegulatoryPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    let path = self.pack_path(query);
    let raw = match ::std::fs::read_to_string(&path) {
      ::std::result::Result::Ok(s) => s,
      ::std::result::Result::Err(_) => {
        return Self::fallback(query, ::std::format!("<missing:{}>", path.display()));
      }
    };
    let pack: PolicyPackToml = match ::toml::from_str(&raw) {
      ::std::result::Result::Ok(p) => p,
      ::std::result::Result::Err(_) => {
        return Self::fallback(query, ::std::format!("<malformed:{}>", path.display()));
      }
    };

    // The global dial wins; else the pack's declared strictness; else Aggressive.
    let strictness = self.strictness_override.clone().unwrap_or_else(|| {
      pack
        .meta
        .strictness
        .as_deref()
        .map(Strictness::parse)
        .unwrap_or(Strictness::Aggressive)
    });

    // Intersect every resolved set with the caller's universe — a pack can never
    // demand a control the caller does not know how to enforce, and can never
    // silently drop below what the caller declared under Aggressive.
    let required: ::std::collections::BTreeSet<ControlKey> = match &strictness {
      Strictness::Aggressive => query.universe.clone(),
      Strictness::AsConfigured => pack
        .subject
        .get(&query.subject)
        .map(|c| {
          c.controls
            .iter()
            .map(|s| ControlKey(s.clone()))
            .filter(|k| query.universe.contains(k))
            .collect()
        })
        // A subject absent from an as-configured pack is fail-closed: require all.
        .unwrap_or_else(|| query.universe.clone()),
      Strictness::Minimal => pack
        .legally_required
        .as_ref()
        .map(|c| {
          c.controls
            .iter()
            .map(|s| ControlKey(s.clone()))
            .filter(|k| query.universe.contains(k))
            .collect()
        })
        .unwrap_or_default(),
      Strictness::Other(_) => query.universe.clone(),
    };

    let legal_review_status = pack.meta.legal_review.and_then(|r| r.status);
    ControlDecision {
      required,
      provenance: PolicyProvenance {
        profile: query.profile.clone(),
        strictness,
        source: path.display().to_string(),
        legal_review_status,
        fell_back: false,
      },
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  fn universe() -> ::std::collections::BTreeSet<super::ControlKey> {
    ["user_attestation", "all_party_consent", "signal_notice", "aph_mandate"]
      .into_iter()
      .map(|s| super::ControlKey(::std::string::String::from(s)))
      .collect()
  }

  fn query(
    profile: &str,
    subject: &str,
  ) -> super::RegulatoryQuery {
    super::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: super::RegulatoryProfile(::std::string::String::from(profile)),
      jurisdiction: ::std::string::String::from("all_party"),
      subject: ::std::string::String::from(subject),
      universe: universe(),
    }
  }

  fn ck(s: &str) -> super::ControlKey {
    super::ControlKey(::std::string::String::from(s))
  }

  #[test]
  fn aggressive_default_requires_entire_universe() {
    let p = super::AggressiveDefaultPolicyAdapter;
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("anything", "telepresence"));
    ::std::assert_eq!(d.required, universe());
    ::std::assert!(!d.provenance.fell_back);
  }

  #[test]
  fn missing_pack_falls_back_aggressive_loudly() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("nonexistent", "telepresence"));
    ::std::assert_eq!(d.required, universe(), "missing pack ⇒ require everything");
    ::std::assert!(d.provenance.fell_back, "fallback must be loud in provenance");
  }

  #[test]
  fn as_configured_honors_subject_table() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("us_all_party");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.toml"),
      "[meta]\nstrictness = \"as_configured\"\n[meta.legal_review]\nstatus = \"draft\"\n\
       [subject.telepresence]\ncontrols = [\"signal_notice\", \"aph_mandate\"]\n\
       [legally_required]\ncontrols = [\"all_party_consent\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("us_all_party", "telepresence"));
    let expect: ::std::collections::BTreeSet<super::ControlKey> =
      [ck("signal_notice"), ck("aph_mandate")].into_iter().collect();
    ::std::assert_eq!(d.required, expect);
    ::std::assert_eq!(d.provenance.legal_review_status.as_deref(), ::std::option::Option::Some("draft"));
    ::std::assert!(!d.provenance.fell_back);
  }

  #[test]
  fn global_override_forces_aggressive_over_a_relaxed_pack() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("us_all_party");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.toml"),
      "[meta]\nstrictness = \"as_configured\"\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n",
    )
    .expect("write pack");
    // The dial forced to Aggressive overrides the pack's as_configured relaxation.
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::Some(super::Strictness::Aggressive),
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("us_all_party", "telepresence"));
    ::std::assert_eq!(d.required, universe(), "override dial ⇒ require everything");
  }

  #[test]
  fn minimal_requires_only_legally_required() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("us_one_party");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.toml"),
      "[meta]\nstrictness = \"minimal\"\n[subject.local_piggyback]\ncontrols = [\"user_attestation\", \"all_party_consent\"]\n[legally_required]\ncontrols = [\"user_attestation\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("us_one_party", "local_piggyback"));
    let expect: ::std::collections::BTreeSet<super::ControlKey> =
      [ck("user_attestation")].into_iter().collect();
    ::std::assert_eq!(d.required, expect);
  }

  #[test]
  fn unknown_subject_in_as_configured_is_fail_closed() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("p");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.toml"),
      "[meta]\nstrictness = \"as_configured\"\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    // "import" is absent from the pack ⇒ require the whole universe.
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("p", "import"));
    ::std::assert_eq!(d.required, universe());
  }
}
