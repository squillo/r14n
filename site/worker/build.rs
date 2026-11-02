// SPDX-License-Identifier: Apache-2.0
//! Derives the two ETags from the vocabulary bytes this crate embeds, so they cannot go stale.
//!
//! The JavaScript Worker pinned both tags as hand-copied constants and relied on a sentence in a
//! README to remember to update them after any edit to `ns/`. Here a tag is a function of the
//! file: the first 32 lowercase hex characters of its sha256, in double quotes -- the exact
//! format the live Worker already serves, so clients holding a cached tag keep validating.
//!
//! Revision History
//! - 2026-11-02: authored -- replaces the hand-pinned constants in `site/src/index.js`.

/// Each constant this script defines, with the vocabulary file it describes. Paths are relative to
/// this crate's directory (where Cargo runs build scripts); `src/site.rs` embeds the same files,
/// and a test recomputes both tags independently from the files on disk.
const VOCABULARY: [(&str, &str); 2] = [
  ("ETAG_JSONLD", "../../ns/context.jsonld"),
  ("ETAG_TTL", "../../ns/rlps.ttl"),
];

/// `"` + first 32 lowercase hex characters of the sha256 + `"`.
fn etag(bytes: &[u8]) -> ::std::string::String {
  let digest = <::sha2::Sha256 as ::sha2::Digest>::digest(bytes);
  let hex: ::std::string::String =
    digest.iter().take(16).map(|b| ::std::format!("{b:02x}")).collect();
  ::std::format!("\"{hex}\"")
}

fn main() {
  let consts: ::std::string::String = VOCABULARY
    .iter()
    .map(|(name, path)| {
      ::std::println!("cargo:rerun-if-changed={path}");
      let bytes = ::std::fs::read(path).unwrap_or_else(|e| ::std::panic!("read {path}: {e}"));
      ::std::format!("const {name}: &str = {:?};\n", etag(&bytes))
    })
    .collect();
  let out = ::std::path::Path::new(&::std::env::var("OUT_DIR").expect("Cargo sets OUT_DIR"))
    .join("etags.rs");
  ::std::fs::write(&out, consts).unwrap_or_else(|e| ::std::panic!("write {}: {e}", out.display()));
}
