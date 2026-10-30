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

/// Why: the data-protection domain's obligations must ALL be satisfiable with no human
/// prompt, and that is a property of the vocabulary rather than of any pack.
///
/// A fail-closed fallback binds the caller's ENTIRE declared universe (invariant 1), so
/// the all-on state is what an unconfigured install is made to do. Reading a sealed body
/// is an automatic act — opening a file, indexing, syncing, a headless sweep — so a
/// presence obligation in this domain would make the fallback unusable and lock a
/// headless agent out of its own data. The presence question belongs to an explicit
/// reveal, which `credential_use` governs.
///
/// ⚠ This is the testable form of a check the consuming lane proposed as
/// "every key in the universe is kind=obligation". That one cannot live here: a universe
/// is the CALLER's declaration, constructed in a consumer's process and invisible to this
/// repo — and it would contradict the domain's one deliberate prohibition. Scoped to
/// catalog CONTENT, which is what this repo can actually see.
#[test]
fn the_data_protection_domain_is_obligations_plus_one_prohibition_and_no_presence_gate() {
  let (_domain, cat, rel) = catalogs()
    .into_iter()
    .find(|(d, _, _)| d == "data_protection")
    .expect("data_protection catalog is shipped");

  let mut obligations = 0usize;
  let mut prohibitions: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for (key, def) in controls(&cat) {
    match def.get("kind").and_then(|k| k.as_str()) {
      ::std::option::Option::Some("obligation") => obligations += 1,
      ::std::option::Option::Some("prohibition") => prohibitions.push(key.clone()),
      other => ::std::panic!("{rel}: {key:?} has kind {other:?}; this domain declares no permits"),
    }
    // A presence gate would be satisfiable only with a human in the loop, which the
    // fallback cannot provide. Checked on the MECHANISM, because that is where an
    // enforcement path is named — `credential_use`'s presence control reaches its gate
    // through `interactive_resolve_gate`.
    let mech = def.get("mechanism").and_then(|m| m.as_str()).unwrap_or("");
    ::std::assert!(
      !mech.contains("interactive") && !mech.contains("presence"),
      "{rel}: {key:?} names mechanism {mech:?}, which implies a human prompt — a \
       fail-closed fallback cannot satisfy it and a headless read would be refused"
    );
  }
  ::std::assert!(obligations > 0, "{rel}: no obligations — the domain would require nothing");
  ::std::assert_eq!(
    prohibitions.len(), 1,
    "{rel}: expected exactly the one documented prohibition, found {prohibitions:?} — \
     adding another needs the header's invisibility caveat read first"
  );
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

/// Every `controls = [...]` list in an emitted Snapp bundle, as
/// (bundle-relative JSON pointer, keys). Reads the ARTIFACT rather than scraping
/// the `.n.md` source: the emitter's bytes are the contract, and a string scan of
/// literate markdown would also pick up prose and `legal_review` values (it does —
/// `requires_signoff` is a status, not a control).
fn snapp_control_lists() -> ::std::vec::Vec<(::std::string::String, ::std::vec::Vec<::std::string::String>)> {
  fn walk(
    v: &::serde_json::Value,
    path: &str,
    out: &mut ::std::vec::Vec<(::std::string::String, ::std::vec::Vec<::std::string::String>)>,
  ) {
    match v {
      ::serde_json::Value::Object(map) => {
        if let ::std::option::Option::Some(::serde_json::Value::Array(list)) = map.get("controls") {
          let keys: ::std::vec::Vec<::std::string::String> = list
            .iter()
            .filter_map(|k| k.as_str().map(::std::string::String::from))
            .collect();
          out.push((::std::string::String::from(path), keys));
        }
        for (k, child) in map {
          walk(child, &::std::format!("{path}/{k}"), out);
        }
      }
      ::serde_json::Value::Array(items) => {
        for (i, child) in items.iter().enumerate() {
          walk(child, &::std::format!("{path}[{i}]"), out);
        }
      }
      _ => {}
    }
  }

  let dir = root().join("snapp");
  let mut out = ::std::vec::Vec::new();
  for entry in ::std::fs::read_dir(&dir).expect("read snapp/") {
    let path = entry.expect("dir entry").path();
    let name = match path.file_name().and_then(|n| n.to_str()) {
      ::std::option::Option::Some(n) if n.ends_with(".json") => ::std::string::String::from(n),
      _ => continue,
    };
    let raw = ::std::fs::read_to_string(&path).expect("read bundle");
    let doc: ::serde_json::Value = ::serde_json::from_str(&raw)
      .unwrap_or_else(|e| ::std::panic!("snapp/{name} is not valid JSON: {e}"));
    walk(&doc, &::std::format!("snapp/{name}"), &mut out);
  }
  out
}

/// Why: invariant 11 — "a new pack needs a catalog for its domain (CI globs both
/// directions)". That glob covers `packs/*.r14n.toml`, and the N Lang baselines
/// ship their packs as Snapp NODES, not published TOML — so they were never in its
/// scope. `baselines/1.0.0`'s `HEALTH_AGGRESSIVE` pack consequently shipped with
/// five control keys (`consent_prompt`, `usage_description`, `on_device_only`,
/// `read_scope_limit`, `audit_log`) that NO catalog declared, and nothing reported
/// it. This closes the other half of the glob.
///
/// Checks UNION membership across all catalogs rather than per-domain, deliberately.
/// Deriving a pack's domain from its node name (`HEALTH_AGGRESSIVE` -> `health`)
/// would be string-matching on an identifier (BOOK §1), and the post-rename pack
/// type no longer carries a `domain` prop to read instead — the key IS the domain in
/// the published form. Union membership is the honest check available, and it still
/// catches the defect that shipped: a key declared nowhere at all.
#[test]
fn every_control_key_in_an_emitted_snapp_bundle_is_declared_by_some_catalog() {
  let declared: ::std::collections::BTreeSet<::std::string::String> = catalogs()
    .iter()
    .flat_map(|(_, cat, _)| controls(cat).keys().cloned().collect::<::std::vec::Vec<_>>())
    .collect();
  ::std::assert!(!declared.is_empty(), "no catalog keys loaded — the glob is broken");

  let lists = snapp_control_lists();
  ::std::assert!(
    !lists.is_empty(),
    "no control list found in any snapp/ bundle — this test would pass vacuously, \
     which is the defect it exists to prevent"
  );

  let mut undeclared: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();
  for (pointer, keys) in &lists {
    for key in keys {
      if !declared.contains(key) {
        undeclared.push(::std::format!("{key} (at {pointer})"));
      }
    }
  }
  undeclared.sort();
  undeclared.dedup();
  ::std::assert!(
    undeclared.is_empty(),
    "emitted Snapp packs reference {} control key(s) no catalog declares, so they ship \
     with an undeclared vocabulary (invariant 11): {undeclared:#?}",
    undeclared.len()
  );
}

/// Why: the Snapp cross-check above reads EMITTED bundles, and the smoke-test packs
/// under `r14n Spec/1.0.0/smoke_tests/` are `#[cfg(test)]` — never emitted, so that
/// check cannot see them. Four domains (`location`, `nfc`, `photo_library`,
/// `smart_home`) consequently shipped with TEN control keys no catalog declared, and
/// BOTH existing guards missed them for complementary reasons: the CI pack lint globs
/// `packs/*.r14n.toml` (these are Snapp nodes), and the bundle check globs `snapp/`
/// (these are never emitted). The overlap of two blind spots is where this sat.
///
/// Scott ruled 2026-10-26 that these are real domains rather than fixtures, so their
/// vocabulary is held to the same standard as a published pack's.
///
/// This check is PER-DOMAIN, not union, and that is possible because these packs
/// declare an `id` prop — a data contract to read, rather than a node name to split
/// (BOOK §1). The emitted-bundle check upstream has no `id` to read and stays union.
///
/// ⚠ LIMITATION, stated rather than hidden: this reads N Lang SOURCE with a regex,
/// which the bundle check deliberately avoids. It is tolerable only because these are
/// plain `.n` files (not literate markdown, where prose polluted the same approach) and
/// because the pattern is anchored to `controls:` arrays after line comments are
/// stripped. If smoke packs ever become emitted, delete this and extend the bundle
/// check instead — reading the artifact beats scraping the source.
#[test]
fn every_control_key_in_a_smoke_test_pack_is_declared_by_its_own_domains_catalog() {
  let dir = root().join("r14n Spec/1.0.0/smoke_tests");
  let mut packs_checked = 0usize;
  let mut undeclared: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::new();

  for entry in ::std::fs::read_dir(&dir).expect("read smoke_tests/") {
    let path = entry.expect("dir entry").path();
    let name = match path.file_name().and_then(|n| n.to_str()) {
      ::std::option::Option::Some(n) if n.starts_with("policies__") && n.ends_with(".n") => {
        ::std::string::String::from(n)
      }
      _ => continue,
    };
    let raw = ::std::fs::read_to_string(&path).expect("read smoke pack");
    // Strip line comments first: a commented-out `controls:` array would otherwise be
    // read as live vocabulary.
    let code: ::std::string::String = raw
      .lines()
      .map(|l| match l.find("//") {
        ::std::option::Option::Some(i) => &l[..i],
        ::std::option::Option::None => l,
      })
      .collect::<::std::vec::Vec<&str>>()
      .join("\n");

    // The pack's declared domain. Read from the `id` PROP, never from the node name.
    let domain = match code.split("id:").nth(1).and_then(|rest| {
      let rest = rest.trim_start();
      let rest = rest.strip_prefix('"')?;
      rest.find('"').map(|end| ::std::string::String::from(&rest[..end]))
    }) {
      ::std::option::Option::Some(d) => d,
      ::std::option::Option::None => {
        ::std::panic!("{name}: no `id:` prop — cannot attribute its keys to a domain")
      }
    };

    let cat_path = root().join(::std::format!("catalog/{domain}.catalog.toml"));
    ::std::assert!(
      cat_path.is_file(),
      "{name} declares domain {domain:?} and catalog/{domain}.catalog.toml does not exist \
       — a pack whose domain has no catalog ships an undeclared vocabulary (invariant 11)"
    );
    let cat: ::toml::Value = ::std::fs::read_to_string(&cat_path)
      .expect("read catalog")
      .parse()
      .expect("catalog TOML");
    let declared: ::std::collections::BTreeSet<::std::string::String> =
      controls(&cat).keys().cloned().collect();

    packs_checked += 1;
    let mut rest = code.as_str();
    let mut found_a_list = false;
    while let ::std::option::Option::Some(i) = rest.find("controls:") {
      rest = &rest[i + "controls:".len()..];
      let open = match rest.find('[') {
        ::std::option::Option::Some(o) => o,
        ::std::option::Option::None => break,
      };
      let close = match rest[open..].find(']') {
        ::std::option::Option::Some(c) => open + c,
        ::std::option::Option::None => break,
      };
      found_a_list = true;
      for key in rest[open + 1..close].split(',') {
        let key = key.trim().trim_matches('"').trim();
        if key.is_empty() {
          continue;
        }
        if !declared.contains(key) {
          undeclared.push(::std::format!("{key} (in {name}, domain {domain})"));
        }
      }
      rest = &rest[close..];
    }
    ::std::assert!(
      found_a_list,
      "{name}: no `controls:` array parsed — the scrape silently matched nothing, which \
       would let this pack's vocabulary go unchecked"
    );
  }

  ::std::assert!(
    packs_checked > 0,
    "no smoke-test pack was checked — this test would pass vacuously"
  );
  undeclared.sort();
  undeclared.dedup();
  ::std::assert!(
    undeclared.is_empty(),
    "smoke-test packs reference {} control key(s) their own domain's catalog does not \
     declare: {undeclared:#?}",
    undeclared.len()
  );
}

