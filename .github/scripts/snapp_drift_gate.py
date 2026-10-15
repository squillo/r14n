#!/usr/bin/env python3
"""Prove the N Lang Snapp sources and their committed artifacts still agree.

WHY THIS EXISTS, and why it does not run `nlang`.

The Snapp tree was authored and then built by nothing — no CI job, no Makefile, no test
referenced it. By the time anyone ran `nlang check` on it again, four separate pieces of
language drift had accumulated, ten authored packs had never once compiled, and the committed
bundle claimed a version its own filename contradicted. Every one of those was silent.

N Lang is proprietary and this repository's CI runs on public runners, so a gate here cannot
invoke the compiler. It does not need to. Every defect in that list was an AGREEMENT failure
between files that are all committed, and agreement is checkable with hashlib and json:

  1. a pack on disk that no index declares will never load (discovery is index-only; there is
     no filesystem-walk fallback, so the file is simply ignored)
  2. an index declaration with no file is a hard build error, found only at build time
  3. a committed bundle missing a declared pack is a stale artifact — the source moved and the
     emit did not follow
  4. a bundle whose `@snapp.version` contradicts its filename misleads every consumer that
     pins by filename
  5. an `integrity` pin that does not match the current bytes of its target breaks the
     dependent's build; re-emitting a framework bundle invalidates every pin on it, by design
  6. two `@`-bearing JSONs for one Snapp name is an ambiguous artifact, and the build-door
     sidecar carries no `@snapp` root at all, so a consumer that globbed it would load a
     bundle with no identity

What this gate CANNOT do is prove the committed bundle is what the current sources would emit.
That needs the compiler. Run the full pipeline locally before committing a source change:

    NLANG_CACHE_DIR=$(mktemp -d) <guard> -- check  -r "r14n Spec/1.0.0"
    NLANG_CACHE_DIR=$(mktemp -d) <guard> -- export -r "r14n Spec/1.0.0" -f json
    # then refresh dependent pins (the export above invalidates them) and repeat for baselines/

Usage: snapp_drift_gate.py [repo_root]   (default: the repo this script lives in)
Exit 0 = sources and artifacts agree. Exit 1 = they do not, and every failure says which.
"""

import hashlib
import json
import os
import re
import sys

CI = os.environ.get("GITHUB_ACTIONS") == "true"
failures: list[str] = []
notes: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)
    print(f"::error::{msg}" if CI else f"  FAIL  {msg}")


def ok(msg: str) -> None:
    print(f"  ok    {msg}")


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def read(path: str) -> str:
    with open(path, encoding="utf-8") as fh:
        return fh.read()


# ---------------------------------------------------------------- 1 + 2: index <-> files
def check_index_agreement(root: str, index_rel: str, glob_dir: str, suffix: str) -> set[str]:
    """Every source declared, every declaration sourced. Returns the declared stem set."""
    index_path = os.path.join(root, index_rel)
    if not os.path.isfile(index_path):
        fail(f"missing index {index_rel} — without it nothing in {glob_dir} loads at all")
        return set()

    declared = set(re.findall(r"^\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{\}\s*;", read(index_path), re.M))
    src_dir = os.path.join(root, glob_dir)
    on_disk = {f[: -len(suffix)] for f in os.listdir(src_dir) if f.endswith(suffix)} if os.path.isdir(src_dir) else set()

    for orphan in sorted(on_disk - declared):
        fail(
            f"{glob_dir}/{orphan}{suffix} exists but {index_rel} does not declare it — "
            f"discovery is index-only, so this file is compiled by NOTHING and anything it "
            f"declares is unverified. Add `mod {orphan} {{}};` at the END of the index."
        )
    for ghost in sorted(declared - on_disk):
        fail(f"{index_rel} declares `mod {ghost} {{}}` but {glob_dir}/{ghost}{suffix} does not exist")

    if on_disk and on_disk == declared:
        ok(f"{index_rel}: all {len(declared)} sources declared, no ghosts")
    return declared


def instance_names(root: str, glob_dir: str, stems: set[str], suffix: str) -> set[str]:
    """The typed instances a loaded source actually declares.

    NOT the file stems. The index declares files (`policies__health_aggressive`); a bundle keys
    packs by the instance inside the file (`health_aggressive`). Comparing the two sets directly
    reports every pack as missing — this function is why the gate reads the sources rather than
    trusting the filename convention, and it also catches an instance RENAMED without a re-emit.
    """
    pat = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*<\s*[\w:]*PolicyPack\s*>", re.M)
    found: set[str] = set()
    for stem in sorted(stems):
        p = os.path.join(root, glob_dir, stem + suffix)
        if os.path.isfile(p):
            found |= set(pat.findall(read(p)))
    return found


