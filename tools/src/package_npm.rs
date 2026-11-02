// SPDX-License-Identifier: Apache-2.0
//! `package-npm` — stage the publishable `@squillo/r14n` npm package from the Rust resolver.
//!
//! wasm-pack emits a `package.json` whose `name` is the CRATE name (`r14n-wasm`) and which
//! carries none of the registry metadata we publish under. This subcommand runs the build, then
//! rewrites exactly those fields — the generated glue, the `.wasm` and the `.d.ts` are
//! wasm-pack's output, untouched — and adds the README and LICENSE. Zero npm dependencies on
//! purpose: the published package has no runtime dependencies, and neither does the thing that
//! builds it.
//!
//! The split is deliberate: [`rewrite_manifest`] and [`render_manifest`] are pure (no I/O, so
//! the published metadata is unit-testable without wasm-pack); [`run`] is the thin wrapper that
//! spawns the build and copies files. This file has no `crate::` dependencies so the integration
//! test can compile it directly — the crate is a binary with no library target.
//!
//! ⚠ NOT LEGAL ADVICE. The package description carries the disclaimer; it is part of what
//! `rewrite_manifest` writes and must not be dropped.
//!
//! Revision History
//! - 2026-11-02: authored — replaces the former Node build script, so the build tooling is Rust.

/// The package identity we publish under. The bare name `r14n` is permanently unavailable on
/// npm (its similarity guard rejects it against `i18n`/`y18n`), so the scoped name is the only
/// option.
pub const PACKAGE_NAME: &str = "@squillo/r14n";

const DESCRIPTION: &str = "RLPS reference resolver for JS/TS — WebAssembly bindings generated from the Rust core. Fail-closed regulatory-control resolution with legal-review provenance. NOT LEGAL ADVICE.";
const HOMEPAGE: &str = "https://r14n.squillo.com";
const KEYWORDS: [&str; 6] = ["compliance", "regulatory", "policy-as-code", "wasm", "rlps", "r14n"];

/// npm normalizes a bare https repository URL to the `git+….git` form at publish time and warns
/// that it did. `bindings/wasm/Cargo.toml` has to keep the bare form (crates.io's convention),
/// so the npm-canonical form is applied here instead of at the Rust source.
const REPOSITORY_URL: &str = "git+https://github.com/squillo/r14n.git";

/// Files added beside wasm-pack's output, as (name inside the package, repo-relative source).
/// One table drives both the copy and the manifest's `files` list, so the package can never
/// declare a file it did not stage (or stage one npm would drop from the tarball).
const STAGED: [(&str, &str); 2] =
  [("README.md", "bindings/npm/README.md"), ("LICENSE", "LICENSE-APACHE")];

/// The manifest wasm-pack generated, rewritten for publication — pure, no I/O.
///
/// Every generated field is kept; the registry-facing fields are overridden; `files` keeps
/// wasm-pack's artifact entries in order and gains the staged docs once (applying this to its
/// own output changes nothing). A manifest this cannot rewrite faithfully is an `Err`, never a
/// package with a defaulted field: a `files` list that silently became empty would publish a
/// tarball with no wasm in it.
pub fn rewrite_manifest(
  generated: &::serde_json::Value,
) -> ::std::result::Result<::serde_json::Value, ::std::string::String> {
  let fields = generated
    .as_object()
    .ok_or_else(|| ::std::string::String::from("generated package.json is not a JSON object"))?;
  // npm refuses to publish a manifest with no version, so a missing one is caught here.
  fields
    .get("version")
    .and_then(::serde_json::Value::as_str)
    .ok_or_else(|| ::std::string::String::from("generated package.json has no string `version`"))?;
  let listed = fields
    .get("files")
    .and_then(::serde_json::Value::as_array)
    .ok_or_else(|| ::std::string::String::from("generated package.json has no `files` array"))?
    .iter()
    .map(|f| {
      f.as_str()
        .map(::std::string::String::from)
        .ok_or_else(|| ::std::format!("generated `files` entry {f} is not a string"))
    })
    .collect::<::std::result::Result<::std::vec::Vec<_>, _>>()?;
  let added = STAGED
    .iter()
    .map(|(name, _)| *name)
    .filter(|name| !listed.iter().any(|l| l.as_str() == *name))
    .map(::std::string::String::from);
  let files: ::std::vec::Vec<::std::string::String> = listed.iter().cloned().chain(added).collect();
  let overrides = [
    ("name", ::serde_json::Value::from(PACKAGE_NAME)),
    ("description", ::serde_json::Value::from(DESCRIPTION)),
    ("homepage", ::serde_json::Value::from(HOMEPAGE)),
    ("keywords", ::serde_json::Value::from(KEYWORDS.to_vec())),
    ("repository", ::serde_json::json!({ "type": "git", "url": REPOSITORY_URL })),
    ("files", ::serde_json::Value::from(files)),
  ];
  ::std::result::Result::Ok(::serde_json::Value::Object(
    fields
      .iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .chain(overrides.into_iter().map(|(k, v)| (::std::string::String::from(k), v)))
      .collect(),
  ))
}

