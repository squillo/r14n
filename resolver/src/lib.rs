// SPDX-License-Identifier: Apache-2.0
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
//! (its regulatory-policy engine) and is fully generic — `serde` + `toml` + `std` only.
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
//! - 2026-08-23: + `InMemoryRegulatoryPolicyAdapter` and the shared
//!   `decide_from_pack_text` pipeline (fs adapter refactored onto it, behavior
//!   unchanged) — the transport for filesystem-less hosts (wasm/Python bindings).
//! - 2026-07-06: extracted as the RLPS reference resolver (r14n) from the Squillo OS
//!   regulatory-policy engine.
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
pub mod wire;

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


/// How the caller determined the jurisdiction it is asking about (spec §4.1).
///
/// TYPED, never a bare string, because the whole point is that a resolver
/// decides on a CLOSED set and an unrecognised word must fall to the safe side.
/// `Other(String)` is the forward-additive escape hatch (§23.5), and it ranks
/// with `Unknown` rather than with a verified source: a spelling this resolver
/// has never heard of is not evidence that attribution succeeded.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::default::Default,
)]
pub enum AttributionSource {
  /// The principal stated its jurisdiction outright.
  Declared,
  /// Derived from an observable signal (a locale, a network location, …).
  Inferred,
  /// Attribution FAILED or was never performed — spec §4.1's fail-closed input.
  #[default]
  Unknown,
  /// Forward-additive escape hatch (§23.5). Treated as `Unknown` when deciding.
  Other(::std::string::String),
}

impl AttributionSource {
  /// Parse the wire string; an unrecognised value becomes `Other`, which
  /// decides as `Unknown` (fail-closed).
  pub fn parse(s: &str) -> Self {
    match s {
      "declared" => Self::Declared,
      "inferred" => Self::Inferred,
      "unknown" => Self::Unknown,
      other => Self::Other(::std::string::String::from(other)),
    }
  }

  /// The wire string (inverse of [`AttributionSource::parse`]).
  pub fn as_str(&self) -> &str {
    match self {
      Self::Declared => "declared",
      Self::Inferred => "inferred",
      Self::Unknown => "unknown",
      Self::Other(other) => other,
    }
  }

  /// Does §4.1 consider this attribution AMBIGUOUS, and therefore blocking?
  ///
  /// `Declared` and `Inferred` are attributions that happened. Everything else
  /// is one that did not, including a spelling this build does not recognise —
  /// ranking an unknown word as "attributed" is how a typo in one caller would
  /// turn a fail-closed condition into a permit.
  pub fn is_ambiguous(&self) -> bool {
    !::std::matches!(self, Self::Declared | Self::Inferred)
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
  /// How the caller determined `jurisdiction` (spec §4.1). `None` means the
  /// caller is not asserting anything about attribution and OPTS OUT of the
  /// §4.1 check — the same contract `as_of: None` has for the temporal
  /// envelope, and the same reason: a field every existing caller omits cannot
  /// start refusing their queries without notice.
  ///
  /// ⚠ So `None` is NOT "attribution succeeded". A gate that must enforce §4.1
  /// has to SUPPLY this; it cannot infer compliance from a resolver that was
  /// never told.
  pub attribution_source: ::std::option::Option<AttributionSource>,
  /// Per-domain posture overrides (spec §3), keyed by domain. The MOST SPECIFIC
  /// setting wins: an entry here beats the global dial, which beats the pack's
  /// own declared posture.
  ///
  /// Carried as a map rather than a single resolved value because a caller
  /// configures its postures once and issues many queries — collapsing it at
  /// the call site would put the precedence rule in every caller, which is
  /// where it would drift.
  pub per_domain_posture_overrides:
    ::std::collections::BTreeMap<::std::string::String, Strictness>,
  /// A `Sec-GPC: 1` signal from the principal (spec §4 step 5). When set, the
  /// subject escalates to `aggressive` regardless of configured posture.
  pub gpc_signal: bool,
  /// An IEEE-7012 `NoRecording` term in the principal's agent terms (spec §4
  /// step 5). Same effect as `gpc_signal`, from a different source.
  pub ieee7012_no_recording: bool,
}


impl RegulatoryQuery {
  /// Does spec §4 step 5 escalate this subject to `aggressive`?
  ///
  /// ESCALATE-ONLY, and that is the whole safety property. `aggressive` is the
  /// maximum posture — the caller's entire declared universe — so forcing it can
  /// only ever require MORE. There is deliberately no inverse: no signal here
  /// can relax a posture, because a relaxation arriving as a request header is
  /// precisely the thing a fail-closed standard must not honour.
  pub fn escalates(&self) -> bool {
    self.gpc_signal || self.ieee7012_no_recording
  }

  /// Which signals escalated, for provenance. Both are reported when both are
  /// present: a receipt that named only the first would understate the basis.
  fn escalation_reasons(&self) -> ::std::vec::Vec<&'static str> {
    let mut out = ::std::vec::Vec::new();
    if self.gpc_signal {
      out.push("gpc_signal");
    }
    if self.ieee7012_no_recording {
      out.push("ieee7012_no_recording");
    }
    out
  }
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

/// A reviewer-key directory entry (spec §6:193-199).
///
/// Only the fields §6 makes load-bearing for a TRUST decision are modelled.
/// `public_key_ed25519` is deliberately absent: this crate cannot verify a
/// signature (AGENTS.md invariant 9 caps its dependencies at serde/toml/std),
/// so holding the key material here would invite a reader to think it does.
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::serde::Deserialize)]
pub struct ReviewerKey {
  /// The directory's identifier for this key — what a pack's
  /// `[meta.legal_review] signature_key_id` names.
  pub key_id: ::std::string::String,
  /// The jurisdiction the reviewer is credentialed in.
  #[serde(default)]
  pub jurisdiction: ::std::option::Option<::std::string::String>,
  /// ISO `YYYY-MM-DD`; a decision dated before this is NOT yet covered.
  #[serde(default)]
  pub valid_from: ::std::option::Option<::std::string::String>,
  /// ISO `YYYY-MM-DD`; §6:192 — expiry is handled IDENTICALLY to revocation.
  #[serde(default)]
  pub valid_until: ::std::option::Option<::std::string::String>,
  /// Present once the key is revoked (§6:188).
  #[serde(default)]
  pub revocation: ::std::option::Option<Revocation>,
}

/// A `revocation` block on a directory entry (spec §6:188).
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::serde::Deserialize)]
pub struct Revocation {
  /// ISO `YYYY-MM-DD`. From this date FORWARD the key is untrusted.
  pub revoked_at: ::std::string::String,
  /// Why, for the receipt. Absent reads `"unspecified"`.
  #[serde(default)]
  pub reason: ::std::option::Option<::std::string::String>,
}

/// The reviewer-key directory a deployment trusts (spec §6:193).
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::serde::Deserialize, ::std::default::Default)]
pub struct ReviewerDirectory {
  /// The listed keys. A key absent from here is UNKNOWN, never trusted.
  #[serde(default)]
  pub keys: ::std::vec::Vec<ReviewerKey>,
}

/// The trust status of a pack's signing key, as of a decision date (spec §6).
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
pub enum KeyStatus {
  /// Listed, in validity, not revoked. The ONLY status that can clear the taint.
  Trusted,
  /// Listed but revoked on/before the decision date (§6:188).
  Revoked {
    /// The date from which the key stopped being trusted.
    revoked_at: ::std::string::String,
    /// The recorded reason.
    reason: ::std::string::String,
  },
  /// Listed but past `valid_until` — §6:192 treats this exactly as revoked.
  Expired {
    /// The date the key stopped being valid.
    valid_until: ::std::string::String,
  },
  /// Listed, but the decision predates `valid_from`.
  NotYetValid {
    /// The date the key starts being valid.
    valid_from: ::std::string::String,
  },
  /// No entry carries this `key_id` — or the pack named no key at all.
  Unknown,
  /// No directory was supplied, so no key CAN be trusted. Distinct from
  /// `Unknown` on purpose: "I have a trust root and your key is not in it" and
  /// "I have no trust root" are different operational facts, and collapsing
  /// them hides a deployment that forgot to configure one.
  NoDirectory,
}

impl KeyStatus {
  /// True only for `Trusted`. A decision escapes advisory-only IFF this holds
  /// AND the caller asserted a verified signature.
  pub fn is_trusted(&self) -> bool {
    ::std::matches!(self, Self::Trusted)
  }

  /// A stable lowercase tag for the receipt.
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Trusted => "trusted",
      Self::Revoked { .. } => "revoked",
      Self::Expired { .. } => "expired",
      Self::NotYetValid { .. } => "not_yet_valid",
      Self::Unknown => "unknown",
      Self::NoDirectory => "no_directory",
    }
  }
}

impl ReviewerDirectory {
  /// Resolve `key_id`'s status as of ISO date `as_of` (spec §6:188-192).
  ///
  /// Ordering is load-bearing and mirrors the CLI's `key_status`: revocation
  /// wins over expiry, which wins over not-yet-valid. ISO dates compare
  /// lexically, which is why no date parsing (and so no date dependency) is
  /// needed here.
  pub fn key_status(&self, key_id: &str, as_of: &str) -> KeyStatus {
    let entry = match self.keys.iter().find(|k| k.key_id == key_id) {
      ::std::option::Option::Some(e) => e,
      ::std::option::Option::None => return KeyStatus::Unknown,
    };
    // §6:188 — "from `revoked_at` FORWARD", so the boundary date is revoked.
    if let ::std::option::Option::Some(rev) = &entry.revocation {
      if !rev.revoked_at.is_empty() && as_of >= rev.revoked_at.as_str() {
        return KeyStatus::Revoked {
          revoked_at: rev.revoked_at.clone(),
          reason: rev
            .reason
            .clone()
            .unwrap_or_else(|| ::std::string::String::from("unspecified")),
        };
      }
    }
    if let ::std::option::Option::Some(until) = &entry.valid_until {
      if as_of > until.as_str() {
        return KeyStatus::Expired { valid_until: until.clone() };
      }
    }
    if let ::std::option::Option::Some(from) = &entry.valid_from {
      if as_of < from.as_str() {
        return KeyStatus::NotYetValid { valid_from: from.clone() };
      }
    }
    KeyStatus::Trusted
  }
}

