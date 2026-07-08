// SPDX-License-Identifier: Apache-2.0
//! Reviewer-key directory verification (spec §6 trust root; council-audit M9).
//!
//! Loads a reviewer-key directory (schema `/registry/reviewer-key.schema.json`)
//! and answers: is the key that signed a pack a TRUSTED signer — listed,
//! non-revoked, non-expired, and (optionally) scoped to the pack's jurisdiction
//! — as of a decision date? This is the tooling half of the trust root; the
//! generic resolver stays directory-agnostic (`PROVENANCE_VERIFIED = false`),
//! and this CLI check is how an operator upgrades a decision out of
//! advisory-only. A directory entry proves WHO may attest, never that a review
//! was correct. NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-07-07: authored — council-audit M9 (reviewer-key directory tooling).
//! - 2026-07-07: council-audit D2 — steward directory signing/verification over
//!   the RFC-8785 (JCS) canonical form with the `signature_ed25519` field absent.

/// Steward-sign a reviewer-key directory: canonicalizes the document (JCS) with
/// `steward.signature_ed25519` ABSENT, signs it, and writes the signature back.
/// The directory MUST already carry `steward.identity` + `steward.public_key_ed25519`.
pub fn sign_directory(
  directory_path: &::std::path::Path,
  seed_path: &::std::path::Path,
) -> ::std::result::Result<(), ::std::string::String> {
  let raw = ::std::fs::read_to_string(directory_path)
    .map_err(|e| ::std::format!("read {}: {e}", directory_path.display()))?;
  let mut doc: ::serde_json::Value =
    ::serde_json::from_str(&raw).map_err(|e| ::std::format!("parse directory: {e}"))?;
  if doc["steward"]["public_key_ed25519"].as_str().is_none() {
    return ::std::result::Result::Err(::std::string::String::from(
      "directory has no steward.public_key_ed25519 — add the steward block before signing",
    ));
  }
  // Canonicalize with the signature field absent (schema requirement).
  if let ::std::option::Option::Some(steward) = doc["steward"].as_object_mut() {
    steward.remove("signature_ed25519");
  }
  let canonical = crate::jcs::canonicalize(&doc)?;
  let (public_key, signature) = crate::keys::sign_payload(seed_path, canonical.as_bytes())?;
  if doc["steward"]["public_key_ed25519"].as_str() != ::std::option::Option::Some(public_key.as_str()) {
    return ::std::result::Result::Err(::std::format!(
      "seed's public key {public_key} does not match steward.public_key_ed25519 in the directory"
    ));
  }
  doc["steward"]["signature_ed25519"] = ::serde_json::Value::String(signature);
  let pretty =
    ::serde_json::to_string_pretty(&doc).map_err(|e| ::std::format!("serialize: {e}"))?;
  ::std::fs::write(directory_path, pretty)
    .map_err(|e| ::std::format!("write {}: {e}", directory_path.display()))
}

/// Verify a directory's steward signature over its JCS canonical form. Returns
/// `Ok(false)` when the directory carries no steward signature (unsigned — the
/// caller decides whether that is acceptable); `Ok(true)` when a valid steward
/// signature is present; `Err` when a signature is present but invalid.
pub fn verify_directory_steward(
  directory_path: &::std::path::Path,
) -> ::std::result::Result<bool, ::std::string::String> {
  let raw = ::std::fs::read_to_string(directory_path)
    .map_err(|e| ::std::format!("read {}: {e}", directory_path.display()))?;
  let doc: ::serde_json::Value =
    ::serde_json::from_str(&raw).map_err(|e| ::std::format!("parse directory: {e}"))?;
  let sig = match doc["steward"]["signature_ed25519"].as_str() {
    ::std::option::Option::Some(s) => s,
    ::std::option::Option::None => return ::std::result::Result::Ok(false),
  };
  let public_key = doc["steward"]["public_key_ed25519"]
    .as_str()
    .ok_or("steward signature present but no steward.public_key_ed25519")?;
  let mut unsigned = doc.clone();
  if let ::std::option::Option::Some(steward) = unsigned["steward"].as_object_mut() {
    steward.remove("signature_ed25519");
  }
  let canonical = crate::jcs::canonicalize(&unsigned)?;
  crate::keys::verify_payload(public_key, sig, canonical.as_bytes())
    .map_err(|e| ::std::format!("steward signature does not verify: {e}"))?;
  ::std::result::Result::Ok(true)
}

/// The trust status of a signing key against a directory, as of a date.
#[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
pub enum KeyStatus {
  /// Listed, in-validity, not revoked (and jurisdiction-matched if required).
  Trusted { key_id: ::std::string::String, jurisdiction: ::std::string::String },
  /// Listed but revoked on/after `revoked_at`.
  Revoked { key_id: ::std::string::String, revoked_at: ::std::string::String, reason: ::std::string::String },
  /// Listed but past `valid_until` (treated as revoked for new decisions, §6).
  Expired { key_id: ::std::string::String, valid_until: ::std::string::String },
  /// Listed but the decision date precedes `valid_from`.
  NotYetValid { key_id: ::std::string::String, valid_from: ::std::string::String },
  /// Listed + in-validity, but scoped to a different jurisdiction than required.
  JurisdictionMismatch { key_id: ::std::string::String, key_jurisdiction: ::std::string::String, required: ::std::string::String },
  /// No directory entry carries this public key.
  Unknown,
}

