// SPDX-License-Identifier: Apache-2.0
//! RLPS conformance-vector runner (maintainer-notes roadmap item 4; spec §9).
//!
//! Runs every vector in `/conformance/level-*.json` whose `capabilities` are
//! all implemented by this reference resolver, and asserts drift-protected run
//! counts — adding a vector fails this suite until the counts (and, if newly
//! runnable, the capability set) are consciously updated. Vectors requiring
//! unimplemented capabilities (RFC-4647 negotiation, delta-merge,
//! most-restrictive merge, GPC escalation, per-domain posture, trust-root
//! revocation) are authored ahead of implementation and skipped LOUDLY here.
//!
//! Revision History
//! - 2026-07-06: authored with the level-1/2/3 vector files.

/// Capabilities this reference resolver implements (keep in lockstep with
/// `/conformance/README.md`'s status paragraph).
const SUPPORTED: &[&str] = &[
  "pack_parse",
  "posture_selection",
  "fail_closed_fallback",
  "fail_closed_unknown_subject",
  "posture_override",
  "data_minimization_guard",
  "temporal_envelope",
  "trust_root_taint",
  "receipt_27560",
  "receipt_kantara",
];

fn vectors_file(name: &str) -> ::serde_json::Value {
  let path = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR"))
    .join("..")
    .join("conformance")
    .join(name);
  let raw = ::std::fs::read_to_string(&path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
  ::serde_json::from_str(&raw)
    .unwrap_or_else(|e| ::std::panic!("parse {}: {e}", path.display()))
}

fn str_vec(v: &::serde_json::Value) -> ::std::vec::Vec<::std::string::String> {
  v.as_array()
    .expect("array")
    .iter()
    .map(|s| ::std::string::String::from(s.as_str().expect("string element")))
    .collect()
}

fn opt_string(v: &::serde_json::Value) -> ::std::option::Option<::std::string::String> {
  v.as_str().map(::std::string::String::from)
}

fn receipt_context(v: &::serde_json::Value) -> ::r14n::receipt::ReceiptContext {
  ::r14n::receipt::ReceiptContext {
    record_id: ::std::string::String::from(v["record_id"].as_str().expect("record_id")),
    issued_at: ::std::string::String::from(v["issued_at"].as_str().expect("issued_at")),
    issued_at_unix: v["issued_at_unix"].as_u64().expect("issued_at_unix"),
    language: ::std::string::String::from(v["language"].as_str().expect("language")),
    pii_principal_id: ::std::string::String::from(
      v["pii_principal_id"].as_str().expect("pii_principal_id"),
    ),
    pii_controller: ::std::string::String::from(
      v["pii_controller"].as_str().expect("pii_controller"),
    ),
    policy_url: opt_string(&v["policy_url"]),
    domain: ::std::string::String::from(v["domain"].as_str().expect("domain")),
    subject: ::std::string::String::from(v["subject"].as_str().expect("subject")),
    operations: str_vec(&v["operations"]),
    jurisdictions: str_vec(&v["jurisdictions"]),
    attribution_source: ::std::string::String::from(
      v["attribution_source"].as_str().expect("attribution_source"),
    ),
    gpc_signal: v["gpc_signal"].as_bool().expect("gpc_signal"),
    ieee7012_no_recording: v["ieee7012_no_recording"].as_bool().expect("ieee7012"),
    sensitive: v["sensitive"].as_bool().expect("sensitive"),
    spi_cat: str_vec(&v["spi_cat"]),
    ai_disclosure: v["ai_disclosure"].as_object().map(|d| ::r14n::receipt::AiDisclosure {
      disclosed_at: ::std::string::String::from(d["disclosed_at"].as_str().expect("disclosed_at")),
      method: ::std::string::String::from(d["method"].as_str().expect("method")),
    }),
    pack_sha256: opt_string(&v["pack_sha256"]),
  }
}

fn assert_pointers(doc: &::serde_json::Value, expectations: &::serde_json::Value, label: &str) {
  for (pointer, want) in expectations.as_object().expect("pointer map") {
    let got = doc.pointer(pointer).unwrap_or_else(|| {
      ::std::panic!("{label}: pointer {pointer} absent in {doc:#}");
    });
    ::std::assert_eq!(got, want, "{label}: mismatch at {pointer}");
  }
}

/// Run one level file; returns (ran, skipped-by-name).
fn run_suite(name: &str) -> (usize, ::std::vec::Vec<::std::string::String>) {
  let doc = vectors_file(name);
  let mut ran = 0usize;
  let mut skipped: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for vector in doc["vectors"].as_array().expect("vectors array") {
    let vname = vector["name"].as_str().expect("vector name");
    let caps = str_vec(&vector["capabilities"]);
    if !caps.iter().all(|c| SUPPORTED.contains(&c.as_str())) {
      skipped.push(::std::string::String::from(vname));
      continue;
    }
    ran += 1;

    // Materialize packs under a fresh root. Reject absolute or parent-escaping
    // keys — a hostile/sloppy vector file (this kit runs third-party vectors)
    // must never write outside the tempdir (council-audit path-traversal guard).
    let tmp = ::tempfile::tempdir().expect("tempdir");
    for (rel, content) in vector["packs"].as_object().expect("packs object") {
      let rel_path = ::std::path::Path::new(rel);
      ::std::assert!(
        rel_path.is_relative()
          && !rel_path
            .components()
            .any(|c| ::std::matches!(c, ::std::path::Component::ParentDir)),
        "{vname}: unsafe pack key {rel:?} (absolute or contains '..')"
      );
      let path = tmp.path().join(rel_path);
      if let ::std::option::Option::Some(parent) = path.parent() {
        ::std::fs::create_dir_all(parent).expect("mkdir");
      }
      ::std::fs::write(&path, content.as_str().expect("pack text")).expect("write pack");
    }

    // Build + run the query.
    let q = &vector["query"];
    let universe: ::std::collections::BTreeSet<::r14n::ControlKey> = str_vec(&q["universe"])
      .into_iter()
      .map(::r14n::ControlKey)
      .collect();
    let query = ::r14n::RegulatoryQuery {
      domain: ::std::string::String::from(q["domain"].as_str().expect("domain")),
      profile: ::r14n::RegulatoryProfile(::std::string::String::from(
        q["profile"].as_str().expect("profile"),
      )),
      jurisdiction: ::std::string::String::from(q["jurisdiction"].as_str().expect("jurisdiction")),
      subject: ::std::string::String::from(q["subject"].as_str().expect("subject")),
      universe,
      as_of: q.get("as_of").and_then(|v| v.as_str()).map(::std::string::String::from),
    };
    let override_ = q["posture_override"].as_str().map(::r14n::Strictness::parse);
    let adapter = ::r14n::TomlRegulatoryPolicyAdapter::new(tmp.path().to_path_buf(), override_);
    let decision = ::r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);

    // Decision expectations — CLOSED SET (council-audit M6): every key in
    // `expect` must be one this runner knows how to assert, else a typo'd or
    // unimplemented expectation (e.g. `verdict`) would assert nothing and pass
    // vacuously. Panic on an unknown key.
    let expect = vector["expect"].as_object().expect("expect object");
    for key in expect.keys() {
      ::std::assert!(
        ::std::matches!(
          key.as_str(),
          "required_controls" | "fell_back" | "legal_review_status" | "verdict"
        ),
        "{vname}: unknown expect key {key:?} — the runner would assert nothing"
      );
    }
    if let ::std::option::Option::Some(rc) = expect.get("required_controls") {
      let got: ::std::vec::Vec<::std::string::String> =
        decision.required.iter().map(|c| c.0.clone()).collect();
      ::std::assert_eq!(got, str_vec(rc), "{vname}: required_controls");
    }
    if let ::std::option::Option::Some(fb) = expect.get("fell_back") {
      ::std::assert_eq!(
        decision.provenance.fell_back,
        fb.as_bool().expect("fell_back bool"),
        "{vname}: fell_back"
      );
    }
    if let ::std::option::Option::Some(ls) = expect.get("legal_review_status") {
      ::std::assert_eq!(
        decision.provenance.legal_review_status.as_deref(),
        ls.as_str(),
        "{vname}: legal_review_status"
      );
    }
    if let ::std::option::Option::Some(vd) = expect.get("verdict") {
      ::std::assert_eq!(decision.verdict.as_str(), vd.as_str().expect("verdict str"), "{vname}: verdict");
    }

    // Receipt expectations (level 3). expect_receipt/expect_kantara REQUIRE a
    // receipt_context — otherwise the assertion silently never runs (M6).
    let has_ctx = vector.get("receipt_context").is_some();
    ::std::assert!(
      has_ctx || (vector.get("expect_receipt").is_none() && vector.get("expect_kantara").is_none()),
      "{vname}: expect_receipt/expect_kantara present without receipt_context"
    );
    if let ::std::option::Option::Some(rctx) = vector.get("receipt_context") {
      let ctx = receipt_context(rctx);
      if let ::std::option::Option::Some(er) = vector.get("expect_receipt") {
        let receipt = ::r14n::receipt::dpv_27560(&decision, &ctx);
        assert_pointers(&receipt, er, vname);
      }
      if let ::std::option::Option::Some(ek) = vector.get("expect_kantara") {
        let shim = ::r14n::receipt::kantara_cr_v1_1(&decision, &ctx);
        assert_pointers(&shim, ek, vname);
      }
    }
  }
  (ran, skipped)
}

