//! Catalog-integrity tests — the shipped Control Catalogs, validated by
//! something.
//!
//! ⚠ WHY THIS FILE EXISTS, stated plainly because it is a self-inflicted gap.
//!
//! CI's pack lint globs `catalog/*.catalog.toml` and validates each one against
//! its domain's packs — but when a domain has NO packs it prints
//! `::notice::catalog <d> has no packs yet` and `continue`s. Three catalogs
//! (`data_residency`, `data_retention`, `analytics_consent`) ship with no
//! upstream packs on purpose: a pack claiming a real jurisdiction needs a named
//! attorney of record (invariant 2), so the vocabulary ships and the packs live
//! in the consuming application.
//!
//! The consequence went unnoticed when those catalogs landed: **nothing in CI
//! ever loaded them.** A typo'd control key, a duplicated table, or a `kind`
//! outside the closed set would have shipped silently. That is invariant 11 —
//! "nothing ships that nothing checks" — and the catalogs added to CLOSE that
//! invariant for the downstream packs were themselves unchecked.
//!
//! Everything here GLOBS. Per invariant 11, adding a catalog must not require
//! editing a list somewhere else, because a hand-maintained list is exactly how
//! the gap above happens.
//!
//! NOT LEGAL ADVICE.

/// Repo root (the tools crate's parent).
fn root() -> ::std::path::PathBuf {
  ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every shipped catalog as (domain-from-filename, parsed TOML, relative path).
/// Globbed, never listed.
fn catalogs() -> ::std::vec::Vec<(::std::string::String, ::toml::Value, ::std::string::String)> {
  let dir = root().join("catalog");
  let mut out = ::std::vec::Vec::new();
  for entry in ::std::fs::read_dir(&dir).expect("read catalog/") {
    let path = entry.expect("dir entry").path();
    let name = match path.file_name().and_then(|n| n.to_str()) {
      ::std::option::Option::Some(n) if n.ends_with(".catalog.toml") => ::std::string::String::from(n),
      _ => continue,
    };
    let raw = ::std::fs::read_to_string(&path)
      .unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()));
    let parsed: ::toml::Value = raw
      .parse()
      .unwrap_or_else(|e| ::std::panic!("{name} is not valid TOML: {e}"));
    let domain = name.trim_end_matches(".catalog.toml").to_string();
    out.push((domain, parsed, ::std::format!("catalog/{name}")));
  }
  ::std::assert!(!out.is_empty(), "no catalogs found — the glob is broken, not the repo");
  out.sort_by(|a, b| a.0.cmp(&b.0));
  out
}

/// The `[control.*]` table of a parsed catalog.
fn controls(cat: &::toml::Value) -> &::toml::map::Map<::std::string::String, ::toml::Value> {
  cat
    .get("control")
    .and_then(|c| c.as_table())
    .expect("catalog has no [control.*] tables")
}

/// Why: invariant 11 — a published file must be reachable by something that
/// validates it. Before this test, a catalog whose domain had no upstream packs
/// was never loaded by CI at all (the pack-lint loop `continue`s on a zero pack
/// count), so a catalog could ship unparseable or empty. This is the only thing
/// that loads every catalog unconditionally.
#[test]
fn every_shipped_catalog_parses_and_declares_at_least_one_control() {
  for (domain, cat, rel) in catalogs() {
    let n = controls(&cat).len();
    ::std::assert!(n > 0, "{rel}: declares no controls — an empty vocabulary for {domain}");
  }
}

/// Why: a control's deontic `kind` decides whether it is REQUIRED of a caller
/// or SUBTRACTED from every posture, so a wrong one inverts the control's
/// meaning. The loader defaults a MISSING kind to `"obligation"`
/// (`tools/src/catalog.rs`), which means an omission silently becomes an
/// assertion — so this test demands the key be PRESENT, which is stricter than
/// the parser on purpose. A typo'd kind is stored verbatim and would otherwise
/// reach `extract` output and any consumer reading it.
#[test]
fn every_control_declares_a_kind_from_the_closed_set() {
  const KINDS: [&str; 3] = ["permit", "obligation", "prohibition"];
  for (_domain, cat, rel) in catalogs() {
    for (key, def) in controls(&cat) {
      let kind = def
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or_else(|| ::std::panic!("{rel}: control {key:?} declares no `kind` — the loader would default it to \"obligation\", which is an assertion nobody wrote"));
      ::std::assert!(
        KINDS.contains(&kind),
        "{rel}: control {key:?} has kind {kind:?}, which is outside {KINDS:?}"
      );
      ::std::assert!(
        def.get("title").and_then(|t| t.as_str()).is_some_and(|t| !t.trim().is_empty()),
        "{rel}: control {key:?} has no title — `extract` emits it, so an empty one ships"
      );
    }
  }
}

/// Why: spec §2.2 ties a domain to its filename on disk, and every consumer
/// resolves a catalog by globbing `catalog/<domain>.catalog.toml` and trusting
/// the name. A catalog whose `[domain] name` disagreed with its filename would
/// validate the wrong domain's packs while every log line named the right one.
#[test]
fn a_catalogs_declared_domain_matches_its_filename() {
  for (domain, cat, rel) in catalogs() {
    let declared = cat
      .get("domain")
      .and_then(|d| d.get("name"))
      .and_then(|n| n.as_str())
      .unwrap_or_else(|| ::std::panic!("{rel}: no [domain] name"));
    ::std::assert_eq!(declared, domain, "{rel}: filename and [domain] name disagree");
  }
}

