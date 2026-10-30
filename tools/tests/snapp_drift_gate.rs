//! The N Lang Snapp sources and their committed artifacts still agree, or this fails.
//!
//! WHY THIS EXISTS, and why it does not run `nlang`.
//!
//! The Snapp tree was authored and then built by nothing — no CI job, no Makefile, no test
//! referenced it. By the time anyone compiled it again, four separate pieces of language
//! drift had accumulated, ten authored packs had never once compiled, and the committed
//! bundle claimed a version its own filename contradicted. Every one of those was silent.
//!
//! N Lang is proprietary and this repository's CI runs on public runners, so a gate here
//! cannot invoke the compiler. It does not need to. Every defect in that list was an
//! AGREEMENT failure between files that are all committed, and agreement is checkable with
//! a hash and a JSON parser:
//!
//!   1. a pack on disk that no index declares will never load — discovery is index-only,
//!      with no filesystem-walk fallback, so the file is simply ignored
//!   2. an index declaration with no file behind it is a hard build error, found only at
//!      build time
//!   3. a committed bundle missing a declared pack is a stale artifact: the source moved
//!      and the emit did not follow
//!   4. a bundle whose `@snapp.version` contradicts its filename misleads every consumer
//!      that pins by filename
//!   5. an `integrity` pin that does not match its target's current bytes breaks the
//!      dependent's build; re-emitting a framework bundle invalidates every pin on it
//!   6. two `@`-bearing JSONs for one Snapp name is an ambiguous artifact, and the
//!      build-door sidecar carries no `@snapp` root at all, so a consumer that globbed it
//!      would load a bundle with no identity
//!
//! What this CANNOT do is prove the committed bundle is what the current sources would
//! emit. That needs the compiler. Run the full pipeline locally before committing a source
//! change — `AGENTS.md` carries the guarded invocation.
//!
//! This was a Python script until 2026-10-16. Python is not a tooling language here; it
//! lives in `tools/` because `tools/` already carries `sha2`, so the port added no
//! dependency anywhere, and because the existing `tools` CI job runs it with no new job to
//! wire up.
//!
//! Style note: FQ paths, no module-top `use` — the house style of this tree.
//!
//! Revision History
//! - 2026-10-16: ported from `.github/scripts/snapp_drift_gate.py`.

mod common;

fn sha256_hex(rel_path: &::std::path::Path) -> ::std::string::String {
  let bytes = ::std::fs::read(rel_path)
    .unwrap_or_else(|e| ::std::panic!("read {}: {e}", rel_path.display()));
  let digest = <::sha2::Sha256 as ::sha2::Digest>::digest(&bytes);
  digest.iter().map(|b| ::std::format!("{b:02x}")).collect()
}

/// `mod NAME {};` declarations in an index, in declaration order.
fn declared_mods(index_rel: &str) -> ::std::vec::Vec<::std::string::String> {
  common::read(index_rel)
    .lines()
    .filter_map(|line| {
      let t = line.trim();
      let rest = t.strip_prefix("mod ")?;
      let name = rest.split_whitespace().next()?;
      // Only a bare `mod NAME {};` declaration — not `mod * { … }` re-exports.
      if name == "*" || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return ::std::option::Option::None;
      }
      if !rest.contains("{}") {
        return ::std::option::Option::None;
      }
      ::std::option::Option::Some(::std::string::String::from(name))
    })
    .collect()
}

/// `pub @use NAME {}` re-exports in an index, in order. A `mod` declaration compiles a file; only
/// a re-export makes what it defines visible to a Snapp that `@deps` this one.
fn reexported_mods(index_rel: &str) -> ::std::vec::Vec<::std::string::String> {
  common::read(index_rel)
    .lines()
    .filter_map(|line| {
      let rest = line.trim().strip_prefix("pub @use ")?;
      ::std::option::Option::Some(::std::string::String::from(rest.split_whitespace().next()?))
    })
    .collect()
}

fn files_with_suffix(dir_rel: &str, suffix: &str) -> ::std::vec::Vec<::std::string::String> {
  let dir = common::root().join(dir_rel);
  let mut out: ::std::vec::Vec<::std::string::String> = ::std::fs::read_dir(&dir)
    .unwrap_or_else(|e| ::std::panic!("read_dir {}: {e}", dir.display()))
    .filter_map(|e| e.ok())
    .filter_map(|e| {
      let name = e.file_name().to_string_lossy().into_owned();
      name.strip_suffix(suffix).map(::std::string::String::from)
    })
    .collect();
  out.sort();
  out
}

