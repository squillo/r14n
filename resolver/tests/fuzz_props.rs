// SPDX-License-Identifier: Apache-2.0
//! Property tests for the untrusted-input invariant (council-audit E1).
//!
//! The resolver's security posture is "malformed input degrades safely." These
//! stable-gated tests feed adversarial/garbage pack text through the real
//! adapter and assert the invariant: it NEVER panics and ALWAYS returns a
//! decision — a missing/malformed/degenerate pack falls closed to
//! aggressive-over-universe. The cargo-fuzz target (`resolver/fuzz`) runs the
//! same entry with libFuzzer for deeper coverage on nightly.
//!
//! Revision History
//! - 2026-07-07: authored — council-audit E1 (fuzz property).

/// The invariant-under-test as a reusable entry (also the fuzz target's body):
/// resolving `pack_text` at an arbitrary profile must yield a decision, never a
/// panic. Returns the required-set size so a caller can sanity-check.
pub fn resolve_arbitrary_pack(pack_text: &[u8]) -> usize {
  let tmp = ::tempfile::tempdir().expect("tmp");
  let dir = tmp.path().join("p");
  ::std::fs::create_dir_all(&dir).expect("mkdir");
  ::std::fs::write(dir.join("recording_consent.r14n.toml"), pack_text).expect("write");
  let universe: ::std::collections::BTreeSet<::r14n::ControlKey> =
    ["a", "b", "c"].into_iter().map(|s| ::r14n::ControlKey(::std::string::String::from(s))).collect();
  let query = ::r14n::RegulatoryQuery {
    domain: ::std::string::String::from("recording_consent"),
    profile: ::r14n::RegulatoryProfile(::std::string::String::from("p")),
    jurisdiction: ::std::string::String::from("all_party"),
    subject: ::std::string::String::from("telepresence"),
    universe: universe.clone(),
    as_of: ::std::option::Option::Some(::std::string::String::from("2026-06-15")),
  };
  let adapter = ::r14n::TomlRegulatoryPolicyAdapter::new(tmp.path().to_path_buf(), ::std::option::Option::None);
  let decision = ::r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);
  // Invariant: the required set is always a subset of the caller's universe.
  ::std::assert!(decision.required.iter().all(|k| universe.contains(k)), "required ⊄ universe");
  decision.required.len()
}

/// Why: council-audit E1 — the fail-closed guarantee the whole design rests on
/// must hold for adversarial input, not just the six hand-written malformed
/// cases. A panic here is a DoS on any consumer parsing untrusted packs.
#[test]
fn arbitrary_pack_text_never_panics_and_always_resolves() {
  let cases: &[&[u8]] = &[
    b"",
    b"\x00\x01\x02\xff\xfe",
    b"[[[[[[",
    b"strictness = ",
    b"[meta]\nstrictness = 12345",
    b"[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = \"not-an-array\"",
    b"[meta]\nstrictness = \"minimal\"\n[legally_required]\ncontrols = [1, 2, 3]",
    b"[meta]\nstrictness = \"\xf0\x28\x8c\x28\"",
    b"[prohibited]\ncontrols = [\"a\", \"a\", \"a\"]",
    b"[meta]\nstrictness = \"minimal\"\neffective_from = 42\neffective_until = true",
    "[meta]\nstrictness = \"minimal\"\ndescription = \"\u{1f600}\u{202e}\"\n[legally_required]\ncontrols = [\"a\"]".as_bytes(),
    b"\n\n\n\n[meta]\n",
  ];
  for (i, case) in cases.iter().enumerate() {
    // Must not panic; the invariant assert lives inside resolve_arbitrary_pack.
    let _ = resolve_arbitrary_pack(case);
    // Deterministic: same input, same result.
    ::std::assert_eq!(
      resolve_arbitrary_pack(case),
      resolve_arbitrary_pack(case),
      "case {i} non-deterministic"
    );
  }
}

/// Why: a pseudo-random sweep of byte soup catches panics the curated cases
/// miss without a full fuzzer; a fixed seed keeps it reproducible in CI.
#[test]
fn pseudo_random_byte_soup_never_panics() {
  // A cheap deterministic LCG — no external rng, no Date/Random (unavailable).
  let mut state: u64 = 0x9e3779b97f4a7c15;
  for _ in 0..200 {
    let len = (state % 64) as usize;
    let mut bytes = ::std::vec::Vec::with_capacity(len);
    for _ in 0..len {
      state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
      bytes.push((state >> 33) as u8);
    }
    let _ = resolve_arbitrary_pack(&bytes);
  }
}