/// A deployment's trust root for pack provenance (spec §6).
///
/// Two separable facts, because this crate can only establish one of them:
/// the DIRECTORY (listing, revocation, expiry — pure date comparison, which
/// §6:188 says "a resolver MUST" perform), and whether the host
/// CRYPTOGRAPHICALLY verified the pack signature (which needs Ed25519 and so
/// lives in `r14n-tools`, per invariant 9).
///
/// `Default` is the safe one: no directory, no verified signature, so every
/// decision stays advisory-only. A deployment that configures nothing gets the
/// same answer the hardcoded constant used to give — but now because it SAID
/// nothing, not because the code could not say otherwise.
#[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::default::Default)]
pub struct TrustRoot {
  /// The directory to resolve signing keys against.
  pub directory: ::std::option::Option<ReviewerDirectory>,
  /// The host asserts it verified the pack's Ed25519 signature against the
  /// directory-listed public key before calling. **This crate cannot check
  /// that claim** — it is the caller's attestation, and a caller that lies
  /// here produces a receipt that overstates its own provenance.
  pub signatures_verified: bool,
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
  /// Spec §6: provenance was established — the host asserted a verified
  /// signature AND the signing key resolved `Trusted` against the directory as
  /// of the decision date. ONLY this can clear the advisory-only taint.
  ///
  /// This was a hardcoded `const false` until trust-root resolution landed, so
  /// every decision was advisory and no test could prove otherwise — a control
  /// read that was never armed.
  pub provenance_verified: bool,
  /// The signing key's status tag as of the decision date: `trusted`,
  /// `revoked`, `expired`, `not_yet_valid`, `unknown` or `no_directory`.
  /// Surfaced so a receipt reader can tell WHY provenance failed, not merely
  /// that it did.
  pub trust_status: ::std::string::String,
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
  /// The controls the resolved pack FORBIDS, intersected with the query universe
  /// (spec §3 `prohibition`).
  ///
  /// Surfaced because `required` alone cannot tell "not required" apart from
  /// "forbidden", and §4.2.3 cannot detect a cross-jurisdiction deontic conflict
  /// without the forbidden set. Held in the same frame as `required`, so
  /// `required` and `prohibited` are always disjoint within one decision: a
  /// prohibition on a control the caller could not enforce anyway changes no
  /// outcome.
  ///
  /// EMPTY UNDER FALLBACK, and that is the asymmetry to design around. A
  /// fail-closed fallback over-requires (the whole universe) AND under-forbids
  /// (nothing at all, because no pack was read). A `fell_back` decision is
  /// therefore NOT evidence that nothing is prohibited, which is exactly why a
  /// prohibition key must never appear in a caller universe.
  pub prohibited: ::std::collections::BTreeSet<ControlKey>,
  /// Unsatisfiable required-vs-prohibited pairs found while merging across
  /// jurisdictions (spec §4.2.3). Non-empty implies `verdict == Block`.
  ///
  /// Recording these is a spec MUST, not a courtesy: a bare `block` tells a
  /// caller it may not proceed but not what to escalate, and §4.2.3 forbids
  /// resolving the conflict by silently dropping either side.
  pub conflicts: ::std::vec::Vec<DeonticConflict>,
  /// How this decision was reached.
  pub provenance: PolicyProvenance,
}


/// The more restrictive of two postures, for the cross-jurisdiction merge
/// (spec §4.2 rule 1).
///
/// Order is Aggressive > AsConfigured > Minimal, and `Other(_)` ranks WITH
/// Aggressive rather than below it: an unrecognised posture is already treated
/// as require-everything everywhere else in this resolver, and ranking an
/// unknown word as permissive is how a typo in one pack would relax a merged
/// session.
fn most_restrictive(a: Strictness, b: Strictness) -> Strictness {
  fn rank(s: &Strictness) -> u8 {
    match s {
      Strictness::Aggressive | Strictness::Other(_) => 2,
      Strictness::AsConfigured => 1,
      Strictness::Minimal => 0,
    }
  }
  if rank(&b) > rank(&a) {
    b
  } else {
    a
  }
}
/// One unsatisfiable required-vs-prohibited pair across two jurisdictions
/// (spec §4.2.3).
///
/// The spec requires the receipt to record "the conflicting control key and both
/// jurisdictions", so all three travel together. A `Vec<ControlKey>` would have
/// satisfied the verdict and failed the record.
#[derive(
  ::std::clone::Clone,
  ::std::fmt::Debug,
  ::std::cmp::PartialEq,
  ::std::cmp::Eq,
  ::std::cmp::PartialOrd,
  ::std::cmp::Ord,
)]
pub struct DeonticConflict {
  /// The control one jurisdiction requires and another forbids.
  pub control: ControlKey,
  /// The jurisdiction whose pack REQUIRES `control`.
  pub required_by: ::std::string::String,
  /// The jurisdiction whose pack marks `control` prohibited.
  pub prohibited_by: ::std::string::String,
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