/// Why: a pack on disk that no index declares is compiled by NOTHING — discovery is
/// index-only with no filesystem-walk fallback — so everything it declares goes
/// unverified. This repository shipped ten such packs: the bundle reported three included
/// files and zero pack instances while every gate stayed green. The inverse is checked in
/// the same test because a declaration with no file behind it is a hard build error that
/// only surfaces when someone runs the compiler, which public CI cannot.
#[test]
fn every_smoke_pack_is_declared_and_every_declaration_has_a_file() {
  const INDEX: &str = "r14n Spec/1.0.0/smoke_tests/mod.n.md";
  const DIR: &str = "r14n Spec/1.0.0/smoke_tests";
  let declared = declared_mods(INDEX);
  let on_disk = files_with_suffix(DIR, ".n");

  for stem in &on_disk {
    ::std::assert!(
      declared.contains(stem),
      "{DIR}/{stem}.n exists but {INDEX} does not declare it — discovery is index-only, so \
       this file is compiled by NOTHING. Add `mod {stem} {{}};` at the END of the index."
    );
  }
  for name in &declared {
    ::std::assert!(
      on_disk.contains(name),
      "{INDEX} declares `mod {name} {{}}` but {DIR}/{name}.n does not exist"
    );
  }
  ::std::assert!(!on_disk.is_empty(), "no smoke packs found — the glob or the path is wrong");
}

/// Why: the same index-only discovery rule that governs the smoke packs governs the KEYWORD docs,
/// and nothing checked it. A `R14N_SPEC_KEYWORD_*` doc that its index does not declare is
/// compiled by NOTHING — the types it defines silently do not exist, and every test written
/// against them fails to resolve, which can read as a test problem rather than a missing
/// declaration. The inverse (a declaration with no file) is a hard build error found only by
/// someone running the compiler, which public CI cannot. The index file itself (`mod.n.md`)
/// is excluded from the on-disk set.
#[test]
fn every_keyword_doc_is_declared_and_every_declaration_has_a_file() {
  const INDEX: &str = "r14n Spec/1.0.0/R14N_SPEC/R14N_SPEC_KEYWORDS/mod.n.md";
  const DIR: &str = "r14n Spec/1.0.0/R14N_SPEC/R14N_SPEC_KEYWORDS";
  let declared = declared_mods(INDEX);
  let on_disk: ::std::vec::Vec<::std::string::String> = files_with_suffix(DIR, ".n.md")
    .into_iter()
    .filter(|stem| stem != "mod")
    .collect();

  for stem in &on_disk {
    ::std::assert!(
      declared.contains(stem),
      "{DIR}/{stem}.n.md exists but {INDEX} does not declare it — discovery is index-only, so \
       the types it defines are compiled by NOTHING. Add `mod {stem} {{}};` to the index."
    );
  }
  for name in &declared {
    ::std::assert!(
      on_disk.contains(name),
      "{INDEX} declares `mod {name} {{}}` but {DIR}/{name}.n.md does not exist"
    );
  }
  ::std::assert!(
    on_disk.len() >= 4,
    "found {} keyword docs — the glob or the path is wrong (policies, catalog, frameworks and \
     snapp_compliance are all expected)",
    on_disk.len()
  );

  // A declaration compiles a doc; only a `pub @use` re-export puts its types in front of a Snapp
  // that `@deps` this one. A declared-but-unexported doc is compiled, green, and unreachable to
  // every consumer, which reads as "the type is missing" rather than "the re-export is".
  let reexported = reexported_mods(INDEX);
  for name in &declared {
    ::std::assert!(
      reexported.contains(name),
      "{INDEX} declares `mod {name} {{}}` but never `pub @use {name} {{}}` — compiled, but no \
       consumer of this Snapp can see its types"
    );
  }
  for name in &reexported {
    ::std::assert!(
      declared.contains(name),
      "{INDEX} re-exports `{name}` but never declares it with `mod {name} {{}};`"
    );
  }
}

/// Why: a bundle is what every consumer actually reads, and two things about it can drift
/// silently. A declared pack absent from it means the source moved and the emit did not
/// follow. A `@snapp.version` that contradicts the filename misleads anything pinning by
/// filename — this repository shipped a bundle claiming `0.0.0` under a `0.1.0-alpha.1`
/// name for weeks.
#[test]
fn the_framework_bundle_carries_every_declared_pack_and_agrees_with_its_filename() {
  const BUNDLE: &str = "snapp/r14n@0.1.0-alpha.1.json";
  let doc: ::serde_json::Value =
    ::serde_json::from_str(&common::read(BUNDLE)).expect("framework bundle is not valid JSON");

  let meta = &doc["@snapp"];
  ::std::assert!(
    meta.is_object(),
    "{BUNDLE} carries no `@snapp` root — that is the build-door sidecar shape, not an \
     export bundle. A consumer loading it gets a bundle with no identity."
  );
  ::std::assert_eq!(
    meta["version"].as_str(),
    ::std::option::Option::Some("0.1.0-alpha.1"),
    "{BUNDLE}: `@snapp.version` disagrees with the filename consumers pin by. Re-emit \
     rather than editing either."
  );

  // Instance names come from the sources, not the filenames: the index declares files
  // (`policies__health_aggressive`) and the bundle keys packs by the instance inside the
  // file (`health_aggressive`). Comparing those sets directly reports every pack missing.
  let packs = doc["packs"].as_object().expect("bundle has no `packs` object");
  for stem in declared_mods("r14n Spec/1.0.0/smoke_tests/mod.n.md") {
    let src = common::read(&::std::format!("r14n Spec/1.0.0/smoke_tests/{stem}.n"));
    for line in src.lines() {
      // `  name: <PolicyPack> {` — the typed instance declaration.
      if let ::std::option::Option::Some((lhs, rhs)) = line.split_once(':') {
        if rhs.trim_start().starts_with('<') && rhs.contains("PolicyPack") {
          let name = lhs.trim();
          ::std::assert!(
            packs.contains_key(name),
            "{stem}.n declares pack `{name}` but it is absent from the emitted bundle — \
             the source moved and the emit did not follow. Re-export."
          );
        }
      }
    }
  }
}

