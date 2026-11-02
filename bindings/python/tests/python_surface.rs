// SPDX-License-Identifier: Apache-2.0
//! The exported Python surface of `r14n`, exercised through a real CPython interpreter.
//!
//! The module object is built in-process by the shim's own initialiser and every call goes through
//! CPython's object model (argument parsing, the `None` default of the strictness dial, exception
//! types). Nothing here is Python source: the calls and assertions are Rust. The boundary's logic
//! is pinned once in the core crate (`r14n::wire`); what these prove is that the shim exposes it
//! under the names, defaults and exception types a Python consumer sees.
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-11-02: authored -- replaces the hand-written Python tests, so the binding is tested in
//!   the language it is generated from.

use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::{json, Value};

const PACK: &str = "[meta]\nstrictness = \"minimal\"\n[subject.telepresence]\ncontrols = [\"user_attestation\"]\n[legally_required]\ncontrols = [\"user_attestation\"]\n";
const PACK_KEY: &str = "minimal/recording_consent.r14n.toml";

fn packs() -> String {
  json!({ PACK_KEY: PACK }).to_string()
}

fn query() -> String {
  json!({
    "domain": "recording_consent",
    "profile": "minimal",
    "jurisdiction": "all_party",
    "subject": "telepresence",
    "universe": ["user_attestation", "signal_notice"],
  })
  .to_string()
}

fn ctx() -> String {
  json!({
    "record_id": "urn:test:1",
    "issued_at": "2026-08-23T12:00:00Z",
    "issued_at_unix": 1787832000,
    "language": "en",
    "pii_principal_id": "user-1",
    "pii_controller": "Example Operator",
    "domain": "recording_consent",
    "subject": "telepresence",
    "attribution_source": "operator_declared",
  })
  .to_string()
}