  /// Resolve across jurisdictions `J1..Jn` and merge MOST-RESTRICTIVELY
  /// (spec §4.2, normative).
  ///
  /// A provided method, not a per-adapter one, deliberately: §4.2 says a
  /// *resolver* MUST merge this way, so every adapter in this crate inherits one
  /// implementation and none can drift. Each jurisdiction names its own pack
  /// chain, so the sub-query carries it as BOTH the profile (which selects the
  /// pack) and the jurisdiction key (which packs may vary controls by).
  ///
  /// The four rules, in order:
  ///   1. merged `required` is the UNION of the per-jurisdiction required sets;
  ///   2. the merged verdict is `block` if ANY per-jurisdiction resolution blocks;
  ///   3. a control required by one jurisdiction and prohibited by another is
  ///      unsatisfiable — verdict `block`, and both jurisdictions recorded;
  ///   4. the merge stays within one domain (§3 per-domain independence holds).
  ///
  /// An empty `jurisdictions` resolves `query` unchanged, so a caller may route
  /// every query through here without branching.
  ///
  /// ⚠ UNION, NOT INTERSECTION, AND NOT A DROP. Merging the other way would
  /// relax a session to the most permissive jurisdiction it touches, and
  /// dropping a conflicting control would silently satisfy a query the spec says
  /// is unsatisfiable. Both are the same class of mistake as "fixing" a
  /// fail-closed fallback into a permissive default.
  fn resolve_across_jurisdictions(
    &self,
    query: &RegulatoryQuery,
    jurisdictions: &[::std::string::String],
  ) -> ControlDecision {
    // ── §4.1 FIRST, before any pack is read ──────────────────────────────────
    // Spec §4.1 makes ambiguous jurisdiction attribution a fail-closed `block`.
    // It is checked HERE, ahead of resolution, because the question it answers
    // is "do we know WHICH law to apply" — and no amount of correctly resolving
    // the wrong jurisdiction's pack repairs a wrong answer to that. A resolver
    // that resolved first and blocked second would still have read a pack it had
    // no basis to choose.
    //
    // `None` opts out (see the field doc): absent means the caller asserted
    // nothing, not that attribution succeeded.
    if let ::std::option::Option::Some(source) = &query.attribution_source {
      if source.is_ambiguous() {
        return ControlDecision {
          verdict: Verdict::Block,
          // The caller's whole universe, NOT an empty set. A blocked decision
          // whose `required` read empty would, to anything that ignored the
          // verdict, be indistinguishable from "nothing is required" — the
          // permissive reading of a refusal. Invariant 1's shape, applied to a
          // block.
          required: query.universe.clone(),
          prohibited: ::std::collections::BTreeSet::new(),
          conflicts: ::std::vec::Vec::new(),
          provenance: PolicyProvenance {
            profile: query.profile.clone(),
            strictness: Strictness::Aggressive,
            source: ::std::format!("<ambiguous-attribution:{}>", source.as_str()),
            legal_review_status: ::std::option::Option::None,
            last_reviewed_against_guidance: ::std::option::Option::None,
            // No pack was consulted, so this IS a fail-closed degrade and must
            // say so — a caller filtering on `fell_back` to find unreviewed
            // decisions has to see this one.
            fell_back: true,
            // No pack was read, so nothing could be attested.
            provenance_verified: false,
            trust_status: ::std::string::String::from("no_directory"),
          },
        };
      }
    }

    if jurisdictions.is_empty() {
      return self.required_controls(query);
    }

    let mut required: ::std::collections::BTreeSet<ControlKey> =
      ::std::collections::BTreeSet::new();
    let mut prohibited: ::std::collections::BTreeSet<ControlKey> =
      ::std::collections::BTreeSet::new();
    // Per jurisdiction, kept separate from the unions so §4.2.3 can name BOTH
    // sides of a conflict. Collapsing to unions first loses which one said what.
    let mut per: ::std::vec::Vec<(::std::string::String, ControlDecision)> =
      ::std::vec::Vec::new();
    let mut blocked = false;
    let mut fell_back = false;
    let mut sources: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
    let mut strictness = ::std::option::Option::None::<Strictness>;

    for j in jurisdictions {
      let sub = RegulatoryQuery {
        domain: query.domain.clone(),
        profile: RegulatoryProfile(j.clone()),
        jurisdiction: j.clone(),
        subject: query.subject.clone(),
        universe: query.universe.clone(),
        as_of: query.as_of.clone(),
        // Carried forward rather than blanked. The §4.1 gate already ran above,
        // so this cannot block again today — but a sub-query IS about the same
        // attribution as its parent, and a field that lies about that would be
        // wrong the moment the check moves or a second reader appears.
        attribution_source: query.attribution_source.clone(),
        per_domain_posture_overrides: query.per_domain_posture_overrides.clone(),
        gpc_signal: query.gpc_signal,
        ieee7012_no_recording: query.ieee7012_no_recording,
      };
      let d = self.required_controls(&sub);
      required.extend(d.required.iter().cloned());
      prohibited.extend(d.prohibited.iter().cloned());
      blocked = blocked || ::std::matches!(d.verdict, Verdict::Block);
      // ANY jurisdiction falling back taints the merge: the merged set is then
      // partly a fail-closed guess, and a caller that read `fell_back = false`
      // would believe a reviewed pack governed all of it.
      fell_back = fell_back || d.provenance.fell_back;
      sources.push(::std::format!("{j}={}", d.provenance.source));
      strictness = ::std::option::Option::Some(match strictness.take() {
        ::std::option::Option::Some(acc) => most_restrictive(acc, d.provenance.strictness.clone()),
        ::std::option::Option::None => d.provenance.strictness.clone(),
      });
      per.push((j.clone(), d));
    }

    // §4.2.3 — required HERE against prohibited THERE. A sub-resolution already
    // subtracted its own pack's prohibitions, so a conflict is cross-jurisdiction
    // by construction and this loop cannot report a pack against itself.
    let mut conflicts: ::std::vec::Vec<DeonticConflict> = ::std::vec::Vec::new();
    for (ja, da) in &per {
      for (jb, db) in &per {
        if ja == jb {
          continue;
        }
        for control in da.required.intersection(&db.prohibited) {
          conflicts.push(DeonticConflict {
            control: control.clone(),
            required_by: ja.clone(),
            prohibited_by: jb.clone(),
          });
        }
      }
    }
    conflicts.sort();
    conflicts.dedup();

    let verdict = if blocked || !conflicts.is_empty() {
      Verdict::Block
    } else {
      Verdict::Permit
    };

    ControlDecision {
      verdict,
      required,
      prohibited,
      conflicts,
      provenance: PolicyProvenance {
        profile: query.profile.clone(),
        strictness: strictness.unwrap_or(Strictness::Aggressive),
        source: ::std::format!("<merge:{}>", sources.join(",")),
        // A merge spans packs whose review states may differ, and there is no
        // honest single value for the pair. Left None rather than picking one:
        // reporting one pack's `approved` for a merged decision would overstate
        // what counsel reviewed.
        legal_review_status: ::std::option::Option::None,
        last_reviewed_against_guidance: ::std::option::Option::None,
        fell_back,
        // A merge spans packs whose provenance may differ, and there is no
        // honest single value — the same reason `legal_review_status` is None
        // above. False is the conservative read, and it costs nothing: a None
        // review status already forces the advisory taint on this path.
        provenance_verified: false,
        trust_status: ::std::string::String::from("unknown"),
      },
    }
  }
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
    // A fallback read NO pack, so it knows of no prohibition. Empty here is the
    // under-forbidding half of the fail-closed asymmetry, not an assertion that
    // nothing is forbidden -- see the field doc on `prohibited`.
    prohibited: ::std::collections::BTreeSet::new(),
    conflicts: ::std::vec::Vec::new(),
    provenance: PolicyProvenance {
      profile: query.profile.clone(),
      strictness: Strictness::Aggressive,
      source,
      legal_review_status: ::std::option::Option::None,
      last_reviewed_against_guidance: ::std::option::Option::None,
      fell_back,
      // Aggressive-over-universe reads no pack, so there is no signature and
      // no key to resolve.
      provenance_verified: false,
      trust_status: ::std::string::String::from("no_directory"),
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
  /// Spec §6:166 — the directory `key_id` whose signature binds this pack.
  /// A pack naming no key can never be attested: the taint stays.
  #[serde(default)]
  signature_key_id: ::std::option::Option<::std::string::String>,
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


/// The `[meta] inherits` tag from a pack text (spec §4.3) — the ONE reader of that field.
///
/// `PackMeta` deliberately does NOT carry it. A typed field would have made two readers of
/// one declaration, and the parsed one would have been the unused one — which is how the two
/// drift and nobody notices, because only the unused reader is ever type-checked.
///
/// Read on the cheap path on purpose: the chain is built BEFORE anything is validated, so a
/// pack that will not parse must still be reachable — `decide_from_pack_text` is where a
/// malformed pack fails closed, and it can only do that if the chain walker put it in the
/// chain first. A full parse here would silently truncate the chain at the bad pack and
/// resolve against its parents while reporting success.
fn parent_tag(raw: &str) -> ::std::option::Option<::std::string::String> {
  for line in raw.lines() {
    let t = line.trim();
    let rest = match t.strip_prefix("inherits") {
      ::std::option::Option::Some(r) => r.trim_start(),
      ::std::option::Option::None => continue,
    };
    let rest = match rest.strip_prefix('=') {
      ::std::option::Option::Some(r) => r.trim(),
      ::std::option::Option::None => continue,
    };
    let value = rest.trim_matches('"');
    if !value.is_empty() {
      return ::std::option::Option::Some(::std::string::String::from(value));
    }
  }
  ::std::option::Option::None
}
/// RFC-4647-style progressive truncation over `/`-delimited subtags (spec §4 step 1).
///
/// `example/region/locality` yields `["example/region/locality", "example/region", "example", ""]`
/// — most specific first, ending at the root profile. A caller tries each in order and the
/// first pack that exists governs.
///
/// Truncation is LOOKUP ONLY. It never changes the decision's reported profile: provenance
/// names what the caller ASKED for, and the `source` names the pack that answered, so a
/// reader can see that a request for a locality was served by its region. Rewriting the
/// profile would hide exactly that.
fn profile_candidates(profile: &str) -> ::std::vec::Vec<::std::string::String> {
  let mut out = ::std::vec::Vec::new();
  let mut remaining = profile;
  loop {
    out.push(::std::string::String::from(remaining));
    match remaining.rfind('/') {
      ::std::option::Option::Some(cut) => remaining = &remaining[..cut],
      ::std::option::Option::None => break,
    }
  }
  // The root profile, reached last. Pushed only when the profile was not already
  // empty, so a root-profile query does not get a duplicate candidate.
  if !profile.is_empty() {
    out.push(::std::string::String::new());
  }
  out
}

/// Delta-merge a resolved inheritance chain into one effective pack (spec §4.3).
///
/// `chain` is CHILD FIRST. The merge rules are the spec's, and each one is a REPLACE rather
/// than a union, which is the whole point of "delta-only": a child that lists a subject is
/// stating the complete table for that subject, not adding rows to the parent's.
///
///   1. `[meta]` scalars — the child's value wins where it has one.
///   2. `[subject.<name>]` — a child's table REPLACES the parent's for that subject; subjects
///      the child does not mention come from the parent unchanged.
///   3. `[legally_required]` / `[prohibited]` — the child's table, when present, REPLACES.
///
/// ⚠ WHY REPLACE AND NOT UNION, for rule 3 especially. A union of floors could only ever grow
/// the floor, which sounds safe — but it would make a child unable to state a floor at all,
/// and the spec gives the child that authority. A union of PROHIBITIONS would be worse in the
/// other direction: it would let a parent forbid a control the child's reviewing attorney
/// permitted, silently, with no way for the child to say so.
fn merge_chain(chain: ::std::vec::Vec<PolicyPackToml>) -> ::std::option::Option<PolicyPackToml> {
  let mut iter = chain.into_iter();
  let mut effective = iter.next()?;
  for parent in iter {
    // Rule 1 — child scalars win; take the parent's only where the child is silent.
    effective.meta.strictness = effective.meta.strictness.or(parent.meta.strictness);
    effective.meta.legal_review = effective.meta.legal_review.or(parent.meta.legal_review);
    effective.meta.last_reviewed_against_guidance = effective
      .meta
      .last_reviewed_against_guidance
      .or(parent.meta.last_reviewed_against_guidance);
    effective.meta.effective_from =
      effective.meta.effective_from.or(parent.meta.effective_from);
    effective.meta.effective_until =
      effective.meta.effective_until.or(parent.meta.effective_until);

    // Rule 2 — per-subject REPLACE: only subjects the child never mentioned are inherited.
    for (name, table) in parent.subject {
      effective.subject.entry(name).or_insert(table);
    }

    // Rule 3 — the child's table replaces when present; otherwise inherit.
    effective.legally_required = effective.legally_required.or(parent.legally_required);
    effective.prohibited = effective.prohibited.or(parent.prohibited);
  }
  ::std::option::Option::Some(effective)
}

/// The whole RLPS decision pipeline over a resolved inheritance CHAIN — shared
/// verbatim by the filesystem adapter and the in-memory adapter so the two can
/// never drift (one behavior, two transports).
///
/// `chain` is `(source, pack text)` pairs, CHILD FIRST, already resolved by RFC-4647
/// truncation and `inherits` following (spec §4 steps 1-2). The `source` surfaced in the
/// decision names the CHILD — the pack that governs — with its inherited parents appended,
/// so a reader can see the whole chain that produced the answer rather than just its head.
///
/// A MALFORMED PACK ANYWHERE IN THE CHAIN fails closed, not just a malformed child. A parent
/// that will not parse means the effective pack is unknown, and an unknown pack is exactly
/// invariant 1's case — silently dropping the unparseable parent would resolve against a
/// partial pack while reporting success.
fn decide_from_pack_text(
  chain: &[(::std::string::String, ::std::string::String)],
  strictness_override: &::std::option::Option<Strictness>,
  query: &RegulatoryQuery,
  trust: &TrustRoot,
) -> ControlDecision {
  let source = match chain.first() {
    ::std::option::Option::Some((s, _)) => {
      if chain.len() == 1 {
        s.clone()
      } else {
        let parents: ::std::vec::Vec<&str> =
          chain[1..].iter().map(|(s, _)| s.as_str()).collect();
        ::std::format!("{s}<-{}", parents.join("<-"))
      }
    }
    ::std::option::Option::None => {
      return aggressive_over_universe(query, ::std::string::String::from("<empty-chain>"), true);
    }
  };

  let mut parsed: ::std::vec::Vec<PolicyPackToml> = ::std::vec::Vec::new();
  for (one_source, raw) in chain {
    match ::toml::from_str::<PolicyPackToml>(raw) {
      ::std::result::Result::Ok(p) => parsed.push(p),
      ::std::result::Result::Err(_) => {
        return aggressive_over_universe(
          query,
          ::std::format!("<malformed:{one_source}>"),
          true,
        );
      }
    }
  }
  let pack = match merge_chain(parsed) {
    ::std::option::Option::Some(p) => p,
    ::std::option::Option::None => {
      return aggressive_over_universe(query, ::std::format!("<empty-chain:{source}>"), true);
    }
  };
  let source = source.as_str();

  // Spec §4 step 5 — a GPC signal or an IEEE-7012 NoRecording term escalates the
  // subject to `aggressive` REGARDLESS of configured posture, so it sits above
  // every rung of the precedence ladder below rather than inside it.
  //
  // Step 5 runs after step 3 (posture) and step 4 (merge) in the spec's order.
  // Applying it here, per pack, reaches the same answer: `aggressive` resolves to
  // the caller's whole universe whatever the pack says, and a union of whole
  // universes is the whole universe. Doing it per pack also means a merge cannot
  // lose the escalation by picking a posture from one jurisdiction.
  //
  // It can only ever make the answer STRICTER — see `escalates`.
  let strictness = if query.escalates() {
    Strictness::Aggressive
  } else {
    query
      .per_domain_posture_overrides
      .get(&query.domain)
      .cloned()
      .or_else(|| strictness_override.clone())
      .unwrap_or_else(|| {
        pack
          .meta
          .strictness
          .as_deref()
          .map(Strictness::parse)
          .unwrap_or(Strictness::Aggressive)
      })
  };

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
      return aggressive_over_universe(
        query,
        ::std::format!("<outside-effective-envelope:{source}>"),
        true,
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
      return aggressive_over_universe(
        query,
        ::std::format!("<minimal-missing-floor:{source}>"),
        true,
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
  //
  // The prohibited set is also RETAINED on the decision now, not just consumed
  // here: §4.2.3 cannot detect a cross-jurisdiction conflict without it, and a
  // caller could not previously tell "not required" from "forbidden" at all.
  // Intersected with the universe so it shares `required`'s frame.
  let prohibited: ::std::collections::BTreeSet<ControlKey> = match &pack.prohibited {
    ::std::option::Option::Some(p) => {
      controls_in_universe(p, &query.universe)
    }
    ::std::option::Option::None => ::std::collections::BTreeSet::new(),
  };
  let required: ::std::collections::BTreeSet<ControlKey> =
    required.into_iter().filter(|k| !prohibited.contains(k)).collect();

  // ── Spec §6 trust root ──────────────────────────────────────────────────
  //
  // §6:188-190 assigns this to the RESOLVER by name: "from `revoked_at`
  // forward a resolver MUST treat packs signed by that key as unattested".
  // Listing, revocation and expiry are pure lexical date comparison, so this
  // crate can do all of it without the crypto it is forbidden (invariant 9)
  // from carrying. The SIGNATURE check is the half it cannot do, and that
  // arrives as the caller's assertion.
  //
  // A decision date is required: "from revoked_at forward" is meaningless
  // without a date to place the decision on, so `as_of: None` resolves
  // `Unknown` and stays tainted rather than guessing a date that might fall
  // the convenient side of a revocation.
  let review = pack.meta.legal_review;
  let signing_key = review.as_ref().and_then(|r| r.signature_key_id.clone());
  let trust_status = match (&trust.directory, &signing_key, &query.as_of) {
    (::std::option::Option::None, _, _) => KeyStatus::NoDirectory,
    (::std::option::Option::Some(dir), ::std::option::Option::Some(key), ::std::option::Option::Some(as_of)) => {
      dir.key_status(key, as_of)
    }
    _ => KeyStatus::Unknown,
  };
  // Both halves must hold. Either alone is insufficient: a verified signature
  // by a revoked key is exactly the case §6 exists to catch, and a trusted
  // directory entry proves nothing about a pack that merely NAMES the key.
  let provenance_verified = trust.signatures_verified && trust_status.is_trusted();
  let legal_review_status = review.and_then(|r| r.status);
  ControlDecision {
    verdict: Verdict::Permit,
    required,
    prohibited,
    conflicts: ::std::vec::Vec::new(),
    provenance: PolicyProvenance {
      profile: query.profile.clone(),
      strictness,
      // An escalated decision reports WHY in its provenance. `strictness` alone
      // would read `aggressive` while the pack declares `minimal`, which to an
      // auditor looks like the pack and the receipt disagree. Uses the same
      // `<marker>` convention as the merge and ambiguous-attribution paths.
      source: if query.escalates() {
        ::std::format!("<escalated:{}>{source}", query.escalation_reasons().join("+"))
      } else {
        ::std::string::String::from(source)
      },
      legal_review_status,
      last_reviewed_against_guidance: pack.meta.last_reviewed_against_guidance,
      fell_back: false,
      provenance_verified,
      trust_status: ::std::string::String::from(trust_status.as_str()),
    },
  }
}

/// Loads policy packs from a root dir mirroring `locales/`:
/// `<root>/<profile>/<domain>.toml`. A global `strictness_override` (the DIAL)
/// wins over each pack's declared strictness when set.
pub struct TomlRegulatoryPolicyAdapter {
  root: ::std::path::PathBuf,
  strictness_override: ::std::option::Option<Strictness>,
  trust: TrustRoot,
}

impl TomlRegulatoryPolicyAdapter {
  /// Construct with a pack root and an optional global strictness override.
  /// `strictness_override = Some(Aggressive)` forces maximum strictness system-wide
  /// regardless of packs (the "turn it all the way up" dial).
  pub fn new(
    root: ::std::path::PathBuf,
    strictness_override: ::std::option::Option<Strictness>,
  ) -> Self {
    Self { root, strictness_override, trust: TrustRoot::default() }
  }

  /// Attach a spec §6 trust root. Additive and opt-in: an adapter built with
  /// `new` alone has none, so every decision stays advisory-only — the same
  /// answer this crate gave before trust resolution existed, but now because
  /// the deployment configured nothing rather than because the code could not
  /// express otherwise.
  pub fn with_trust_root(mut self, trust: TrustRoot) -> Self {
    self.trust = trust;
    self
  }


  /// Prefer the RLPS pack extension `<domain>.r14n.toml`; fall back to the
  /// legacy `<domain>.toml` (backward compatible — Squillo's `policies/` trees
  /// keep resolving unchanged).
  /// Resolve the pack path for one profile tag, preferring the RLPS extension.
  fn path_for(&self, profile: &str, domain: &str) -> ::std::option::Option<::std::path::PathBuf> {
    let dir = if profile.is_empty() { self.root.clone() } else { self.root.join(profile) };
    for name in [::std::format!("{domain}.r14n.toml"), ::std::format!("{domain}.toml")] {
      let candidate = dir.join(&name);
      if candidate.is_file() {
        return ::std::option::Option::Some(candidate);
      }
    }
    ::std::option::Option::None
  }

  /// The inheritance chain, child first (spec §4 steps 1-2): negotiate the profile by
  /// RFC-4647 truncation, then follow `inherits` from whatever that found.
  fn chain(
    &self,
    query: &RegulatoryQuery,
  ) -> ::std::vec::Vec<(::std::string::String, ::std::string::String)> {
    let mut out = ::std::vec::Vec::new();
    let mut seen: ::std::collections::BTreeSet<::std::string::String> =
      ::std::collections::BTreeSet::new();
    let mut want = query.profile.0.clone();
    loop {
      // Truncation applies to EVERY link, not just the first: a parent tag may itself
      // name a profile that only exists further up the chain.
      let found = profile_candidates(&want)
        .into_iter()
        .find_map(|c| self.path_for(&c, &query.domain).map(|p| (c, p)));
      let (tag, path) = match found {
        ::std::option::Option::Some(f) => f,
        ::std::option::Option::None => break,
      };
      // A cycle, or a diamond revisiting one tag, stops here rather than looping. The
      // packs resolved so far still govern — refusing the whole decision because a
      // parent pointed back would fail closed on an authoring mistake in a file the
      // caller may not own.
      if !seen.insert(tag.clone()) {
        break;
      }
      let raw = match ::std::fs::read_to_string(&path) {
        ::std::result::Result::Ok(s) => s,
        ::std::result::Result::Err(_) => break,
      };
      let parent = parent_tag(&raw);
      out.push((path.display().to_string(), raw));
      match parent {
        ::std::option::Option::Some(p) => want = p,
        ::std::option::Option::None => break,
      }
    }
    out
  }

  /// Aggressive fallback decision (fail-closed) with loud provenance.
  fn fallback(query: &RegulatoryQuery, source: ::std::string::String) -> ControlDecision {
    aggressive_over_universe(query, source, true)
  }
}

impl __private_seal::RegulatoryPolicyPortSeal for TomlRegulatoryPolicyAdapter {}

impl RegulatoryPolicyPort for TomlRegulatoryPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    let chain = self.chain(query);
    if chain.is_empty() {
      return Self::fallback(
        query,
        ::std::format!("<missing:{}>", self.root.join(&query.profile.0).display()),
      );
    }
    decide_from_pack_text(&chain, &self.strictness_override, query, &self.trust)
  }
}

/// Resolves policy packs held entirely in memory — the transport for hosts
/// with no filesystem (the wasm/JS and Python bindings, tests, embedded
/// callers). Keys mirror the on-disk layout relative to the pack root:
/// `<profile>/<domain>.r14n.toml` (preferred) or the legacy
/// `<profile>/<domain>.toml`. Decision behavior is IDENTICAL to
/// [`TomlRegulatoryPolicyAdapter`] by construction — both call
/// [`decide_from_pack_text`]; only the pack lookup differs.
pub struct InMemoryRegulatoryPolicyAdapter {
  packs: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
  strictness_override: ::std::option::Option<Strictness>,
  trust: TrustRoot,
}

impl InMemoryRegulatoryPolicyAdapter {
  /// Construct from `<profile>/<domain>.r14n.toml → TOML text` entries and an
  /// optional global strictness override (the same DIAL as the fs adapter).
  pub fn new(
    packs: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
    strictness_override: ::std::option::Option<Strictness>,
  ) -> Self {
    Self { packs, strictness_override, trust: TrustRoot::default() }
  }

  /// Attach a spec §6 trust root. Additive and opt-in: an adapter built with
  /// `new` alone has none, so every decision stays advisory-only — the same
  /// answer this crate gave before trust resolution existed, but now because
  /// the deployment configured nothing rather than because the code could not
  /// express otherwise.
  pub fn with_trust_root(mut self, trust: TrustRoot) -> Self {
    self.trust = trust;
    self
  }


  /// Prefer the RLPS pack key `<profile>/<domain>.r14n.toml`; fall back to the
  /// legacy `<profile>/<domain>.toml` — the same probe order as the fs adapter.
  fn key_for(
    &self,
    profile: &str,
    domain: &str,
  ) -> ::std::option::Option<(::std::string::String, &::std::string::String)> {
    for name in [
      ::std::format!("{domain}.r14n.toml"),
      ::std::format!("{domain}.toml"),
    ] {
      let key =
        if profile.is_empty() { name.clone() } else { ::std::format!("{profile}/{name}") };
      if let ::std::option::Option::Some(text) = self.packs.get(&key) {
        return ::std::option::Option::Some((key, text));
      }
    }
    ::std::option::Option::None
  }

  /// The inheritance chain, child first (spec §4 steps 1-2). Mirrors the fs adapter's
  /// `chain` exactly; only the transport differs.
  fn chain(
    &self,
    query: &RegulatoryQuery,
  ) -> ::std::vec::Vec<(::std::string::String, ::std::string::String)> {
    let mut out = ::std::vec::Vec::new();
    let mut seen: ::std::collections::BTreeSet<::std::string::String> =
      ::std::collections::BTreeSet::new();
    let mut want = query.profile.0.clone();
    loop {
      let found = profile_candidates(&want)
        .into_iter()
        .find_map(|c| self.key_for(&c, &query.domain).map(|(k, t)| (c, k, t)));
      let (tag, key, text) = match found {
        ::std::option::Option::Some(f) => f,
        ::std::option::Option::None => break,
      };
      if !seen.insert(tag) {
        break;
      }
      let parent = parent_tag(text);
      out.push((key, text.clone()));
      match parent {
        ::std::option::Option::Some(p) => want = p,
        ::std::option::Option::None => break,
      }
    }
    out
  }
}

impl __private_seal::RegulatoryPolicyPortSeal for InMemoryRegulatoryPolicyAdapter {}

impl RegulatoryPolicyPort for InMemoryRegulatoryPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    let chain = self.chain(query);
    if chain.is_empty() {
      return aggressive_over_universe(
        query,
        ::std::format!("<missing:{}/{}.r14n.toml>", query.profile.0, query.domain),
        true,
      );
    }
    decide_from_pack_text(&chain, &self.strictness_override, query, &self.trust)
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────



/// Spec §4 step 5 escalation. The conformance vectors pin that each signal escalates; these
/// pin the properties a resolver could get wrong while still passing them.
#[cfg(test)]
mod escalation_tests {
  // Declares `minimal` AND carries a prohibition, so escalation and the §3
  // data-minimization guard are exercised against each other.
  const PACK: &str = "[meta]\nstrictness = \"minimal\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";

  fn decide(gpc: bool, ieee: bool, per_domain: &[(&str, crate::Strictness)]) -> crate::ControlDecision {
    let adapter = crate::InMemoryRegulatoryPolicyAdapter::new(
      [(
        ::std::string::String::from("esc/recording_consent.r14n.toml"),
        ::std::string::String::from(PACK),
      )]
      .into_iter()
      .collect(),
      ::std::option::Option::None,
    );
    let query = crate::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: crate::RegulatoryProfile(::std::string::String::from("esc")),
      jurisdiction: ::std::string::String::from("all_party"),
      subject: ::std::string::String::from("telepresence"),
      universe: ["attestation", "signal_notice", "aph_mandate"]
        .into_iter()
        .map(|c| crate::ControlKey(::std::string::String::from(c)))
        .collect(),
      as_of: ::std::option::Option::None,
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: per_domain
        .iter()
        .map(|(d, s)| (::std::string::String::from(*d), s.clone()))
        .collect(),
      gpc_signal: gpc,
      ieee7012_no_recording: ieee,
    };
    crate::RegulatoryPolicyPort::required_controls(&adapter, &query)
  }

  fn required(d: &crate::ControlDecision) -> ::std::vec::Vec<&str> {
    d.required.iter().map(|c| c.as_str()).collect()
  }

  /// Why: escalation must beat EVERY rung of the posture ladder, not just the pack's declared
  /// posture — "regardless of configured posture" (§4 step 5). A per-domain override is the
  /// most specific rung and therefore the one most likely to be checked first and win by
  /// accident; a resolver that folded escalation into the ladder instead of above it would
  /// pass the conformance vectors, which supply no override at all.
  #[test]
  fn escalation_beats_the_most_specific_posture_override() {
    let no_signal = decide(false, false, &[("recording_consent", crate::Strictness::Minimal)]);
    ::std::assert_eq!(required(&no_signal), ["attestation"], "baseline: the override governs");

    for (gpc, ieee) in [(true, false), (false, true), (true, true)] {
      let d = decide(gpc, ieee, &[("recording_consent", crate::Strictness::Minimal)]);
      ::std::assert_eq!(
        required(&d),
        ["attestation", "signal_notice"],
        "escalation lost to a per-domain override (gpc={gpc}, ieee={ieee})"
      );
    }
  }

  /// Why: escalation raises the posture, it does NOT bypass the §3 data-minimization guard.
  /// Spec §3 says no posture may ADD a prohibited control, "including Aggressive" — so an
  /// escalated decision is the universe MINUS the prohibition, never the raw universe. A
  /// resolver that short-circuited to `query.universe` on a signal would hand back a control
  /// the pack forbids, which is the under-forbidding direction and the worse one.
  #[test]
  fn escalation_does_not_override_a_prohibition() {
    let d = decide(true, false, &[]);
    ::std::assert!(
      !d.required.iter().any(|c| c.as_str() == "aph_mandate"),
      "escalation added a PROHIBITED control: {:?}",
      required(&d)
    );
    ::std::assert!(
      d.prohibited.iter().any(|c| c.as_str() == "aph_mandate"),
      "the prohibition is not surfaced on an escalated decision"
    );
  }

  /// Why: an escalated decision reads `strictness = aggressive` while the pack declares
  /// `minimal`, which to an auditor looks like the pack and the receipt disagree. The
  /// provenance has to say WHY, and it has to name BOTH signals when both fired — reporting
  /// only the first would understate the basis for the decision.
  #[test]
  fn provenance_names_the_signals_that_escalated() {
    ::std::assert_eq!(
      decide(false, false, &[]).provenance.source,
      "esc/recording_consent.r14n.toml",
      "an unescalated decision must not carry an escalation marker"
    );
    let gpc = decide(true, false, &[]).provenance.source;
    ::std::assert!(gpc.starts_with("<escalated:gpc_signal>"), "got {gpc}");
    let ieee = decide(false, true, &[]).provenance.source;
    ::std::assert!(ieee.starts_with("<escalated:ieee7012_no_recording>"), "got {ieee}");
    let both = decide(true, true, &[]).provenance.source;
    ::std::assert!(
      both.starts_with("<escalated:gpc_signal+ieee7012_no_recording>"),
      "both signals fired but provenance named fewer: {both}"
    );
  }

  /// Why: a pack GOVERNED this decision — escalation changed which table was read, not
  /// whether a reviewed pack applied. If `fell_back` were set, every gate that filters for
  /// unreviewed decisions would flag an escalated one, and the signal that actually means
  /// "no pack governed" would lose its meaning.
  #[test]
  fn escalation_is_not_a_fallback() {
    let d = decide(true, false, &[]);
    ::std::assert!(!d.provenance.fell_back, "escalation reported as a fail-closed fallback");
    ::std::assert_eq!(d.verdict, crate::Verdict::Permit);
  }
}

/// RFC-4647 negotiation and delta-merge (spec §4 steps 1-2, §4.3). The two conformance
/// vectors pin one truncation and one inheritance case; these pin the directions a resolver
/// can get wrong while passing both.
#[cfg(test)]
mod inheritance_tests {
  const PARENT: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[subject.import]\ncontrols = [\"attestation\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";
  const CHILD: &str = "[meta]\nstrictness = \"as_configured\"\ninherits = \"example\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\", \"attestation\"]\n";

  fn adapter(packs: &[(&str, &str)]) -> crate::InMemoryRegulatoryPolicyAdapter {
    crate::InMemoryRegulatoryPolicyAdapter::new(
      packs
        .iter()
        .map(|(k, v)| (::std::string::String::from(*k), ::std::string::String::from(*v)))
        .collect(),
      ::std::option::Option::None,
    )
  }

  fn decide(packs: &[(&str, &str)], profile: &str, subject: &str) -> crate::ControlDecision {
    let query = crate::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: crate::RegulatoryProfile(::std::string::String::from(profile)),
      jurisdiction: ::std::string::String::from("all_party"),
      subject: ::std::string::String::from(subject),
      universe: ["attestation", "signal_notice", "aph_mandate"]
        .into_iter()
        .map(|c| crate::ControlKey(::std::string::String::from(c)))
        .collect(),
      as_of: ::std::option::Option::None,
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
      gpc_signal: false,
      ieee7012_no_recording: false,
    };
    crate::RegulatoryPolicyPort::required_controls(&adapter(packs), &query)
  }