# ---------------------------------------------------------------- 3 + 4: bundle agreement
def check_bundle(root: str, bundle_rel: str, expect_packs: set[str], packs_key: str) -> None:
    path = os.path.join(root, bundle_rel)
    if not os.path.isfile(path):
        fail(f"missing emitted bundle {bundle_rel}")
        return
    try:
        doc = json.loads(read(path))
    except json.JSONDecodeError as exc:
        fail(f"{bundle_rel} is not valid JSON: {exc}")
        return

    meta = doc.get("@snapp")
    if not isinstance(meta, dict):
        fail(
            f"{bundle_rel} carries no `@snapp` root — this is the build-door sidecar shape, not "
            f"an export bundle. A consumer loading it gets a bundle with no identity."
        )
        return

    # 4: the filename is what consumers pin by, so the content must agree with it.
    stem = os.path.basename(path)
    if stem.endswith(".json"):
        stem = stem[: -len(".json")]
    if "@" in stem:
        fname_name, fname_ver = stem.split("@", 1)
        if meta.get("version") != fname_ver:
            fail(
                f"{bundle_rel}: `@snapp.version` is {meta.get('version')!r} but the filename says "
                f"{fname_ver!r}. Consumers pin by filename; re-emit rather than editing either."
            )
        else:
            ok(f"{bundle_rel}: version {fname_ver} matches filename")
        if meta.get("name") not in (fname_name, fname_name.replace("-", "_")):
            fail(f"{bundle_rel}: `@snapp.name` {meta.get('name')!r} != filename stem {fname_name!r}")

    # 3: a declared pack absent from the artifact means the emit did not follow the source.
    if expect_packs:
        present = doc.get(packs_key)
        present_keys = set(present.keys()) if isinstance(present, dict) else set()
        missing = expect_packs - present_keys
        if missing:
            fail(
                f"{bundle_rel}: declared pack(s) absent from the emitted artifact: "
                f"{sorted(missing)}. The source moved and the emit did not follow — re-export."
            )
        else:
            ok(f"{bundle_rel}: all {len(expect_packs)} declared packs present under `{packs_key}`")


# ---------------------------------------------------------------- 5: integrity pins
def check_pins(root: str, source_rel: str) -> None:
    path = os.path.join(root, source_rel)
    if not os.path.isfile(path):
        notes.append(f"{source_rel} absent — no pins to check")
        return
    body = read(path)
    # `from:` then `integrity:` within one @deps entry; both are literal string fields.
    entries = re.findall(
        r'from\s*:\s*"([^"]+)"\s*\n\s*integrity\s*:\s*"sha256-([a-f0-9]{64})"', body
    )
    if not entries:
        notes.append(f"{source_rel}: no local sha256 pins found")
        return
    for rel_target, pinned in entries:
        target = os.path.normpath(os.path.join(os.path.dirname(path), rel_target))
        if not os.path.isfile(target):
            fail(f"{source_rel} pins {rel_target} which does not exist")
            continue
        actual = sha256_file(target)
        if actual != pinned:
            fail(
                f"{source_rel}: stale integrity pin for {rel_target}\n"
                f"           pinned: sha256-{pinned}\n"
                f"           actual: sha256-{actual}\n"
                f"           Re-emitting a framework bundle invalidates every pin on it. Refresh "
                f"the pin in the SAME change as the re-emit, or the dependent fails to build."
            )
        else:
            ok(f"{source_rel}: pin for {os.path.basename(rel_target)} is current")


# ---------------------------------------------------------------- 6: one artifact per name
def check_artifact_ambiguity(root: str, dist_rel: str) -> None:
    d = os.path.join(root, dist_rel)
    if not os.path.isdir(d):
        fail(f"missing {dist_rel}/")
        return
    by_name: dict[str, list[str]] = {}
    for f in sorted(os.listdir(d)):
        if not f.endswith(".json") or "@" not in f:
            continue
        by_name.setdefault(f.split("@", 1)[0], []).append(f)
    for name, files in sorted(by_name.items()):
        if len(files) > 1:
            fail(
                f"{dist_rel}/ holds {len(files)} `@`-bearing JSONs for Snapp {name!r}: {files}. "
                f"A consumer globbing this cannot tell which is the bundle. Keep exactly one."
            )
    if by_name and not any(len(v) > 1 for v in by_name.values()):
        ok(f"{dist_rel}/: one bundle each for {sorted(by_name)}")


def main() -> int:
    root = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "..", "..")
    root = os.path.abspath(root)
    print(f"Snapp drift gate — {root}")

    stems = check_index_agreement(root, "r14n Spec/1.0.0/smoke_tests/mod.n.md", "r14n Spec/1.0.0/smoke_tests", ".n")
    packs = instance_names(root, "r14n Spec/1.0.0/smoke_tests", stems, ".n")
    if stems and not packs:
        fail("declared smoke-test sources contain no `<PolicyPack>` instances — check the parser")
    check_bundle(root, "snapp/r14n@0.1.0-alpha.1.json", packs, "packs")
    check_bundle(root, "snapp/r14n_baselines@1.0.0.json", set(), "policies")
    check_pins(root, "baselines/1.0.0/lib.n.md")
    check_artifact_ambiguity(root, "snapp")

    for n in notes:
        print(f"  note  {n}")
    if failures:
        print(f"\n{len(failures)} drift failure(s). Sources and committed artifacts disagree.")
        return 1
    print("\nSources and committed artifacts agree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
