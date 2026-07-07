//! `r14n publish` — content-addressed supersession into the LOCAL registry
//! index (`/registry/index.json`, schema `/registry/pack-index.schema.json`).
//!
//! COUNSEL-GATED BY DESIGN: this never touches a network. "Publishing" appends
//! a version entry (id, monotonic version, sha256, supersedes-chain, optional
//! detached-signature embed) to the local index; making anything PUBLIC —
//! pushing the repo, crates.io, a hosted registry — is a human decision behind
//! the legal review (maintainer-notes governing constraint 3). NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI) + item 6
//!   (pack versioning + content-addressed supersession).

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
  let packs = index["packs"].as_array().ok_or("index.packs must be an array")?;
  let mut latest_version = 0u64;
  let mut latest_sha: ::std::option::Option<::std::string::String> = ::std::option::Option::None;
  for entry in packs {
    if entry["id"].as_str() == ::std::option::Option::Some(id) {
      let version = entry["version"].as_u64().unwrap_or(0);
      if version >= latest_version {
        latest_version = version;
        latest_sha = entry["sha256"].as_str().map(::std::string::String::from);
      }
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
    "path": pack.display().to_string(),
    "published_at_unix": published_at_unix,
    "supersedes": latest_sha,
  });

  // Embed the detached signature when present (spec §6 envelope).
  let sig_path = ::std::path::PathBuf::from(::std::format!("{}.sig", pack.display()));
  if let ::std::result::Result::Ok(sig_raw) = ::std::fs::read_to_string(&sig_path) {
    let sig_doc: ::serde_json::Value =
      ::serde_json::from_str(&sig_raw).map_err(|e| ::std::format!("parse {}: {e}", sig_path.display()))?;
    if sig_doc["sha256"].as_str() != ::std::option::Option::Some(sha256.as_str()) {
      return ::std::result::Result::Err(::std::string::String::from(
        "detached signature does not match the pack bytes — re-sign before publishing",
      ));
    }
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
    ::std::assert!(err.contains("re-sign"), "{err}");
  }
}
