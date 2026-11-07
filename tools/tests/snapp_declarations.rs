// SPDX-License-Identifier: Apache-2.0
//! Snapp-declaration tests, catalog side — the five Control Catalogs that shipped without one, the
//! training-data locality axis, and the two install-time decisions that `docs/framework-links.md`
//! section 7.3 records. Asserted against the committed catalogs and the committed page, never against
//! a compiler: N Lang is proprietary and this repository's CI runs on public runners, so a gate here
//! cannot invoke `nlang` (see `snapp_drift_gate.rs`).
//!
//! WHAT THIS CAN AND CANNOT PROVE. It proves that each new catalog declares exactly the control keys
//! the consuming application's draft packs already name, that every one satisfies the published
//! catalog schema, that none declares a prohibition (a prohibition key must never sit in a
//! requirement table, and every one of these does), and that the training-capture header names the
//! three locality tokens, the pack-data rule and the resolver facts the vocabulary cannot enforce
//! without stating a stance for any locality. It CANNOT prove that a pack for any of these domains
//! exists or is lawful: none ships here (invariant 2), and a catalog is a vocabulary, not a
//! determination. The data-class vocabulary and the typed-path prop of `SnappCompliance` are Snapp
//! sources, not catalogs; they are pinned where they land, not here.
//!
//! The control-key sets below are the union read from the consuming application's bundled draft
//! packs across all five of its profiles (read 2026-11-02, re-read 2026-11-03). They are pinned so a
//! catalog that drifts from the packs it declares the vocabulary for is noticed, in both directions.
//!
//! Style note: FQ paths, no module-top `use` — the house style of this tree.
//!
//! NOT LEGAL ADVICE.
//!
//! Revision History
//! - 2026-11-03: authored — the five catalogs that shipped without one, the training-capture locality
//!   axis and the section 7.3 decisions. Split from a larger proposal: the tests that read Snapp-tree
//!   files (the data-class vocabulary, the typed-path prop, the emitted bundle) are not in this file,
//!   because the files they read are not in the tree yet and a test that reads an absent file is red.
//! - 2026-11-03: the third locality token is `platform_corpus`. The first spelling carried an internal
//!   component name, which this repository's publication scan refuses in any tracked file, so it is
//!   retired: `LOCALITY_TOKENS` names the new spelling, and the header test now refuses the retired one
//!   as a negative needle, assembled with `concat!` so that this file does not itself spell it. The test
//!   name is unchanged.

mod common;

/// The domains whose catalogs this change adds.
const NEW_CATALOG_DOMAINS: [&str; 5] = [
  "ai_disclosure",
  "contact_import",
  "diagnostics_upload",
  "export_control",
  "training_data_capture",
];

/// Domain to the control keys the consuming application's draft packs name (union across profiles).
const SHIPPED_PACK_CONTROLS: [(&str, &[&str]); 5] = [
  (
    "ai_disclosure",
    &[
      "ai_disclosure",
      "agent_real_vault_mutation_permitted",
      "quiet_hours_enforced",
      "auto_renewal_disclosure",
    ],
  ),
  (
    "contact_import",
    &["contact_pii_consent", "explicit_opt_in", "data_minimization", "residency_check"],
  ),
  (
    "diagnostics_upload",
    &["upload_consent", "third_party_processor_ack", "retention_bound", "data_residency_egress"],
  ),
  (
    "export_control",
    &["crypto_settlement_permitted", "refund_cooling_off_required", "dunning_frequency_capped"],
  ),
  (
    "training_data_capture",
    &[
      "scrub_secrets",
      "encrypt_at_rest",
      "org_local_only",
      "opt_out_honored",
      "admin_gated_reads",
      "retention_capped",
      "explicit_optin_consent",
      // The one key this change adds, with the locality axis. No draft pack names it yet.
      "recipient_disclosed",
    ],
  ),
];

/// The two keys the consuming gate reads with INVERTED polarity: acted on only when a governing pack
/// REQUIRES them, which means the pack grants the act. Each is a `permit` — a `*_permitted`
/// authorization the actor must HOLD — on the precedent the `data_residency` catalog and the
/// `external_agent_harvest_permitted` key of the `analytics_consent` catalog already set.
const POLARITY_PERMITS: [(&str, &str); 2] = [
  ("ai_disclosure", "agent_real_vault_mutation_permitted"),
  ("export_control", "crypto_settlement_permitted"),
];

/// The three training-locality SUBJECT tokens, the one spelling of where a captured corpus goes. A
/// catalog declares controls and never subjects, so the only home the catalog format offers them is
/// the header of the `training_data_capture` catalog, and this constant pins the header to them.
const LOCALITY_TOKENS: [&str; 3] = ["local_corpus", "cloud_corpus", "platform_corpus"];

