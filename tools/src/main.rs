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

mod catalog;
mod extract;
mod keys;
mod merge;
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
}

/// Pull `--flag value` out of the arg list (removing both tokens).
fn take_flag(
  args: &mut ::std::vec::Vec<::std::string::String>,
  flag: &str,
) -> ::std::option::Option<::std::string::String> {
  let idx = args.iter().position(|a| a == flag)?;
  if idx + 1 >= args.len() {
    ::std::eprintln!("error: {flag} needs a value");
    ::std::process::exit(2);
  }
  args.remove(idx);
  ::std::option::Option::Some(args.remove(idx))
}

fn require(value: ::std::option::Option<::std::string::String>, what: &str) -> ::std::string::String {
  match value {
    ::std::option::Option::Some(v) => v,
    ::std::option::Option::None => {
      ::std::eprintln!("error: {what} is required\n\n{USAGE}");
      ::std::process::exit(2);
    }
  }
}

fn parse_command() -> Command {
  let mut args: ::std::vec::Vec<::std::string::String> = ::std::env::args().skip(1).collect();
  if args.is_empty() {
    ::std::eprintln!("{USAGE}");
    ::std::process::exit(2);
  }
  let sub = args.remove(0);
  match sub.as_str() {
    "extract" => Command::Extract {
      catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog"), "--catalog")),
      profile: require(take_flag(&mut args, "--profile"), "--profile"),
      out: take_flag(&mut args, "-o").map(::std::path::PathBuf::from),
    },
    "merge" => Command::Merge {
      pack: ::std::path::PathBuf::from(require(take_flag(&mut args, "--pack"), "--pack")),
      catalog: ::std::path::PathBuf::from(require(take_flag(&mut args, "--catalog"), "--catalog")),
      out: take_flag(&mut args, "-o").map(::std::path::PathBuf::from),
    },
    "validate" => {
      let catalog = take_flag(&mut args, "--catalog").map(::std::path::PathBuf::from);
      if args.is_empty() {
        ::std::eprintln!("error: validate needs at least one pack path\n\n{USAGE}");
        ::std::process::exit(2);
      }
      Command::Validate {
        packs: args.into_iter().map(::std::path::PathBuf::from).collect(),
        catalog,
      }
    }
    "keygen" => Command::Keygen {
      out: ::std::path::PathBuf::from(require(take_flag(&mut args, "--out"), "--out")),
    },
    "sign" => {
      let key = ::std::path::PathBuf::from(require(take_flag(&mut args, "--key"), "--key"));
      let key_id = take_flag(&mut args, "--key-id");
      if args.is_empty() {
        ::std::eprintln!("error: sign needs a pack path\n\n{USAGE}");
        ::std::process::exit(2);
      }
      Command::Sign { pack: ::std::path::PathBuf::from(args.remove(0)), key, key_id }
    }
    "verify" => {
      let sig = take_flag(&mut args, "--sig").map(::std::path::PathBuf::from);
      if args.is_empty() {
        ::std::eprintln!("error: verify needs a pack path\n\n{USAGE}");
        ::std::process::exit(2);
      }
      Command::Verify { pack: ::std::path::PathBuf::from(args.remove(0)), sig }
    }
    "publish" => {
      let id = require(take_flag(&mut args, "--id"), "--id (canonically <profile>/<domain>)");
      let registry = take_flag(&mut args, "--registry")
        .map(::std::path::PathBuf::from)
        .unwrap_or_else(|| ::std::path::PathBuf::from("registry/index.json"));
      if args.is_empty() {
        ::std::eprintln!("error: publish needs a pack path\n\n{USAGE}");
        ::std::process::exit(2);
      }
      Command::Publish { pack: ::std::path::PathBuf::from(args.remove(0)), id, registry }
    }
    "--help" | "-h" | "help" => {
      ::std::println!("{USAGE}");
      ::std::process::exit(0);
    }
    other => {
      ::std::eprintln!("error: unknown subcommand `{other}`\n\n{USAGE}");
      ::std::process::exit(2);
    }
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
  match parse_command() {
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
