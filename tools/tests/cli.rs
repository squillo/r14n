// SPDX-License-Identifier: Apache-2.0
//! CLI integration tests — drive the BUILT `r14n` binary end-to-end (council
//! audit N7: the `main()` dispatch layer, exit codes, and the publish
//! lint-before-index gate had zero coverage; the unit tests stop at `parse_args`
//! and the library functions).
//!
//! Revision History
//! - 2026-07-07: authored — council-audit N7 (CLI dispatch coverage).

/// Absolute path to the built `r14n` binary (Cargo sets CARGO_BIN_EXE_<name>).
fn r14n() -> ::std::process::Command {
  ::std::process::Command::new(::std::env!("CARGO_BIN_EXE_r14n"))
}

fn scratch() -> ::tempfile::TempDir {
  ::tempfile::tempdir().expect("tempdir")
}

const CATALOG: &str = "[domain]\nname = \"recording_consent\"\ndescription = \"d\"\n\n\
   [control.attestation]\nkind = \"obligation\"\ntitle = \"a\"\n\n\
   [control.signal_notice]\nkind = \"obligation\"\ntitle = \"b\"\n";

/// Why: the whole point of the tool is the extract→validate→sign→verify→publish
/// pipeline; a regression in any main dispatch arm (which no unit test exercises)
/// would ship green. This drives the real binary through the happy path.
#[test]
fn full_lifecycle_happy_path_through_the_binary() {
  let dir = scratch();
  let catalog = dir.path().join("recording_consent.catalog.toml");
  ::std::fs::write(&catalog, CATALOG).expect("write catalog");

  // extract → a template (to stdout).
  let out = r14n()
    .args(["extract", "--catalog"])
    .arg(&catalog)
    .args(["--profile", "demo"])
    .output()
    .expect("run extract");
  assert!(out.status.success(), "extract failed: {}", String::from_utf8_lossy(&out.stderr));
  assert!(String::from_utf8_lossy(&out.stdout).contains("NOT LEGAL ADVICE"));

  // A hand-authored valid pack.
  let pack = dir.path().join("recording_consent.r14n.toml");
  ::std::fs::write(
    &pack,
    "[meta]\nstrictness = \"minimal\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
     [legally_required]\ncontrols = [\"attestation\"]\n",
  )
  .expect("write pack");

  // validate --catalog → OK, exit 0.
  let st = r14n().arg("validate").arg(&pack).arg("--catalog").arg(&catalog).status().expect("validate");
  assert!(st.success(), "validate should pass");

  // keygen → sign → verify.
  let st = r14n().args(["keygen", "--out"]).arg(dir.path().join("rk")).status().expect("keygen");
  assert!(st.success());
  let st = r14n()
    .arg("sign")
    .arg(&pack)
    .arg("--key")
    .arg(dir.path().join("rk.seed"))
    .args(["--key-id", "reviewer-1"])
    .status()
    .expect("sign");
  assert!(st.success(), "sign should succeed");
  let st = r14n().arg("verify").arg(&pack).status().expect("verify");
  assert!(st.success(), "verify should succeed on a fresh signature");

  // publish → local index, exit 0.
  let index = dir.path().join("registry").join("index.json");
  let st = r14n()
    .arg("publish")
    .arg(&pack)
    .args(["--id", "demo/recording_consent", "--registry"])
    .arg(&index)
    .status()
    .expect("publish");
  assert!(st.success(), "publish should succeed");
  assert!(index.exists(), "publish must write the index");
}

/// Why: council-audit — the "lint before indexing" gate exists ONLY in `main`
/// (publish() the library fn validates nothing). A regression that dropped it
/// would publish invalid packs with every unit test still green; this is the
/// integration-level guard.
#[test]
fn publish_refuses_a_pack_that_fails_validation() {
  let dir = scratch();
  // No floor + no review status ⇒ validate errors.
  let pack = dir.path().join("bad.r14n.toml");
  ::std::fs::write(&pack, "[meta]\nstrictness = \"minimal\"\n").expect("write");
  let index = dir.path().join("index.json");
  let out = r14n()
    .arg("publish")
    .arg(&pack)
    .args(["--id", "bad/recording_consent", "--registry"])
    .arg(&index)
    .output()
    .expect("run publish");
  assert!(!out.status.success(), "publish must reject an invalid pack");
  assert!(
    String::from_utf8_lossy(&out.stderr).contains("not publishing"),
    "stderr: {}",
    String::from_utf8_lossy(&out.stderr)
  );
  assert!(!index.exists(), "a refused publish must not write the index");
}

/// Why: council-audit — `validate` must exit non-zero if ANY pack in a
/// multi-pack invocation fails, else CI treating exit code as pass/fail would
/// green-light a batch containing a broken pack.
#[test]
fn validate_exits_nonzero_when_any_pack_fails() {
  let dir = scratch();
  let good = dir.path().join("good.r14n.toml");
  ::std::fs::write(
    &good,
    "[meta]\nstrictness = \"minimal\"\n\n[meta.legal_review]\nstatus = \"draft\"\n\n\
     [legally_required]\ncontrols = [\"attestation\"]\n",
  )
  .expect("write good");
  let bad = dir.path().join("bad.r14n.toml");
  ::std::fs::write(&bad, "[meta]\nstrictness = \"minimal\"\n").expect("write bad");
  let st = r14n().arg("validate").arg(&good).arg(&bad).status().expect("validate");
  assert!(!st.success(), "a batch with one bad pack must exit non-zero");
}

/// Why: a bare/unknown invocation is the first thing a user hits — it must exit
/// with the documented code (2), not panic or silently succeed.
#[test]
fn unknown_subcommand_exits_two() {
  let out = r14n().arg("frobnicate").output().expect("run");
  assert_eq!(out.status.code(), Some(2), "unknown subcommand ⇒ exit 2");
}

/// Why: council-audit M9 — the trust root is only real if `verify --directory`
/// actually gates on it end-to-end. A signer in the directory (in-validity,
/// jurisdiction-matched) exits 0; the SAME valid signature from a signer NOT in
/// the directory must exit non-zero (advisory-only), driven through the binary.
#[test]
fn verify_directory_trust_gate_end_to_end() {
  let dir = scratch();
  let profile = dir.path().join("us_all_party");
  std::fs::create_dir_all(&profile).expect("mkdir");
  let pack = profile.join("recording_consent.r14n.toml");
  std::fs::write(&pack, "[meta]\nstrictness = \"minimal\"\n").expect("write pack");

  // Keygen + sign; read the signer's public key that keygen wrote.
  assert!(r14n().args(["keygen", "--out"]).arg(dir.path().join("rk")).status().expect("keygen").success());
  assert!(r14n().arg("sign").arg(&pack).arg("--key").arg(dir.path().join("rk.seed")).status().expect("sign").success());
  let pubkey = std::fs::read_to_string(dir.path().join("rk.pub")).expect("pub").trim().to_string();

  // Directory listing THIS key, US-CA, in-validity.
  let good_dir = dir.path().join("good-directory.json");
  std::fs::write(
    &good_dir,
    format!(
      "{{\"directory_version\":\"1\",\"keys\":[{{\"key_id\":\"r1\",\"public_key_ed25519\":\"{pubkey}\",\
       \"reviewer_identity\":\"Counsel\",\"jurisdiction\":\"US-CA\",\"credential_type\":\"bar_license\",\
       \"credential_id\":\"1\",\"valid_from\":\"2026-01-01\",\"valid_until\":\"2026-12-31\"}}]}}"
    ),
  )
  .expect("write good dir");
  let st = r14n()
    .arg("verify").arg(&pack)
    .arg("--directory").arg(&good_dir)
    .args(["--as-of", "2026-06-15", "--jurisdiction", "US-CA"])
    .status()
    .expect("verify");
  assert!(st.success(), "a listed, in-validity, jurisdiction-matched key must be trusted");

  // An empty directory ⇒ the same valid signature is advisory-only ⇒ non-zero.
  let empty_dir = dir.path().join("empty-directory.json");
  std::fs::write(&empty_dir, "{\"directory_version\":\"1\",\"keys\":[]}").expect("write empty dir");
  let st = r14n()
    .arg("verify").arg(&pack)
    .arg("--directory").arg(&empty_dir)
    .args(["--as-of", "2026-06-15"])
    .status()
    .expect("verify");
  assert!(!st.success(), "an unlisted signer must not be trusted (advisory-only)");
}
