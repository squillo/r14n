//! # RLPS decision receipts — ISO/IEC TS 27560 + W3C DPV serialization (spec §8)
//!
//! Serializes a [`crate::ControlDecision`] as the RLPS DECISION-provenance
//! receipt: a JSON-LD consent-record document structured per the ISO/IEC
//! TS 27560:2023 record sections and typed with W3C DPV vocabulary where DPV
//! terms are established (`dpv:DataSubject`, `dpv:DataController`,
//! `dpv:hasJurisdiction`), plus a flat Kantara Consent Receipt v1.1
//! compatibility shim.
//!
//! ⚠ **NOT LEGAL ADVICE.** A receipt proves what was DECIDED and by which pack,
//! never that the decision was lawful. Decisions carry `advisory_only = true`
//! (spec §6 trust-root taint) unless provenance was cryptographically verified.
//! The reference resolver does NOT consult the reviewer-key directory, so
//! [`PROVENANCE_VERIFIED`] is `false` and **every reference-resolver receipt is
//! advisory-only** — a self-declared `legal_review.status = "approved"` string
//! does NOT clear the taint (it never proves *who* attested). `advisory_only`
//! is therefore `fell_back` OR `status != "approved"` OR `!PROVENANCE_VERIFIED`.
//!
//! Recording/receipt terms (`rlps:ControlDecisionReceipt`,
//! `rlps:RegulatedActivity`, `rlps:AudioRecording`, …) are minted in the
//! RLPS-OWNED namespace [`RLPS_NS`] so interop degrades gracefully if DPVCG
//! declines to adopt them (spec §8 / an internal design memo must-fix #6): the `@context` is
//! self-contained (`@vocab` = [`RLPS_NS`]) and coerces the term-bearing keys
//! (`operations`, `event_type`, and `id` → `@id`) so minted terms expand to
//! namespace IRIs rather than string literals. Term registry:
//! `/docs/namespace.md`.
//!
//! v0.1 fidelity note: the section layout (record / pii_principal /
//! pii_controller / processing / event) is structured after **ISO/IEC TS
//! 27560's abstract record layout** and reuses DPV terms where DPV defines them
//! (`dpv:DataSubject`/`dpv:DataController`/`dpv:hasJurisdiction`). It is NOT the
//! DPVCG "dpv-27560" JSON-LD profile (which serializes `dpv:ConsentRecord` with
//! `dct:conformsTo` / `dpv:hasDataSubject` / `dpv:hasProcess`); adopting those
//! profile properties is future work. The Kantara CR v1.1 emission is a flat
//! *compatibility shim* — it records which controls were REQUIRED, NOT that
//! consent was obtained (see [`kantara_cr_v1_1`]). Signing (Ed25519 over the
//! receipt) is the `/tools` CLI's job — this module only serializes.
//!
//! Output is deterministic: [`dpv_27560_json`] / [`kantara_cr_v1_1_json`]
//! serialize through an explicit key-sorting canonicalizer, so the byte string
//! is stable regardless of whether a consumer's build enables the
//! `serde_json/preserve_order` feature (a prerequisite for content-addressing +
//! signing that a bare BTreeMap assumption would not survive under Cargo feature
//! unification).
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 3 (27560/DPV receipt + Kantara CR v1.1
//!   shim + AI-Act §50 disclosure evidence).
//! - 2026-07-07: council-audit fidelity fixes — advisory taint no longer cleared
//!   by a self-declared status (`PROVENANCE_VERIFIED`); `@context` coerces minted
//!   terms to IRIs; dropped the false dpv-27560-profile lineage claim, the
//!   fabricated Kantara `consentType: EXPLICIT`, and the hardcoded
//!   `legal_basis_hint`; explicit sorting canonicalizer for determinism.

/// The RLPS-owned JSON-LD namespace (spec §8). Minted under squillo.com
/// control; a `w3id.org` alias is planned post-publication. Terms remain valid
/// RLPS identifiers even if DPVCG never adopts them (graceful degradation).
pub const RLPS_NS: &str = "https://rlps.squillo.com/ns#";

/// W3C Data Privacy Vocabulary namespace (only established DPV terms are used:
/// `dpv:DataSubject`, `dpv:DataController`, `dpv:hasJurisdiction`).
pub const DPV_NS: &str = "https://w3id.org/dpv#";

/// The disclaimer every receipt carries verbatim (spec §Scope).
pub const NOT_LEGAL_ADVICE: &str = "NOT LEGAL ADVICE. This receipt records a control decision and its provenance; it is not a statement of law or a compliance claim. 'aggressive' != 'compliant'.";

/// Receipt schema identifier carried in `record.schema_version`.
pub const SCHEMA_VERSION: &str = "rlps-receipt/0.1+iso27560";

/// Whether the reference resolver cryptographically verifies pack provenance
/// (signature by a directory-listed, non-revoked reviewer key). It does NOT in
/// v0.1 — directory verification is deferred to registry tooling — so this is
/// `false` and every reference-resolver decision is advisory-only (spec §6).
pub const PROVENANCE_VERIFIED: bool = false;

const KANTARA_VERSION: &str = "KI-CR-v1.1.0";

/// Recursively rebuild a JSON value with object keys in sorted order — the
/// canonical form the `_json` serializers emit, so output bytes are stable
/// regardless of the `serde_json/preserve_order` feature (see module doc).
fn canonicalize(value: &::serde_json::Value) -> ::serde_json::Value {
  match value {
    ::serde_json::Value::Object(map) => {
      let sorted: ::std::collections::BTreeMap<::std::string::String, ::serde_json::Value> =
        map.iter().map(|(k, v)| (k.clone(), canonicalize(v))).collect();
      ::serde_json::Value::Object(sorted.into_iter().collect())
    }
    ::serde_json::Value::Array(arr) => {
      ::serde_json::Value::Array(arr.iter().map(canonicalize).collect())
    }
    other => other.clone(),
  }
}

/// EU AI Act Art. 50 disclosure evidence: WHEN and HOW the "an AI system is
/// present" notice was delivered (spec §8 — the receipt MUST log disclosure
/// timestamp + method for AI-notetaker sessions in EU profiles).
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct AiDisclosure {
  /// RFC 3339 timestamp of the disclosure.
  pub disclosed_at: ::std::string::String,
  /// Delivery method (e.g. `"in_meeting_announcement"`, `"banner"`).
  pub method: ::std::string::String,
}

/// Caller-supplied receipt context — everything the receipt records that a
/// [`crate::ControlDecision`] does not itself carry (identifiers, timestamps,
/// the query's domain/subject, jurisdiction attribution per spec §4.1, and
/// escalation signals per spec §4). Timestamps are caller-supplied strings so
/// the crate stays `std`-only (no clock/formatting dependency).
#[derive(::std::clone::Clone, ::std::fmt::Debug)]
pub struct ReceiptContext {
  /// Unique receipt identifier (content-address or UUID — caller's choice).
  pub record_id: ::std::string::String,
  /// RFC 3339 issuance timestamp.
  pub issued_at: ::std::string::String,
  /// The same instant as Unix epoch seconds (Kantara `consentTimestamp`).
  pub issued_at_unix: u64,
  /// BCP-47 language tag of the notice/receipt context.
  pub language: ::std::string::String,
  /// The PII principal (data subject) identifier.
  pub pii_principal_id: ::std::string::String,
  /// The PII controller (operator) identity.
  pub pii_controller: ::std::string::String,
  /// Privacy-policy URL (Kantara `policyUrl`); `None` ⇒ a URN placeholder.
  pub policy_url: ::std::option::Option<::std::string::String>,
  /// The queried compliance domain (e.g. `"recording_consent"`).
  pub domain: ::std::string::String,
  /// The queried subject within the domain (e.g. `"twin_attend"`).
  pub subject: ::std::string::String,
  /// Processing operations, as namespace terms (e.g. `"rlps:AudioRecording"`).
  pub operations: ::std::vec::Vec<::std::string::String>,
  /// Applicable jurisdictions (ISO 3166-1/-2), per spec §4.1 an INPUT.
  pub jurisdictions: ::std::vec::Vec<::std::string::String>,
  /// How jurisdiction was attributed (e.g. `"operator_declared"`,
  /// `"participant_geo"`, `"network_transit"`) — spec §4.1 provenance.
  pub attribution_source: ::std::string::String,
  /// A `Sec-GPC: 1` signal was present (spec §4 escalation).
  pub gpc_signal: bool,
  /// An IEEE 7012 `NoRecording` term was present (spec §4 escalation).
  pub ieee7012_no_recording: bool,
  /// Kantara `sensitive` flag (special-category data involved).
  pub sensitive: bool,
  /// Kantara `spiCat` — sensitive-PII categories, when `sensitive`.
  pub spi_cat: ::std::vec::Vec<::std::string::String>,
  /// EU AI Act Art. 50 disclosure evidence, when applicable.
  pub ai_disclosure: ::std::option::Option<AiDisclosure>,
  /// SHA-256 of the pack that produced the decision (content address).
  pub pack_sha256: ::std::option::Option<::std::string::String>,
}

/// Spec §6: a decision is advisory-only unless provenance was cryptographically
/// verified (which the reference resolver never does — [`PROVENANCE_VERIFIED`]).
/// A self-declared `status = "approved"` string can never clear the taint on its
/// own; it does not prove *who* attested.
fn advisory_only(prov: &crate::PolicyProvenance) -> bool {
  prov.fell_back
    || !PROVENANCE_VERIFIED
    || !::std::matches!(
      prov.legal_review_status.as_deref(),
      ::std::option::Option::Some("approved")
    )
}

/// The decision's control keys, in stable sorted order.
fn control_strings(
  decision: &crate::ControlDecision,
) -> ::std::vec::Vec<::std::string::String> {
  decision.required.iter().map(|c| c.0.clone()).collect()
}

/// Serialize a decision as the ISO/IEC TS 27560 + W3C DPV JSON-LD receipt
/// (spec §8). Returns the JSON document as a [`::serde_json::Value`]; use
/// [`dpv_27560_json`] for the canonical (sorted-key, pretty) string form.
pub fn dpv_27560(
  decision: &crate::ControlDecision,
  ctx: &ReceiptContext,
) -> ::serde_json::Value {
  let mut events = ::std::vec![::serde_json::json!({
    "event_time": ctx.issued_at,
    "event_type": "rlps:ControlDecision",
    "event_state": "permit",
  })];
  if let ::std::option::Option::Some(d) = &ctx.ai_disclosure {
    events.push(::serde_json::json!({
      "event_time": d.disclosed_at,
      "event_type": "rlps:AiDisclosure",
      "event_state": d.method,
    }));
  }
  let mut root = ::serde_json::json!({
    "@context": {
      "@vocab": RLPS_NS,
      "rlps": RLPS_NS,
      "dpv": DPV_NS,
      // Node identity + IRI coercion so minted terms expand to namespace IRIs,
      // not string literals (council-audit NN4).
      "id": "@id",
      "operations": { "@id": "rlps:operations", "@type": "@id" },
      "event_type": { "@id": "rlps:event_type", "@type": "@id" },
    },
    "@type": "rlps:ControlDecisionReceipt",
    "record": {
      "schema_version": SCHEMA_VERSION,
      "record_id": ctx.record_id,
      "issued_at": ctx.issued_at,
      "language": ctx.language,
    },
    "pii_principal": { "@type": "dpv:DataSubject", "id": ctx.pii_principal_id },
    "pii_controller": { "@type": "dpv:DataController", "id": ctx.pii_controller },
    "processing": {
      "@type": "rlps:RegulatedActivity",
      "domain": ctx.domain,
      "subject": ctx.subject,
      "operations": ctx.operations,
    },
    "jurisdiction": {
      "dpv:hasJurisdiction": ctx.jurisdictions,
      "attribution_source": ctx.attribution_source,
    },
    "decision": {
      // The infallible reference resolver always permits (fail-closed = require
      // everything + fell_back flag); a Level-2 resolver emits "block" here.
      "verdict": "permit",
      // The posture value is never collapsed — audit evidence (spec §7 entity 7).
      "posture": decision.provenance.strictness.as_str(),
      "required_controls": control_strings(decision),
      "escalation": {
        "gpc_signal": ctx.gpc_signal,
        "ieee7012_no_recording": ctx.ieee7012_no_recording,
      },
    },
    "provenance": {
      "profile": decision.provenance.profile.0,
      "pack_source": decision.provenance.source,
      "legal_review_status": decision.provenance.legal_review_status,
      "fell_back": decision.provenance.fell_back,
      "provenance_verified": PROVENANCE_VERIFIED,
      "advisory_only": advisory_only(&decision.provenance),
    },
    "event": events,
    "disclaimer": NOT_LEGAL_ADVICE,
  });
  if let ::std::option::Option::Some(sha) = &ctx.pack_sha256 {
    root["provenance"]["pack_sha256"] = ::serde_json::Value::String(sha.clone());
  }
  if let ::std::option::Option::Some(d) = &ctx.ai_disclosure {
    // Record the disclosure FACTS (when + how). Naming a governing statute is a
    // legal conclusion the resolver must not assert (council-audit NN7) — the
    // legal basis is the pack author's / counsel's call, carried in the pack.
    root["ai_disclosure"] = ::serde_json::json!({
      "disclosed_at": d.disclosed_at,
      "method": d.method,
    });
  }
  root
}

/// [`dpv_27560`] as a deterministic pretty JSON string (sorted keys).
pub fn dpv_27560_json(
  decision: &crate::ControlDecision,
  ctx: &ReceiptContext,
) -> ::std::string::String {
  ::serde_json::to_string_pretty(&canonicalize(&dpv_27560(decision, ctx)))
    .expect("Value serialization cannot fail")
}

/// Serialize a decision as a flat Kantara Consent Receipt v1.1 (spec §8 —
/// "a flat Kantara CR v1.1 shim MAY be emitted"). RLPS-specific decision
/// provenance rides in the `rlps` extension block.
pub fn kantara_cr_v1_1(
  decision: &crate::ControlDecision,
  ctx: &ReceiptContext,
) -> ::serde_json::Value {
  let policy_url = ctx
    .policy_url
    .clone()
    .unwrap_or_else(|| ::std::string::String::from("urn:rlps:policy:unspecified"));
  let mut root = ::serde_json::json!({
    "version": KANTARA_VERSION,
    "jurisdiction": ctx.jurisdictions.join(" "),
    "consentTimestamp": ctx.issued_at_unix,
    "collectionMethod": "rlps_resolver",
    "consentReceiptID": ctx.record_id,
    "language": ctx.language,
    "piiPrincipalId": ctx.pii_principal_id,
    "piiControllers": [{ "piiController": ctx.pii_controller }],
    "policyUrl": policy_url,
    "services": [{
      "service": ctx.domain,
      "purposes": [{
        "purpose": ctx.subject,
        "purposeCategory": [ctx.domain],
        // No consentType / thirdPartyDisclosure: an RLPS receipt records which
        // controls are REQUIRED, NOT that consent was obtained. Asserting
        // consentType:"EXPLICIT" would fabricate consent evidence (council-audit
        // NN6). The record kind is stated in the top-level `notice`.
        "piiCategory": ctx.operations,
        "primaryPurpose": true,
        "termination": "per_pack_effective_until",
      }],
    }],
    "sensitive": ctx.sensitive,
    "spiCat": ctx.spi_cat,
    // Prominent at TOP LEVEL (not only in the rlps block a Kantara reader skips):
    // this is a control-prescription, not proof of collected consent.
    "notice": ::std::format!(
      "RLPS control-prescription receipt — records REQUIRED controls, NOT obtained consent. {NOT_LEGAL_ADVICE}"
    ),
    "rlps": {
      "posture": decision.provenance.strictness.as_str(),
      "required_controls": control_strings(decision),
      "pack_source": decision.provenance.source,
      "legal_review_status": decision.provenance.legal_review_status,
      "fell_back": decision.provenance.fell_back,
      "provenance_verified": PROVENANCE_VERIFIED,
      "advisory_only": advisory_only(&decision.provenance),
      "gpc_signal": ctx.gpc_signal,
      "disclaimer": NOT_LEGAL_ADVICE,
    },
  });
  if let ::std::option::Option::Some(sha) = &ctx.pack_sha256 {
    root["rlps"]["pack_sha256"] = ::serde_json::Value::String(sha.clone());
  }
  if let ::std::option::Option::Some(d) = &ctx.ai_disclosure {
    root["rlps"]["ai_disclosure"] = ::serde_json::json!({
      "disclosed_at": d.disclosed_at,
      "method": d.method,
    });
  }
  root
}

/// [`kantara_cr_v1_1`] as a deterministic pretty JSON string (sorted keys).
pub fn kantara_cr_v1_1_json(
  decision: &crate::ControlDecision,
  ctx: &ReceiptContext,
) -> ::std::string::String {
  ::serde_json::to_string_pretty(&canonicalize(&kantara_cr_v1_1(decision, ctx)))
    .expect("Value serialization cannot fail")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  fn decision(
    fell_back: bool,
    status: ::std::option::Option<&str>,
  ) -> crate::ControlDecision {
    crate::ControlDecision {
      required: ["signal_notice", "attestation"]
        .into_iter()
        .map(|s| crate::ControlKey(::std::string::String::from(s)))
        .collect(),
      provenance: crate::PolicyProvenance {
        profile: crate::RegulatoryProfile(::std::string::String::from("aggressive")),
        strictness: crate::Strictness::AsConfigured,
        source: ::std::string::String::from("packs/aggressive/recording_consent.r14n.toml"),
        legal_review_status: status.map(::std::string::String::from),
        fell_back,
      },
    }
  }

  fn ctx() -> super::ReceiptContext {
    super::ReceiptContext {
      record_id: ::std::string::String::from("rec-0001"),
      // issued_at and issued_at_unix MUST be the same instant (2026-07-06T00:00:00Z).
      issued_at: ::std::string::String::from("2026-07-06T00:00:00Z"),
      issued_at_unix: 1_783_296_000,
      language: ::std::string::String::from("en"),
      pii_principal_id: ::std::string::String::from("principal-42"),
      pii_controller: ::std::string::String::from("Example Operator LLC"),
      policy_url: ::std::option::Option::None,
      domain: ::std::string::String::from("recording_consent"),
      subject: ::std::string::String::from("twin_attend"),
      operations: ::std::vec![::std::string::String::from("rlps:AudioRecording")],
      jurisdictions: ::std::vec![::std::string::String::from("DE")],
      attribution_source: ::std::string::String::from("operator_declared"),
      gpc_signal: false,
      ieee7012_no_recording: false,
      sensitive: false,
      spi_cat: ::std::vec::Vec::new(),
      ai_disclosure: ::std::option::Option::None,
      pack_sha256: ::std::option::Option::Some(::std::string::String::from("abc123")),
    }
  }

  /// Why: spec §2.7/§8 — the receipt's whole purpose is audit evidence: the
  /// posture value (never collapsed), the pack SHA, the exact required set,
  /// and the disclaimer must all survive serialization or the receipt proves
  /// nothing.
  #[test]
  fn receipt_carries_posture_pack_sha_and_sorted_controls() {
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("approved")), &ctx());
    ::std::assert_eq!(v["decision"]["posture"], "as_configured", "posture never collapsed");
    ::std::assert_eq!(v["provenance"]["pack_sha256"], "abc123");
    // BTreeSet order: attestation < signal_notice.
    ::std::assert_eq!(
      v["decision"]["required_controls"],
      ::serde_json::json!(["attestation", "signal_notice"])
    );
    ::std::assert_eq!(v["disclaimer"], super::NOT_LEGAL_ADVICE);
  }

  /// Why: council-audit NN1 — writing `status = "approved"` is self-attestation
  /// and must NOT clear the advisory taint on its own (it never proves WHO
  /// attested). The reference resolver verifies no provenance, so EVERY receipt
  /// is advisory-only and carries provenance_verified=false — even for an
  /// "approved" pack. If this ever flips, a one-line TOML edit defeats §6.
  #[test]
  fn approved_status_alone_does_not_clear_the_advisory_taint() {
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("approved")), &ctx());
    ::std::assert_eq!(v["provenance"]["legal_review_status"], "approved", "claim preserved");
    ::std::assert_eq!(v["provenance"]["provenance_verified"], false, "resolver verifies nothing");
    ::std::assert_eq!(
      v["provenance"]["advisory_only"], true,
      "self-declared approved must stay advisory until crypto verification exists"
    );
    ::std::assert!(!super::PROVENANCE_VERIFIED, "v0.1 constant guards the above");
  }

  /// Why: council-audit NN4 — the minted terms are the whole point of the
  /// RLPS-owned namespace; the @context must coerce the term-bearing keys so
  /// they expand to IRIs, not string literals. Pins the coercion is present.
  #[test]
  fn context_coerces_minted_terms_to_iris() {
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("draft")), &ctx());
    ::std::assert_eq!(v["@context"]["id"], "@id", "bare id must map to node identity");
    ::std::assert_eq!(v["@context"]["operations"]["@type"], "@id", "operations values are IRIs");
    ::std::assert_eq!(v["@context"]["event_type"]["@type"], "@id", "event_type values are IRIs");
  }

  /// Why: spec §5/§6 — a fallen-back decision was governed by NO reviewed
  /// pack; if the receipt didn't carry the advisory taint, the fail-closed
  /// path would look like a counsel-reviewed decision to every auditor.
  #[test]
  fn fallback_taints_receipt_advisory_only() {
    let v = super::dpv_27560(&decision(true, ::std::option::Option::None), &ctx());
    ::std::assert_eq!(v["provenance"]["fell_back"], true);
    ::std::assert_eq!(v["provenance"]["advisory_only"], true, "fallback ⇒ advisory-only");
  }

  /// Why: spec §6 trust-root — "legal_review: draft" is self-attestation, and
  /// without the advisory taint on non-approved statuses, writing `approved`
  /// would be the only gate and anyone could skip it by never claiming it.
  #[test]
  fn self_attested_pack_is_advisory_only() {
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("draft")), &ctx());
    ::std::assert_eq!(v["provenance"]["fell_back"], false);
    ::std::assert_eq!(v["provenance"]["advisory_only"], true, "non-approved review ⇒ advisory");
  }

  /// Why: EU AI Act Art. 50 (spec §8) makes the disclosure LOG the compliance
  /// evidence — timestamp + method must land in both the dedicated block and
  /// the 27560 event stream, or the obligation is enforced but unprovable.
  #[test]
  fn ai_act_50_disclosure_is_logged_with_timestamp_and_method() {
    let mut c = ctx();
    c.ai_disclosure = ::std::option::Option::Some(super::AiDisclosure {
      disclosed_at: ::std::string::String::from("2026-07-06T11:59:00Z"),
      method: ::std::string::String::from("in_meeting_announcement"),
    });
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("approved")), &c);
    ::std::assert_eq!(v["ai_disclosure"]["disclosed_at"], "2026-07-06T11:59:00Z");
    ::std::assert_eq!(v["ai_disclosure"]["method"], "in_meeting_announcement");
    let events = v["event"].as_array().expect("event array");
    ::std::assert_eq!(events.len(), 2, "decision event + disclosure event");
    ::std::assert_eq!(events[1]["event_type"], "rlps:AiDisclosure");
  }

  /// Why: spec §4.1 makes jurisdiction attribution a provenance-carrying INPUT
  /// — the receipt must record both the jurisdictions and their source, or
  /// contested-attribution disputes (VoIP transit, remote employees) can't be
  /// reconstructed from the audit trail.
  #[test]
  fn jurisdiction_attribution_is_recorded() {
    let v = super::dpv_27560(&decision(false, ::std::option::Option::Some("approved")), &ctx());
    ::std::assert_eq!(v["jurisdiction"]["dpv:hasJurisdiction"], ::serde_json::json!(["DE"]));
    ::std::assert_eq!(v["jurisdiction"]["attribution_source"], "operator_declared");
  }

  /// Why: the Kantara CR v1.1 shim exists for consumers we don't control —
  /// its REQUIRED fields (version/id/epoch timestamp/service mapping) are the
  /// interop contract, and the RLPS extension block must not lose the taint.
  #[test]
  fn kantara_shim_shape_and_extension_block() {
    let v = super::kantara_cr_v1_1(&decision(false, ::std::option::Option::Some("draft")), &ctx());
    ::std::assert_eq!(v["version"], "KI-CR-v1.1.0");
    ::std::assert_eq!(v["consentReceiptID"], "rec-0001");
    ::std::assert_eq!(v["consentTimestamp"], 1_783_296_000u64);
    ::std::assert_eq!(v["policyUrl"], "urn:rlps:policy:unspecified", "None ⇒ URN placeholder");
    ::std::assert_eq!(v["services"][0]["service"], "recording_consent");
    ::std::assert_eq!(v["services"][0]["purposes"][0]["purpose"], "twin_attend");
    ::std::assert_eq!(v["rlps"]["posture"], "as_configured");
    ::std::assert_eq!(v["rlps"]["advisory_only"], true);
  }

  /// Why: council-audit NN6 — the shim must NOT fabricate consent evidence. An
  /// RLPS receipt records which controls are REQUIRED, not that consent was
  /// obtained; asserting consentType:"EXPLICIT" could be read by a Kantara-native
  /// auditor as proof of collected explicit consent. The top-level `notice` must
  /// state the record kind where such a reader will see it.
  #[test]
  fn kantara_shim_does_not_fabricate_consent() {
    let v = super::kantara_cr_v1_1(&decision(true, ::std::option::Option::None), &ctx());
    let purpose = &v["services"][0]["purposes"][0];
    ::std::assert!(purpose.get("consentType").is_none(), "must not assert consentType");
    ::std::assert!(purpose.get("thirdPartyDisclosure").is_none(), "must not assert disclosure");
    ::std::assert!(
      v["notice"].as_str().expect("notice").contains("NOT obtained consent"),
      "record-kind notice must be top-level"
    );
  }

  /// Why: escalation signals (GPC / IEEE-7012) change WHICH posture governed
  /// (spec §4) and policyUrl is a Kantara-required field — dropping either
  /// would make receipts under-document exactly the decisions regulators care
  /// about most.
  #[test]
  fn escalation_signals_and_policy_url_are_recorded() {
    let mut c = ctx();
    c.gpc_signal = true;
    c.ieee7012_no_recording = true;
    c.policy_url =
      ::std::option::Option::Some(::std::string::String::from("https://example.invalid/privacy"));
    let d = decision(false, ::std::option::Option::Some("approved"));
    let receipt = super::dpv_27560(&d, &c);
    ::std::assert_eq!(receipt["decision"]["escalation"]["gpc_signal"], true);
    ::std::assert_eq!(receipt["decision"]["escalation"]["ieee7012_no_recording"], true);
    let shim = super::kantara_cr_v1_1(&d, &c);
    ::std::assert_eq!(shim["policyUrl"], "https://example.invalid/privacy");
    ::std::assert_eq!(shim["rlps"]["gpc_signal"], true);
  }

  /// Why: deterministic bytes are the precondition for content-addressing and
  /// Ed25519 signing (module doc) — the canonicalizer must emit object keys in
  /// sorted order so the same decision hashes identically regardless of the
  /// serde_json/preserve_order feature. Assert the top-level order explicitly,
  /// not just self-equality (which passes even under insertion order).
  #[test]
  fn serialization_is_deterministic_and_key_sorted() {
    let d = decision(false, ::std::option::Option::Some("approved"));
    let c = ctx();
    let json = super::dpv_27560_json(&d, &c);
    ::std::assert_eq!(json, super::dpv_27560_json(&d, &c), "stable across calls");
    // Re-parse and confirm a nested object's keys are sorted in the byte stream.
    let ctx_pos = json.find("\"@context\"").expect("@context present");
    let type_pos = json.find("\"@type\"").expect("@type present");
    ::std::assert!(ctx_pos < type_pos, "@context sorts before @type");
  }

  /// Why: council-audit NN11 — the two receipt forms date one decision from the
  /// SAME instant (dpv emits the ISO string, Kantara the epoch); a drifting
  /// fixture (they were ~4 days apart) would ship an example that dates itself
  /// inconsistently. Pin the fixture's epoch == its ISO string.
  #[test]
  fn fixture_epoch_matches_its_iso_instant() {
    let c = ctx();
    // 2026-07-06T00:00:00Z == 1783296000 (days since epoch * 86400).
    ::std::assert_eq!(c.issued_at, "2026-07-06T00:00:00Z");
    ::std::assert_eq!(c.issued_at_unix, 1_783_296_000);
  }
}