/// The manifest as written to disk: two-space indent and a trailing newline.
pub fn render_manifest(
  manifest: &::serde_json::Value,
) -> ::std::result::Result<::std::string::String, ::std::string::String> {
  ::serde_json::to_string_pretty(manifest)
    .map(|text| ::std::format!("{text}\n"))
    .map_err(|e| ::std::format!("serialize package.json: {e}"))
}

/// Every path the manifest promises a consumer (`main`, `types`, each `files` entry) that
/// `exists` cannot find — sorted and de-duplicated, pure. This is the check that a package which
/// installs is also a package which loads: the metadata and the staged bytes must agree, and
/// nothing else in the pipeline executes the published bundle.
pub fn missing_promised(
  manifest: &::serde_json::Value,
  exists: impl ::std::ops::Fn(&str) -> bool,
) -> ::std::vec::Vec<::std::string::String> {
  ["main", "types"]
    .iter()
    .filter_map(|key| manifest.get(*key).and_then(::serde_json::Value::as_str))
    .chain(
      manifest
        .get("files")
        .and_then(::serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(::serde_json::Value::as_str),
    )
    .filter(|path| !exists(path))
    .map(::std::string::String::from)
    .collect::<::std::collections::BTreeSet<_>>()
    .into_iter()
    .collect()
}

/// Build with wasm-pack into `out_dir`. `Err` carries the exit code to propagate: wasm-pack's
/// own when it ran and failed, else 1. wasm-pack's stdout and stderr are inherited, never
/// captured, so its diagnostics reach the user untouched.
fn build_wasm(
  wasm_dir: &::std::path::Path,
  out_dir: &::std::path::Path,
) -> ::std::result::Result<(), (i32, ::std::string::String)> {
  // A missing working directory also surfaces as `NotFound` from the spawn, which would read
  // as "wasm-pack is not installed" — so check it first.
  if !wasm_dir.is_dir() {
    return ::std::result::Result::Err((
      1,
      ::std::format!("{} is not a directory (the CLI must run from its own checkout)", wasm_dir.display()),
    ));
  }
  let status = ::std::process::Command::new("wasm-pack")
    .args(["build", "--target", "nodejs", "--out-dir"])
    .arg(out_dir)
    .args(["--out-name", "r14n"])
    .current_dir(wasm_dir)
    .status()
    .map_err(|e| match e.kind() {
      ::std::io::ErrorKind::NotFound => (
        1,
        ::std::string::String::from(
          "wasm-pack not found on PATH — install it (`cargo install wasm-pack`) and re-run",
        ),
      ),
      _ => (1, ::std::format!("spawn wasm-pack: {e}")),
    })?;
  if status.success() {
    ::std::result::Result::Ok(())
  } else {
    ::std::result::Result::Err((status.code().unwrap_or(1), ::std::format!("wasm-pack failed ({status})")))
  }
}

/// Rewrite the generated manifest in place and copy the staged files beside it.
fn stage(
  repo: &::std::path::Path,
  out_dir: &::std::path::Path,
) -> ::std::result::Result<(), ::std::string::String> {
  let manifest_path = out_dir.join("package.json");
  let raw = ::std::fs::read_to_string(&manifest_path)
    .map_err(|e| ::std::format!("read {}: {e}", manifest_path.display()))?;
  let generated: ::serde_json::Value = ::serde_json::from_str(&raw)
    .map_err(|e| ::std::format!("parse {}: {e}", manifest_path.display()))?;
  let manifest = rewrite_manifest(&generated)?;
  ::std::fs::write(&manifest_path, render_manifest(&manifest)?)
    .map_err(|e| ::std::format!("write {}: {e}", manifest_path.display()))?;
  STAGED.iter().try_for_each(|(name, source)| {
    ::std::fs::copy(repo.join(source), out_dir.join(name))
      .map(|_| ())
      .map_err(|e| ::std::format!("copy {source} → {}: {e}", out_dir.join(name).display()))
  })?;
  match missing_promised(&manifest, |path| out_dir.join(path).is_file()).as_slice() {
    [] => ::std::result::Result::Ok(()),
    missing => ::std::result::Result::Err(::std::format!(
      "package.json promises files the staged package lacks: {}",
      missing.join(", ")
    )),
  }
}

/// Build and stage the npm package (`out` defaults to `dist/npm` under the repo root).
/// `Ok` is the message to print; `Err` is (exit code, message).
pub fn run(
  out: ::std::option::Option<&::std::path::Path>,
) -> ::std::result::Result<::std::string::String, (i32, ::std::string::String)> {
  // The repo root is the tools crate's parent, as the integration tests locate it.
  let repo = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR"))
    .parent()
    .ok_or_else(|| (1, ::std::string::String::from("the tools crate has no parent directory")))?;
  let out_dir = match out {
    ::std::option::Option::Some(path) => ::std::path::absolute(path)
      .map_err(|e| (1, ::std::format!("resolve {}: {e}", path.display())))?,
    ::std::option::Option::None => repo.join("dist/npm"),
  };
  build_wasm(&repo.join("bindings/wasm"), &out_dir)?;
  stage(repo, &out_dir).map_err(|message| (1, message))?;
  ::std::result::Result::Ok(::std::format!("{PACKAGE_NAME} staged at {}", out_dir.display()))
}