  fn required(d: &crate::ControlDecision) -> ::std::vec::Vec<&str> {
    d.required.iter().map(|c| c.as_str()).collect()
  }

  const CHAIN: [(&str, &str); 2] = [
    ("example/recording_consent.r14n.toml", PARENT),
    ("example/region/recording_consent.r14n.toml", CHILD),
  ];

  /// Why: §4.3 rule 2 is REPLACE per subject, not union. The child here lists `telepresence`
  /// with two controls where the parent lists one; a union would return the same set, so the
  /// vectors cannot tell replace from union on that subject. This checks the direction that
  /// CAN differ — a child narrowing a subject the parent made broader.
  #[test]
  fn a_child_subject_table_replaces_the_parents_rather_than_unioning() {
    const NARROWING: &str = "[meta]\nstrictness = \"as_configured\"\ninherits = \"example\"\n\n[subject.import]\ncontrols = [\"signal_notice\"]\n";
    let packs = [
      ("example/recording_consent.r14n.toml", PARENT),
      ("example/region/recording_consent.r14n.toml", NARROWING),
    ];
    // The parent requires `attestation` for `import`; the child replaces that table with
    // `signal_notice`. A union would return both and look like a stricter, plausible answer.
    ::std::assert_eq!(
      required(&decide(&packs, "example/region", "import")),
      ["signal_notice"],
      "the child's subject table did not REPLACE the parent's"
    );
  }

