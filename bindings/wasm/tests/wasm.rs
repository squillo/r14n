// SPDX-License-Identifier: Apache-2.0
//! Wasm32-only boundary suite: proves the exported bindgen surface works inside
//! an actual wasm runtime (wasm-pack test --node), not just as native Rust.
//! These tests call the exported functions exactly as the generated JS glue
//! does — JSON strings in, a JSON string or a thrown error out — against the
//! packs this repository really ships, so a drift between the core, the ABI
//! and the published packs shows up here and not in a consumer's install.
//! On native targets this whole file compiles to an empty test binary — the
//! native gate is the module tests in src/lib.rs and the wire tests in the
//! resolver crate.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::wasm_bindgen_test;

/// The ABI's fallible return shape, named once so the helpers below read as
/// what they are.
type Abi = ::std::result::Result<::std::string::String, ::wasm_bindgen::JsError>;

/// The shipped `minimal` recording_consent pack, byte for byte, under the key
/// the in-memory adapter looks it up by (`<profile>/<domain>.r14n.toml`).
const MINIMAL_KEY: &str = "minimal/recording_consent.r14n.toml";
const MINIMAL_PACK: &str =
  ::std::include_str!("../../../packs/minimal/recording_consent.r14n.toml");

/// The caller's declared universe. Kept in sorted order because `required` is
/// emitted in set order, so a fail-closed or aggressive decision equals this
/// list exactly.
const UNIVERSE: [&str; 2] = ["attestation", "signal_notice"];

/// The `packs_json` argument: an object of pack key to TOML text. Serialized by
/// a JSON encoder so the pack's quotes and newlines are escaped by machine.
fn packs() -> ::std::string::String {
  ::serde_json::to_string(&::std::collections::BTreeMap::from([(MINIMAL_KEY, MINIMAL_PACK)]))
    .expect("a map of strings serializes")
}

/// The `query_json` argument every test shares.
fn query() -> ::std::string::String {
  ::serde_json::json!({
    "domain": "recording_consent",
    "profile": "minimal",
    "jurisdiction": "all_party",
    "subject": "telepresence",
    "universe": UNIVERSE,
  })
  .to_string()
}

/// Resolve `query()` over the ABI and parse the decision. Panics (failing the
/// test) if well-formed input comes back as an error.
fn decision(packs: &str, dial: ::std::option::Option<&str>) -> ::serde_json::Value {
  let out = r14n_wasm::resolve(packs, &query(), dial.map(::std::string::String::from))
    .expect("well-formed caller JSON must resolve, never throw");
  ::serde_json::from_str(&out).expect("the ABI returns valid JSON")
}

/// The text of the error an ABI call threw, as the debug form of the thrown
/// value (which carries the JS `Error` message). Panics if the call did NOT
/// throw — that is the failure these tests exist to catch.
fn thrown(out: Abi) -> ::std::string::String {
  ::std::format!("{:?}", out.expect_err("caller junk must throw, not resolve"))
}

/// Why: fail-closed (spec §4/§5) is the property the whole standard rests on, and
/// it must hold across the real wasm ABI — a missing pack demands the caller's
/// ENTIRE universe, flagged `fell_back`, and neither throws nor resolves empty.
/// Asserting the exact `required` list (not a substring) is what rules out a
/// partial fallback that names only one of the declared controls.
#[wasm_bindgen_test]
fn resolve_fails_closed_over_the_wasm_abi() {
  let d = decision("{}", ::std::option::Option::None);
  ::std::assert_eq!(d["provenance"]["fell_back"], true);
  ::std::assert_eq!(d["required"], ::serde_json::json!(UNIVERSE));
}

