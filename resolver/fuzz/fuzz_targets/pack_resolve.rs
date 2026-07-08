// SPDX-License-Identifier: Apache-2.0
//! libFuzzer target: arbitrary bytes → a pack → resolve. Asserts the fail-closed
//! invariant (never panic, decision ⊆ universe). Run: `cargo +nightly fuzz run
//! pack_resolve`. The stable mirror is resolver/tests/fuzz_props.rs.
#![no_main]

::libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  let tmp = ::tempfile::tempdir().expect("tmp");
  let dir = tmp.path().join("p");
  ::std::fs::create_dir_all(&dir).expect("mkdir");
  ::std::fs::write(dir.join("recording_consent.r14n.toml"), data).expect("write");
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
  let adapter =
    ::r14n::TomlRegulatoryPolicyAdapter::new(tmp.path().to_path_buf(), ::std::option::Option::None);
  let decision = ::r14n::RegulatoryPolicyPort::required_controls(&adapter, &query);
  assert!(decision.required.iter().all(|k| universe.contains(k)));
});