  /// Why: the half of §4.3 rule 2 that makes it delta-ONLY — a subject the child never
  /// mentions must come from the parent unchanged. A resolver that treated an inheriting
  /// child as the complete pack would hit the unknown-subject path instead and return the
  /// whole universe, which is fail-closed and therefore looks defensible rather than wrong.
  #[test]
  fn a_subject_the_child_omits_is_inherited_not_fail_closed() {
    let d = decide(&CHAIN, "example/region", "import");
    ::std::assert_eq!(required(&d), ["attestation"], "the unlisted subject was not inherited");
    ::std::assert_eq!(d.required.len(), 1, "an unknown-subject fallback would require all three");
  }

  /// Why: §4.3 rule 3 — the child's floor and prohibitions REPLACE when present and are
  /// INHERITED when absent. This child declares neither, so both must come from the parent;
  /// losing the inherited `[prohibited]` would silently restore a control the parent forbids,
  /// which is the under-forbidding direction.
  #[test]
  fn an_absent_floor_and_prohibition_are_inherited() {
    let d = decide(&CHAIN, "example/region", "telepresence");
    ::std::assert!(
      d.prohibited.iter().any(|c| c.as_str() == "aph_mandate"),
      "the parent's prohibition was lost across the merge: {:?}",
      d.prohibited
    );
    ::std::assert!(
      !d.required.iter().any(|c| c.as_str() == "aph_mandate"),
      "a prohibited control came back: {:?}",
      required(&d)
    );
  }

  /// Why: §4 step 1 truncation must NOT rewrite the reported profile. Provenance names what
  /// the caller asked for and `source` names the pack that answered, so a reader can see that
  /// a request for a locality was served by its region. A resolver that overwrote the profile
  /// with the tag it matched would erase the only evidence that negotiation happened.
  #[test]
  fn truncation_serves_a_parent_pack_without_rewriting_the_asked_profile() {
    let packs = [("example/region/recording_consent.r14n.toml", CHILD)];
    let d = decide(&packs, "example/region/locality", "telepresence");
    ::std::assert_eq!(
      d.provenance.profile.0, "example/region/locality",
      "truncation rewrote the profile the caller asked for"
    );
    ::std::assert!(
      d.provenance.source.contains("example/region/"),
      "source does not name the pack that actually answered: {}",
      d.provenance.source
    );
    ::std::assert!(!d.provenance.fell_back, "a negotiated pack DID govern");
  }

  /// Why: an inheritance cycle is an authoring mistake in files the caller may not own, and
  /// it must not hang or recurse without bound. It also must not fail the whole decision —
  /// the packs resolved before the cycle still govern. Asserted on a self-inheriting pack,
  /// the tightest possible loop.
  #[test]
  fn a_self_inheriting_pack_terminates_and_still_governs() {
    const LOOP: &str = "[meta]\nstrictness = \"as_configured\"\ninherits = \"example\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
    let packs = [("example/recording_consent.r14n.toml", LOOP)];
    let d = decide(&packs, "example", "telepresence");
    ::std::assert_eq!(required(&d), ["signal_notice"], "the self-inheriting pack did not govern");
    ::std::assert!(!d.provenance.fell_back, "a cycle must not fail the decision closed");
  }

