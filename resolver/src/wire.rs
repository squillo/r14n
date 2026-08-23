// SPDX-License-Identifier: Apache-2.0
//! # The JSON wire layer — one boundary, every host
//!
//! Bindings are GENERATED from this crate (CONTRIBUTING.md bindings policy), and
//! this module is what they are generated *over*: JSON text in, JSON text out.
//! The wasm/JS and Python shims add an FFI attribute and nothing else, so there
//! is exactly one place where the boundary shape is defined and exactly one
//! place it is tested. A JSON string is also the same artifact in every host —
//! it diffs in a test failure, and it cannot half-deserialize.
//!
//! Hosts here have no filesystem, so packs arrive as a JSON object mapping
//! `"<profile>/<domain>.r14n.toml"` to the pack's TOML text and resolve through
//! [`crate::InMemoryRegulatoryPolicyAdapter`].
//!
//! **NOT LEGAL ADVICE** — a decision is a control-prescription, never a
//! statement of law.
//!
//! Revision History
//! - 2026-08-23: authored — extracted from the wasm binding so the wasm and
//!   Python shims share one tested boundary (bindings policy revision
//!   2026-08-23d).

/// A caller-facing boundary error. Distinct from a *pack* failure: a pack that
/// is missing or malformed resolves fail-closed (spec §4/§5) and is NOT an
/// error, whereas junk from the caller is — a host must be able to tell "I sent
/// bad input" from "no reviewed pack governed this decision".
pub type WireError = ::std::string::String;

/// The wire form of [`crate::RegulatoryQuery`]. `universe` is a JSON array
/// (JSON has no sets); order is irrelevant and duplicates collapse.
#[derive(::serde::Deserialize)]
struct WireQuery {
  domain: ::std::string::String,
  profile: ::std::string::String,
  jurisdiction: ::std::string::String,
  subject: ::std::string::String,
  universe: ::std::vec::Vec<::std::string::String>,
  #[serde(default)]
  as_of: ::std::option::Option<::std::string::String>,
}

impl WireQuery {
  fn into_core(self) -> crate::RegulatoryQuery {
    crate::RegulatoryQuery {
      domain: self.domain,
      profile: crate::RegulatoryProfile(self.profile),
      jurisdiction: self.jurisdiction,
      subject: self.subject,
      universe: self.universe.into_iter().map(crate::ControlKey).collect(),
      as_of: self.as_of,
    }
  }
}

/// The wire form of [`crate::PolicyProvenance`].
#[derive(::serde::Serialize)]
struct WireProvenance {
  profile: ::std::string::String,
  strictness: ::std::string::String,
  source: ::std::string::String,
  legal_review_status: ::std::option::Option<::std::string::String>,
  last_reviewed_against_guidance: ::std::option::Option<::std::string::String>,
  fell_back: bool,
}

/// The wire form of [`crate::ControlDecision`]. `required` is emitted in
/// BTreeSet order, so the JSON is byte-stable across hosts and runs.
#[derive(::serde::Serialize)]
struct WireDecision {
  verdict: ::std::string::String,
  required: ::std::vec::Vec<::std::string::String>,
  provenance: WireProvenance,
  disclaimer: &'static str,
}

impl WireDecision {
  fn from_core(d: &crate::ControlDecision) -> Self {
    Self {
      verdict: ::std::string::String::from(d.verdict.as_str()),
      required: d.required.iter().map(|c| ::std::string::String::from(c.as_str())).collect(),
      provenance: WireProvenance {
        profile: d.provenance.profile.0.clone(),
        strictness: ::std::string::String::from(d.provenance.strictness.as_str()),
        source: d.provenance.source.clone(),
        legal_review_status: d.provenance.legal_review_status.clone(),
        last_reviewed_against_guidance: d.provenance.last_reviewed_against_guidance.clone(),
        fell_back: d.provenance.fell_back,
      },
      disclaimer: crate::receipt::NOT_LEGAL_ADVICE,
    }
  }
}