/// Why: Level 1 is the floor every conforming resolver must clear (spec §9.1)
/// — the reference implementation skipping ANY level-1 vector would mean the
/// spec's own reference can't demonstrate minimal viability.
#[test]
fn conformance_level_1_minimal_viable_runs_all_vectors() {
  let (ran, skipped) = run_suite("level-1.json");
  ::std::assert_eq!(skipped.len(), 0, "level 1 must be fully runnable, skipped: {skipped:?}");
  ::std::assert_eq!(ran, 7, "level-1 vector count drifted — update consciously");
}

/// Why: level-2 vectors are authored AHEAD of the implementation (negotiation,
/// delta-merge, most-restrictive…) — pinning BOTH the run and skip counts makes
/// silent coverage loss impossible AND forces a conscious update the day a
/// capability lands (the skip count drops, this test fails, SUPPORTED grows).
#[test]
fn conformance_level_2_configured_supported_subset() {
  let (ran, skipped) = run_suite("level-2.json");
  ::std::assert_eq!(ran, 8, "level-2 supported-vector count drifted");
  ::std::assert_eq!(
    skipped.len(),
    8,
    "level-2 skip count drifted (capability implemented? update SUPPORTED + counts): {skipped:?}"
  );
}

/// Why: level 3 is the provenance story (taint + receipts) — these vectors
/// exercise the resolve→receipt pipeline END-TO-END, which the per-module unit
/// tests deliberately don't (they construct decisions by hand).
#[test]
fn conformance_level_3_comprehensive_supported_subset() {
  let (ran, skipped) = run_suite("level-3.json");
  ::std::assert_eq!(ran, 5, "level-3 supported-vector count drifted");
  ::std::assert_eq!(skipped.len(), 1, "level-3 skip count drifted: {skipped:?}");
}