  /// Why: a malformed pack ANYWHERE in the chain has to fail closed, not just a malformed
  /// child. Dropping an unparseable parent would resolve against a partial pack and report
  /// success — the pack's real content is unknown, which is precisely invariant 1's case.
  #[test]
  fn a_malformed_parent_fails_the_whole_chain_closed() {
    let packs = [
      ("example/recording_consent.r14n.toml", "this is not toml {{{"),
      ("example/region/recording_consent.r14n.toml", CHILD),
    ];
    let d = decide(&packs, "example/region", "telepresence");
    ::std::assert!(d.provenance.fell_back, "a malformed PARENT was silently dropped");
    ::std::assert_eq!(d.required.len(), 3, "a fail-closed decision requires the whole universe");
    ::std::assert!(
      d.provenance.source.starts_with("<malformed:"),
      "provenance does not name the malformed pack: {}",
      d.provenance.source
    );
  }

  /// Why: the truncation candidate list is the whole of §4 step 1, and it is easy to write one
  /// that drops the root or duplicates it. `gdpr/eu/de → gdpr/eu → gdpr → root` is the spec's
  /// own worked example, transliterated to fictional subtags.
  #[test]
  fn the_candidate_list_truncates_to_root_exactly_once() {
    ::std::assert_eq!(
      crate::profile_candidates("example/region/locality"),
      ["example/region/locality", "example/region", "example", ""]
    );
    ::std::assert_eq!(crate::profile_candidates("example"), ["example", ""]);
    ::std::assert_eq!(crate::profile_candidates(""), [""], "root must not yield a duplicate");
  }
}


/// Spec §4.1 — ambiguous jurisdiction attribution is a fail-closed block. These sit beside
/// the single conformance vector because the vector pins ONE shape (`unknown`), and the
/// ways this can be implemented wrongly are mostly about the OTHER inputs.
#[cfg(test)]
mod attribution_tests {
  fn query(source: ::std::option::Option<crate::AttributionSource>) -> crate::RegulatoryQuery {
    crate::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: crate::RegulatoryProfile(::std::string::String::from("amb")),
      jurisdiction: ::std::string::String::from("all_party"),
      subject: ::std::string::String::from("telepresence"),
      universe: ["attestation", "signal_notice"]
        .into_iter()
        .map(|c| crate::ControlKey(::std::string::String::from(c)))
        .collect(),
      as_of: ::std::option::Option::None,
      attribution_source: source,
      per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
      gpc_signal: false,
      ieee7012_no_recording: false,
    }
  }

  fn adapter() -> crate::InMemoryRegulatoryPolicyAdapter {
    crate::InMemoryRegulatoryPolicyAdapter::new(
      [(
        ::std::string::String::from("amb/recording_consent.r14n.toml"),
        ::std::string::String::from(
          "[meta]\nstrictness = \"minimal\"\n\n[subject.telepresence]\ncontrols = [\"attestation\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n",
        ),
      )]
      .into_iter()
      .collect(),
      ::std::option::Option::None,
    )
  }

  fn decide(source: ::std::option::Option<crate::AttributionSource>) -> crate::ControlDecision {
    crate::RegulatoryPolicyPort::resolve_across_jurisdictions(&adapter(), &query(source), &[])
  }

  /// Why: §4.1 is the gap `conformance/README.md` calls the one a reader is most likely to
  /// assume closed, and the failure it guards is applying the wrong jurisdiction's law
  /// confidently. The pack here RESOLVES cleanly under `minimal` — so a resolver that read
  /// the pack first would return a tidy permit, and nothing in that answer would look wrong.
  #[test]
  fn an_unknown_attribution_blocks_even_when_a_pack_would_have_resolved() {
    let permitted = decide(::std::option::Option::Some(crate::AttributionSource::Declared));
    ::std::assert_eq!(permitted.verdict, crate::Verdict::Permit, "a declared attribution must resolve");

    let blocked = decide(::std::option::Option::Some(crate::AttributionSource::Unknown));
    ::std::assert_eq!(blocked.verdict, crate::Verdict::Block, "unknown attribution must block");
    ::std::assert!(
      blocked.provenance.source.contains("ambiguous-attribution"),
      "the provenance must say WHY it blocked, not just that it did: {}",
      blocked.provenance.source
    );
  }

  /// Why: a blocked decision whose `required` set read EMPTY would, to anything that ignored
  /// the verdict, be indistinguishable from "nothing is required" — the permissive reading of
  /// a refusal. Invariant 1 gives the fail-closed shape as the caller's whole universe, and a
  /// block has to keep it; and `fell_back` must be true because no pack was consulted, or a
  /// gate filtering for unreviewed decisions would miss this one entirely.
  #[test]
  fn a_block_still_requires_the_whole_universe_and_reports_falling_back() {
    let d = decide(::std::option::Option::Some(crate::AttributionSource::Unknown));
    let got: ::std::vec::Vec<&str> = d.required.iter().map(|c| c.as_str()).collect();
    ::std::assert_eq!(got, ["attestation", "signal_notice"], "a block must not read as empty");
    ::std::assert!(d.provenance.fell_back, "no pack was read, so this is a fail-closed degrade");
  }

  /// Why: `None` has to stay the opt-out, because every caller written before §4.1 omits the
  /// field and would otherwise start being refused without notice. This is the same contract
  /// `as_of: None` already has for the temporal envelope — stated as a test so the two cannot
  /// drift into different meanings for "absent".
  #[test]
  fn an_absent_attribution_opts_out_rather_than_blocking() {
    let d = decide(::std::option::Option::None);
    ::std::assert_eq!(d.verdict, crate::Verdict::Permit);
    ::std::assert!(!d.provenance.fell_back, "opting out is not a fail-closed degrade");
  }

  /// Why: an unrecognised spelling must rank with `Unknown`, not with a verified source.
  /// Ranking an unknown word as "attributed" is how one caller's typo turns a fail-closed
  /// condition into a permit — the same reasoning as an unknown POSTURE ranking with
  /// aggressive, and the round-trip is pinned so the escape hatch stays auditable.
  #[test]
  fn an_unrecognised_source_decides_as_unknown_and_round_trips() {
    let other = crate::AttributionSource::parse("gps-triangulated");
    ::std::assert!(other.is_ambiguous(), "an unrecognised source must not read as attributed");
    ::std::assert_eq!(other.as_str(), "gps-triangulated", "the escape hatch must round-trip verbatim");
    ::std::assert_eq!(decide(::std::option::Option::Some(other)).verdict, crate::Verdict::Block);

    for s in ["declared", "inferred"] {
      ::std::assert!(!crate::AttributionSource::parse(s).is_ambiguous(), "{s} is an attribution");
    }
    ::std::assert!(crate::AttributionSource::parse("unknown").is_ambiguous());
  }
}


/// Per-domain posture precedence (spec §3). The conformance vector pins the one ordering
/// that matters most; these pin the rest of the ladder, where a wrong answer still looks
/// like a successful resolution.
#[cfg(test)]
mod per_domain_posture_tests {
  const PACK: &str = "[meta]\nstrictness = \"aggressive\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";

  fn decide(
    global: ::std::option::Option<crate::Strictness>,
    per_domain: &[(&str, crate::Strictness)],
  ) -> crate::ControlDecision {
    let adapter = crate::InMemoryRegulatoryPolicyAdapter::new(
      [(
        ::std::string::String::from("pd/recording_consent.r14n.toml"),
        ::std::string::String::from(PACK),
      )]
      .into_iter()
      .collect(),
      global,
    );
    let query = crate::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: crate::RegulatoryProfile(::std::string::String::from("pd")),
      jurisdiction: ::std::string::String::from("all_party"),
      subject: ::std::string::String::from("telepresence"),
      universe: ["attestation", "signal_notice", "aph_mandate"]
        .into_iter()
        .map(|c| crate::ControlKey(::std::string::String::from(c)))
        .collect(),
      as_of: ::std::option::Option::None,
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: per_domain
        .iter()
        .map(|(d, s)| (::std::string::String::from(*d), s.clone()))
        .collect(),
      gpc_signal: false,
      ieee7012_no_recording: false,
    };
    crate::RegulatoryPolicyPort::required_controls(&adapter, &query)
  }

  fn required(d: &crate::ControlDecision) -> ::std::vec::Vec<&str> {
    d.required.iter().map(|c| c.as_str()).collect()
  }

  /// Why: the whole ladder, asserted in one place because the failure mode is a resolution
  /// that SUCCEEDS at the wrong posture. Each rung produces a different set from the same
  /// pack, so an off-by-one in the precedence returns a plausible answer rather than an
  /// error — the pack declares `aggressive` (whole universe), its subject table is
  /// `[signal_notice]`, and its floor is `[attestation]`.
  #[test]
  fn the_most_specific_posture_wins_at_every_rung() {
    // Nothing supplied: the pack's own declared posture governs.
    ::std::assert_eq!(
      required(&decide(::std::option::Option::None, &[])),
      ["aph_mandate", "attestation", "signal_notice"],
      "the pack's declared aggressive posture did not govern"
    );
    // Global dial beats the pack.
    ::std::assert_eq!(
      required(&decide(::std::option::Option::Some(crate::Strictness::AsConfigured), &[])),
      ["signal_notice"],
      "the global dial did not beat the pack's declared posture"
    );
    // Per-domain beats the global dial — the vector's case.
    ::std::assert_eq!(
      required(&decide(
        ::std::option::Option::Some(crate::Strictness::Aggressive),
        &[("recording_consent", crate::Strictness::Minimal)],
      )),
      ["attestation"],
      "a per-domain override did not beat the global dial"
    );
    // Per-domain beats the pack with no global dial set.
    ::std::assert_eq!(
      required(&decide(
        ::std::option::Option::None,
        &[("recording_consent", crate::Strictness::Minimal)],
      )),
      ["attestation"],
      "a per-domain override did not beat the pack's declared posture"
    );
  }

  /// Why: §3's per-domain INDEPENDENCE. An override for another domain must not reach this
  /// one — and the failure is silent, because applying someone else's posture still returns
  /// a well-formed set. A map lookup that ignored the key, or took the first entry, would
  /// pass every single-entry test.
  #[test]
  fn an_override_for_a_different_domain_does_not_leak_into_this_one() {
    ::std::assert_eq!(
      required(&decide(
        ::std::option::Option::None,
        &[("some_other_domain", crate::Strictness::Minimal)],
      )),
      ["aph_mandate", "attestation", "signal_notice"],
      "another domain's posture override changed THIS domain's answer"
    );
    // And with both present, only the matching key applies.
    ::std::assert_eq!(
      required(&decide(
        ::std::option::Option::None,
        &[
          ("some_other_domain", crate::Strictness::Aggressive),
          ("recording_consent", crate::Strictness::Minimal),
        ],
      )),
      ["attestation"],
      "the override was selected by something other than the domain key"
    );
  }
}
/// Cross-jurisdiction merge (spec §4.2). These sit beside the conformance
/// vectors deliberately: the vectors pin the two outcomes the standard
/// publishes, and these pin the DIRECTIONS the implementation could silently
/// go wrong in without any vector noticing.
#[cfg(test)]
mod merge_tests {
  fn adapter(
    packs: &[(&str, &str)],
  ) -> crate::InMemoryRegulatoryPolicyAdapter {
    crate::InMemoryRegulatoryPolicyAdapter::new(
      packs
        .iter()
        .map(|(k, v)| (::std::string::String::from(*k), ::std::string::String::from(*v)))
        .collect(),
      ::std::option::Option::None,
    )
  }

