// SPDX-License-Identifier: Apache-2.0
//! # r14n-wasm — WebAssembly/JS bindings for the RLPS reference resolver
//!
//! GENERATED-from-Rust bindings (CONTRIBUTING.md bindings policy): every
//! decision comes from the same `r14n` core the Rust ecosystem links. This
//! crate is deliberately a **thin shim** — it adds `#[wasm_bindgen]` and maps
//! the error type, nothing else. The boundary itself (JSON in, JSON out; packs
//! as a `"<profile>/<domain>.r14n.toml" → TOML text` object, because wasm has
//! no filesystem) lives in [`r14n_core::wire`] and is tested there, once, for
//! both this and the Python binding.
//!
//! **NOT LEGAL ADVICE** — a decision is a control-prescription, never a
//! statement of law.
//!
//! Revision History
//! - 2026-08-23: authored — thin shim over `r14n::wire` (bindings policy
//!   revision 2026-08-23d).

use wasm_bindgen::prelude::wasm_bindgen;

/// Resolve required controls. `packs_json` maps `"<profile>/<domain>.r14n.toml"`
/// to pack TOML text; `query_json` is the RegulatoryQuery; `strictness_override`
/// is the optional global dial (`"aggressive"` / `"as_configured"` /
/// `"minimal"`). Returns the decision as a JSON string.
///
/// A missing or malformed *pack* is not an error — it resolves fail-closed to
/// the caller's full universe with `provenance.fell_back = true`. A thrown error
/// always means the caller's JSON was unusable.
#[wasm_bindgen]
pub fn resolve(
  packs_json: &str,
  query_json: &str,
  strictness_override: ::std::option::Option<::std::string::String>,
) -> ::std::result::Result<::std::string::String, wasm_bindgen::JsError> {
  r14n_core::wire::resolve_json(packs_json, query_json, strictness_override.as_deref())
    .map_err(|e| wasm_bindgen::JsError::new(&e))
}

/// Resolve AND serialize both receipt forms in one call (spec §8) — the ISO/IEC
/// TS 27560 + W3C DPV JSON-LD document and the Kantara CR v1.1 shim.
/// `ctx_json` is the caller-supplied ReceiptContext. Returns
/// `{"decision":…, "receipt_dpv27560":…, "receipt_kantara_cr_v1_1":…}`.
#[wasm_bindgen]
pub fn resolve_with_receipts(
  packs_json: &str,
  query_json: &str,
  ctx_json: &str,
  strictness_override: ::std::option::Option<::std::string::String>,
) -> ::std::result::Result<::std::string::String, wasm_bindgen::JsError> {
  r14n_core::wire::resolve_with_receipts_json(
    packs_json,
    query_json,
    ctx_json,
    strictness_override.as_deref(),
  )
  .map_err(|e| wasm_bindgen::JsError::new(&e))
}

/// The binding version (tracks the core resolver version).
#[wasm_bindgen]
pub fn version() -> ::std::string::String {
  ::std::string::String::from(::std::env!("CARGO_PKG_VERSION"))
}

/// The RLPS vocabulary namespace IRI that receipts mint terms in.
#[wasm_bindgen]
pub fn rlps_ns() -> ::std::string::String {
  ::std::string::String::from(r14n_core::receipt::RLPS_NS)
}
