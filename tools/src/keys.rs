//! `r14n keygen` / `sign` / `verify` — Ed25519 pack signing (spec §6).
//!
//! Signatures are DETACHED (`<pack>.sig`, JSON): signing the exact pack bytes
//! avoids the sign-a-file-containing-its-own-signature circularity and keeps
//! the pack byte-identical to what reviewers read. A signature proves WHO
//! signed, never that the review was correct (NOT LEGAL ADVICE; spec §6:
//! the envelope, not the trust decision). Key encodings match
//! `/registry/reviewer-key.schema.json`: unpadded base64 (32-byte public key,
//! 64-byte signature); seeds are stored as 64 hex chars.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI).

fn hex_encode(bytes: &[u8]) -> ::std::string::String {
  bytes.iter().map(|b| ::std::format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> ::std::result::Result<::std::vec::Vec<u8>, ::std::string::String> {
  if s.len() % 2 != 0 {
    return ::std::result::Result::Err(::std::string::String::from("odd-length hex"));
  }
  (0..s.len())
    .step_by(2)
    .map(|i| {
      u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| ::std::format!("bad hex at {i}: {e}"))
    })
    .collect()
}

fn b64(bytes: &[u8]) -> ::std::string::String {
  ::base64::Engine::encode(&::base64::engine::general_purpose::STANDARD_NO_PAD, bytes)
}

fn b64_decode(s: &str) -> ::std::result::Result<::std::vec::Vec<u8>, ::std::string::String> {
  ::base64::Engine::decode(&::base64::engine::general_purpose::STANDARD_NO_PAD, s)
    .map_err(|e| ::std::format!("bad base64: {e}"))
}

/// SHA-256 of raw bytes, hex-encoded (the pack content address).
pub fn sha256_hex(bytes: &[u8]) -> ::std::string::String {
  let digest = <::sha2::Sha256 as ::sha2::Digest>::digest(bytes);
  hex_encode(&digest)
}

/// Generate a keypair: writes `<prefix>.seed` (hex, private — keep out of git)
/// and `<prefix>.pub` (unpadded base64, the reviewer-directory encoding).
pub fn keygen(
  prefix: &::std::path::Path,
) -> ::std::result::Result<(::std::path::PathBuf, ::std::path::PathBuf), ::std::string::String> {
  let mut rng = ::rand_core::OsRng;
  let signing = ::ed25519_dalek::SigningKey::generate(&mut rng);
  let seed_path = prefix.with_extension("seed");
  let pub_path = prefix.with_extension("pub");
  ::std::fs::write(&seed_path, hex_encode(&signing.to_bytes()))
    .map_err(|e| ::std::format!("write {}: {e}", seed_path.display()))?;
  ::std::fs::write(&pub_path, b64(signing.verifying_key().as_bytes()))
    .map_err(|e| ::std::format!("write {}: {e}", pub_path.display()))?;
  ::std::result::Result::Ok((seed_path, pub_path))
}

fn load_signing_key(
  seed_path: &::std::path::Path,
) -> ::std::result::Result<::ed25519_dalek::SigningKey, ::std::string::String> {
  let raw = ::std::fs::read_to_string(seed_path)
    .map_err(|e| ::std::format!("read {}: {e}", seed_path.display()))?;
  let bytes = hex_decode(raw.trim())?;
  let seed: [u8; 32] = bytes
    .try_into()
    .map_err(|_| ::std::string::String::from("seed must be exactly 32 bytes (64 hex chars)"))?;
  ::std::result::Result::Ok(::ed25519_dalek::SigningKey::from_bytes(&seed))
}

/// Sign a pack file: writes the detached `<pack>.sig` JSON and returns its path.
pub fn sign_file(
  pack: &::std::path::Path,
  seed_path: &::std::path::Path,
  key_id: ::std::option::Option<&str>,
) -> ::std::result::Result<::std::path::PathBuf, ::std::string::String> {
  let bytes =
    ::std::fs::read(pack).map_err(|e| ::std::format!("read {}: {e}", pack.display()))?;
  let signing = load_signing_key(seed_path)?;
  let signature = ::ed25519_dalek::Signer::sign(&signing, &bytes);
  let mut sig_doc = ::serde_json::json!({
    "schema": "rlps-sig/0.1",
    "sha256": sha256_hex(&bytes),
    "public_key_ed25519": b64(signing.verifying_key().as_bytes()),
    "signature_ed25519": b64(&signature.to_bytes()),
  });
  if let ::std::option::Option::Some(id) = key_id {
    sig_doc["key_id"] = ::serde_json::Value::String(::std::string::String::from(id));
  }
  let sig_path = ::std::path::PathBuf::from(::std::format!("{}.sig", pack.display()));
  let pretty = ::serde_json::to_string_pretty(&sig_doc)
    .expect("Value serialization cannot fail");
  ::std::fs::write(&sig_path, pretty)
    .map_err(|e| ::std::format!("write {}: {e}", sig_path.display()))?;
  ::std::result::Result::Ok(sig_path)
}

/// Verify a pack against its detached signature. Checks the SHA-256 content
/// address AND the Ed25519 signature over the exact bytes.
pub fn verify_file(
  pack: &::std::path::Path,
  sig_path: &::std::path::Path,
) -> ::std::result::Result<(), ::std::string::String> {
  let bytes =
    ::std::fs::read(pack).map_err(|e| ::std::format!("read {}: {e}", pack.display()))?;
  let sig_raw = ::std::fs::read_to_string(sig_path)
    .map_err(|e| ::std::format!("read {}: {e}", sig_path.display()))?;
  let sig_doc: ::serde_json::Value =
    ::serde_json::from_str(&sig_raw).map_err(|e| ::std::format!("parse sig: {e}"))?;
  let want_sha = sig_doc["sha256"].as_str().ok_or("sig missing sha256")?;
  let got_sha = sha256_hex(&bytes);
  if want_sha != got_sha {
    return ::std::result::Result::Err(::std::format!(
      "content address mismatch: sig has {want_sha}, file is {got_sha} — the pack changed after signing"
    ));
  }
  let pub_bytes: [u8; 32] = b64_decode(
    sig_doc["public_key_ed25519"].as_str().ok_or("sig missing public_key_ed25519")?,
  )?
  .try_into()
  .map_err(|_| ::std::string::String::from("public key must be 32 bytes"))?;
  let verifying = ::ed25519_dalek::VerifyingKey::from_bytes(&pub_bytes)
    .map_err(|e| ::std::format!("bad public key: {e}"))?;
  let sig_bytes = b64_decode(
    sig_doc["signature_ed25519"].as_str().ok_or("sig missing signature_ed25519")?,
  )?;
  let signature = ::ed25519_dalek::Signature::from_slice(&sig_bytes)
    .map_err(|e| ::std::format!("bad signature: {e}"))?;
  ::ed25519_dalek::Verifier::verify(&verifying, &bytes, &signature)
    .map_err(|e| ::std::format!("signature INVALID: {e}"))
}

#[cfg(test)]
mod tests {
  /// Why: sign/verify is the trust-root envelope (spec §6) — a roundtrip that
  /// silently failed to bind the exact bytes would let a pack change after
  /// counsel signed it. Tampering must fail on the CONTENT ADDRESS first (the
  /// loud, explainable error).
  #[test]
  fn keygen_sign_verify_roundtrip_and_tamper_detection() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, _pubkey) = super::keygen(&tmp.path().join("reviewer")).expect("keygen");
    let pack = tmp.path().join("recording_consent.r14n.toml");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write pack");
    let sig = super::sign_file(&pack, &seed, ::std::option::Option::Some("reviewer-1"))
      .expect("sign");
    super::verify_file(&pack, &sig).expect("verify fresh signature");
    // Tamper ⇒ both the content address and the signature must fail.
    ::std::fs::write(&pack, "[meta]\nstrictness = \"minimal\"\n").expect("tamper");
    let err = super::verify_file(&pack, &sig).expect_err("tampered pack must fail");
    ::std::assert!(err.contains("content address mismatch"), "{err}");
  }

  /// Why: a matching sha256 with a corrupted signature is the forgery-shaped
  /// failure (content untouched, attestation invalid) — verify must reject on
  /// the Ed25519 check itself, not only on the cheaper hash comparison.
  #[test]
  fn corrupted_signature_with_matching_sha_still_fails_verification() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, _pubkey) = super::keygen(&tmp.path().join("k")).expect("keygen");
    let pack = tmp.path().join("p.r14n.toml");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write");
    let sig_path = super::sign_file(&pack, &seed, ::std::option::Option::None).expect("sign");
    // Corrupt ONLY the signature field; sha256 + public key stay valid.
    let mut doc: ::serde_json::Value =
      ::serde_json::from_str(&::std::fs::read_to_string(&sig_path).expect("read sig"))
        .expect("sig json");
    let sig = ::std::string::String::from(doc["signature_ed25519"].as_str().expect("sig"));
    let flipped = if sig.starts_with('A') { "B" } else { "A" };
    doc["signature_ed25519"] = ::serde_json::Value::String(
      ::std::format!("{flipped}{}", &sig[1..]),
    );
    ::std::fs::write(&sig_path, doc.to_string()).expect("write corrupted sig");
    let err = super::verify_file(&pack, &sig_path).expect_err("corrupted sig must fail");
    ::std::assert!(err.contains("signature INVALID"), "{err}");
  }

  /// Why: keygen/sign outputs feed the reviewer-key directory — if the base64
  /// encodings drifted from reviewer-key.schema.json's unpadded 43/86-char
  /// patterns, published directories would fail schema validation downstream.
  #[test]
  fn sig_document_matches_registry_encodings() {
    let tmp = ::tempfile::tempdir().expect("tmp");
    let (seed, pub_path) = super::keygen(&tmp.path().join("k")).expect("keygen");
    let pack = tmp.path().join("p.r14n.toml");
    ::std::fs::write(&pack, "[meta]\nstrictness = \"aggressive\"\n").expect("write");
    let sig = super::sign_file(&pack, &seed, ::std::option::Option::None).expect("sign");
    let doc: ::serde_json::Value =
      ::serde_json::from_str(&::std::fs::read_to_string(&sig).expect("read sig")).expect("json");
    // Unpadded base64: 43 chars for 32 bytes, 86 for 64 (reviewer-key.schema.json).
    ::std::assert_eq!(doc["public_key_ed25519"].as_str().expect("pub").len(), 43);
    ::std::assert_eq!(doc["signature_ed25519"].as_str().expect("sig").len(), 86);
    ::std::assert_eq!(
      doc["public_key_ed25519"].as_str().expect("pub"),
      ::std::fs::read_to_string(&pub_path).expect("read pub"),
      "sig embeds the same public key keygen wrote"
    );
  }
}