impl KeyStatus {
  /// True only for `Trusted` — the caller escapes advisory-only iff this holds.
  pub fn is_trusted(&self) -> bool {
    ::std::matches!(self, KeyStatus::Trusted { .. })
  }
}

/// Resolve the status of the base64 `public_key` against the directory file, as
/// of ISO date `as_of`, optionally requiring the key's jurisdiction to equal
/// `required_jurisdiction`.
pub fn key_status(
  directory_path: &::std::path::Path,
  public_key: &str,
  as_of: &str,
  required_jurisdiction: ::std::option::Option<&str>,
) -> ::std::result::Result<KeyStatus, ::std::string::String> {
  let raw = ::std::fs::read_to_string(directory_path)
    .map_err(|e| ::std::format!("read {}: {e}", directory_path.display()))?;
  let doc: ::serde_json::Value =
    ::serde_json::from_str(&raw).map_err(|e| ::std::format!("parse directory: {e}"))?;
  let keys = doc["keys"].as_array().ok_or("directory has no `keys` array")?;

  let entry = match keys
    .iter()
    .find(|k| k["public_key_ed25519"].as_str() == ::std::option::Option::Some(public_key))
  {
    ::std::option::Option::Some(e) => e,
    ::std::option::Option::None => return ::std::result::Result::Ok(KeyStatus::Unknown),
  };
  let key_id = ::std::string::String::from(entry["key_id"].as_str().unwrap_or("<unknown>"));
  let jurisdiction = ::std::string::String::from(entry["jurisdiction"].as_str().unwrap_or(""));

  // Revocation wins (ISO dates compare lexically). Revoked on/after revoked_at.
  if let ::std::option::Option::Some(rev) = entry.get("revocation") {
    let revoked_at = rev["revoked_at"].as_str().unwrap_or("");
    if !revoked_at.is_empty() && as_of >= revoked_at {
      return ::std::result::Result::Ok(KeyStatus::Revoked {
        key_id,
        revoked_at: ::std::string::String::from(revoked_at),
        reason: ::std::string::String::from(rev["reason"].as_str().unwrap_or("unspecified")),
      });
    }
  }
  if let ::std::option::Option::Some(until) = entry["valid_until"].as_str() {
    if as_of > until {
      return ::std::result::Result::Ok(KeyStatus::Expired {
        key_id,
        valid_until: ::std::string::String::from(until),
      });
    }
  }
  if let ::std::option::Option::Some(from) = entry["valid_from"].as_str() {
    if as_of < from {
      return ::std::result::Result::Ok(KeyStatus::NotYetValid {
        key_id,
        valid_from: ::std::string::String::from(from),
      });
    }
  }
  if let ::std::option::Option::Some(req) = required_jurisdiction {
    if jurisdiction != req {
      return ::std::result::Result::Ok(KeyStatus::JurisdictionMismatch {
        key_id,
        key_jurisdiction: jurisdiction,
        required: ::std::string::String::from(req),
      });
    }
  }
  ::std::result::Result::Ok(KeyStatus::Trusted { key_id, jurisdiction })
}

#[cfg(test)]
mod tests {
  fn directory() -> (::tempfile::TempDir, ::std::path::PathBuf) {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let path = tmp.path().join("directory.json");
    ::std::fs::write(
      &path,
      "{\"directory_version\":\"1\",\"keys\":[\
        {\"key_id\":\"reviewer-1\",\"public_key_ed25519\":\"AAA\",\"reviewer_identity\":\"Counsel LLP\",\
         \"jurisdiction\":\"US-CA\",\"credential_type\":\"bar_license\",\"credential_id\":\"1\",\
         \"valid_from\":\"2026-01-01\",\"valid_until\":\"2026-12-31\"},\
        {\"key_id\":\"reviewer-2\",\"public_key_ed25519\":\"BBB\",\"reviewer_identity\":\"Lapsed LLP\",\
         \"jurisdiction\":\"US-CA\",\"credential_type\":\"bar_license\",\"credential_id\":\"2\",\
         \"valid_from\":\"2026-01-01\",\"revocation\":{\"revoked_at\":\"2026-06-01\",\"reason\":\"credential_lapsed\"}}]}",
    )
    .expect("write directory");
    (tmp, path)
  }