/// Why: the happy path across the real wasm ABI against the real shipped pack —
/// verdict, the sorted required set, the map key as provenance source, and the
/// standing disclaimer. `required` is the pack's floor ONLY (`attestation`):
/// under `minimal` the universe's other control, which a subject table names,
/// must not leak in, so a resolver that ignored the posture would fail here.
#[wasm_bindgen_test]
fn resolve_returns_the_core_decision_over_the_wasm_abi() {
  let d = decision(&packs(), ::std::option::Option::None);
  ::std::assert_eq!(d["verdict"], "permit");
  ::std::assert_eq!(d["required"], ::serde_json::json!(["attestation"]));
  ::std::assert_eq!(d["provenance"]["source"], MINIMAL_KEY);
  ::std::assert_eq!(d["provenance"]["fell_back"], false);
  ::std::assert!(d["disclaimer"].as_str().expect("disclaimer").starts_with("NOT LEGAL ADVICE"));
}

/// Why: a caller's unusable JSON must reach a host as a thrown error naming the
/// offending argument — distinct from a pack-level fallback, so a consumer can
/// tell "I sent junk" from "no reviewed pack governed this". `expect_err` is the
/// load-bearing line: it fails if any argument resolved to a decision, and in
/// particular to a permissive one. A panic would abort the wasm instance and
/// take the whole suite down, so completing is also the no-panic proof.
#[wasm_bindgen_test]
fn malformed_caller_json_throws_rather_than_resolving() {
  ::std::assert!(thrown(r14n_wasm::resolve(&packs(), "{not json", ::std::option::Option::None))
    .contains("query_json"));
  ::std::assert!(thrown(r14n_wasm::resolve("[]", &query(), ::std::option::Option::None))
    .contains("packs_json"));
  ::std::assert!(thrown(r14n_wasm::resolve_with_receipts(
    &packs(),
    &query(),
    "{not json",
    ::std::option::Option::None
  ))
  .contains("ctx_json"));
}

/// Why: the strictness dial is the caller's system-wide override and must reach
/// the core over the boundary. The first assertion is the control: without it a
/// pack that already resolved as `aggressive` would make the dial assertions
/// pass vacuously, proving nothing about the override.
#[wasm_bindgen_test]
fn the_strictness_dial_reaches_the_core_over_the_wasm_abi() {
  let baseline = decision(&packs(), ::std::option::Option::None);
  ::std::assert_eq!(baseline["provenance"]["strictness"], "minimal");
  let dialed = decision(&packs(), ::std::option::Option::Some("aggressive"));
  ::std::assert_eq!(dialed["provenance"]["strictness"], "aggressive");
  ::std::assert_eq!(dialed["required"], ::serde_json::json!(UNIVERSE));
}

/// Why: receipts are the auditable artifact (spec §8). Both forms must serialize
/// through one call and BOTH must stay advisory-only, since the reference
/// resolver verifies no provenance and a shipped pack awaiting counsel sign-off
/// is not an attestation. Checking the Kantara form's flag, not just its
/// presence, is what stops the shim from silently dropping the taint.
#[wasm_bindgen_test]
fn resolve_with_receipts_emits_both_forms_advisory_only() {
  let ctx = ::serde_json::json!({
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
  .to_string();
  let out = r14n_wasm::resolve_with_receipts(&packs(), &query(), &ctx, ::std::option::Option::None)
    .expect("well-formed caller JSON must resolve, never throw");
  let v: ::serde_json::Value = ::serde_json::from_str(&out).expect("the ABI returns valid JSON");
  ::std::assert_eq!(v["decision"]["verdict"], "permit");
  ::std::assert_eq!(v["receipt_dpv27560"]["provenance"]["advisory_only"], true);
  ::std::assert!(v["receipt_kantara_cr_v1_1"].is_object());
  ::std::assert_eq!(v["receipt_kantara_cr_v1_1"]["rlps"]["advisory_only"], true);
}

/// Why: version() and rlps_ns() are the cheapest cross-boundary sanity checks
/// a JS consumer will run first. The namespace is pinned to the exact published
/// IRI, not a prefix: receipts mint their vocabulary terms under it, so a
/// stray suffix would send every minted term to a host that does not serve it.
#[wasm_bindgen_test]
fn version_and_namespace_answer() {
  ::std::assert!(!r14n_wasm::version().is_empty());
  ::std::assert_eq!(r14n_wasm::rlps_ns(), "https://r14n.squillo.com/ns#");
}