/// Why: invariant 3 — the disclaimer travels with EVERY artifact, and a catalog
/// is the artifact a reader most easily mistakes for a legal determination: it
/// is a list of named obligations with no visible author. Checked on the raw
/// bytes rather than the parse, because a comment is where it lives.
#[test]
fn the_not_legal_advice_disclaimer_travels_with_every_catalog() {
  for (_domain, _cat, rel) in catalogs() {
    let raw = ::std::fs::read_to_string(root().join(&rel)).expect("read catalog");
    ::std::assert!(
      raw.contains("NOT LEGAL ADVICE"),
      "{rel}: no NOT-LEGAL-ADVICE disclaimer (invariant 3)"
    );
  }
}

/// Why: the credential_use catalog states this as rule 1 of two that are "both
/// load-bearing, both cheap to get wrong" — a prohibition key MUST NOT appear
/// in `[legally_required]`, in a `[subject.*]` table, or anywhere else a pack
/// states a requirement, because a pack that both requires and forbids a
/// control is not satisfiable. **Until this test, that rule was prose and
/// nothing enforced it**: `r14n validate` keys off catalog MEMBERSHIP alone and
/// never reads `kind` (the only reader outside the loader prints it as a
/// comment in `extract` output). Globs both directions so a new pack or a newly
/// prohibited key is covered without editing a list.
#[test]
fn no_prohibition_key_appears_in_any_requirement_table() {
  let mut pairs_checked = 0usize;
  for (domain, cat, cat_rel) in catalogs() {
    let prohibited: ::std::collections::BTreeSet<&str> = controls(&cat)
      .iter()
      .filter(|(_, d)| d.get("kind").and_then(|k| k.as_str()) == ::std::option::Option::Some("prohibition"))
      .map(|(k, _)| k.as_str())
      .collect();
    if prohibited.is_empty() {
      continue;
    }
    for pack_rel in shipped_packs_for(&domain) {
      pairs_checked += 1;
      let raw = ::std::fs::read_to_string(root().join(&pack_rel)).expect("read pack");
      let pack: ::toml::Value = raw.parse().unwrap_or_else(|e| ::std::panic!("{pack_rel}: {e}"));

      let mut required: ::std::vec::Vec<(::std::string::String, ::std::string::String)> =
        ::std::vec::Vec::new();
      if let ::std::option::Option::Some(list) =
        pack.get("legally_required").and_then(|t| t.get("controls")).and_then(|c| c.as_array())
      {
        for c in list {
          if let ::std::option::Option::Some(s) = c.as_str() {
            required.push((::std::string::String::from("[legally_required]"), ::std::string::String::from(s)));
          }
        }
      }
      if let ::std::option::Option::Some(subjects) = pack.get("subject").and_then(|s| s.as_table()) {
        for (name, body) in subjects {
          if let ::std::option::Option::Some(list) =
            body.get("controls").and_then(|c| c.as_array())
          {
            for c in list {
              if let ::std::option::Option::Some(s) = c.as_str() {
                required.push((::std::format!("[subject.{name}]"), ::std::string::String::from(s)));
              }
            }
          }
        }
      }

      for (table, key) in required {
        ::std::assert!(
          !prohibited.contains(key.as_str()),
          "{pack_rel}: {table} requires {key:?}, which {cat_rel} declares a PROHIBITION — \
           a pack that both requires and forbids a control is not satisfiable (rule 1)"
        );
      }
    }
  }
  ::std::assert!(
    pairs_checked > 0,
    "no catalog+pack pair was checked — this test would pass vacuously, which is the \
     defect it exists to prevent"
  );
}

/// Every shipped pack for one domain, at either nesting depth (`packs/<profile>/`
/// and the nested `packs/<profile>/<region>/` form spec §2.4 allows).
fn shipped_packs_for(domain: &str) -> ::std::vec::Vec<::std::string::String> {
  fn walk(dir: &::std::path::Path, want: &str, out: &mut ::std::vec::Vec<::std::path::PathBuf>) {
    let entries = match ::std::fs::read_dir(dir) {
      ::std::result::Result::Ok(e) => e,
      ::std::result::Result::Err(_) => return,
    };
    for entry in entries {
      let path = entry.expect("dir entry").path();
      if path.is_dir() {
        walk(&path, want, out);
      } else if path.file_name().and_then(|n| n.to_str()) == ::std::option::Option::Some(want) {
        out.push(path);
      }
    }
  }
  let base = root().join("packs");
  let want = ::std::format!("{domain}.r14n.toml");
  let mut hits = ::std::vec::Vec::new();
  walk(&base, &want, &mut hits);
  let mut rels: ::std::vec::Vec<::std::string::String> = hits
    .iter()
    .map(|p| {
      ::std::format!(
        "packs/{}",
        p.strip_prefix(&base).expect("under packs/").display()
      )
    })
    .collect();
  rels.sort();
  rels
}

/// Why: invariant 10 — renames and removals are DEPRECATIONS because a control
/// key is long-lived audit evidence, and a receipt already issued names the key
/// in its old domain. A deprecated control that does not name its successor
/// leaves the holder of such a receipt with no forward path, which is the whole
/// reason the shipped `aph_mandate` deprecation carries `superseded_by`.
#[test]
fn a_deprecated_control_names_its_successor() {
  let mut seen = 0usize;
  for (_domain, cat, rel) in catalogs() {
    for (key, def) in controls(&cat) {
      if def.get("deprecated").is_none() {
        continue;
      }
      seen += 1;
      let successors = def
        .get("superseded_by")
        .and_then(|s| s.as_array())
        .unwrap_or_else(|| ::std::panic!("{rel}: {key:?} is deprecated with no `superseded_by`"));
      ::std::assert!(
        !successors.is_empty(),
        "{rel}: {key:?} has an EMPTY `superseded_by` — worse than absent, it reads as answered"
      );
    }
  }
  ::std::assert!(seen > 0, "no deprecated control found — this test is not exercising anything");
}
