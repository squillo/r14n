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
//! - 2026-07-06: + `receipt` module (ISO 27560 / W3C DPV + Kantara CR v1.1) and
//!   `Strictness::as_str` — additive; mirror to the Squillo twin (roadmap item 3).
//! - 2026-07-06: pack_path prefers `<domain>.r14n.toml` (falls back `.toml`);
//!   `[prohibited]` table enforced as the §3 data-minimization guard across ALL
//!   postures — behavior additions; mirror to the Squillo twin (items 4/5 prep).
//! - 2026-07-06: DRY pass — `control_set`/`controls_in_universe` helpers replace
//!   the repeated posture-arm closures; `aggressive_over_universe` unifies the
//!   default adapter + fallback constructors. Behavior-identical (tests green).
//! - 2026-07-07: fail-closed fix (council audit BLOCKER) — a `minimal` pack with
//!   a missing OR empty `[legally_required]` floor now degrades to
//!   aggressive-over-universe (`fell_back = true`) instead of an empty set.
//!   BEHAVIOR CHANGE — mirror to the Squillo twin.

// ── Modules ──────────────────────────────────────────────────────────────────

/// ISO/IEC TS 27560 + W3C DPV decision-receipt serialization (spec §8).
pub mod receipt;

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

  /// The wire string for this posture (inverse of [`Strictness::parse`]) —
  /// surfaced verbatim in decision receipts (never collapsed; audit evidence).
  pub fn as_str(&self) -> &str {
    match self {
      Self::Aggressive => "aggressive",
      Self::AsConfigured => "as_configured",
      Self::Minimal => "minimal",
      Self::Other(other) => other,
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
  /// The decision date as ISO `YYYY-MM-DD` (spec §7). When set, a pack whose
  /// effective-date envelope does not cover it fails closed (spec §4). `None`
  /// disables the temporal check (the caller opts out of envelope enforcement).
  pub as_of: ::std::option::Option<::std::string::String>,
}

/// The decision verdict (spec §2.6). The infallible reference resolver always
/// produces `Permit` (fail-closed = require the full universe with `fell_back`,
/// which is still a permit of everything); a Level-2 resolver that can BLOCK
/// (deontic conflict, envelope expiry, ambiguous attribution) uses `Block`.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::default::Default,
)]
pub enum Verdict {
  /// The query resolves to a required-control set.
  #[default]
  Permit,
  /// The query is unsatisfiable / fail-closed to a hard block.
  Block,
}

impl Verdict {
  /// The wire string (`"permit"` / `"block"`) surfaced in receipts.
  pub fn as_str(&self) -> &str {
    match self {
      Self::Permit => "permit",
      Self::Block => "block",
    }
  }
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
  /// The pack's interpretation-currency date (spec §7 `last_reviewed_against_guidance`),
  /// `None` if the pack does not declare one. Surfaced in the receipt so a
  /// consumer can see staleness rather than silently trust an in-window pack.
  pub last_reviewed_against_guidance: ::std::option::Option<::std::string::String>,
  /// True when the port degraded to the aggressive fallback (pack missing /
  /// malformed) — a loud signal that no reviewed pack governed this decision.
  pub fell_back: bool,
}

/// The resolved decision: the set of controls required for the query, plus
/// provenance.
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
pub struct ControlDecision {
  /// permit | block (spec §2.6). Receipts read the verdict from HERE, never a
  /// hardcoded literal.
  pub verdict: Verdict,
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

/// The one way an Aggressive-over-universe decision is built: shared by the
/// default adapter (`fell_back = false`) and every fail-closed fallback path
/// (`fell_back = true`) so the fail-closed shape cannot drift between them.
fn aggressive_over_universe(
  query: &RegulatoryQuery,
  source: ::std::string::String,
  fell_back: bool,
) -> ControlDecision {
  ControlDecision {
    verdict: Verdict::Permit,
    required: query.universe.clone(),
    provenance: PolicyProvenance {
      profile: query.profile.clone(),
      strictness: Strictness::Aggressive,
      source,
      legal_review_status: ::std::option::Option::None,
      last_reviewed_against_guidance: ::std::option::Option::None,
      fell_back,
    },
  }
}

/// Requires EVERY control in the query universe, always. The safe default when
/// no packs are configured (identical to the pre-config hardcoded behavior).
pub struct AggressiveDefaultPolicyAdapter;

impl __private_seal::RegulatoryPolicyPortSeal for AggressiveDefaultPolicyAdapter {}

impl RegulatoryPolicyPort for AggressiveDefaultPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    aggressive_over_universe(
      query,
      ::std::string::String::from("<aggressive-default>"),
      false,
    )
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
  /// Controls this profile FORBIDS (spec §3 data-minimization guard input):
  /// no posture may ADD a control listed here.
  #[serde(default)]
  prohibited: ::std::option::Option<ControlList>,
}

