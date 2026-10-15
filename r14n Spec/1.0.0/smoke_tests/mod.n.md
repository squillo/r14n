---
section: "r14n Specification"
subsection: "Smoke Tests"
name: "Smoke Test Module Index"
version: "1_0_0"
---

# r14n Specification Smoke Tests — Module Index

Deterministic declaration index for the smoke-test packs. Per the N Language Book,
`1. Introduction/1. Introduction.md` §1.1 "The Determinism Invariant": declaration order IS
load order, and **undeclared sibling files never load**.

**Why this file exists.** Without it, these ten packs were present on disk and compiled by
nothing. The Snapp's emitted bundle reported `included_files: 3` and contained zero pack
instances, so every `<PolicyPack>` instance here — the only exercise the type definition has —
was dead weight. A type whose instances never compile is a type whose constraints are untested:
the `values` closed sets, the `format` patterns, and `min_items` on `subjects` all go unchecked.
That is the gap this index closes, and it is why a pack added to this directory must also be
declared below.

Copyright 2026 Squillo, Inc. <legal@squillo.com>. These Snapp sources are licensed under
Apache-2.0 like the rest of this repository's code (see the repo `LICENSE`); N Lang itself — the
language, compiler, runtime, and Snapp tooling — is proprietary to Squillo, Inc.

> **⚠ NOT LEGAL ADVICE.** These are Squillo's own `aggressive` / `minimal` baselines for Apple
> capability families, exercising the type definition. They are not statements of law, and
> `aggressive` ≠ `compliant`. Real-jurisdiction packs are counsel-gated and held out
> (`GOVERNANCE.md` hard gate 2).

---

## Module Declarations

Declared in domain order, each domain's `aggressive` baseline before its `minimal` one — the
same fail-closed-first reading order the postures resolve in. New packs append at the END
(last-position-additive); never reorder, never renumber.

```nlang
mod policies__health_aggressive {};
mod policies__health_minimal {};
mod policies__location_aggressive {};
mod policies__location_minimal {};
mod policies__nfc_aggressive {};
mod policies__nfc_minimal {};
mod policies__photo_library_aggressive {};
mod policies__photo_library_minimal {};
mod policies__smart_home_aggressive {};
mod policies__smart_home_minimal {};
```

---

## References

- [../R14N_SPEC/R14N_SPEC_KEYWORDS/R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](../R14N_SPEC/R14N_SPEC_KEYWORDS/R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md)
  — the `PolicyPack` / `Subject` type blocks these packs instantiate
- r14n `spec/RLPS-v0.1.md` — the Regulatory Localization Pack Specification
