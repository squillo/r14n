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

/// The whole RLPS decision pipeline over an already-loaded pack text — shared
/// verbatim by the filesystem adapter and the in-memory adapter so the two can
/// never drift (one behavior, two transports). `source` is the provenance
/// string surfaced in the decision (a path for the fs adapter, a map key for
/// the in-memory one).
fn decide_from_pack_text(
  raw: &str,
  source: &str,
  strictness_override: &::std::option::Option<Strictness>,
  query: &RegulatoryQuery,
) -> ControlDecision {
  let pack: PolicyPackToml = match ::toml::from_str(raw) {
    ::std::result::Result::Ok(p) => p,
    ::std::result::Result::Err(_) => {
      return aggressive_over_universe(query, ::std::format!("<malformed:{source}>"), true);
    }
  };

  // The global dial wins; else the pack's declared strictness; else Aggressive.
  let strictness = strictness_override.clone().unwrap_or_else(|| {
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

  let legal_review_status = pack.meta.legal_review.and_then(|r| r.status);
  ControlDecision {
    verdict: Verdict::Permit,
    required,
    prohibited,
    conflicts: ::std::vec::Vec::new(),
    provenance: PolicyProvenance {
      profile: query.profile.clone(),
      strictness,
      source: ::std::string::String::from(source),
      legal_review_status,
      last_reviewed_against_guidance: pack.meta.last_reviewed_against_guidance,
      fell_back: false,
    },
  }
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
    decide_from_pack_text(
      &raw,
      &path.display().to_string(),
      &self.strictness_override,
      query,
    )
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
}

impl InMemoryRegulatoryPolicyAdapter {
  /// Construct from `<profile>/<domain>.r14n.toml → TOML text` entries and an
  /// optional global strictness override (the same DIAL as the fs adapter).
  pub fn new(
    packs: ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
    strictness_override: ::std::option::Option<Strictness>,
  ) -> Self {
    Self { packs, strictness_override }
  }

  /// Prefer the RLPS pack key `<profile>/<domain>.r14n.toml`; fall back to the
  /// legacy `<profile>/<domain>.toml` — the same probe order as the fs adapter.
  fn pack_entry(
    &self,
    query: &RegulatoryQuery,
  ) -> ::std::option::Option<(::std::string::String, &::std::string::String)> {
    let r14n = ::std::format!("{}/{}.r14n.toml", query.profile.0, query.domain);
    if let ::std::option::Option::Some(text) = self.packs.get(&r14n) {
      return ::std::option::Option::Some((r14n, text));
    }
    let legacy = ::std::format!("{}/{}.toml", query.profile.0, query.domain);
    self.packs.get(&legacy).map(|text| (legacy, text))
  }
}

impl __private_seal::RegulatoryPolicyPortSeal for InMemoryRegulatoryPolicyAdapter {}

impl RegulatoryPolicyPort for InMemoryRegulatoryPolicyAdapter {
  fn required_controls(&self, query: &RegulatoryQuery) -> ControlDecision {
    match self.pack_entry(query) {
      ::std::option::Option::Some((source, raw)) => {
        decide_from_pack_text(raw, &source, &self.strictness_override, query)
      }
      ::std::option::Option::None => aggressive_over_universe(
        query,
        ::std::format!("<missing:{}/{}.r14n.toml>", query.profile.0, query.domain),
        true,
      ),
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────



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
