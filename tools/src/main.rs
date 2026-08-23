// SPDX-License-Identifier: Apache-2.0
//! # r14n — the RLPS pack-lifecycle CLI (roadmap item 5)
//!
//! `extract` (catalog → fail-closed pack template) · `merge` (regulation-change
//! delta; flags ONLY changed controls for legal re-review — the gettext-msgmerge
//! analogue) · `validate` (the cross-file linter behind `schema/pack.schema.json`)
//! · `keygen` / `sign` / `verify` (Ed25519, detached) · `publish` (LOCAL
//! content-addressed registry index only — network publication is manual and
//! counsel-gated).
//!
//! ⚠ NOT LEGAL ADVICE. Every subcommand operates on configuration artifacts;
//! none of them states what any jurisdiction's law is.
//!
//! Revision History
//! - 2026-07-06: authored — roadmap item 5 (pack-lifecycle CLI).
//! - 2026-07-06: DRY/testability pass — `parse_args` is pure (Result, no
//!   exits) so the CLI surface is unit-tested; `Help` is a Command variant.

mod catalog;
mod directory;
mod extract;
mod jcs;
mod keys;
mod merge;
mod packtoml;
mod publish;
mod validate;

const USAGE: &str = "r14n — RLPS pack-lifecycle CLI (NOT LEGAL ADVICE)

USAGE:
  r14n extract  --catalog <file> --profile <name> [-o <out>]
  r14n merge    --pack <file> --catalog <file> [-o <out>]
  r14n validate <pack>... [--catalog <file>]
  r14n keygen   --out <prefix>            (writes <prefix>.seed + <prefix>.pub)
  r14n sign     <pack> --key <seedfile> [--key-id <id>]
  r14n verify   <pack> [--sig <file>] [--directory <dir.json> [--as-of <YYYY-MM-DD>] [--jurisdiction <j>]]
  r14n sign-directory <dir.json> --key <seedfile>   (steward-signs over the JCS canonical form)
  r14n publish  <pack> --id <profile/domain> [--registry <index.json>]

`publish` writes only the LOCAL registry index; making anything public is a
human decision behind the counsel gate (see GOVERNANCE.md).";

/// Parsed invocation — one variant per subcommand (exhaustive dispatch).
#[derive(::std::fmt::Debug)]
enum Command {
  Extract {
    catalog: ::std::path::PathBuf,
    profile: ::std::string::String,
    out: ::std::option::Option<::std::path::PathBuf>,
  },
  Merge {
    pack: ::std::path::PathBuf,
    catalog: ::std::path::PathBuf,
    out: ::std::option::Option<::std::path::PathBuf>,
  },
  Validate {
    packs: ::std::vec::Vec<::std::path::PathBuf>,
    catalog: ::std::option::Option<::std::path::PathBuf>,
  },
  Keygen {
    out: ::std::path::PathBuf,
  },
  Sign {
    pack: ::std::path::PathBuf,
    key: ::std::path::PathBuf,
    key_id: ::std::option::Option<::std::string::String>,
  },
  Verify {
    pack: ::std::path::PathBuf,
    sig: ::std::option::Option<::std::path::PathBuf>,
    directory: ::std::option::Option<::std::path::PathBuf>,
    as_of: ::std::option::Option<::std::string::String>,
    jurisdiction: ::std::option::Option<::std::string::String>,
  },
  SignDirectory {
    directory: ::std::path::PathBuf,
    key: ::std::path::PathBuf,
  },
  Publish {
    pack: ::std::path::PathBuf,
    id: ::std::string::String,
    registry: ::std::path::PathBuf,
  },
  Help,
}

