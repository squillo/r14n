---
section: "r14n Specification"
name: "r14n Spec Module Index"
version: "1_0_0"
---
# r14n Specification — Module Index

Deterministic declaration index for the R14N_SPEC directory.
Per the N Language Book `1. Introduction/1. Introduction.md` §1.1 "The Determinism Invariant":
declaration order IS load order.

R14N defines the core block type schema + global defaults. R14N_SPEC_KEYWORDS
declares the block predicate keyword sub-modules. Load order: schema before
keywords (mirrors the Taxa gold standard).

## Module Declarations

```nlang
mod R14N {};
mod R14N_SPEC_KEYWORDS {};
```

## Public Re-exports

All keywords are re-exported at the root level to maintain backwards compatibility:

```nlang
mod * {
  pub use R14N {}
  pub use R14N_SPEC_KEYWORDS {}
}
```