/// The wire form of [`crate::receipt::AiDisclosure`].
#[derive(::serde::Deserialize)]
struct WireAiDisclosure {
  disclosed_at: ::std::string::String,
  method: ::std::string::String,
}

/// The wire form of [`crate::receipt::ReceiptContext`]. Escalation flags and
/// list fields default, so a minimal caller writes a minimal object.
#[derive(::serde::Deserialize)]
struct WireReceiptContext {
  record_id: ::std::string::String,
  issued_at: ::std::string::String,
  issued_at_unix: u64,
  language: ::std::string::String,
  pii_principal_id: ::std::string::String,
  pii_controller: ::std::string::String,
  #[serde(default)]
  policy_url: ::std::option::Option<::std::string::String>,
  domain: ::std::string::String,
  subject: ::std::string::String,
  #[serde(default)]
  operations: ::std::vec::Vec<::std::string::String>,
  #[serde(default)]
  jurisdictions: ::std::vec::Vec<::std::string::String>,
  attribution_source: ::std::string::String,
  #[serde(default)]
  gpc_signal: bool,
  #[serde(default)]
  ieee7012_no_recording: bool,
  #[serde(default)]
  sensitive: bool,
  #[serde(default)]
  spi_cat: ::std::vec::Vec<::std::string::String>,
  #[serde(default)]
  ai_disclosure: ::std::option::Option<WireAiDisclosure>,
  #[serde(default)]
  pack_sha256: ::std::option::Option<::std::string::String>,
}

impl WireReceiptContext {
  fn into_core(self) -> crate::receipt::ReceiptContext {
    crate::receipt::ReceiptContext {
      record_id: self.record_id,
      issued_at: self.issued_at,
      issued_at_unix: self.issued_at_unix,
      language: self.language,
      pii_principal_id: self.pii_principal_id,
      pii_controller: self.pii_controller,
      policy_url: self.policy_url,
      domain: self.domain,
      subject: self.subject,
      operations: self.operations,
      jurisdictions: self.jurisdictions,
      attribution_source: self.attribution_source,
      gpc_signal: self.gpc_signal,
      ieee7012_no_recording: self.ieee7012_no_recording,
      sensitive: self.sensitive,
      spi_cat: self.spi_cat,
      ai_disclosure: self.ai_disclosure.map(|a| crate::receipt::AiDisclosure {
        disclosed_at: a.disclosed_at,
        method: a.method,
      }),
      pack_sha256: self.pack_sha256,
    }
  }
}

/// Parse the packs object + query and run the core resolver.
fn decide(
  packs_json: &str,
  query_json: &str,
  strictness_override: ::std::option::Option<&str>,
) -> ::std::result::Result<crate::ControlDecision, WireError> {
  let packs: ::std::collections::BTreeMap<::std::string::String, ::std::string::String> =
    ::serde_json::from_str(packs_json)
      .map_err(|e| ::std::format!("packs_json is not an object of path->toml strings: {e}"))?;
  let query: WireQuery = ::serde_json::from_str(query_json)
    .map_err(|e| ::std::format!("query_json is not a valid RegulatoryQuery: {e}"))?;
  let dial = strictness_override.map(crate::Strictness::parse);
  let adapter = crate::InMemoryRegulatoryPolicyAdapter::new(packs, dial);
  ::std::result::Result::Ok(crate::RegulatoryPolicyPort::required_controls(
    &adapter,
    &query.into_core(),
  ))
}