/// The `r14n` module as Python sees it, built by the shim's own `#[pymodule]` initialiser.
fn module(py: Python<'_>) -> Bound<'_, PyModule> {
  let m = PyModule::new(py, "r14n").expect("create module object");
  ::r14n::r14n(&m).expect("shim initialiser populates the module");
  m
}

/// `module.<name>(*args)` as a JSON value, or the Python exception it raised.
fn call_json(py: Python<'_>, name: &str, args: &[&str]) -> Result<Value, PyErr> {
  let args = pyo3::types::PyTuple::new(py, args)?;
  let text: String = module(py).getattr(name)?.call1(args)?.extract()?;
  Ok(serde_json::from_str(&text).expect("the shim returns JSON text"))
}

/// Why: the module attributes are the cheapest sanity checks a consumer runs first, and `RLPS_NS`
/// must match the live vocabulary host.
#[test]
fn module_attributes() {
  Python::attach(|py| {
    let m = module(py);
    let version: String = m.getattr("__version__").unwrap().extract().unwrap();
    let ns: String = m.getattr("RLPS_NS").unwrap().extract().unwrap();
    let disclaimer: String = m.getattr("NOT_LEGAL_ADVICE").unwrap().extract().unwrap();
    assert!(!version.is_empty());
    assert_eq!(ns, "https://r14n.squillo.com/ns#");
    assert!(disclaimer.starts_with("NOT LEGAL ADVICE"));
  });
}

/// Why: a positive control for every lookup above -- an attribute the shim does not export must
/// raise, so "the attribute exists" is a real assertion and not a lookup that cannot fail.
#[test]
fn an_unexported_name_raises_attribute_error() {
  Python::attach(|py| {
    let err = module(py).getattr("no_such_function").unwrap_err();
    assert!(err.is_instance_of::<pyo3::exceptions::PyAttributeError>(py));
  });
}

/// Why: the happy path across the shim -- verdict, the sorted required set, and the pack key as
/// provenance source.
#[test]
fn happy_path_returns_the_core_decision() {
  Python::attach(|py| {
    let d = call_json(py, "resolve", &[&packs(), &query()]).unwrap();
    assert_eq!(d["verdict"], "permit");
    assert_eq!(d["required"], json!(["user_attestation"]));
    assert_eq!(d["provenance"]["source"], PACK_KEY);
    assert_eq!(d["provenance"]["fell_back"], false);
    assert!(d["disclaimer"].as_str().unwrap().starts_with("NOT LEGAL ADVICE"));
  });
}

/// Why: fail-closed (spec §4/§5) is the property the standard rests on -- a missing pack demands
/// the caller's whole universe, loudly, and must not raise or resolve empty.
#[test]
fn missing_pack_fails_closed() {
  Python::attach(|py| {
    let d = call_json(py, "resolve", &["{}", &query()]).unwrap();
    assert_eq!(d["provenance"]["fell_back"], true);
    assert_eq!(d["required"], json!(["signal_notice", "user_attestation"]));
  });
}

/// Why: a caller bug must be distinguishable from a policy fallback -- bad JSON is a `ValueError`
/// naming the argument, never a silent aggressive decision.
#[test]
fn malformed_input_raises_value_error() {
  Python::attach(|py| {
    let err = call_json(py, "resolve", &[&packs(), "{not json"]).unwrap_err();
    assert!(err.is_instance_of::<pyo3::exceptions::PyValueError>(py));
    assert!(err.to_string().contains("query_json"), "got: {err}");
  });
}

/// Why: the global dial must reach the core through the optional third argument and be reported
/// back.
#[test]
fn strictness_override_dial() {
  Python::attach(|py| {
    let d = call_json(py, "resolve", &[&packs(), &query(), "aggressive"]).unwrap();
    assert_eq!(d["provenance"]["strictness"], "aggressive");
    assert_eq!(d["required"], json!(["signal_notice", "user_attestation"]));
  });
}

/// Why: receipts are the auditable artifact (spec §8); both forms must serialize and stay
/// advisory-only, since the reference resolver verifies no provenance (spec §6).
#[test]
fn both_receipt_forms_advisory_only() {
  Python::attach(|py| {
    let out = call_json(py, "resolve_with_receipts", &[&packs(), &query(), &ctx()]).unwrap();
    assert_eq!(out["decision"]["verdict"], "permit");
    assert_eq!(out["receipt_dpv27560"]["decision"]["verdict"], "permit");
    assert_eq!(out["receipt_dpv27560"]["provenance"]["advisory_only"], true);
    assert!(out["receipt_kantara_cr_v1_1"].is_object());
  });
}

/// Why: three JSON inputs mean three distinct error surfaces; a bad context must not be blamed on
/// the query.
#[test]
fn malformed_context_names_ctx_json() {
  Python::attach(|py| {
    let err = call_json(py, "resolve_with_receipts", &[&packs(), &query(), "{}"]).unwrap_err();
    assert!(err.is_instance_of::<pyo3::exceptions::PyValueError>(py));
    assert!(err.to_string().contains("ctx_json"), "got: {err}");
  });
}

fn manifest(text: &str) -> toml::Table {
  text.parse().expect("a well-formed TOML manifest")
}

/// Why: CPython derives the initialiser symbol it calls (`PyInit_<name>`) from the module name, so
/// the Cargo `[lib] name`, maturin's `module-name` and the distribution name must agree -- a
/// mismatch builds a wheel that installs fine and fails on `import`. The wheel is no longer
/// imported by a Python test, so this is the guard on that failure mode.
#[test]
fn the_import_name_agrees_across_cargo_and_maturin() {
  let cargo = manifest(include_str!("../Cargo.toml"));
  let pyproject = manifest(include_str!("../pyproject.toml"));
  let lib = cargo["lib"]["name"].as_str().unwrap();
  assert_eq!(pyproject["tool"]["maturin"]["module-name"].as_str().unwrap(), lib);
  assert_eq!(pyproject["project"]["name"].as_str().unwrap(), lib);
}

/// Why: `abi3-py39` is what makes one wheel serve every Python from the floor up, and
/// `requires-python` is what pip enforces; if they differ the wheel claims a range it cannot load
/// on (or refuses one it could).
#[test]
fn the_abi3_floor_equals_requires_python() {
  let cargo = manifest(include_str!("../Cargo.toml"));
  let pyproject = manifest(include_str!("../pyproject.toml"));
  let requires = pyproject["project"]["requires-python"].as_str().unwrap();
  let floor = requires.strip_prefix(">=").expect("a `>=` floor").replace('.', "");
  let features = cargo["dependencies"]["pyo3"]["features"].as_array().unwrap();
  assert!(
    features.iter().any(|f| f.as_str() == Some(&format!("abi3-py{floor}"))),
    "requires-python {requires} but pyo3 features are {features:?}"
  );
}
