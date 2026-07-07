//! # r14n — the RLPS pack-lifecycle CLI (maintainer-notes roadmap item 5)
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
mod extract;
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
  r14n verify   <pack> [--sig <file>]
  r14n publish  <pack> --id <profile/domain> [--registry <index.json>]

`publish` writes only the LOCAL registry index; making anything public is a
human decision behind the counsel gate (see the maintainer notes).";

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
  },
  Publish {
    pack: ::std::path::PathBuf,
    id: ::std::string::String,
    registry: ::std::path::PathBuf,
  },
  Help,
}

/// Pull `--flag value` out of the arg list (removing both tokens).
/// `Err` when the flag is present but dangling without a value.
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
  args.remove(idx);
  ::std::result::Result::Ok(::std::option::Option::Some(args.remove(idx)))
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
    "extract" => ::std::result::Result::Ok(Command::Extract {
      catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog")?, "--catalog")?),
      profile: require(take_flag(&mut args, "--profile")?, "--profile")?,
      out: take_flag(&mut args, "-o")?.map(::std::path::PathBuf::from),
    }),
    "merge" => ::std::result::Result::Ok(Command::Merge {
      pack: ::std::path::PathBuf::from(require(take_flag(&mut args, "--pack")?, "--pack")?),
      catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog")?, "--catalog")?),
      out: take_flag(&mut args, "-o")?.map(::std::path::PathBuf::from),
    }),
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
    "keygen" => ::std::result::Result::Ok(Command::Keygen {
      out: ::std::path::PathBuf::from(require(take_flag(&mut args, "--out")?, "--out")?),
    }),
    "sign" => {
      let key = ::std::path::PathBuf::from(require(take_flag(&mut args, "--key")?, "--key")?);
      let key_id = take_flag(&mut args, "--key-id")?;
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("sign needs a pack path"));
      }
      ::std::result::Result::Ok(Command::Sign {
        pack: ::std::path::PathBuf::from(args.remove(0)),
        key,
        key_id,
      })
    }
    "verify" => {
      let sig = take_flag(&mut args, "--sig")?.map(::std::path::PathBuf::from);
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("verify needs a pack path"));
      }
      ::std::result::Result::Ok(Command::Verify {
        pack: ::std::path::PathBuf::from(args.remove(0)),
        sig,
      })
    }
    "publish" => {
      let id = require(take_flag(&mut args, "--id")?, "--id (canonically <profile>/<domain>)")?;
      let registry = take_flag(&mut args, "--registry")?
        .map(::std::path::PathBuf::from)
        .unwrap_or_else(|| ::std::path::PathBuf::from("registry/index.json"));
      if args.is_empty() {
        return ::std::result::Result::Err(::std::string::String::from("publish needs a pack path"));
      }
      ::std::result::Result::Ok(Command::Publish {
        pack: ::std::path::PathBuf::from(args.remove(0)),
        id,
        registry,
      })
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
    Command::Verify { pack, sig } => {
      let sig_path = sig.unwrap_or_else(|| {
        ::std::path::PathBuf::from(::std::format!("{}.sig", pack.display()))
      });
      crate::keys::verify_file(&pack, &sig_path).unwrap_or_else(|e| fail(&e));
      ::std::eprintln!("{}: signature + content address OK", pack.display());
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