/// Resolve required controls, JSON in and JSON out.
///
/// `packs_json` maps `"<profile>/<domain>.r14n.toml"` to pack TOML text;
/// `query_json` is the [`crate::RegulatoryQuery`]; `strictness_override` is the
/// optional global dial (`"aggressive"` / `"as_configured"` / `"minimal"`).
///
/// A missing or malformed *pack* is not an error — it resolves fail-closed to
/// the caller's full universe with `provenance.fell_back = true`. An `Err` here
/// always means the *caller's* JSON was unusable.
pub fn resolve_json(
  packs_json: &str,
  query_json: &str,
  strictness_override: ::std::option::Option<&str>,
) -> ::std::result::Result<::std::string::String, WireError> {
  let decision = decide(packs_json, query_json, strictness_override)?;
  ::serde_json::to_string(&WireDecision::from_core(&decision))
    .map_err(|e| ::std::format!("decision serialization failed: {e}"))
}

/// Resolve and serialize both receipt forms in one call (spec §8), returning
/// `{"decision":…, "receipt_dpv27560":…, "receipt_kantara_cr_v1_1":…}`.
/// `ctx_json` is the caller-supplied [`crate::receipt::ReceiptContext`].
pub fn resolve_with_receipts_json(
  packs_json: &str,
  query_json: &str,
  ctx_json: &str,
  strictness_override: ::std::option::Option<&str>,
) -> ::std::result::Result<::std::string::String, WireError> {
  let decision = decide(packs_json, query_json, strictness_override)?;
  let ctx: WireReceiptContext = ::serde_json::from_str(ctx_json)
    .map_err(|e| ::std::format!("ctx_json is not a valid ReceiptContext: {e}"))?;
  let ctx = ctx.into_core();
  let out = ::serde_json::json!({
    "decision": ::serde_json::to_value(WireDecision::from_core(&decision))
      .map_err(|e| ::std::format!("decision serialization failed: {e}"))?,
    "receipt_dpv27560": crate::receipt::dpv_27560(&decision, &ctx),
    "receipt_kantara_cr_v1_1": crate::receipt::kantara_cr_v1_1(&decision, &ctx),
  });
  ::serde_json::to_string(&out).map_err(|e| ::std::format!("receipt serialization failed: {e}"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  const PACK: &str = "[meta]\nstrictness = \"minimal\"\n[subject.telepresence]\ncontrols = [\"user_attestation\"]\n[legally_required]\ncontrols = [\"user_attestation\"]\n";

  fn packs_json() -> ::std::string::String {
    ::serde_json::json!({ "minimal/recording_consent.r14n.toml": PACK }).to_string()
  }

  fn query_json() -> ::std::string::String {
    ::serde_json::json!({
      "domain": "recording_consent",
      "profile": "minimal",
      "jurisdiction": "all_party",
      "subject": "telepresence",
      "universe": ["user_attestation", "signal_notice"],
    })
    .to_string()
  }

  fn ctx_json() -> ::std::string::String {
    ::serde_json::json!({
      "record_id": "urn:test:1",
      "issued_at": "2026-08-23T12:00:00Z",
      "issued_at_unix": 1_787_832_000u64,
      "language": "en",
      "pii_principal_id": "user-1",
      "pii_controller": "Example Operator",
      "domain": "recording_consent",
      "subject": "telepresence",
      "attribution_source": "operator_declared",
    })
    .to_string()
  }

  /// Why: this is the contract every binding inherits — a happy-path resolve
  /// must surface the verdict, the sorted required set, the pack key as
  /// provenance source, and the standing disclaimer.
  #[test]
  fn resolve_json_round_trips_the_core_decision() {
    let out = super::resolve_json(&packs_json(), &query_json(), ::std::option::Option::None)
      .expect("resolve");
    let v: ::serde_json::Value = ::serde_json::from_str(&out).expect("valid JSON out");
    ::std::assert_eq!(v["verdict"], "permit");
    ::std::assert_eq!(v["required"], ::serde_json::json!(["user_attestation"]));
    ::std::assert_eq!(v["provenance"]["source"], "minimal/recording_consent.r14n.toml");
    ::std::assert_eq!(v["provenance"]["fell_back"], false);
    ::std::assert!(
      v["disclaimer"].as_str().expect("disclaimer").starts_with("NOT LEGAL ADVICE")
    );
  }

  /// Why: fail-closed (spec §4/§5) must survive the JSON boundary — a missing
  /// pack demands the caller's whole universe and says so, rather than erroring
  /// or resolving empty.
  #[test]
  fn a_missing_pack_fails_closed_over_the_boundary() {
    let out = super::resolve_json("{}", &query_json(), ::std::option::Option::None)
      .expect("resolve");
    let v: ::serde_json::Value = ::serde_json::from_str(&out).expect("valid JSON out");
    ::std::assert_eq!(v["provenance"]["fell_back"], true);
    ::std::assert_eq!(
      v["required"],
      ::serde_json::json!(["signal_notice", "user_attestation"])
    );
  }

  /// Why: a caller's bad JSON must be a loud error, never a silent fallback —
  /// the fallback is reserved for pack failures, and conflating the two would
  /// hide a bug behind a policy decision.
  #[test]
  fn malformed_caller_json_errors_and_names_the_field() {
    let e = super::resolve_json(&packs_json(), "{not json", ::std::option::Option::None)
      .expect_err("must error");
    ::std::assert!(e.contains("query_json"), "names the offending field: {e}");
    let e = super::resolve_json("[]", &query_json(), ::std::option::Option::None)
      .expect_err("must error");
    ::std::assert!(e.contains("packs_json"), "names the offending field: {e}");
  }

  /// Why: the strictness dial is the caller's system-wide override; it must
  /// reach the core and be reported in provenance.
  #[test]
  fn the_strictness_dial_reaches_the_core() {
    let out = super::resolve_json(
      &packs_json(),
      &query_json(),
      ::std::option::Option::Some("aggressive"),
    )
    .expect("resolve");
    let v: ::serde_json::Value = ::serde_json::from_str(&out).expect("valid JSON out");
    ::std::assert_eq!(v["provenance"]["strictness"], "aggressive");
    ::std::assert_eq!(
      v["required"],
      ::serde_json::json!(["signal_notice", "user_attestation"])
    );
  }

  /// Why: receipts are the auditable artifact (spec §8). Both forms must
  /// serialize, read the verdict from the decision, and stay advisory-only —
  /// the reference resolver verifies no provenance (spec §6).
  #[test]
  fn resolve_with_receipts_json_emits_both_forms_advisory_only() {
    let out = super::resolve_with_receipts_json(
      &packs_json(),
      &query_json(),
      &ctx_json(),
      ::std::option::Option::None,
    )
    .expect("resolve_with_receipts");
    let v: ::serde_json::Value = ::serde_json::from_str(&out).expect("valid JSON out");
    ::std::assert_eq!(v["decision"]["verdict"], "permit");
    ::std::assert_eq!(v["receipt_dpv27560"]["decision"]["verdict"], "permit");
    ::std::assert_eq!(v["receipt_dpv27560"]["provenance"]["advisory_only"], true);
    ::std::assert!(v["receipt_kantara_cr_v1_1"].is_object());
  }

  /// Why: a bad receipt context must be named as such, not blamed on the query
  /// — three JSON inputs mean three distinct error surfaces.
  #[test]
  fn a_malformed_receipt_context_names_ctx_json() {
    let e = super::resolve_with_receipts_json(
      &packs_json(),
      &query_json(),
      "{}",
      ::std::option::Option::None,
    )
    .expect_err("must error");
    ::std::assert!(e.contains("ctx_json"), "names the offending field: {e}");
  }

  /// Why: the emitted JSON must be byte-stable — bindings and downstream
  /// receipt consumers compare these strings, so a nondeterministic set order
  /// would look like a behavior change.
  #[test]
  fn output_is_byte_stable_across_runs() {
    let a = super::resolve_json(&packs_json(), &query_json(), ::std::option::Option::None)
      .expect("resolve");
    let b = super::resolve_json(&packs_json(), &query_json(), ::std::option::Option::None)
      .expect("resolve");
    ::std::assert_eq!(a, b);
  }
}
