// SPDX-License-Identifier: Apache-2.0
//! `r14n publish` — content-addressed supersession into the LOCAL registry
//! index (`/registry/index.json`, schema `/registry/pack-index.schema.json`).
//!
//! COUNSEL-GATED BY DESIGN: this never touches a network. "Publishing" appends
//! a version entry (id, monotonic version, sha256, supersedes-chain, optional
//! detached-signature embed) to the local index; making anything PUBLIC —
//! pushing the repo, crates.io, a hosted registry — is a human decision behind
//! the legal review (GOVERNANCE.md hard gate 1). NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI) + item 6
//!   (pack versioning + content-addressed supersession).
//! - 2026-07-07: council-audit integrity fixes — publish now CRYPTOGRAPHICALLY
//!   verifies an embedded signature (not just its sha), errors on a
//!   wrong-typed/duplicate version in the existing index instead of coercing it
//!   to 0, and stores the pack path RELATIVE to the index (no absolute-path leak).

/// The pack path recorded in the index, relative to the index file's directory
/// (council-audit: absolute local paths must not leak into a shareable index).
/// Falls back to the file name when no relative path can be computed.
fn index_relative_path(pack: &::std::path::Path, index_path: &::std::path::Path) -> ::std::string::String {
  let base = index_path.parent().unwrap_or_else(|| ::std::path::Path::new("."));
  if let ::std::result::Result::Ok(rel) = pack.strip_prefix(base) {
    return rel.display().to_string();
  }
  pack
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_else(|| pack.display().to_string())
}

/// Publish `pack` under `id` (canonically `<profile>/<domain>`) into the index
//  at `index_path`; returns the new entry.
pub fn publish(
  pack: &::std::path::Path,
  index_path: &::std::path::Path,
  id: &str,
  published_at_unix: u64,
) -> ::std::result::Result<::serde_json::Value, ::std::string::String> {
  let bytes =
    ::std::fs::read(pack).map_err(|e| ::std::format!("read {}: {e}", pack.display()))?;
  let sha256 = crate::keys::sha256_hex(&bytes);

  let mut index: ::serde_json::Value = match ::std::fs::read_to_string(index_path) {
    ::std::result::Result::Ok(raw) => {
      ::serde_json::from_str(&raw).map_err(|e| ::std::format!("parse {}: {e}", index_path.display()))?
    }
    ::std::result::Result::Err(_) => ::serde_json::json!({
      "index_version": "rlps-pack-index/0.1",
      "packs": [],
    }),
  };

  // Latest prior version for this id (content-addressed supersession chain).
  // A wrong-typed version on a matching id is a CORRUPT index, not a v0 — refuse
  // rather than silently mint a contradictory chain (council-audit integrity).
  let packs = index["packs"].as_array().ok_or("index.packs must be an array")?;
  let mut latest_version = 0u64;
  let mut latest_sha: ::std::option::Option<::std::string::String> = ::std::option::Option::None;
  let mut seen_versions: ::std::collections::BTreeSet<u64> = ::std::collections::BTreeSet::new();
  for entry in packs {
    if entry["id"].as_str() != ::std::option::Option::Some(id) {
      continue;
    }
    let version = entry["version"].as_u64().ok_or_else(|| {
      ::std::format!("corrupt index: entry for {id} has a non-integer version {}", entry["version"])
    })?;
    if !seen_versions.insert(version) {
      return ::std::result::Result::Err(::std::format!(
        "corrupt index: duplicate version {version} for {id}"
      ));
    }
    if version >= latest_version {
      latest_version = version;
      latest_sha = entry["sha256"].as_str().map(::std::string::String::from);
    }
  }
  if latest_sha.as_deref() == ::std::option::Option::Some(sha256.as_str()) {
    return ::std::result::Result::Err(::std::format!(
      "{id} @ {sha256} is already the latest published version (v{latest_version})"
    ));
  }

  let mut entry = ::serde_json::json!({
    "id": id,
    "version": latest_version + 1,
    "sha256": sha256,
    "path": index_relative_path(pack, index_path),
    "published_at_unix": published_at_unix,
    "supersedes": latest_sha,
  });

  // Embed the detached signature when present (spec §6 envelope). The signature
  // is CRYPTOGRAPHICALLY verified (Ed25519 over the pack bytes) before embedding
  // — a sha match alone is not attestation; a forged sig must never enter the
  // index as apparently-attested (council-audit).
  let sig_path = ::std::path::PathBuf::from(::std::format!("{}.sig", pack.display()));
  if sig_path.is_file() {
    crate::keys::verify_file(pack, &sig_path)
      .map_err(|e| ::std::format!("refusing to publish: signature does not verify ({e})"))?;
    let sig_raw = ::std::fs::read_to_string(&sig_path)
      .map_err(|e| ::std::format!("read {}: {e}", sig_path.display()))?;
    let sig_doc: ::serde_json::Value =
      ::serde_json::from_str(&sig_raw).map_err(|e| ::std::format!("parse {}: {e}", sig_path.display()))?;
    entry["signature"] = sig_doc;
  }

  index["packs"]
    .as_array_mut()
    .ok_or("index.packs must be an array")?
    .push(entry.clone());
  if let ::std::option::Option::Some(parent) = index_path.parent() {
    ::std::fs::create_dir_all(parent).map_err(|e| ::std::format!("mkdir {}: {e}", parent.display()))?;
  }
  let pretty =
    ::serde_json::to_string_pretty(&index).expect("Value serialization cannot fail");
  ::std::fs::write(index_path, pretty)
    .map_err(|e| ::std::format!("write {}: {e}", index_path.display()))?;
  ::std::result::Result::Ok(entry)
}

