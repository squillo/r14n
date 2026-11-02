// SPDX-License-Identifier: Apache-2.0
//! `package-npm` tests — the published npm metadata, and the thin I/O wrapper around it.
//!
//! None of these needs wasm-pack. The manifest rewrite is pure, so it is tested directly; the
//! wrapper is driven through the built binary with a stand-in `wasm-pack` on a scratch PATH, so
//! the spawn, the exit-status propagation and the staging are covered without a wasm toolchain.
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-11-02: authored — the npm package build moved from a Node script into the CLI.

mod common;

// The tools crate is a binary with no library target, so the pure half is compiled straight
// into this test crate from its source file: the tests exercise the shipped code, not a copy.
// The module's I/O half goes unused here by design.
#[path = "../src/package_npm.rs"]
#[allow(dead_code)]
mod package_npm;

/// A manifest shaped like wasm-pack's output for `--target nodejs --out-name r14n`.
fn generated() -> ::serde_json::Value {
  ::serde_json::json!({
    "name": "r14n-wasm",
    "description": "WebAssembly/JS bindings for the RLPS reference resolver (r14n)",
    "version": "0.1.0",
    "license": "Apache-2.0",
    "repository": { "type": "git", "url": "https://github.com/squillo/r14n" },
    "files": ["r14n_bg.wasm", "r14n.js", "r14n.d.ts"],
    "main": "r14n.js",
    "types": "r14n.d.ts"
  })
}

fn without(key: &str) -> ::serde_json::Value {
  let mut manifest = generated();
  manifest.as_object_mut().expect("object").remove(key);
  manifest
}

fn with_files(files: ::serde_json::Value) -> ::serde_json::Value {
  let mut manifest = generated();
  manifest["files"] = files;
  manifest
}

fn rewritten(generated: &::serde_json::Value) -> ::serde_json::Value {
  package_npm::rewrite_manifest(generated).expect("a well-formed manifest must rewrite")
}

fn files_of(manifest: &::serde_json::Value) -> ::std::vec::Vec<::std::string::String> {
  manifest["files"]
    .as_array()
    .expect("files array")
    .iter()
    .map(|f| ::std::string::String::from(f.as_str().expect("files entry is a string")))
    .collect()
}

/// Why: the package is published under the scoped name (the bare name is blocked by npm's
/// similarity guard) with registry metadata wasm-pack does not emit. A regression ships a
/// package named after the crate, or drops the NOT-LEGAL-ADVICE disclaimer the description
/// carries (AGENTS.md invariant 3), or grows a runtime dependency the published package
/// promises not to have.
#[test]
fn published_metadata_is_the_scoped_identity() {
  let manifest = rewritten(&generated());
  ::std::assert_eq!(manifest["name"], "@squillo/r14n");
  ::std::assert!(manifest["description"].as_str().expect("description").contains("NOT LEGAL ADVICE"));
  ::std::assert_eq!(manifest["homepage"], "https://r14n.squillo.com");
  ::std::assert_eq!(
    manifest["repository"],
    ::serde_json::json!({ "type": "git", "url": "git+https://github.com/squillo/r14n.git" })
  );
  ::std::assert_eq!(
    manifest["keywords"],
    ::serde_json::json!(["compliance", "regulatory", "policy-as-code", "wasm", "rlps", "r14n"])
  );
  ::std::assert!(manifest.get("dependencies").is_none(), "zero runtime dependencies");
  let files = files_of(&manifest);
  ::std::assert!(files.iter().any(|f| f == "README.md"), "README ships in the tarball");
  ::std::assert!(files.iter().any(|f| f == "LICENSE"), "LICENSE ships in the tarball");
}

/// Why: only the registry-facing fields are ours to override. `main`, `types`, `version` and
/// `license` come from wasm-pack and the crate; overwriting or dropping them would point `require`
/// at the wrong entry or publish under a version the crate never declared.
#[test]
fn generated_fields_we_do_not_own_pass_through_untouched() {
  let manifest = rewritten(&generated());
  for key in ["version", "license", "main", "types"] {
    ::std::assert_eq!(manifest[key], generated()[key], "`{key}` must pass through");
  }
}