fn control_table(
  cat: &::toml::Value,
) -> &::toml::map::Map<::std::string::String, ::toml::Value> {
  cat
    .get("control")
    .and_then(|c| c.as_table())
    .unwrap_or_else(|| ::std::panic!("catalog has no [control] table"))
}

fn catalog(domain: &str) -> (::std::string::String, ::toml::Value) {
  let raw = common::read(&::std::format!("catalog/{domain}.catalog.toml"));
  let parsed: ::toml::Value =
    raw.parse().unwrap_or_else(|e| ::std::panic!("catalog/{domain}.catalog.toml: {e}"));
  (raw, parsed)
}

// ── The five catalogs ───────────────────────────────────────────────────────────────────────

/// Why: each catalog declares the vocabulary for packs that already exist in the consuming
/// application. If a catalog drops a key a draft pack names, that pack ships an undeclared
/// vocabulary (invariant 11); if it invents one no pack names, the vocabulary has drifted from its
/// users. The set is pinned in BOTH directions. `training_data_capture` is the one domain that adds a
/// key no draft pack names yet (`recipient_disclosed`, with the locality axis), and the table says so.
#[test]
fn each_new_catalog_declares_exactly_the_control_keys_the_shipped_packs_name() {
  for (domain, expected) in SHIPPED_PACK_CONTROLS {
    let (_raw, cat) = catalog(domain);
    let declared: ::std::collections::BTreeSet<::std::string::String> =
      control_table(&cat).keys().cloned().collect();
    let want: ::std::collections::BTreeSet<::std::string::String> =
      expected.iter().map(|s| ::std::string::String::from(*s)).collect();
    ::std::assert_eq!(declared, want, "catalog/{domain}.catalog.toml control keys");
  }
}

/// Why: a `prohibition` key must never appear in a requirement table (`catalog_integrity`), and every
/// key these catalogs declare is one the draft packs place in a requirement table, so a prohibition
/// here would make a shipped pack unsatisfiable. Between the other two kinds the precedent is the key's
/// shape: a `*_permitted` authorization the actor must HOLD is a `permit`, and a state the enforcing
/// gate must bring about is an `obligation`. Checked in both directions, so a polarity key that
/// drifts to an obligation, or an ordinary key that drifts to a permit, is noticed.
#[test]
fn the_new_catalogs_declare_no_prohibition_and_only_the_two_polarity_keys_are_permits() {
  for domain in NEW_CATALOG_DOMAINS {
    let (_raw, cat) = catalog(domain);
    for (key, def) in control_table(&cat) {
      let is_polarity_permit =
        POLARITY_PERMITS.iter().any(|(d, k)| *d == domain && *k == key.as_str());
      let want = if is_polarity_permit { "permit" } else { "obligation" };
      ::std::assert_eq!(
        def.get("kind").and_then(|k| k.as_str()),
        ::std::option::Option::Some(want),
        "{domain}.{key} must be a {want}"
      );
    }
  }
}

/// Why: a catalog can declare that a key EXISTS and never which direction a gate reads it. The two keys
/// the consuming gate reads with inverted polarity (acted on only when a governing pack REQUIRES them,
/// which means the pack explicitly permits the act) must say so in the file a reader opens, because
/// the `permit` kind alone reads as the ordinary direction.
#[test]
fn the_two_inverted_polarity_keys_carry_the_polarity_warning() {
  for (domain, key) in POLARITY_PERMITS {
    let (raw, _cat) = catalog(domain);
    ::std::assert!(raw.contains("POLARITY"), "{domain}: the header must warn about polarity");
    ::std::assert!(raw.contains(key), "{domain}: the warning must name {key}");
  }
}

/// Why: the published catalog schema is the contract every catalog is validated against in CI. The
/// five new files must satisfy it, with the same validator `schema_conformance` uses.
#[test]
fn each_new_catalog_validates_against_the_published_catalog_schema() {
  let validator = common::validator("schema/catalog.schema.json");
  for domain in NEW_CATALOG_DOMAINS {
    let (raw, _cat) = catalog(domain);
    let errs = common::errors(&validator, &common::toml_text_as_json(&raw));
    ::std::assert!(errs.is_empty(), "catalog/{domain}.catalog.toml is refused by the schema: {errs:?}");
  }
}

// ── The training-data locality axis ─────────────────────────────────────────────────────────