#[derive(::serde::Deserialize)]
struct PackMeta {
  #[serde(default)]
  strictness: ::std::option::Option<::std::string::String>,
  #[serde(default)]
  legal_review: ::std::option::Option<LegalReview>,
  /// Spec §7 interpretation-currency date; carried into the decision receipt.
  #[serde(default)]
  last_reviewed_against_guidance: ::std::option::Option<::std::string::String>,
  /// Spec §2.5/§7 text-in-effect envelope (ISO `YYYY-MM-DD`). A decision date
  /// outside `[effective_from, effective_until]` fails closed (spec §4).
  #[serde(default)]
  effective_from: ::std::option::Option<::std::string::String>,
  #[serde(default)]
  effective_until: ::std::option::Option<::std::string::String>,
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

/// A `[<table>] controls = [...]` list as a key set.
fn control_set(list: &ControlList) -> ::std::collections::BTreeSet<ControlKey> {
  list.controls.iter().map(|s| ControlKey(s.clone())).collect()
}

/// The list intersected with the caller's universe — a pack can never demand a
/// control the caller does not know how to enforce.
fn controls_in_universe(
  list: &ControlList,
  universe: &::std::collections::BTreeSet<ControlKey>,
) -> ::std::collections::BTreeSet<ControlKey> {
  control_set(list).into_iter().filter(|k| universe.contains(k)).collect()
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

  /// Prefer the RLPS pack extension `<domain>.r14n.toml`; fall back to the
  /// legacy `<domain>.toml` (backward compatible — Squillo's `policies/` trees
  /// keep resolving unchanged).
  fn pack_path(&self, query: &RegulatoryQuery) -> ::std::path::PathBuf {
    let dir = self.root.join(&query.profile.0);
    let r14n = dir.join(::std::format!("{}.r14n.toml", query.domain));
    if r14n.is_file() {
      r14n
    } else {
      dir.join(::std::format!("{}.toml", query.domain))
    }
  }

  /// Aggressive fallback decision (fail-closed) with loud provenance.
  fn fallback(query: &RegulatoryQuery, source: ::std::string::String) -> ControlDecision {
    aggressive_over_universe(query, source, true)
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

    // Text-in-effect envelope (spec §2.5/§4/§7): when the caller supplies a
    // decision date and the pack declares an envelope, a date outside
    // `[effective_from, effective_until]` means no in-effect pack governs — fail
    // closed to aggressive-over-universe, loudly. ISO `YYYY-MM-DD` compares
    // lexically. `as_of = None` opts out of the temporal check.
    if let ::std::option::Option::Some(as_of) = &query.as_of {
      let before_start = pack
        .meta
        .effective_from
        .as_deref()
        .is_some_and(|from| as_of.as_str() < from);
      let after_end = pack
        .meta
        .effective_until
        .as_deref()
        .is_some_and(|until| as_of.as_str() > until);
      if before_start || after_end {
        return Self::fallback(
          query,
          ::std::format!("<outside-effective-envelope:{}>", path.display()),
        );
      }
    }

    // Fail-closed (spec §4/§5): a `minimal` pack whose `[legally_required]` floor
    // is missing OR empty has no enforceable minimum — it MUST NOT resolve to an
    // empty required set. Degrade to aggressive-over-universe, loudly, exactly
    // like a missing/malformed pack. (The linter rejects this at author time; the
    // resolver is the runtime guarantee against unlinted / adversarial packs.)
    if ::std::matches!(strictness, Strictness::Minimal) {
      let has_floor = pack
        .legally_required
        .as_ref()
        .is_some_and(|c| !c.controls.is_empty());
      if !has_floor {
        return Self::fallback(
          query,
          ::std::format!("<minimal-missing-floor:{}>", path.display()),
        );
      }
    }

    // Every resolved set is intersected with the caller's universe
    // (`controls_in_universe`) — a pack can never demand a control the caller
    // does not know how to enforce, and can never silently drop below what the
    // caller declared under Aggressive.
    let required: ::std::collections::BTreeSet<ControlKey> = match &strictness {
      Strictness::Aggressive => query.universe.clone(),
      Strictness::AsConfigured => pack
        .subject
        .get(&query.subject)
        .map(|c| controls_in_universe(c, &query.universe))
        // A subject absent from an as-configured pack is fail-closed: require all.
        .unwrap_or_else(|| query.universe.clone()),
      Strictness::Minimal => pack
        .legally_required
        .as_ref()
        .map(|c| controls_in_universe(c, &query.universe))
        .unwrap_or_default(),
      Strictness::Other(_) => query.universe.clone(),
    };

    // Data-minimization guard (RLPS spec §3): no posture may ADD a control the
    // resolved pack marks prohibited — subtract [prohibited] from EVERY posture's
    // set, including Aggressive ("aggressive" = union of permitted-or-required,
    // never prohibited).
    let required: ::std::collections::BTreeSet<ControlKey> = match &pack.prohibited {
      ::std::option::Option::Some(p) => {
        let prohibited = control_set(p);
        required.into_iter().filter(|k| !prohibited.contains(k)).collect()
      }
      ::std::option::Option::None => required,
    };

    let legal_review_status = pack.meta.legal_review.and_then(|r| r.status);
    ControlDecision {
      verdict: Verdict::Permit,
      required,
      provenance: PolicyProvenance {
        profile: query.profile.clone(),
        strictness,
        source: path.display().to_string(),
        legal_review_status,
        last_reviewed_against_guidance: pack.meta.last_reviewed_against_guidance,
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
      as_of: ::std::option::Option::None,
    }
  }

  fn ck(s: &str) -> super::ControlKey {
    super::ControlKey(::std::string::String::from(s))
  }

  /// Why: the default adapter is the zero-config safety net (spec §5) — if it
  /// ever required less than the full universe, an unconfigured deployment
  /// would silently under-enforce, the exact failure fail-closed exists to
  /// prevent.
  #[test]
  fn aggressive_default_requires_entire_universe() {
    let p = super::AggressiveDefaultPolicyAdapter;
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("anything", "telepresence"));
    ::std::assert_eq!(d.required, universe());
    ::std::assert!(!d.provenance.fell_back);
  }

  /// Why: spec §5 — absence MUST degrade to aggressive-over-universe AND be
  /// loud (`fell_back`); a quiet fallback would let a typo'd profile dir look
  /// like a reviewed deployment in every receipt downstream.
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

  /// Why: `as_configured` is the counsel-reviewed production posture (spec §3)
  /// — the resolver must apply EXACTLY the subject table (no more, no less)
  /// and surface the pack's review status into provenance, or the "what
  /// counsel signed is what runs" contract breaks.
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

  /// Why: the global dial is the operator's "turn it all the way up" control
  /// (spec §3) — if a relaxed pack could win over the override, an operator
  /// could not force maximum strictness during an incident or migration.
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

  /// Why: `minimal` exists so a counsel-confirmed floor can run without the
  /// over-collection of aggressive (spec §3) — but ONLY the floor: if subject
  /// tables leaked in, "minimal" would be a lie in both directions.
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

  /// Why: spec §3 — a subject the pack never contemplated (a new recording
  /// rail shipped after the review) must fail CLOSED to the full universe;
  /// defaulting to empty would make every new feature launch unenforced.
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

  /// Why: the RLPS wire name is `<domain>.r14n.toml` (spec §2.2) while
  /// Squillo's live `policies/` trees use bare `.toml` — the preference order
  /// is the twin-compat contract; inverting it would make a repo ship packs
  /// its own reference resolver ignores.
  #[test]
  fn r14n_toml_extension_is_preferred_over_legacy_toml() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("baseline");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    // Both files exist; the .r14n.toml one must win.
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = [\"signal_notice\"]\n",
    )
    .expect("write r14n pack");
    ::std::fs::write(
      dir.join("recording_consent.toml"),
      "[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = [\"aph_mandate\"]\n",
    )
    .expect("write legacy pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("baseline", "telepresence"));
    let expect: ::std::collections::BTreeSet<super::ControlKey> =
      [ck("signal_notice")].into_iter().collect();
    ::std::assert_eq!(d.required, expect, ".r14n.toml pack must govern");
    ::std::assert!(d.provenance.source.ends_with("recording_consent.r14n.toml"));
  }

  /// Why: an internal design memo must-fix #1 — "aggressive" is a posture, not a legal
  /// ordering; requiring a control a jurisdiction PROHIBITS (the shipped
  /// aggressive-floor-superset bug) is itself a violation. This pins
  /// aggressive = union of permitted-or-required, never prohibited (spec §3).
  #[test]
  fn prohibited_controls_are_never_required_even_under_aggressive() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("dm_guard");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    // The pack prohibits aph_mandate; aggressive must NOT add it (spec §3:
    // aggressive = union of permitted-or-required, never prohibited).
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"aggressive\"\n\
       [subject.telepresence]\ncontrols = [\"signal_notice\", \"aph_mandate\"]\n\
       [legally_required]\ncontrols = [\"user_attestation\"]\n\
       [prohibited]\ncontrols = [\"aph_mandate\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("dm_guard", "telepresence"));
    let expect: ::std::collections::BTreeSet<super::ControlKey> =
      [ck("user_attestation"), ck("all_party_consent"), ck("signal_notice")]
        .into_iter()
        .collect();
    ::std::assert_eq!(d.required, expect, "universe minus prohibited");
    ::std::assert!(!d.required.contains(&ck("aph_mandate")));
  }