/// Pull `--flag value` out of the arg list (removing both tokens). `Err` when
/// the flag is dangling (no value / value is itself a `--flag`) or repeated
/// (council-audit NN13 — a missing value must not silently swallow the next flag).
fn take_flag(
  args: &mut ::std::vec::Vec<::std::string::String>,
  flag: &str,
) -> ::std::result::Result<::std::option::Option<::std::string::String>, ::std::string::String> {
  let idx = match args.iter().position(|a| a == flag) {
    ::std::option::Option::Some(i) => i,
    ::std::option::Option::None => return ::std::result::Result::Ok(::std::option::Option::None),
  };
  if idx + 1 >= args.len() {
    return ::std::result::Result::Err(::std::format!("{flag} needs a value"));
  }
  if args[idx + 1].starts_with("--") {
    return ::std::result::Result::Err(::std::format!(
      "{flag} needs a value, but got the flag `{}`",
      args[idx + 1]
    ));
  }
  if args.iter().filter(|a| a.as_str() == flag).count() > 1 {
    return ::std::result::Result::Err(::std::format!("{flag} given more than once"));
  }
  args.remove(idx);
  ::std::result::Result::Ok(::std::option::Option::Some(args.remove(idx)))
}

/// After a subcommand has consumed its flags + positional, nothing may remain —
/// a leftover token is a typo or a dropped arg, not something to ignore silently.
fn expect_no_leftovers(
  args: &[::std::string::String],
) -> ::std::result::Result<(), ::std::string::String> {
  if args.is_empty() {
    ::std::result::Result::Ok(())
  } else {
    ::std::result::Result::Err(::std::format!("unexpected argument(s): {}", args.join(" ")))
  }
}

fn require(
  value: ::std::option::Option<::std::string::String>,
  what: &str,
) -> ::std::result::Result<::std::string::String, ::std::string::String> {
  value.ok_or_else(|| ::std::format!("{what} is required"))
}

/// Parse an argument vector (without argv[0]) into a [`Command`] — pure, so
/// the CLI surface is unit-testable; `main` owns printing + exit codes.
fn parse_args(
  mut args: ::std::vec::Vec<::std::string::String>,
) -> ::std::result::Result<Command, ::std::string::String> {
  if args.is_empty() {
    return ::std::result::Result::Ok(Command::Help);
  }
  let sub = args.remove(0);
  match sub.as_str() {
    "extract" => {
      let cmd = Command::Extract {
        catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog")?, "--catalog")?),
        profile: require(take_flag(&mut args, "--profile")?, "--profile")?,
        out: take_flag(&mut args, "-o")?.map(::std::path::PathBuf::from),
      };
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(cmd)
    }
    "merge" => {
      let cmd = Command::Merge {
        pack: ::std::path::PathBuf::from(require(take_flag(&mut args, "--pack")?, "--pack")?),
        catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog")?, "--catalog")?),
        out: take_flag(&mut args, "-o")?.map(::std::path::PathBuf::from),
      };
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(cmd)
    }
    "validate" => {
      let catalog = take_flag(&mut args, "--catalog")?.map(::std::path::PathBuf::from);
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from(
          "validate needs at least one pack path",
        ));
      }
      ::std::result::Result::Ok(Command::Validate {
        packs: args.into_iter().map(::std::path::PathBuf::from).collect(),
        catalog,
      })
    }
    "keygen" => {
      let cmd = Command::Keygen {
        out: ::std::path::PathBuf::from(require(take_flag(&mut args, "--out")?, "--out")?),
      };
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(cmd)
    }
    "sign" => {
      let key = ::std::path::PathBuf::from(require(take_flag(&mut args, "--key")?, "--key")?);
      let key_id = take_flag(&mut args, "--key-id")?;
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("sign needs a pack path"));
      }
      let pack = ::std::path::PathBuf::from(args.remove(0));
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(Command::Sign { pack, key, key_id })
    }
    "verify" => {
      let sig = take_flag(&mut args, "--sig")?.map(::std::path::PathBuf::from);
      let directory = take_flag(&mut args, "--directory")?.map(::std::path::PathBuf::from);
      let as_of = take_flag(&mut args, "--as-of")?;
      let jurisdiction = take_flag(&mut args, "--jurisdiction")?;
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("verify needs a pack path"));
      }
      let pack = ::std::path::PathBuf::from(args.remove(0));
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(Command::Verify { pack, sig, directory, as_of, jurisdiction })
    }
    "sign-directory" => {
      let key = ::std::path::PathBuf::from(require(take_flag(&mut args, "--key")?, "--key")?);
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from(
          "sign-directory needs a directory path",
        ));
      }
      let directory = ::std::path::PathBuf::from(args.remove(0));
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(Command::SignDirectory { directory, key })
    }
    "publish" => {
      let id = require(take_flag(&mut args, "--id")?, "--id (canonically <profile>/<domain>)")?;
      let registry = take_flag(&mut args, "--registry")?
        .map(::std::path::PathBuf::from)
        .unwrap_or_else(|| ::std::path::PathBuf::from("registry/index.json"));
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("publish needs a pack path"));
      }
      let pack = ::std::path::PathBuf::from(args.remove(0));
      expect_no_leftovers(&args)?;
      ::std::result::Result::Ok(Command::Publish { pack, id, registry })
    }
    "--help" | "-h" | "help" => ::std::result::Result::Ok(Command::Help),
    other => ::std::result::Result::Err(::std::format!("unknown subcommand `{other}`")),
  }
}

