//! EU AI Act Art. 50 receipt example (spec §8).
//!
//! Scenario: an AI notetaker (a "twin") attends a meeting; the operator has
//! declared Germany as the applicable jurisdiction; the Squillo `aggressive`
//! baseline governs (fail-closed — the full declared control universe,
//! including `ai_disclosure`); the Art.-50 "an AI system is present" notice
//! was announced in-meeting one minute before capture. The receipt logs the
//! disclosure timestamp + method, the posture, and the full provenance.
//!
//! ⚠ NOT LEGAL ADVICE. The jurisdiction here is an illustrative resolver
//! INPUT (spec §4.1) — this example makes NO jurisdiction-pack claim; the
//! `aggressive` baseline is a posture demo, not a statement of German or EU
//! law. Timestamps are fixed so the output is reproducible
//! (`cargo run --example ai_act_50` regenerates
//! `/docs/examples/receipt-ai-act-50.json`).
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 3 AI-Act §50 example.

fn main() {
  let universe: ::std::collections::BTreeSet<::r14n::ControlKey> = [
    "attestation",
    "signal_notice",
    "indicator_mount",
    "announcement",
    "aph_mandate",
    "ai_disclosure",
  ]
  .into_iter()
  .map(|s| ::r14n::ControlKey(::std::string::String::from(s)))
  .collect();

  let query = ::r14n::RegulatoryQuery {
    domain: ::std::string::String::from("recording_consent"),
    profile: ::r14n::RegulatoryProfile(::std::string::String::from("aggressive")),
    jurisdiction: ::std::string::String::from("all_party"),
    subject: ::std::string::String::from("twin_attend"),
    universe,
  };
  let adapter = ::r14n::AggressiveDefaultPolicyAdapter;
  let decision = ::r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);

  let ctx = ::r14n::receipt::ReceiptContext {
    record_id: ::std::string::String::from("urn:rlps:receipt:example:ai-act-50"),
    issued_at: ::std::string::String::from("2026-08-02T09:00:00Z"),
    issued_at_unix: 1_785_661_200,
    language: ::std::string::String::from("de"),
    pii_principal_id: ::std::string::String::from("participant:example"),
    pii_controller: ::std::string::String::from("Example Operator GmbH"),
    policy_url: ::std::option::Option::Some(::std::string::String::from(
      "https://example.invalid/privacy",
    )),
    domain: ::std::string::String::from("recording_consent"),
    subject: ::std::string::String::from("twin_attend"),
    operations: ::std::vec![
      ::std::string::String::from("rlps:AudioRecording"),
      ::std::string::String::from("rlps:VoiceRecording"),
    ],
    jurisdictions: ::std::vec![::std::string::String::from("DE")],
    attribution_source: ::std::string::String::from("operator_declared"),
    gpc_signal: false,
    ieee7012_no_recording: false,
    sensitive: false,
    spi_cat: ::std::vec::Vec::new(),
    ai_disclosure: ::std::option::Option::Some(::r14n::receipt::AiDisclosure {
      disclosed_at: ::std::string::String::from("2026-08-02T08:59:00Z"),
      method: ::std::string::String::from("in_meeting_announcement"),
    }),
    pack_sha256: ::std::option::Option::None,
  };

  let combined = ::serde_json::json!({
    "dpv_27560": ::r14n::receipt::dpv_27560(&decision, &ctx),
    "kantara_cr_v1_1": ::r14n::receipt::kantara_cr_v1_1(&decision, &ctx),
  });
  ::std::println!(
    "{}",
    ::serde_json::to_string_pretty(&combined).expect("Value serialization cannot fail")
  );
}