  fn query(universe: &[&str]) -> crate::RegulatoryQuery {
    crate::RegulatoryQuery {
      domain: ::std::string::String::from("recording_consent"),
      profile: crate::RegulatoryProfile(::std::string::String::from("base")),
      jurisdiction: ::std::string::String::from("multi"),
      subject: ::std::string::String::from("telepresence"),
      universe: universe
        .iter()
        .map(|c| crate::ControlKey(::std::string::String::from(*c)))
        .collect(),
      as_of: ::std::option::Option::None,
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
      gpc_signal: false,
      ieee7012_no_recording: false,
    }
  }

  fn js(names: &[&str]) -> ::std::vec::Vec<::std::string::String> {
    names.iter().map(|n| ::std::string::String::from(*n)).collect()
  }

  const A: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"attestation\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n";
  const B: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"signal_notice\"]\n";

  /// Why: §4.2 rule 1 says UNION, and the failure that matters is the merge
  /// going the other way. An intersection — or taking the first jurisdiction and
  /// stopping — returns a STRICT SUBSET, which reads as a successful decision
  /// and silently relaxes a session to its most permissive jurisdiction. That is
  /// the same class of defect as "fixing" a fail-closed fallback into a
  /// permissive default, and nothing about the output would look wrong.
  #[test]
  fn the_merge_unions_and_is_a_superset_of_every_part() {
    let ad = adapter(&[
      ("ja/recording_consent.r14n.toml", A),
      ("jb/recording_consent.r14n.toml", B),
    ]);
    let q = query(&["attestation", "signal_notice", "aph_mandate"]);
    let merged = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(&ad, &q, &js(&["ja", "jb"]));

    let got: ::std::vec::Vec<&str> = merged.required.iter().map(|c| c.as_str()).collect();
    ::std::assert_eq!(got, ["attestation", "signal_notice"], "merge is not the union");

    // And a superset of each part, checked against the parts themselves rather
    // than a literal, so this keeps holding if the fixtures change.
    for j in ["ja", "jb"] {
      let one = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(&ad, &q, &js(&[j]));
      ::std::assert!(
        one.required.is_subset(&merged.required),
        "{j} requires something the merge dropped: {:?} vs {:?}",
        one.required,
        merged.required
      );
    }
  }

  /// Why: §4.2.3 makes recording the conflict a MUST — "the receipt MUST record
  /// the conflicting control key and both jurisdictions". A resolver that returns
  /// a bare `block` satisfies the verdict and fails the record: the caller is
  /// told it may not proceed and cannot tell anyone WHY, which is unactionable
  /// at exactly the moment someone needs to escalate. Both directions of the
  /// pair are asserted, because naming only the prohibiting side would read as
  /// correct in a one-conflict test.
  #[test]
  fn a_deontic_conflict_names_the_control_and_both_jurisdictions() {
    const REQUIRES: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"aph_mandate\"]\n\n[legally_required]\ncontrols = [\"aph_mandate\"]\n";
    const PROHIBITS: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"signal_notice\"]\n\n[legally_required]\ncontrols = [\"signal_notice\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";
    let ad = adapter(&[
      ("j_requires/recording_consent.r14n.toml", REQUIRES),
      ("j_prohibits/recording_consent.r14n.toml", PROHIBITS),
    ]);
    let q = query(&["attestation", "signal_notice", "aph_mandate"]);
    let d = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(
      &ad,
      &q,
      &js(&["j_requires", "j_prohibits"]),
    );

    ::std::assert_eq!(d.verdict, crate::Verdict::Block, "an unsatisfiable query must block");
    ::std::assert_eq!(d.conflicts.len(), 1, "expected exactly one conflict: {:?}", d.conflicts);
    ::std::assert_eq!(d.conflicts[0].control.as_str(), "aph_mandate");
    ::std::assert_eq!(d.conflicts[0].required_by, "j_requires");
    ::std::assert_eq!(d.conflicts[0].prohibited_by, "j_prohibits");
    // §4.2.3 also forbids resolving the conflict by dropping a side. The control
    // stays in `required` AND in `prohibited`, so the contradiction is visible
    // rather than tidied away into a set that looks satisfiable.
    ::std::assert!(
      d.required.iter().any(|c| c.as_str() == "aph_mandate"),
      "the conflicting control was dropped from `required` — that is the silent resolution \
       §4.2.3 forbids"
    );
    ::std::assert!(d.prohibited.iter().any(|c| c.as_str() == "aph_mandate"));
  }

  /// Why: a conflict is CROSS-jurisdiction by definition. Within one pack the
  /// data-minimization guard already subtracts `[prohibited]` from `required`,
  /// so a self-conflict is impossible — but an implementation that compared
  /// every pair INCLUDING a pack against itself would report one anyway, and
  /// would block a single perfectly satisfiable jurisdiction. That failure is
  /// invisible in the two-jurisdiction vectors.
  #[test]
  fn a_pack_that_prohibits_what_it_does_not_require_does_not_conflict_with_itself() {
    const SELF: &str = "[meta]\nstrictness = \"as_configured\"\n\n[subject.telepresence]\ncontrols = [\"attestation\", \"aph_mandate\"]\n\n[legally_required]\ncontrols = [\"attestation\"]\n\n[prohibited]\ncontrols = [\"aph_mandate\"]\n";
    let ad = adapter(&[("solo/recording_consent.r14n.toml", SELF)]);
    let q = query(&["attestation", "signal_notice", "aph_mandate"]);
    let d = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(&ad, &q, &js(&["solo"]));
    ::std::assert!(d.conflicts.is_empty(), "self-conflict reported: {:?}", d.conflicts);
    ::std::assert_eq!(d.verdict, crate::Verdict::Permit);
    ::std::assert!(!d.required.iter().any(|c| c.as_str() == "aph_mandate"), "guard did not subtract");
    ::std::assert!(d.prohibited.iter().any(|c| c.as_str() == "aph_mandate"), "prohibited not surfaced");
  }

  /// Why: invariant 1 is that a missing pack fails closed, and a MERGE must not
  /// launder that. If one jurisdiction has no pack, part of the merged set is a
  /// fail-closed guess — so `fell_back` has to stay true for the whole decision.
  /// Reporting `false` because the OTHER jurisdiction had a reviewed pack would
  /// tell a caller a reviewed pack governed all of it, which is the one thing
  /// provenance exists to prevent.
  #[test]
  fn any_jurisdiction_falling_back_taints_the_whole_merge() {
    let ad = adapter(&[("ja/recording_consent.r14n.toml", A)]);
    let q = query(&["attestation", "signal_notice", "aph_mandate"]);
    let d = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(
      &ad,
      &q,
      &js(&["ja", "absent_jurisdiction"]),
    );
    ::std::assert!(d.provenance.fell_back, "a merge over a missing pack reported fell_back = false");
    // And the fallback half contributes the WHOLE universe, per invariant 1.
    ::std::assert_eq!(d.required.len(), q.universe.len(), "fallback did not require the universe");
  }

  /// Why: an empty jurisdiction list must be the identity, because the wire
  /// boundary routes EVERY query through this door so hosts never branch. If it
  /// were not the identity, adding the merge would have silently changed the
  /// answer for every existing single-jurisdiction caller and binding.
  #[test]
  fn an_empty_jurisdiction_list_resolves_exactly_as_a_single_query() {
    let ad = adapter(&[("base/recording_consent.r14n.toml", A)]);
    let q = query(&["attestation", "signal_notice"]);
    let direct = crate::RegulatoryPolicyPort::required_controls(&ad, &q);
    let through = crate::RegulatoryPolicyPort::resolve_across_jurisdictions(&ad, &q, &[]);
    ::std::assert_eq!(direct.required, through.required);
    ::std::assert_eq!(direct.verdict, through.verdict);
    ::std::assert_eq!(direct.provenance.source, through.provenance.source);
    ::std::assert_eq!(direct.provenance.fell_back, through.provenance.fell_back);
  }

