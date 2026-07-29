---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "Reserved Block Predicate Keywords"
version: "1_0_0"
---

# r14n Specification Reserved Block Predicate Keywords [DRAFT 1_0_0]

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 RFC2119 RFC8174 when, and only when, they appear in all capitals, as shown here.

The keywords "N Lang" and "N" are ubiquitous of "N Language".

This document is currently privately licensed. Copyright Scott Wyatt <legal@squillo.com>, 2026 Copyright Squillo INC. 2026 <legal@squillo.com>.

---

## Summary

r14n block predicate keywords express regulatory policy packs — a `(profile × domain × subject) → required-controls` mapping, resolved fail-closed — as terse, readable N Lang programs that desugar to the core-book instance form. These keywords extend N's list of Block Predicate Keywords.

**Deliberate minimalism (the Raconteur principle):** r14n ships exactly ONE keyword in 1.0.0. A keyword represents a WHOLE domain concept — a regulatory policy pack — never a field alias.

---

## Module Declarations

```nlang
mod R14N_SPEC_KEYWORD_POLICIES_1_0_0 {};
```

## Public Re-exports

```nlang
mod * {
  pub use R14N_SPEC_KEYWORD_POLICIES_1_0_0 {}
}
```

## Keywords

The keyword form is DRAFT (parser Wave I.4 — keyword→wire desugaring is NOT yet in the toolchain); the core-book form via `@deps` on the `r14n` contract Snapp is the v1 wire authored today.

### New Keyword Blocks

#### `policies`

Declares a regulatory policy pack: a `(profile × domain) → required-controls` prescription with a per-`subject` table and a `legally_required` floor. Fold position decides declare-vs-refine; a Snapp overlay may only TIGHTEN (raise the floor, never relax it).

- [R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md)

---

## References

- r14n `spec/RLPS-v0.1.md` — the Regulatory Localization Pack Specification
- [R14N.n.md](../R14N.n.md) — global defaults document