/// The complete capability vocabulary the conformance suite recognizes (spec §9
/// levels). A vector tag outside this set is a typo that would be silently
/// SKIPPED forever by every harness — this vocabulary is the guard.
const KNOWN_CAPABILITIES: &[&str] = &[
  // Level 1
  "pack_parse",
  "posture_selection",
  "fail_closed_fallback",
  // Level 2
  "fail_closed_unknown_subject",
  "posture_override",
  "data_minimization_guard",
  "temporal_envelope",
  "rfc4647_negotiation",
  "delta_merge",
  "most_restrictive_merge",
  "deontic_conflict",
  "gpc_escalation",
  "ieee7012_escalation",
  "ambiguous_attribution",
  "per_domain_posture",
  // Level 3
  "trust_root_taint",
  "receipt_27560",
  "receipt_kantara",
  "trust_root_revocation",
];

/// Why: council-audit missing-item — a typo'd capability string in a new vector
/// makes it permanently skipped by every harness while the author bumps the
/// pinned skip count believing it authored-ahead. This asserts every tag in
/// every level file belongs to the known vocabulary, and that the resolver's
/// SUPPORTED set is itself a subset of that vocabulary.
#[test]
fn every_vector_capability_is_in_the_known_vocabulary() {
  for file in ["level-1.json", "level-2.json", "level-3.json"] {
    let doc = vectors_file(file);
    for vector in doc["vectors"].as_array().expect("vectors") {
      let vname = vector["name"].as_str().expect("name");
      for cap in str_vec(&vector["capabilities"]) {
        ::std::assert!(
          KNOWN_CAPABILITIES.contains(&cap.as_str()),
          "{file}:{vname} uses unknown capability {cap:?} — typo or add it to KNOWN_CAPABILITIES"
        );
      }
    }
  }
  for cap in SUPPORTED {
    ::std::assert!(
      KNOWN_CAPABILITIES.contains(cap),
      "SUPPORTED lists {cap:?} which is not in the capability vocabulary"
    );
  }
}
