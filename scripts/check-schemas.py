#!/usr/bin/env python3
"""Validate the repo's producers against their JSON Schemas (council-audit
missing-item: the three schemas had no in-repo machine consumer).

Checks, all against schema/ + registry/:
  - every packs/**/*.r14n.toml against schema/pack.schema.json
  - every conformance/level-*.json reviewer_directory fixture against
    registry/reviewer-key.schema.json
  - the pack.schema.json / pack-index.schema.json / reviewer-key.schema.json
    documents are themselves valid Draft 2020-12

Exit 0 = all good; non-zero on the first failure. Requires `jsonschema` and (py<3.11)
`tomli`. NOT LEGAL ADVICE — this validates FORMAT, not legal correctness.
"""
import json
import pathlib
import sys

try:
    import tomllib  # py3.11+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib

import jsonschema

ROOT = pathlib.Path(__file__).resolve().parent.parent


def _load(p):
    return json.loads((ROOT / p).read_text())


def main() -> int:
    failures = []

    pack_schema = _load("schema/pack.schema.json")
    reviewer_schema = _load("registry/reviewer-key.schema.json")
    index_schema = _load("registry/pack-index.schema.json")
    receipt_schema = _load("schema/receipt.schema.json")

    for schema, name in (
        (pack_schema, "pack.schema.json"),
        (reviewer_schema, "reviewer-key.schema.json"),
        (index_schema, "pack-index.schema.json"),
        (receipt_schema, "receipt.schema.json"),
    ):
        try:
            jsonschema.Draft202012Validator.check_schema(schema)
            print(f"schema OK: {name}")
        except jsonschema.SchemaError as e:
            failures.append(f"{name} is not valid Draft 2020-12: {e.message}")

    pack_validator = jsonschema.Draft202012Validator(pack_schema)
    for pack in sorted((ROOT / "packs").rglob("*.r14n.toml")):
        data = tomllib.loads(pack.read_text())
        errs = sorted(pack_validator.iter_errors(data), key=str)
        rel = pack.relative_to(ROOT)
        if errs:
            failures.append(f"{rel}: {[e.message for e in errs]}")
        else:
            print(f"pack OK: {rel}")

    # Reviewer-directory fixtures embedded in conformance vectors.
    reviewer_validator = jsonschema.Draft202012Validator(reviewer_schema)
    for level in sorted((ROOT / "conformance").glob("level-*.json")):
        doc = json.loads(level.read_text())
        for vector in doc.get("vectors", []):
            directory = vector.get("reviewer_directory")
            if directory is None:
                continue
            errs = sorted(reviewer_validator.iter_errors(directory), key=str)
            label = f"{level.name}:{vector['name']}.reviewer_directory"
            # Fixtures may omit steward (they exercise key entries only) — allow that.
            errs = [e for e in errs if e.validator != "required" or "steward" not in str(e.message)]
            if errs:
                failures.append(f"{label}: {[e.message for e in errs]}")
            else:
                print(f"reviewer-dir fixture OK: {label}")

    # The shipped worked example receipt must validate against the receipt schema.
    receipt_validator = jsonschema.Draft202012Validator(receipt_schema)
    example = json.loads((ROOT / "docs/examples/receipt-ai-act-50.json").read_text())
    errs = sorted(receipt_validator.iter_errors(example["dpv_27560"]), key=str)
    if errs:
        failures.append(f"docs/examples/receipt-ai-act-50.json dpv_27560: {[e.message for e in errs]}")
    else:
        print("receipt example OK: docs/examples/receipt-ai-act-50.json (dpv_27560)")

    if failures:
        print("\nFAILURES:", file=sys.stderr)
        for f in failures:
            print(f"  - {f}", file=sys.stderr)
        return 1
    print("\nall schema checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