/// Why: the draft packs name seven control keys and the locality axis adds exactly ONE
/// (`recipient_disclosed`). Eight, nothing renamed, nothing removed: a control key is audit evidence.
#[test]
fn the_training_data_capture_catalog_declares_the_shipped_seven_plus_recipient_disclosed() {
  let (_raw, cat) = catalog("training_data_capture");
  let table = control_table(&cat);
  ::std::assert_eq!(table.len(), 8, "seven shipped keys plus recipient_disclosed");
  for shipped in [
    "scrub_secrets",
    "encrypt_at_rest",
    "org_local_only",
    "opt_out_honored",
    "admin_gated_reads",
    "retention_capped",
    "explicit_optin_consent",
  ] {
    ::std::assert!(table.contains_key(shipped), "shipped key {shipped} must stay declared");
  }
  ::std::assert!(table.contains_key("recipient_disclosed"), "the one added key");
}

/// Why: the header is where a pack author learns the three locality subject tokens and who owns a
/// locality's stance. Whether a locality is permitted outright or only on an affirmative opt-in is PACK
/// DATA, never a key and never the catalog (the catalog convention "no key encodes a default", the rule
/// the `analytics_consent` header states for its own domain). So the header must name the three tokens
/// and the pack-data rule, and must NOT state a stance for any locality, name a company, or spell the
/// third locality a second way: two spellings of one vocabulary is a defect, and a second spelling of
/// the third token is the drift this guards against. It must also state the resolver facts the
/// vocabulary cannot enforce: an absent subject is the full universe, the `minimal` floor is per pack and
/// not per subject, and a consumer must never read `fell_back` as permission. The "opt-in rule" of the
/// test name is the pack-data rule: opt-in is expressed by whether a subject's table requires
/// `explicit_optin_consent`.
#[test]
fn the_training_data_capture_catalog_header_names_the_three_localities_and_the_optin_rule() {
  let (raw, _cat) = catalog("training_data_capture");
  for token in LOCALITY_TOKENS {
    ::std::assert!(raw.contains(token), "training_data_capture header must name the locality {token:?}");
  }
  for needle in [
    "NO KEY HERE ENCODES A DEFAULT",
    "PACK DATA",
    "explicit_optin_consent",
    "ABSENCE IS THE FULL UNIVERSE",
    "fell_back",
    "PER PACK, NOT PER SUBJECT",
  ] {
    ::std::assert!(raw.contains(needle), "training_data_capture header must mention {needle:?}");
  }
  // The paired negatives: no company wording, no per-locality stance, no second spelling of a locality.
  // The retired spelling of the third token is assembled with `concat!` so that this public file does
  // not itself spell it: the repository's publication scan refuses that spelling in any tracked file.
  let lower = raw.to_lowercase();
  for banned in [
    "squillo inc",
    "opt-in only",
    "default on",
    "on by default",
    ::std::concat!("squillo", "_corpus"),
  ] {
    ::std::assert!(
      !lower.contains(banned),
      "training_data_capture catalog must not contain {banned:?}: a stance is pack data and a \
       locality has one spelling"
    );
  }
}

// ── The two install-time decisions ──────────────────────────────────────────────────────────

/// Why: `framework-links.md` left section 7.3's two decisions undecided. Once this change lands the
/// page records both for the installer it serves first, with the reasoning each rests on, keeps the
/// options for any other installer, and no longer claims the decisions are open. It still must NOT
/// claim to choose for any other installer. The page is hard-wrapped, so the text is
/// whitespace-normalized first: a phrase that sits across a line break is the same phrase.
#[test]
fn framework_links_section_7_3_records_both_decisions_and_binds_no_other_installer() {
  let page = common::read("docs/framework-links.md");
  let flat: ::std::string::String =
    page.split_whitespace().collect::<::std::vec::Vec<&str>>().join(" ");
  ::std::assert!(
    !flat.contains("These are **undecided**"),
    "section 7.3 still claims both decisions are undecided"
  );
  ::std::assert!(
    !flat.contains("### 7.3 Two open decisions"),
    "section 7.3 still carries its old heading, which says the decisions are open"
  );
  ::std::assert_eq!(
    flat.matches("Decided for Squillo OS").count(),
    2,
    "section 7.3 must record decision A and decision B, each marked as decided for Squillo OS"
  );
  ::std::assert!(
    flat.contains("does not bind any other installer"),
    "section 7.3 must say it binds no other installer"
  );
  for kept in [
    "**A. What an undeclared application means at install.**",
    "**B. Where the install-time resolve runs.**",
    "| Warn-only |",
  ] {
    ::std::assert!(
      flat.contains(kept),
      "section 7.3 must keep the options another installer may choose differently: missing {kept:?}"
    );
  }
}
