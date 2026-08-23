// SPDX-License-Identifier: Apache-2.0
//! Wasm32-only smoke harness: proves the exported bindgen surface works inside
//! an actual wasm runtime (wasm-pack test --node), not just as native Rust.
//! On native targets this whole file compiles to an empty test binary — the
//! native gate is the module tests in src/lib.rs.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::wasm_bindgen_test;

/// Why: the exported resolve() must produce the same fail-closed decision the
/// core makes, through the real wasm ABI — this is the byte-crosses-the-
/// boundary proof the native tests cannot give.
#[wasm_bindgen_test]
fn resolve_fails_closed_over_the_wasm_abi() {
  let query = r#"{"domain":"recording_consent","profile":"minimal","jurisdiction":"all_party","subject":"telepresence","universe":["user_attestation","signal_notice"]}"#;
  let out = r14n_wasm::resolve("{}", query, ::std::option::Option::None).expect("resolve");
  ::std::assert!(out.contains("\"fell_back\":true"));
  ::std::assert!(out.contains("user_attestation"));
}

/// Why: version() and rlps_ns() are the cheapest cross-boundary sanity checks
/// a JS consumer will run first; pin they answer.
#[wasm_bindgen_test]
fn version_and_namespace_answer() {
  ::std::assert!(!r14n_wasm::version().is_empty());
  ::std::assert!(r14n_wasm::rlps_ns().starts_with("https://r14n.squillo.com/ns#"));
}
