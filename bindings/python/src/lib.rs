// SPDX-License-Identifier: Apache-2.0
//! # r14n — Python bindings for the RLPS reference resolver
//!
//! GENERATED-from-Rust bindings (CONTRIBUTING.md bindings policy): every
//! decision comes from the same `r14n` core the Rust ecosystem links. This
//! crate is deliberately a **thin shim** — it adds `#[pyfunction]` and maps the
//! error type, nothing else. The boundary itself (JSON in, JSON out; packs as a
//! `"<profile>/<domain>.r14n.toml" → TOML text` object) lives in
//! [`r14n_core::wire`] and is tested there, once, for both this and the
//! wasm/JS binding.
//!
//! **NOT LEGAL ADVICE** — a decision is a control-prescription, never a
//! statement of law.
//!
//! Revision History
//! - 2026-08-23: authored — thin shim over `r14n::wire` (bindings policy
//!   revision 2026-08-23d).

use pyo3::prelude::{pyfunction, pymodule, Bound, PyModule, PyModuleMethods, PyResult};
use pyo3::wrap_pyfunction;

/// Map a wire-layer caller error onto `ValueError` — the Python-native answer
/// to "your input was unusable". Pack-level failures never arrive here: they
/// resolve fail-closed and come back as an ordinary decision.
fn value_error(e: ::std::string::String) -> pyo3::PyErr {
  pyo3::exceptions::PyValueError::new_err(e)
}

/// resolve(packs_json, query_json, strictness_override=None) -> str
///
/// Resolve required controls. `packs_json` maps
/// `"<profile>/<domain>.r14n.toml"` to pack TOML text; `query_json` is the
/// RegulatoryQuery; `strictness_override` is the optional global dial
/// (`"aggressive"` / `"as_configured"` / `"minimal"`). Returns the decision as
/// a JSON string.
///
/// A missing or malformed *pack* is not an error — it resolves fail-closed to
/// the caller's full universe with `provenance.fell_back = true`. `ValueError`
/// always means the caller's JSON was unusable.
#[pyfunction]
#[pyo3(signature = (packs_json, query_json, strictness_override=None))]
fn resolve(
  packs_json: &str,
  query_json: &str,
  strictness_override: ::std::option::Option<::std::string::String>,
) -> PyResult<::std::string::String> {
  r14n_core::wire::resolve_json(packs_json, query_json, strictness_override.as_deref())
    .map_err(value_error)
}

/// resolve_with_receipts(packs_json, query_json, ctx_json, strictness_override=None) -> str
///
/// Resolve AND serialize both receipt forms in one call (spec §8) — the ISO/IEC
/// TS 27560 + W3C DPV JSON-LD document and the Kantara CR v1.1 shim.
/// `ctx_json` is the caller-supplied ReceiptContext. Returns
/// `{"decision": …, "receipt_dpv27560": …, "receipt_kantara_cr_v1_1": …}`.
#[pyfunction]
#[pyo3(signature = (packs_json, query_json, ctx_json, strictness_override=None))]
fn resolve_with_receipts(
  packs_json: &str,
  query_json: &str,
  ctx_json: &str,
  strictness_override: ::std::option::Option<::std::string::String>,
) -> PyResult<::std::string::String> {
  r14n_core::wire::resolve_with_receipts_json(
    packs_json,
    query_json,
    ctx_json,
    strictness_override.as_deref(),
  )
  .map_err(value_error)
}

/// The `r14n` Python module. `__version__` and `RLPS_NS` are module attributes
/// so the common lookups need no function call.
#[pymodule]
fn r14n(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction!(resolve, m)?)?;
  m.add_function(wrap_pyfunction!(resolve_with_receipts, m)?)?;
  m.add("__version__", ::std::env!("CARGO_PKG_VERSION"))?;
  m.add("RLPS_NS", r14n_core::receipt::RLPS_NS)?;
  m.add("NOT_LEGAL_ADVICE", r14n_core::receipt::NOT_LEGAL_ADVICE)?;
  ::std::result::Result::Ok(())
}