fn write_or_stdout(out: ::std::option::Option<::std::path::PathBuf>, text: &str) {
  match out {
    ::std::option::Option::Some(path) => {
      if let ::std::result::Result::Err(e) = ::std::fs::write(&path, text) {
        ::std::eprintln!("error: write {}: {e}", path.display());
        ::std::process::exit(1);
      }
      ::std::eprintln!("wrote {}", path.display());
    }
    ::std::option::Option::None => ::std::print!("{text}"),
  }
}

fn fail(message: &::std::string::String) -> ! {
  ::std::eprintln!("error: {message}");
  ::std::process::exit(1);
}

fn main() {
  let command = match parse_args(::std::env::args().skip(1).collect()) {
    ::std::result::Result::Ok(c) => c,
    ::std::result::Result::Err(e) => {
      ::std::eprintln!("error: {e}\n\n{USAGE}");
      ::std::process::exit(2);
    }
  };
  match command {
    Command::Help => ::std::println!("{USAGE}"),
    Command::Extract { catalog, profile, out } => {
      let cat = crate::catalog::load(&catalog).unwrap_or_else(|e| fail(&e));
      write_or_stdout(out, &crate::extract::template(&cat, &profile));
    }
    Command::Merge { pack, catalog, out } => {
      let cat = crate::catalog::load(&catalog).unwrap_or_else(|e| fail(&e));
      let text = ::std::fs::read_to_string(&pack)
        .unwrap_or_else(|e| fail(&::std::format!("read {}: {e}", pack.display())));
      let (merged, report) = crate::merge::merge(&text, &cat).unwrap_or_else(|e| fail(&e));
      for key in &report.added {
        ::std::eprintln!("NEEDS-LEGAL-REVIEW: + {key}");
      }
      for key in &report.stale {
        ::std::eprintln!("STALE: - {key}");
      }
      if report.added.is_empty() && report.stale.is_empty() {
        ::std::eprintln!("no drift: pack matches the catalog");
      }
      write_or_stdout(out, &merged);
    }
    Command::Validate { packs, catalog } => {
      let cat = catalog.map(|p| crate::catalog::load(&p).unwrap_or_else(|e| fail(&e)));
      let mut failed = false;
      for pack in &packs {
        let text = ::std::fs::read_to_string(pack)
          .unwrap_or_else(|e| fail(&::std::format!("read {}: {e}", pack.display())));
        let findings = crate::validate::validate(&text, cat.as_ref());
        for w in &findings.warnings {
          ::std::eprintln!("{}: warning: {w}", pack.display());
        }
        for e in &findings.errors {
          ::std::eprintln!("{}: error: {e}", pack.display());
        }
        if findings.ok() {
          ::std::eprintln!("{}: OK ({} warning(s))", pack.display(), findings.warnings.len());
        } else {
          failed = true;
        }
      }
      if failed {
        ::std::process::exit(1);
      }
    }
    Command::Keygen { out } => {
      let (seed, public) = crate::keys::keygen(&out).unwrap_or_else(|e| fail(&e));
      ::std::eprintln!(
        "wrote {} (PRIVATE — keep out of git) and {}",
        seed.display(),
        public.display()
      );
    }
    Command::Sign { pack, key, key_id } => {
      let sig = crate::keys::sign_file(&pack, &key, key_id.as_deref())
        .unwrap_or_else(|e| fail(&e));
      ::std::eprintln!("wrote {} (a signature proves WHO signed, not that a review was correct)", sig.display());
    }
    Command::Verify { pack, sig, directory, as_of, jurisdiction } => {
      let sig_path = sig.unwrap_or_else(|| {
        ::std::path::PathBuf::from(::std::format!("{}.sig", pack.display()))
      });
      let public_key = crate::keys::verify_file(&pack, &sig_path).unwrap_or_else(|e| fail(&e));
      ::std::eprintln!("{}: signature + identity + content address OK", pack.display());
      // Optional trust-root check against a reviewer-key directory (M9). Without
      // it, a valid signature only proves WHO signed, not that they may attest.
      if let ::std::option::Option::Some(dir) = directory {
        // The directory's own steward signature must verify (if present) before
        // we trust anything it lists (D2). An unsigned directory is a warning.
        match crate::directory::verify_directory_steward(&dir) {
          ::std::result::Result::Ok(true) => ::std::eprintln!("directory: steward signature OK"),
          ::std::result::Result::Ok(false) => {
            ::std::eprintln!("directory: WARNING — no steward signature (unsigned trust root)")
          }
          ::std::result::Result::Err(e) => fail(&e),
        }
        let when = as_of.unwrap_or_else(|| ::std::string::String::from("9999-12-31"));
        let status = crate::directory::key_status(&dir, &public_key, &when, jurisdiction.as_deref())
          .unwrap_or_else(|e| fail(&e));
        if status.is_trusted() {
          ::std::eprintln!("trust root: {status:?} — decision may escape advisory-only");
        } else {
          ::std::eprintln!(
            "trust root: {status:?} — decision remains ADVISORY-ONLY (signer not a trusted reviewer as of {when})"
          );
          ::std::process::exit(3);
        }
      }
    }
    Command::SignDirectory { directory, key } => {
      crate::directory::sign_directory(&directory, &key).unwrap_or_else(|e| fail(&e));
      ::std::eprintln!("{}: steward-signed over its JCS canonical form", directory.display());
    }
    Command::Publish { pack, id, registry } => {
      // Lint before indexing — never index a pack that fails the format rules.
      let text = ::std::fs::read_to_string(&pack)
        .unwrap_or_else(|e| fail(&::std::format!("read {}: {e}", pack.display())));
      let findings = crate::validate::validate(&text, ::std::option::Option::None);
      if !findings.ok() {
        for e in &findings.errors {
          ::std::eprintln!("{}: error: {e}", pack.display());
        }
        fail(&::std::string::String::from("pack fails validation — not publishing"));
      }
      let now = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH)
        .expect("system clock before 1970")
        .as_secs();
      let entry = crate::publish::publish(&pack, &registry, &id, now).unwrap_or_else(|e| fail(&e));
      ::std::eprintln!(
        "published {id} v{} → {} (LOCAL index only; public release stays counsel-gated)",
        entry["version"],
        registry.display()
      );
    }
  }
}