  /// Why: the merged posture is reported in provenance, and ranking an
  /// UNRECOGNISED posture as permissive would let a typo in one pack relax a
  /// merged session's reported strictness. `Other(_)` ranks with Aggressive
  /// everywhere else in this resolver; this pins that it does here too.
  #[test]
  fn an_unknown_posture_ranks_with_aggressive_not_below_minimal() {
    ::std::assert_eq!(
      crate::most_restrictive(
        crate::Strictness::Minimal,
        crate::Strictness::Other(::std::string::String::from("typo"))
      ),
      crate::Strictness::Other(::std::string::String::from("typo"))
    );
    ::std::assert_eq!(
      crate::most_restrictive(crate::Strictness::Minimal, crate::Strictness::AsConfigured),
      crate::Strictness::AsConfigured
    );
    ::std::assert_eq!(
      crate::most_restrictive(crate::Strictness::Aggressive, crate::Strictness::Minimal),
      crate::Strictness::Aggressive
    );
  }
}
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
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
      gpc_signal: false,
      ieee7012_no_recording: false,
    }
  }

  const IN_MEMORY_MINIMAL_PACK: &str = "[meta]\nstrictness = \"minimal\"\n[subject.telepresence]\ncontrols = [\"user_attestation\", \"all_party_consent\"]\n[legally_required]\ncontrols = [\"user_attestation\"]\n";

  fn in_memory(
    key: &str,
    text: &str,
  ) -> super::InMemoryRegulatoryPolicyAdapter {
    let mut packs = ::std::collections::BTreeMap::new();
    packs.insert(::std::string::String::from(key), ::std::string::String::from(text));
    super::InMemoryRegulatoryPolicyAdapter::new(packs, ::std::option::Option::None)
  }

  /// Why: the in-memory adapter is the bindings' transport (spec §4) — it must
  /// resolve a well-formed pack exactly like the fs adapter, with the map key
  /// as the provenance source and no fallback taint.
  #[test]
  fn in_memory_resolves_a_pack_with_key_as_source() {
    let adapter = in_memory(
      "minimal/recording_consent.r14n.toml",
      IN_MEMORY_MINIMAL_PACK,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&adapter, &query("minimal", "telepresence"));
    ::std::assert!(!d.provenance.fell_back);
    ::std::assert_eq!(d.provenance.source, "minimal/recording_consent.r14n.toml");
    ::std::assert!(d.requires(&super::ControlKey(::std::string::String::from("user_attestation"))));
  }

  /// Why: fail-closed (spec §4/§5) must hold identically off-filesystem — a
  /// missing key degrades to aggressive-over-universe with loud provenance,
  /// never an empty set.
  #[test]
  fn in_memory_missing_pack_fails_closed_to_universe() {
    let adapter = in_memory(
      "minimal/recording_consent.r14n.toml",
      IN_MEMORY_MINIMAL_PACK,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&adapter, &query("gdpr/eu", "telepresence"));
    ::std::assert!(d.provenance.fell_back);
    ::std::assert_eq!(d.required, universe());
    ::std::assert!(d.provenance.source.starts_with("<missing:gdpr/eu/"));
  }

  /// Why: the fs and in-memory adapters share `decide_from_pack_text`, and this
  /// pins that contract — identical pack text yields an identical decision
  /// (verdict, required set, strictness, review status) modulo the source string.
  #[test]
  fn in_memory_and_fs_adapters_agree_on_identical_pack_text() {
    let dir = ::tempfile::tempdir().expect("tempdir");
    let profile_dir = dir.path().join("minimal");
    ::std::fs::create_dir_all(&profile_dir).expect("mkdir");
    ::std::fs::write(
      profile_dir.join("recording_consent.r14n.toml"),
      IN_MEMORY_MINIMAL_PACK,
    )
    .expect("write pack");
    let fs_adapter = super::TomlRegulatoryPolicyAdapter::new(
      dir.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let mem_adapter = in_memory(
      "minimal/recording_consent.r14n.toml",
      IN_MEMORY_MINIMAL_PACK,
    );
    let q = query("minimal", "telepresence");
    let from_fs = super::RegulatoryPolicyPort::required_controls(&fs_adapter, &q);
    let from_mem = super::RegulatoryPolicyPort::required_controls(&mem_adapter, &q);
    ::std::assert_eq!(from_fs.verdict, from_mem.verdict);
    ::std::assert_eq!(from_fs.required, from_mem.required);
    ::std::assert_eq!(from_fs.provenance.strictness, from_mem.provenance.strictness);
    ::std::assert_eq!(
      from_fs.provenance.legal_review_status,
      from_mem.provenance.legal_review_status
    );
    ::std::assert_eq!(from_fs.provenance.fell_back, from_mem.provenance.fell_back);
  }

  /// Why: the legacy `<domain>.toml` probe order (backward compatibility with
  /// pre-RLPS pack trees) must survive the in-memory transport too.
  #[test]
  fn in_memory_falls_back_to_legacy_toml_key() {
    let adapter = in_memory("minimal/recording_consent.toml", IN_MEMORY_MINIMAL_PACK);
    let d = super::RegulatoryPolicyPort::required_controls(&adapter, &query("minimal", "telepresence"));
    ::std::assert!(!d.provenance.fell_back);
    ::std::assert_eq!(d.provenance.source, "minimal/recording_consent.toml");
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

  /// Why: council-audit must-fix #1 — "aggressive" is a posture, not a legal
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

  /// Why: council-audit NN15 — a profile is a `<regime>/<jurisdiction>` tag
  /// (spec §2.4), so its pack lives at a NESTED directory
  /// `<root>/<regime>/<jurisdiction>/<domain>.r14n.toml`. If `pack_path` didn't
  /// handle the `/` in the profile, the grammar-conformant example (and every
  /// future jurisdiction pack) would silently fail to load and fall back.
  #[test]
  fn nested_regime_jurisdiction_profile_resolves() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let dir = tmp.path().join("example").join("region");
    ::std::fs::create_dir_all(&dir).expect("mkdir");
    ::std::fs::write(
      dir.join("recording_consent.r14n.toml"),
      "[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = [\"user_attestation\"]\n",
    )
    .expect("write pack");
    let p = super::TomlRegulatoryPolicyAdapter::new(
      tmp.path().to_path_buf(),
      ::std::option::Option::None,
    );
    let d = super::RegulatoryPolicyPort::required_controls(&p, &query("example/region", "telepresence"));
    ::std::assert_eq!(d.required, [ck("user_attestation")].into_iter().collect());
    ::std::assert!(!d.provenance.fell_back, "the nested profile pack must actually load");
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
      attribution_source: ::std::option::Option::None,
      per_domain_posture_overrides: ::std::collections::BTreeMap::new(),
      gpc_signal: false,
      ieee7012_no_recording: false,
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

#[cfg(test)]
mod trust_root_tests {
  fn dir(json: &str) -> crate::ReviewerDirectory {
    ::serde_json::from_str(json).expect("directory")
  }

  /// A directory with one key revoked on 2026-06-01.
  fn revoked_dir() -> crate::ReviewerDirectory {
    dir(
      "{\"keys\":[{\"key_id\":\"reviewer-1\",\"jurisdiction\":\"US-CA\",\
       \"valid_from\":\"2026-01-01\",\
       \"revocation\":{\"revoked_at\":\"2026-06-01\",\"reason\":\"credential_lapsed\"}}]}",
    )
  }

  /// Why: spec §6:188 — "from `revoked_at` FORWARD". The revocation date
  /// itself is INSIDE the revoked window, so `>=` is the correct comparison
  /// and `>` would leave a one-day hole on the exact day a credential lapsed.
  /// No vector reaches this boundary, so only this test holds it.
  #[test]
  fn the_revocation_date_itself_is_already_revoked() {
    ::std::assert!(
      ::std::matches!(
        revoked_dir().key_status("reviewer-1", "2026-06-01"),
        crate::KeyStatus::Revoked { .. }
      ),
      "the boundary day must be revoked, not the last trusted day"
    );
  }

  /// Why: spec §6:190 — "decisions issued before `revoked_at` are not
  /// retroactively invalidated". Revocation is forward-dated, so a decision
  /// dated before it keeps its attestation; treating revocation as retroactive
  /// would silently re-taint every receipt ever issued under that key.
  #[test]
  fn revocation_is_not_retroactive() {
    ::std::assert!(revoked_dir().key_status("reviewer-1", "2026-05-31").is_trusted());
  }

  /// Why: spec §6:192 — "key expiry (`valid_until` passed) MUST be handled
  /// IDENTICALLY to revocation for new decisions". A directory that enforced
  /// revocation but ignored expiry would trust a lapsed credential forever.
  #[test]
  fn expiry_is_handled_identically_to_revocation() {
    let d = dir("{\"keys\":[{\"key_id\":\"k\",\"valid_until\":\"2026-03-01\"}]}");
    ::std::assert!(::std::matches!(
      d.key_status("k", "2026-03-02"),
      crate::KeyStatus::Expired { .. }
    ));
    ::std::assert!(d.key_status("k", "2026-03-01").is_trusted(), "the last valid day is valid");
  }

  /// Why: a decision dated before the key was issued was not covered by it.
  /// Backdating a decision must not borrow a credential that did not yet exist.
  #[test]
  fn a_decision_before_valid_from_is_not_yet_covered() {
    let d = dir("{\"keys\":[{\"key_id\":\"k\",\"valid_from\":\"2026-01-01\"}]}");
    ::std::assert!(::std::matches!(
      d.key_status("k", "2025-12-31"),
      crate::KeyStatus::NotYetValid { .. }
    ));
  }

  /// Why: spec §6 — a key absent from the directory is NOT trusted. The
  /// directory is an allow-list; an unlisted key must never resolve `Trusted`
  /// merely because nothing contradicted it.
  #[test]
  fn an_unlisted_key_is_unknown_never_trusted() {
    ::std::assert_eq!(revoked_dir().key_status("nobody", "2026-01-02"), crate::KeyStatus::Unknown);
    ::std::assert!(!revoked_dir().key_status("nobody", "2026-01-02").is_trusted());
  }

  /// Why: revocation MUST win over expiry when both apply — it carries the
  /// reason a receipt reader needs ("credential_lapsed"), and it is the
  /// stronger statement. Ordering is load-bearing, so it gets a test rather
  /// than relying on the order the branches happen to be written in.
  #[test]
  fn revocation_outranks_expiry_when_both_apply() {
    let d = dir(
      "{\"keys\":[{\"key_id\":\"k\",\"valid_until\":\"2026-02-01\",\
       \"revocation\":{\"revoked_at\":\"2026-03-01\",\"reason\":\"credential_lapsed\"}}]}",
    );
    ::std::assert!(
      ::std::matches!(d.key_status("k", "2026-04-01"), crate::KeyStatus::Revoked { .. }),
      "revoked must be reported over expired"
    );
  }

  /// Why: "no trust root configured" and "your key is not in my trust root"
  /// are different operational facts. Collapsing them hides a deployment that
  /// never configured a directory, which is the likeliest real-world cause of
  /// a permanently advisory receipt.
  #[test]
  fn no_directory_is_distinct_from_an_unknown_key() {
    ::std::assert_ne!(crate::KeyStatus::NoDirectory, crate::KeyStatus::Unknown);
    ::std::assert_eq!(crate::KeyStatus::NoDirectory.as_str(), "no_directory");
    ::std::assert_eq!(crate::KeyStatus::Unknown.as_str(), "unknown");
    ::std::assert!(!crate::KeyStatus::NoDirectory.is_trusted());
  }

  /// Why: `Trusted` is the ONLY status that may clear the taint. If a future
  /// arm were added and `is_trusted` kept matching loosely, a new failure mode
  /// could silently become an attested one.
  #[test]
  fn only_trusted_is_trusted() {
    for s in [
      crate::KeyStatus::Revoked {
        revoked_at: ::std::string::String::from("2026-01-01"),
        reason: ::std::string::String::from("r"),
      },
      crate::KeyStatus::Expired { valid_until: ::std::string::String::from("2026-01-01") },
      crate::KeyStatus::NotYetValid { valid_from: ::std::string::String::from("2026-01-01") },
      crate::KeyStatus::Unknown,
      crate::KeyStatus::NoDirectory,
    ] {
      ::std::assert!(!s.is_trusted(), "{s:?} must not be trusted");
    }
    ::std::assert!(crate::KeyStatus::Trusted.is_trusted());
  }

  /// Why: the default TrustRoot must be the SAFE one. A deployment that
  /// configures nothing has to keep the advisory taint, or the migration from
  /// the old hardcoded constant would silently clear taints nobody earned.
  #[test]
  fn the_default_trust_root_attests_nothing() {
    let t = crate::TrustRoot::default();
    ::std::assert!(t.directory.is_none());
    ::std::assert!(!t.signatures_verified);
  }
}