  /// Why: the data-minimization guard must hold on EVERY posture path (spec
  /// §3), including the floor — this is the resolve-time backstop for the
  /// authoring error the `r14n validate` linter rejects (floor ∩ prohibited).
  #[test]
  fn prohibited_is_subtracted_even_from_the_minimal_floor() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("dm_floor");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    // An authoring error (floor ∩ prohibited ≠ ∅ — the linter rejects it) must
    // still fail toward NOT adding the prohibited control at resolve time.
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\n\
       [legally_required]\ncontrols = [\"user_attestation\", \"aph_mandate\"]\n\
       [prohibited]\ncontrols = [\"aph_mandate\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("dm_floor", "telepresence"));
    let expect: ::std::collections::BTreeSet<super::ControlKey> =
      [ck("user_attestation")].into_iter().collect();
    ::std::assert_eq!(d.required, expect, "floor minus prohibited");
  }

  /// Why: council-audit BLOCKER — a `minimal` pack with NO floor previously
  /// resolved to an empty set (fail-OPEN), directly contradicting spec §4
  /// ("missing floor MUST block") and the counsel brief's "fail-closed
  /// everywhere". It must degrade to aggressive-over-universe, loudly.
  #[test]
  fn minimal_pack_missing_floor_fails_closed() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("no_floor");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("no_floor", "telepresence"));
    ::std::assert_eq!(d.required, universe(), "no floor ⇒ require everything, not nothing");
    ::std::assert!(d.provenance.fell_back, "the fail-closed degrade must be loud");
  }

  /// Why: an EMPTY floor (`controls = []`) is the same fail-open hazard as a
  /// missing one — a resolver that treated present-but-empty as "zero required"
  /// would let `[legally_required]\ncontrols = []` silently enforce nothing.
  #[test]
  fn minimal_pack_empty_floor_fails_closed() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("empty_floor");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = []\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("empty_floor", "telepresence"));
    ::std::assert_eq!(d.required, universe(), "empty floor ⇒ require everything");
    ::std::assert!(d.provenance.fell_back);
  }

  /// Why: council-audit M4 — spec §4 MUSTs that a pack outside its effective-date
  /// envelope blocks; the resolver never even read the fields, so an expired pack
  /// governed forever. With a decision date supplied, an out-of-window pack must
  /// fail closed (loud fallback); an in-window one resolves normally; and a
  /// caller that supplies no date opts out (unchanged behavior).
  #[test]
  fn pack_outside_effective_envelope_fails_closed() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("dated");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\neffective_from = \"2026-01-01\"\neffective_until = \"2026-06-30\"\n\
       [legally_required]\ncontrols = [\"user_attestation\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let dated = |as_of: ::std::option::Option<&str>| super::RegulatoryQuery {
      as_of: as_of.map(::std::string::String::from),
      ..query("dated", "telepresence")
    };
    // Expired (after the window) ⇒ loud fallback to the whole universe.
    let d = super::RegulatoryPolicyPort::required_controls(&p, &dated(::std::option::Option::Some("2026-07-01")));
    ::std::assert_eq!(d.required, universe(), "expired pack ⇒ fail closed");
    ::std::assert!(d.provenance.fell_back);
    // Not yet effective (before the window) ⇒ same.
    let d = super::RegulatoryPolicyPort::required_controls(&p, &dated(::std::option::Option::Some("2025-12-31")));
    ::std::assert!(d.provenance.fell_back, "not-yet-effective ⇒ fail closed");
    // In window ⇒ resolves normally (minimal floor).
    let d = super::RegulatoryPolicyPort::required_controls(&p, &dated(::std::option::Option::Some("2026-03-15")));
    ::std::assert_eq!(d.required, [ck("user_attestation")].into_iter().collect());
    ::std::assert!(!d.provenance.fell_back);
    // No date supplied ⇒ temporal check disabled (opt-out).
    let d = super::RegulatoryPolicyPort::required_controls(&p, &dated(::std::option::Option::None));
    ::std::assert!(!d.provenance.fell_back, "no as_of ⇒ no temporal enforcement");
  }

  /// Why: receipts surface the posture VERBATIM as audit evidence (spec §2.7)
  /// via `as_str` while packs enter via `parse` — if the pair ever diverged,
  /// audit trails would misreport which posture actually governed.
  #[test]
  fn strictness_wire_string_round_trips() {
    for wire in ["aggressive", "as_configured", "minimal", "turbo"] {
      ::std::assert_eq!(
        super::Strictness::parse(wire).as_str(),
        wire,
        "parse/as_str must be inverses on every wire value"
      );
    }
    ::std::assert_eq!(
      super::Strictness::parse("turbo"),
      super::Strictness::Other(::std::string::String::from("turbo")),
      "unknown values take the forward-additive Other arm"
    );
  }
}