#[cfg(test)]
mod tests {
  fn parse(args: &[&str]) -> ::std::result::Result<super::Command, ::std::string::String> {
    super::parse_args(args.iter().map(|s| ::std::string::String::from(*s)).collect())
  }

  /// Why: the CLI surface is the tools' public API — a parser regression
  /// (dropped flag, reordered positional) would break every documented
  /// invocation in /tools/README.md without any other test noticing, because
  /// the subcommand logic itself is tested below the parser.
  #[test]
  fn parses_every_documented_subcommand_shape() {
    ::std::assert!(::std::matches!(
      parse(&["extract", "--catalog", "c.toml", "--profile", "p"]),
      ::std::result::Result::Ok(super::Command::Extract { .. })
    ));
    ::std::assert!(::std::matches!(
      parse(&["merge", "--pack", "p.toml", "--catalog", "c.toml", "-o", "out.toml"]),
      ::std::result::Result::Ok(super::Command::Merge { .. })
    ));
    match parse(&["validate", "a.toml", "b.toml", "--catalog", "c.toml"]) {
      ::std::result::Result::Ok(super::Command::Validate { packs, catalog }) => {
        ::std::assert_eq!(packs.len(), 2, "flags must not eat positional pack paths");
        ::std::assert!(catalog.is_some());
      }
      other => ::std::panic!("validate parse failed: {other:?}"),
    }
    ::std::assert!(::std::matches!(
      parse(&["sign", "p.toml", "--key", "k.seed", "--key-id", "r1"]),
      ::std::result::Result::Ok(super::Command::Sign { .. })
    ));
    ::std::assert!(::std::matches!(
      parse(&["verify", "p.toml"]),
      ::std::result::Result::Ok(super::Command::Verify { .. })
    ));
    match parse(&["publish", "p.toml", "--id", "prof/dom"]) {
      ::std::result::Result::Ok(super::Command::Publish { registry, .. }) => ::std::assert_eq!(
        registry,
        ::std::path::PathBuf::from("registry/index.json"),
        "publish must default to the local counsel-safe index"
      ),
      other => ::std::panic!("publish parse failed: {other:?}"),
    }
  }