/// Why: `files` is what npm packs. If the rewrite replaced wasm-pack's list instead of extending
/// it, the tarball would ship the README and LICENSE and NO .wasm or glue, and every other check
/// would still pass. Order is pinned too: the generated entries stay first, in wasm-pack's order.
#[test]
fn files_keep_the_wasm_artifacts_and_gain_the_docs_after_them() {
  let manifest = rewritten(&generated());
  ::std::assert_eq!(
    files_of(&manifest),
    ["r14n_bg.wasm", "r14n.js", "r14n.d.ts", "README.md", "LICENSE"]
  );
}

/// Why: re-running the rewrite over its own output (a re-staged directory) must not stack
/// duplicate `files` entries, and the whole manifest must be a fixed point — otherwise the
/// result depends on how many times the build ran.
#[test]
fn rewriting_twice_changes_nothing() {
  let once = rewritten(&generated());
  let twice = rewritten(&once);
  ::std::assert_eq!(once, twice);
  let files = files_of(&twice);
  let unique: ::std::collections::BTreeSet<_> = files.iter().collect();
  ::std::assert_eq!(files.len(), unique.len(), "duplicate `files` entries: {files:?}");
}

/// Why: negative control. A manifest the rewrite cannot trust must be an error, never a package
/// with a defaulted field — an empty `files` would publish a tarball with no wasm in it, and a
/// version-less manifest cannot be published at all. Each malformed shape is asserted to name
/// the offending field so the failure is diagnosable from the CI log.
#[test]
fn a_manifest_missing_a_required_field_is_an_error_not_a_default() {
  let cases = [
    (without("files"), "files"),
    (without("version"), "version"),
    (with_files(::serde_json::json!("r14n.js")), "files"),
    (with_files(::serde_json::json!(["r14n.js", 3])), "not a string"),
    (::serde_json::json!([]), "not a JSON object"),
    (::serde_json::Value::Null, "not a JSON object"),
  ];
  for (manifest, expect) in cases {
    let err = package_npm::rewrite_manifest(&manifest)
      .expect_err("a malformed manifest must not produce a package");
    ::std::assert!(err.contains(expect), "error `{err}` should mention `{expect}`");
  }
}

/// Why: the file on disk is the artifact npm reads. The old writer emitted two-space JSON with a
/// trailing newline and left non-ASCII literal (the description has an em dash); a serializer
/// change that escaped it or dropped the newline would churn every staged package.json.
#[test]
fn rendered_manifest_is_two_space_json_with_a_trailing_newline() {
  let manifest = rewritten(&generated());
  let text = package_npm::render_manifest(&manifest).expect("render");
  ::std::assert!(text.starts_with("{\n  \""), "two-space indent: {text}");
  ::std::assert!(text.ends_with("}\n") && !text.ends_with("\n\n"), "exactly one trailing newline");
  ::std::assert!(text.contains("— WebAssembly"), "non-ASCII stays literal");
  let back: ::serde_json::Value = ::serde_json::from_str(&text).expect("rendered text is JSON");
  ::std::assert_eq!(back, manifest);
}

/// Why: positive control for the completeness check. A package whose every promised path exists
/// must report nothing, or the negative cases below could pass by flagging everything.
#[test]
fn a_package_that_has_everything_it_promises_reports_nothing_missing() {
  let manifest = rewritten(&generated());
  ::std::assert!(package_npm::missing_promised(&manifest, |_| true).is_empty());
}

/// Why: the old Node smoke test loaded the built bundle, so a manifest naming a `.wasm` the stage
/// did not contain failed CI. Nothing executes the bundle now; this check is what keeps the
/// metadata and the bytes in agreement. Exactly the absent file must be named — not everything.
#[test]
fn a_promised_wasm_binary_that_was_not_staged_is_named() {
  let manifest = rewritten(&generated());
  ::std::assert_eq!(
    package_npm::missing_promised(&manifest, |path| path != "r14n_bg.wasm"),
    ["r14n_bg.wasm"]
  );
}