/// `<key>` followed by optional spaces, then `=` or `:`, then the value.
///
/// Both assignment spellings are legal N Lang and both appear in this tree, and the key is
/// routinely padded for column alignment (`from      = "…"`). A parser that matched one exact
/// spelling read this file correctly right up until someone aligned a column — which is what
/// happened, and this gate caught it rather than silently skipping the pin it could no longer
/// see. Skipping is the failure that matters here: an unparsed `from` leaves nothing to check
/// the `integrity` against.
fn assignment<'a>(line: &'a str, key: &str) -> ::std::option::Option<&'a str> {
  let rest = line.trim().strip_prefix(key)?;
  let rest = rest.trim_start();
  let rest = rest.strip_prefix('=').or_else(|| rest.strip_prefix(':'))?;
  ::std::option::Option::Some(rest.trim())
}

/// Why: re-emitting a framework bundle changes its bytes and so invalidates every
/// `integrity` pin on it, by design — that is the dependency system working. But the
/// refresh has to land in the SAME change as the re-emit, or the dependent fails to build
/// for the next person, with a DEP-010 that looks like tampering rather than a stale pin.
#[test]
fn every_local_integrity_pin_matches_its_targets_current_bytes() {
  const SOURCE: &str = "baselines/1.0.0/lib.n.md";
  let body = common::read(SOURCE);
  let mut checked = 0usize;
  let mut pending_from: ::std::option::Option<::std::string::String> =
    ::std::option::Option::None;

  for line in body.lines() {
    let t = line.trim();
    if let ::std::option::Option::Some(rest) = assignment(t, "from") {
      pending_from =
        rest.trim_matches('"').split('"').next().map(::std::string::String::from);
    } else if let ::std::option::Option::Some(rest) = assignment(t, "integrity") {
      let pinned = rest.trim().trim_matches('"').trim_start_matches("sha256-");
      let target = pending_from
        .take()
        .expect("an `integrity` line with no preceding `from` — malformed @deps entry");
      let target_path = common::root().join("baselines/1.0.0").join(&target);
      ::std::assert!(
        target_path.is_file(),
        "{SOURCE} pins {target} which does not exist"
      );
      let actual = sha256_hex(&target_path);
      ::std::assert_eq!(
        actual, pinned,
        "{SOURCE}: STALE integrity pin for {target}. Re-emitting a framework bundle \
         invalidates every pin on it — refresh the pin in the same change as the re-emit."
      );
      checked += 1;
    }
  }
  ::std::assert!(checked > 0, "no sha256 pins found in {SOURCE} — the parser or the file moved");
}

/// Why: a healthy dist holds exactly one `@`-bearing JSON per Snapp name. `nlang build
/// --emit-snapp` drops a `<name>@<ver>.n.json` sidecar beside the export bundle that
/// carries NO `@snapp` root, so a consumer globbing the directory can pick the one with no
/// identity. It is gitignored for that reason; this asserts the committed state agrees.
#[test]
fn the_snapp_dir_holds_exactly_one_bundle_per_name() {
  let dir = common::root().join("snapp");
  let mut by_name: ::std::collections::BTreeMap<::std::string::String, ::std::vec::Vec<::std::string::String>> =
    ::std::collections::BTreeMap::new();
  for entry in ::std::fs::read_dir(&dir).expect("read_dir snapp/").filter_map(|e| e.ok()) {
    let name = entry.file_name().to_string_lossy().into_owned();
    if !name.ends_with(".json") || !name.contains('@') {
      continue;
    }
    let stem = name.split('@').next().unwrap_or_default().to_string();
    by_name.entry(stem).or_default().push(name);
  }
  ::std::assert!(!by_name.is_empty(), "snapp/ holds no `@`-bearing bundle at all");
  for (stem, files) in &by_name {
    ::std::assert_eq!(
      files.len(),
      1,
      "snapp/ holds {} `@`-bearing JSONs for Snapp {stem:?}: {files:?}. A consumer globbing \
       this cannot tell which is the bundle.",
      files.len()
    );
  }
}