#[cfg(test)]
mod tests {
  /// Why: the supersession chain IS the spec §7 text-in-effect story — a broken
  /// version increment or supersedes pointer would corrupt the audit trail
  /// receipts cite by sha, and silent double-publishing of identical bytes
  /// would mint phantom versions.
  #[test]
  fn publish_chains_supersession_and_rejects_identical_bytes() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let pack = tmp.path().join("recording_consent.r14n.toml");
    let index = tmp.path().join("registry").join("index.json");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write v1");

    let e1 = super::publish(&pack, &index, "aggressive/recording_consent", 1_783_641_600)
      .expect("publish v1");
    ::std::assert_eq!(e1["version"], 1);
    ::std::assert_eq!(e1["supersedes"], ::serde_json::Value::Null);
    let sha1 = ::std::string::String::from(e1["sha256"].as_str().expect("sha"));

    // Identical bytes ⇒ refuse (content-addressed idempotence).
    let err = super::publish(&pack, &index, "aggressive/recording_consent", 1_783_641_601)
      .expect_err("identical republish must fail");
    ::std::assert!(err.contains("already the latest"), "{err}");

    // Changed bytes ⇒ v2 supersedes v1's sha.
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\ndescription = \"v2\"\n")
      .expect("write v2");
    let e2 = super::publish(&pack, &index, "aggressive/recording_consent", 1_783_641_700)
      .expect("publish v2");
    ::std::assert_eq!(e2["version"], 2);
    ::std::assert_eq!(e2["supersedes"].as_str(), ::std::option::Option::Some(sha1.as_str()));

    // A different id starts its own chain.
    let e3 = super::publish(&pack, &index, "minimal/recording_consent", 1_783_641_800)
      .expect("publish other id");
    ::std::assert_eq!(e3["version"], 1);
  }

  /// Why: an index entry with a STALE signature (signed bytes ≠ published
  /// bytes) would look attested while attesting nothing — publish must embed
  /// only a sha-matching sig and hard-refuse otherwise (spec §6 envelope).
  #[test]
  fn publish_embeds_matching_detached_signature_and_rejects_stale_one() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, _pub) = crate::keys::keygen(&tmp.path().join("k")).expect("keygen");
    let pack = tmp.path().join("p.r14n.toml");
    let index = tmp.path().join("index.json");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write");
    crate::keys::sign_file(&pack, &seed, ::std::option::Option::None).expect("sign");

    let entry = super::publish(&pack, &index, "aggressive/p", 1).expect("publish signed");
    ::std::assert!(entry["signature"]["signature_ed25519"].is_string());

    // Change the pack but keep the OLD sig ⇒ publish must refuse.
    ::std::fs::write(&pack, "[meta]\nstrictness = \"minimal\"\n").expect("tamper");
    let err = super::publish(&pack, &index, "aggressive/p", 2).expect_err("stale sig");
    ::std::assert!(err.contains("does not verify"), "{err}");
  }

  /// Why: council-audit — a `.sig` with a MATCHING sha256 but a forged/garbage
  /// Ed25519 signature previously published as apparently-attested (exit 0),
  /// because publish checked only the hash. It must cryptographically verify and
  /// refuse; the index is the artifact receipts cite for provenance.
  #[test]
  fn publish_refuses_a_forged_signature_with_matching_sha() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let pack = tmp.path().join("p.r14n.toml");
    let index = tmp.path().join("index.json");
    let bytes = b"[meta]\nstrictness = \"aggressive\"\n";
    ::std::fs::write(&pack, bytes).expect("write");
    let sha = crate::keys::sha256_hex(bytes);
    // Correct sha, but public key / signature are structurally-valid-length garbage.
    let forged = ::serde_json::json!({
      "schema": "rlps-sig/0.1",
      "sha256": sha,
      "public_key_ed25519": "A".repeat(43),
      "signature_ed25519": "B".repeat(86),
    });
    ::std::fs::write(
      ::std::format!("{}.sig", pack.display()),
      ::serde_json::to_string(&forged).expect("json"),
    )
    .expect("write forged sig");
    let err = super::publish(&pack, &index, "forged/test", 1).expect_err("forged sig must fail");
    ::std::assert!(err.contains("does not verify"), "{err}");
    ::std::assert!(!index.exists(), "a refused publish must not write the index");
  }

  /// Why: council-audit — a wrong-typed version (e.g. the string "3", trivial to
  /// hand-edit into the committed index) was read as 0, so publish minted a v1
  /// that "supersedes" a v3 — a self-contradictory chain. A corrupt version must
  /// be refused, and duplicate versions for one id rejected.
  #[test]
  fn publish_refuses_corrupt_or_duplicate_versions() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let pack = tmp.path().join("p.r14n.toml");
    let index = tmp.path().join("index.json");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write");
    ::std::fs::write(
      &index,
      "{\"index_version\":\"rlps-pack-index/0.1\",\"packs\":[\
        {\"id\":\"x/y\",\"version\":\"3\",\"sha256\":\"deadbeef\",\"path\":\"p\",\"published_at_unix\":1,\"supersedes\":null}]}",
    )
    .expect("seed corrupt index");
    let err = super::publish(&pack, &index, "x/y", 2).expect_err("corrupt version");
    ::std::assert!(err.contains("non-integer version"), "{err}");
  }

  /// Why: council-audit — the index is a shareable artifact; storing an absolute
  /// local path leaks the operator's directory layout. The path must be recorded
  /// relative to the index's own directory.
  #[test]
  fn publish_stores_index_relative_path_not_absolute() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let sub = tmp.path().join("packs").join("demo");
    ::std::fs::create_dir_all(&sub).expect("mkdir");
    let pack = sub.join("recording_consent.r14n.toml");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write");
    let index = tmp.path().join("registry").join("index.json");
    // pack is passed ABSOLUTE (as scripts/CI do).
    let entry = super::publish(&pack, &index, "demo/recording_consent", 1).expect("publish");
    let stored = entry["path"].as_str().expect("path");
    ::std::assert!(!stored.starts_with('/'), "stored path must be relative, got {stored}");
    ::std::assert!(!stored.contains(tmp.path().to_str().unwrap()), "must not leak the abs prefix");
  }
}