  /// Why: bad invocations must produce a diagnosable Err — not a panic, not a
  /// silently-misparsed Command that then touches files. These are the paths a
  /// user hits FIRST when learning the tool.
  #[test]
  fn malformed_invocations_error_instead_of_misparsing() {
    ::std::assert!(parse(&["extract", "--profile", "p"]).is_err(), "missing required --catalog");
    ::std::assert!(parse(&["extract", "--catalog"]).is_err(), "dangling flag value");
    ::std::assert!(parse(&["validate", "--catalog", "c.toml"]).is_err(), "no pack paths");
    ::std::assert!(parse(&["sign", "--key", "k.seed"]).is_err(), "sign without a pack");
    ::std::assert!(parse(&["frobnicate"]).is_err(), "unknown subcommand");
  }

  /// Why: council-audit NN13 — a flag missing its value must NOT swallow the
  /// next flag, a repeated flag must NOT leak to a positional, and leftover
  /// tokens (typos / dropped args) must error rather than be silently ignored —
  /// all three previously mis-parsed into a Command that then touched files.
  #[test]
  fn arg_traps_error_instead_of_silently_misparsing() {
    // Missing value swallows the next flag.
    ::std::assert!(
      parse(&["sign", "p.toml", "--key", "--key-id", "r1"]).is_err(),
      "--key with no value must not consume --key-id"
    );
    // Repeated flag.
    ::std::assert!(
      parse(&["validate", "ok.toml", "--catalog", "c.toml", "--catalog", "c2.toml"]).is_err(),
      "duplicate --catalog must error"
    );
    // Leftover positional after the consumed one.
    ::std::assert!(
      parse(&["verify", "p.toml", "extra-junk"]).is_err(),
      "unexpected trailing arg must error"
    );
    ::std::assert!(
      parse(&["extract", "--catalog", "c.toml", "--profile", "p", "junk"]).is_err(),
      "extract takes no positional"
    );
  }

  /// Why: bare `r14n` and `--help`/`-h`/`help` must all land on Help — exiting
  /// with an error on a bare invocation is hostile, and Help being a Command
  /// variant (not an inline exit) is what keeps this testable at all.
  #[test]
  fn help_paths_resolve_to_the_help_command() {
    for args in [&[][..], &["--help"][..], &["-h"][..], &["help"][..]] {
      ::std::assert!(::std::matches!(
        parse(args),
        ::std::result::Result::Ok(super::Command::Help)
      ));
    }
  }
}
