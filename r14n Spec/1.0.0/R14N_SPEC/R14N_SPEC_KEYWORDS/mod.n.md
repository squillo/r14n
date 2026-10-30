---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "Reserved Block Predicate Keywords"
version: "1_0_0"
---

# r14n Specification Reserved Block Predicate Keywords [DRAFT 1_0_0]

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 RFC2119 RFC8174 when, and only when, they appear in all capitals, as shown here.

The keywords "N Lang" and "N" are ubiquitous of "N Language".

Copyright 2026 Squillo, Inc. <legal@squillo.com>. These Snapp sources are licensed under Apache-2.0 like the rest of this repository's code (see the repo `LICENSE`); N Lang itself — the language, compiler, runtime, and Snapp tooling — is proprietary to Squillo, Inc.

---

## Summary

r14n block predicate keywords express regulatory policy packs — a `(profile × domain × subject) → required-controls` mapping, resolved fail-closed — as terse, readable N Lang programs that desugar to the core-book instance form. These keywords extend N's list of Block Predicate Keywords.

**Deliberate minimalism (the Raconteur principle):** a keyword represents a WHOLE domain concept — a regulatory policy pack, a control catalog, a Snapp's compliance declaration — never a field alias. The keyword blocks are `policies`, `catalog` and `snapp_compliance`; the `frameworks` document declares types only (the OPTIONAL `frameworks` prop of a catalog control), not a keyword.

> **⚠ NOT LEGAL ADVICE.** Nothing declared here states what any law requires or that any configuration is lawful, certified or compliant. A framework link relates a control to an edition of a framework; a compliance declaration states what a Snapp says it touches. Neither is a determination.

---

## Module Declarations

```nlang
mod R14N_SPEC_KEYWORD_POLICIES_1_0_0 {};
mod R14N_SPEC_KEYWORD_CATALOG_1_0_0 {};
mod R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0 {};
mod R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0 {};
```

## Public Re-exports

```nlang
mod * {
  pub @use R14N_SPEC_KEYWORD_POLICIES_1_0_0 {}
  pub @use R14N_SPEC_KEYWORD_CATALOG_1_0_0 {}
  pub @use R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0 {}
  pub @use R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0 {}
}
```

## Keywords

The keyword form is DRAFT (parser Wave I.4 — keyword→wire desugaring is NOT yet in the toolchain); the core-book form via `@deps` on the `r14n` contract Snapp is the v1 wire authored today.

### New Keyword Blocks

#### `policies`

Declares a regulatory policy pack: a `(profile × domain) → required-controls` prescription with a per-`subject` table and a `legally_required` floor. Fold position decides declare-vs-refine; a Snapp overlay may only TIGHTEN (raise the floor, never relax it).

- [R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md)

#### `catalog`

Declares a domain's Control Catalog: the jurisdiction-neutral vocabulary of control keys every pack for that domain references. A control MAY carry an OPTIONAL `frameworks` link (below).

- [R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md](R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md)

#### `snapp_compliance`

Declares what a Snapp says it touches: the `(domain, subject)` pairs it exercises (a keyed table, domain to subjects) and the data classes it handles (SUBJECT tokens, never profiles). A declaration, not a determination.

- [R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0.n.md](R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0.n.md)

### Types Without a Keyword

#### `frameworks`

The types behind the OPTIONAL `frameworks` prop of a catalog control: `FrameworkLinks` (a CLOSED set of framework editions, one optional direct prop each), `FrameworkLink` (a closed `relation`, optional `criteria`, optional `review`) and `FrameworkReview`. Ships ahead of its content: no shipped catalog populates it.

- [R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md](R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md)

---

## References

- r14n `spec/RLPS-v0.1.md` — the Regulatory Localization Pack Specification
- [R14N.n.md](../R14N.n.md) — global defaults document