  /// Why: spec §6 — a listed, in-validity key is the ONLY thing that upgrades a
  /// decision out of advisory-only; a jurisdiction-scoped key must also match
  /// the pack's jurisdiction or the trust does not transfer.
  #[test]
  fn listed_in_validity_matching_jurisdiction_is_trusted() {
    let (_t, dir) = directory();
    let s = super::key_status(&dir, "AAA", "2026-06-15", ::std::option::Option::Some("US-CA")).expect("status");
    ::std::assert!(s.is_trusted(), "{s:?}");
    // Wrong jurisdiction ⇒ not trusted.
    let s = super::key_status(&dir, "AAA", "2026-06-15", ::std::option::Option::Some("DE")).expect("status");
    ::std::assert_eq!(
      s,
      super::KeyStatus::JurisdictionMismatch {
        key_id: ::std::string::String::from("reviewer-1"),
        key_jurisdiction: ::std::string::String::from("US-CA"),
        required: ::std::string::String::from("DE"),
      }
    );
  }

  /// Why: spec §6 revocation semantics — from revoked_at forward the key is
  /// untrusted; a decision dated before revocation is still trusted (revocation
  /// is not retroactive). Both directions must hold.
  #[test]
  fn revocation_is_forward_dated_not_retroactive() {
    let (_t, dir) = directory();
    let after = super::key_status(&dir, "BBB", "2026-07-01", ::std::option::Option::None).expect("status");
    ::std::assert!(::std::matches!(after, super::KeyStatus::Revoked { .. }), "{after:?}");
    let before = super::key_status(&dir, "BBB", "2026-05-01", ::std::option::Option::None).expect("status");
    ::std::assert!(before.is_trusted(), "pre-revocation decision stays trusted: {before:?}");
  }

  /// Why: spec §6 — expiry (`valid_until` passed) is handled identically to
  /// revocation for new decisions; a not-yet-valid key is likewise untrusted.
  #[test]
  fn expiry_and_not_yet_valid_are_untrusted() {
    let (_t, dir) = directory();
    let expired = super::key_status(&dir, "AAA", "2027-01-01", ::std::option::Option::None).expect("status");
    ::std::assert!(::std::matches!(expired, super::KeyStatus::Expired { .. }), "{expired:?}");
    let early = super::key_status(&dir, "AAA", "2025-06-01", ::std::option::Option::None).expect("status");
    ::std::assert!(::std::matches!(early, super::KeyStatus::NotYetValid { .. }), "{early:?}");
  }

  /// Why: a key absent from the directory must be Unknown (advisory), never
  /// silently treated as trusted — that is the whole point of the directory.
  #[test]
  fn unlisted_key_is_unknown() {
    let (_t, dir) = directory();
    let s = super::key_status(&dir, "ZZZ", "2026-06-15", ::std::option::Option::None).expect("status");
    ::std::assert_eq!(s, super::KeyStatus::Unknown);
  }

  /// Why: council-audit D2 — the reviewer-key.schema.json REQUIRES a steward
  /// signature over the JCS canonical form; without producing + verifying one,
  /// the trust root is unattested. A steward-sign→verify roundtrip must hold,
  /// and any post-signing tamper must break verification.
  #[test]
  fn steward_sign_verify_roundtrip_and_tamper_detection() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, pub_path) = crate::keys::keygen(&tmp.path().join("steward")).expect("keygen");
    let steward_pub = ::std::fs::read_to_string(&pub_path).expect("pub");
    let dir = tmp.path().join("directory.json");
    ::std::fs::write(
      &dir,
      ::std::format!(
        "{{\"directory_version\":\"1\",\"steward\":{{\"identity\":\"RLPS Steward\",\
          \"public_key_ed25519\":\"{steward_pub}\"}},\"keys\":[]}}"
      ),
    )
    .expect("write directory");
    // Unsigned ⇒ verify returns Ok(false).
    ::std::assert_eq!(super::verify_directory_steward(&dir).expect("verify"), false);
    super::sign_directory(&dir, &seed).expect("sign directory");
    ::std::assert_eq!(super::verify_directory_steward(&dir).expect("verify signed"), true);
    // Tamper: add a key after signing ⇒ steward signature must no longer verify.
    let mut doc: ::serde_json::Value =
      ::serde_json::from_str(&::std::fs::read_to_string(&dir).expect("read")).expect("json");
    doc["keys"].as_array_mut().expect("keys").push(::serde_json::json!({
      "key_id": "sneaked-in", "public_key_ed25519": "AAA", "reviewer_identity": "x",
      "jurisdiction": "US-CA", "credential_type": "bar_license", "credential_id": "1",
      "valid_from": "2026-01-01"
    }));
    ::std::fs::write(&dir, doc.to_string()).expect("write tampered");
    ::std::assert!(super::verify_directory_steward(&dir).is_err(), "tampered directory must fail");
  }

  /// Why: signing with a seed whose public key is not the declared steward key
  /// must be refused — otherwise anyone could "steward-sign" a directory.
  #[test]
  fn signing_with_a_non_steward_key_is_refused() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, _p) = crate::keys::keygen(&tmp.path().join("wrong")).expect("keygen");
    let dir = tmp.path().join("directory.json");
    ::std::fs::write(
      &dir,
      "{\"directory_version\":\"1\",\"steward\":{\"identity\":\"S\",\"public_key_ed25519\":\"AAA\"},\"keys\":[]}",
    )
    .expect("write");
    ::std::assert!(super::sign_directory(&dir, &seed).is_err(), "wrong key must be refused");
  }
}