/// Why: `main` is the file `require("@squillo/r14n")` loads, and it is also listed in `files`; a
/// missing entry point must be reported once, not twice, so the error stays readable and stable.
#[test]
fn a_missing_entry_point_is_reported_once() {
  let manifest = rewritten(&generated());
  ::std::assert_eq!(package_npm::missing_promised(&manifest, |path| path != "r14n.js"), ["r14n.js"]);
}

fn r14n() -> ::std::process::Command {
  ::std::process::Command::new(::std::env!("CARGO_BIN_EXE_r14n"))
}

fn stderr_of(out: &::std::process::Output) -> ::std::string::String {
  ::std::string::String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Install a stand-in `wasm-pack` (a shell script) and return a PATH that finds it first. The
/// system directories stay on PATH because the script itself calls `mkdir`.
#[cfg(unix)]
fn fake_wasm_pack(dir: &::std::path::Path, body: &str) -> ::std::string::String {
  use ::std::os::unix::fs::PermissionsExt;
  let script = dir.join("wasm-pack");
  ::std::fs::write(&script, ::std::format!("#!/bin/sh\n{body}\n")).expect("write stand-in");
  ::std::fs::set_permissions(&script, ::std::fs::Permissions::from_mode(0o755)).expect("chmod");
  ::std::format!("{}:/usr/bin:/bin", dir.display())
}

/// Why: a missing wasm-pack is the first failure a new contributor hits. It must say so (not
/// surface a bare "No such file"), exit non-zero so CI cannot go green on nothing, and leave no
/// half-staged package behind for a later step to publish.
#[test]
fn a_missing_wasm_pack_is_a_clear_error_and_stages_nothing() {
  let dir = ::tempfile::tempdir().expect("tempdir");
  let empty_path = dir.path().join("empty");
  ::std::fs::create_dir(&empty_path).expect("mkdir");
  let out_dir = dir.path().join("pkg");
  let out = r14n()
    .args(["package-npm", "--out"])
    .arg(&out_dir)
    .env("PATH", &empty_path)
    .output()
    .expect("run package-npm");
  ::std::assert_eq!(out.status.code(), Some(1));
  ::std::assert!(stderr_of(&out).contains("wasm-pack not found"), "stderr: {}", stderr_of(&out));
  ::std::assert!(!out_dir.exists(), "a failed build must not leave a staged package");
}

/// Why: the build's own status and diagnostics are the only evidence of why it failed. Flattening
/// the exit code to 1, or capturing stderr, hides a compiler error behind a generic message; and a
/// failed build must never fall through to staging.
#[cfg(unix)]
#[test]
fn a_failing_wasm_pack_propagates_its_status_and_stderr() {
  let dir = ::tempfile::tempdir().expect("tempdir");
  let path = fake_wasm_pack(dir.path(), "echo boom >&2\nexit 7");
  let out_dir = dir.path().join("pkg");
  let out = r14n()
    .args(["package-npm", "--out"])
    .arg(&out_dir)
    .env("PATH", path)
    .output()
    .expect("run package-npm");
  ::std::assert_eq!(out.status.code(), Some(7), "wasm-pack's own status propagates");
  ::std::assert!(stderr_of(&out).contains("boom"), "stderr: {}", stderr_of(&out));
  ::std::assert!(!out_dir.join("package.json").exists(), "a failed build must not stage");
}

/// Why: the end-to-end contract of the subcommand. It must invoke wasm-pack with the nodejs
/// target and the `r14n` out-name from the binding crate's directory (a wrong target or cwd
/// builds a package nobody can `require`), rewrite the manifest it produced, and copy the
/// repository LICENSE and a README that carries the disclaimer into the package.
#[cfg(unix)]
#[test]
fn stages_the_package_around_wasm_pack_output() {
  let dir = ::tempfile::tempdir().expect("tempdir");
  let path = fake_wasm_pack(
    dir.path(),
    r#"for a in "$@"; do [ "$prev" = "--out-dir" ] && out="$a"; prev="$a"; done
mkdir -p "$out"
printf '%s\n' "$@" > "$out/args.txt"
pwd -P > "$out/cwd.txt"
: > "$out/r14n_bg.wasm"; : > "$out/r14n.js"; : > "$out/r14n.d.ts"
printf '%s' '{"name":"r14n-wasm","version":"0.1.0","main":"r14n.js","types":"r14n.d.ts","files":["r14n_bg.wasm","r14n.js","r14n.d.ts"]}' > "$out/package.json""#,
  );
  let out_dir = dir.path().join("pkg");
  let out = r14n()
    .args(["package-npm", "--out"])
    .arg(&out_dir)
    .env("PATH", path)
    .output()
    .expect("run package-npm");
  ::std::assert!(out.status.success(), "stderr: {}", stderr_of(&out));
  ::std::assert!(stderr_of(&out).contains("@squillo/r14n staged at"), "stderr: {}", stderr_of(&out));

  let args = ::std::fs::read_to_string(out_dir.join("args.txt")).expect("args.txt");
  ::std::assert_eq!(
    args.lines().collect::<::std::vec::Vec<_>>(),
    ["build", "--target", "nodejs", "--out-dir", out_dir.to_str().expect("utf8 path"), "--out-name", "r14n"]
  );
  let cwd = ::std::fs::read_to_string(out_dir.join("cwd.txt")).expect("cwd.txt");
  ::std::assert_eq!(
    ::std::path::Path::new(cwd.trim()),
    ::std::fs::canonicalize(common::root().join("bindings/wasm")).expect("bindings/wasm")
  );

  let text = ::std::fs::read_to_string(out_dir.join("package.json")).expect("package.json");
  ::std::assert!(text.ends_with("}\n"));
  let manifest: ::serde_json::Value = ::serde_json::from_str(&text).expect("package.json is JSON");
  ::std::assert_eq!(manifest["name"], "@squillo/r14n");
  ::std::assert_eq!(
    files_of(&manifest),
    ["r14n_bg.wasm", "r14n.js", "r14n.d.ts", "README.md", "LICENSE"]
  );
  ::std::assert_eq!(
    ::std::fs::read_to_string(out_dir.join("LICENSE")).expect("staged LICENSE"),
    common::read("LICENSE-APACHE")
  );
  ::std::assert!(
    ::std::fs::read_to_string(out_dir.join("README.md")).expect("staged README").contains("NOT LEGAL ADVICE"),
    "the staged README must carry the disclaimer"
  );
}

/// Why: end-to-end negative control for the completeness check. A build whose manifest lists a
/// `.wasm` it never wrote is a package that installs and then fails on `require`. The subcommand
/// must exit non-zero and name the missing file — exit 0 here is how a broken package gets
/// published, because nothing else loads the bundle.
#[cfg(unix)]
#[test]
fn a_build_that_omits_a_listed_artifact_fails_the_stage_and_names_it() {
  let dir = ::tempfile::tempdir().expect("tempdir");
  let path = fake_wasm_pack(
    dir.path(),
    r#"for a in "$@"; do [ "$prev" = "--out-dir" ] && out="$a"; prev="$a"; done
mkdir -p "$out"
: > "$out/r14n.js"; : > "$out/r14n.d.ts"
printf '%s' '{"name":"r14n-wasm","version":"0.1.0","main":"r14n.js","types":"r14n.d.ts","files":["r14n_bg.wasm","r14n.js","r14n.d.ts"]}' > "$out/package.json""#,
  );
  let out = r14n()
    .args(["package-npm", "--out"])
    .arg(dir.path().join("pkg"))
    .env("PATH", path)
    .output()
    .expect("run package-npm");
  ::std::assert_eq!(out.status.code(), Some(1), "stderr: {}", stderr_of(&out));
  ::std::assert!(stderr_of(&out).contains("r14n_bg.wasm"), "stderr: {}", stderr_of(&out));
  ::std::assert!(!stderr_of(&out).contains("r14n.js,"), "only the absent file is named: {}", stderr_of(&out));
}
