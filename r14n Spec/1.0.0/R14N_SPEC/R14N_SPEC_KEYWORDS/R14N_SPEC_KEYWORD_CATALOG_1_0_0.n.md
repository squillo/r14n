---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "catalog"
version: "1_0_0"
---

```nlang
@name: "r14n_catalog"
@author: "Scott Wyatt <legal@squillo.com>"
@license: "SEE LICENSE IN LICENSE"
```

# r14n Specification: Reserved Block Predicate Keywords: `catalog` [DRAFT 1_0_0]

> **⚠ NOT LEGAL ADVICE.** A Control Catalog declares that a control key EXISTS and what KIND
> of thing it is. It never states what any jurisdiction requires, and a complete catalog is
> not a compliant configuration.

## Purpose

The Control Catalog is the **jurisdiction-neutral vocabulary every pack references**: one
entry per control key, with a deontic kind and a title. Packs reference these keys; they do
not redefine them (spec §2).

It was the last published r14n artifact with no type. Ten catalogs and 59 control keys were
hand-written TOML, guarded only by a Rust test. That test is real — it checks parse, closed-set
kinds, domain/filename agreement, the disclaimer, prohibition rule 1 and deprecation
successors, each mutation-verified — but it is a guard bolted beside the artifact rather than a
contract the artifact is built from.

## What the type enforces that the TOML could not

Measured 2026-10-26 against this framework's own types: `values` closed sets ARE enforced and
required-prop presence IS enforced, while a `format` facet on a scalar prop is NOT (two
tripwires in `baselines/1.0.0/policy_pack_tests.n` pin that). So `kind` becomes genuinely
closed here, and the control KEY gets its format enforced a different way —

⚠ **The control KEY's shape is still not enforced, and I was wrong to claim it was.** This
block constrains `[control.<key>]` with a REGEX PROP PATTERN rather than a `format` facet, on
the reasoning that a keyed table makes the key a PROP NAME and a regex prop pattern is a key
matcher — with no wildcard arm declared, a key outside the pattern should have nothing to
match (precedence Exact > Regex > Wildcard, per the N Language type-system
reference on custom types).

MEASURED 2026-10-26: it is not refused. `control_table_rejects_a_key_outside_the_regex_pattern`
in the smoke suite put `SignalNotice` in a `ControlTable` and it compiled. The arm is now an
inverted tripwire: green while the pattern goes unenforced, red the day it does not. The
pattern stays in the type because it DOCUMENTS the shape spec §2 gives a control key, and
because replacing it with a wildcard arm would be strictly weaker.

So the key format is enforced in `tools/tests/catalog_integrity.rs`, the layer that can
actually do it — alongside the other rules no type expresses (prohibition rule 1, deprecation
successors, the disclaimer). What the type DOES enforce is the control DEFINITION: `kind` is a
genuinely closed set here and `title` is genuinely required, both proven by paired arms. That
is more than the TOML carried on its own, and it is less than I first wrote.

## Shape, and why it mirrors the TOML exactly

The emitted node must equal the published `<domain>.catalog.toml` document, because the Book's
`--format toml` definition (3. Reference/14. Implementation/8. Binary Format §Export, :364-369)
emits the same document as `json` rendered as TOML 1.0 in the `json` order. So `domain` and
`control` are siblings, `control` is a keyed table and not a list of rows, and nothing here is
nested a level deeper than the artifact nests it. A shape that needed projecting before it
validated is the defect this mirrors away from.

```nlang
blocks "catalog" {
  atom        = list
  description = "{Catalog} Object Block — the Control Catalog ledger, one entry per domain"
  blocks * {
    type = <ControlCatalog>
  }
}

pub mod * blocks "ControlCatalog" {
  atom        = list
  description = "{ControlCatalog} Object Block — one domain's control vocabulary"

  props {
    domain {
      type        = <CatalogDomain>
      description = "The domain this catalog declares. NESTED to mirror the published `[domain]` table rather than flattened to `domain_name`, so the typed source and the artifact a reader opens use one shape."
      priority    = 1
    }
    control {
      type        = <ControlTable>
      description = "The control keys, as a KEYED table — `[control.<key>]` in the artifact. A list of `{key, ...}` rows would emit an array and need projecting before it matched the published document."
      priority    = 2
    }
  }
}

pub mod * blocks "CatalogDomain" {
  atom        = list
  description = "{CatalogDomain} Object Block — which domain a catalog speaks for"

  props {
    name {
      atom        = str
      format      = "^[a-z][a-z0-9_]{0,63}$"
      description = "The domain name, which spec §2.2 makes the pack FILENAME on disk. A catalog whose name disagrees with its filename validates the wrong domain's packs while every log line names the right one."
      priority    = 1
    }
    description {
      atom        = str
      description = "What this domain governs, in one sentence a reviewer reads before the key list."
      priority    = 2
    }
  }
}

pub mod * blocks "ControlTable" {
  atom        = list
  description = "{ControlTable} Object Block — control keys to their definitions, keyed by the key itself"

  props {
    // The key IS the prop name, so its shape is enforced by a REGEX PROP PATTERN.
    // No wildcard arm is declared: a key outside this pattern matches nothing and is
    // refused, which is the enforcement a `format` facet on a value prop does not give.
    /^[a-z][a-z0-9_]{0,63}$/ {
      type        = <ControlDef>
      description = "One control key's definition."
    }
  }
}

pub mod * blocks "ControlDef" {
  atom        = list
  description = "{ControlDef} Object Block — one control key: what kind of thing it is, and what it names"

  props {
    kind {
      atom        = str
      values      = ["permit", "obligation", "prohibition"]
      description = "The deontic kind. A PROHIBITION is subtracted from every resolved set under every posture, so it must never appear in a requirement table — a pack that both requires and forbids a control is not satisfiable. Closed because a wrong kind inverts the control's meaning rather than degrading it."
      priority    = 1
    }
    title {
      atom        = str
      description = "A short human-readable title. `r14n extract` emits it into generated pack templates, so an empty one ships into other people's files."
      priority    = 2
    }
    description? {
      atom        = str
      description = "What the control names, and where its satisfier lives. OPTIONAL in shape, expected in practice — the catalog is where a reader learns what a key MEANS, since the key itself is opaque."
      priority    = 3
    }
    trigger? {
      atom        = str
      description = "When the enforcing gate must satisfy it (e.g. `before_capture`, `before_redeem`, `always`). A facet, not a schedule: the gate owns timing."
      priority    = 4
    }
    mechanism? {
      atom        = str
      description = "A stable name for HOW the control is discharged, for a receipt reader correlating a decision with an enforcement path."
      priority    = 5
    }
    deprecated? {
      atom        = date
      description = "The date this key was deprecated. A control key is long-lived audit evidence (invariant 10), so a retired key is DEPRECATED rather than deleted: receipts already issued name it, and a reader of one needs it to still resolve."
      priority    = 6
    }
    superseded_by? {
      atom        = list
      items       { atom = str }
      description = "What to use instead. Cross-domain successors are spelled `<domain>::<key>`, because a key is namespaced by its owning domain (spec §2) and a bare name would be ambiguous. A deprecation without this leaves the holder of an issued receipt no forward path."
      priority    = 7
    }
    frameworks? {
      type        = <FrameworkLinks>
      description = "OPTIONAL typed links from this control to the compliance-framework EDITIONS it relates to (see the `frameworks` keyword doc). A pointer for a human reviewer, never a determination: it does not say the control is sufficient for, or satisfies, any criterion, and it is NOT LEGAL ADVICE. A resolver MUST NOT vary a decision on it. Absent means no link has been made, which is not the same as no relationship existing. ADDITIVE: a control without this prop validates exactly as before."
      priority    = 8
    }
    verified_by? {
      atom        = str
      description = "A pointer to the external specification section that defines how this control is verified (an interoperability spec's anchor). For a human tracing a control to its defining text; never dereferenced by a resolver. Declared here because shipped catalogs use it and an undeclared prop is REFUSED by default, so without this a catalog carrying it could not be authored through the typed path."
      priority    = 9
    }
    attestation_mode? {
      atom        = str
      description = "Which attestation mode this control requires, where a domain has more than one and they are NOT interchangeable. Open rather than closed: the shipped values are a two-member set in one domain, and closing it here would refuse a domain whose modes this vocabulary has not met. Two controls may share a mechanism and differ only here; the control KEY carries the distinction in a receipt."
      priority    = 10
    }
    subject_rights? {
      atom        = list
      items       { atom = str }
      min_items   = 1
      unique_items = true
      description = "Data-subject rights this control bears on (for example withdrawal). Informational: it tells a reviewer which right a control serves, and asserts nothing about whether that right applies in any jurisdiction."
      priority    = 11
    }
    legal_basis_hint? {
      atom        = str
      description = "A HINT at the consent or processing model a control is usually associated with. The name hedges on purpose and the hedge is load-bearing: a catalog is jurisdiction-neutral and NOT LEGAL ADVICE, so this is a pointer for a human reviewer choosing which packs to author - never a determination, never evidence a basis is available, and a resolver MUST NOT vary a decision on it."
      priority    = 12
    }
  }
}
```

## Framework links (added 2026-10-30)

A control MAY carry an OPTIONAL `frameworks` prop of type `FrameworkLinks`: typed links to the
compliance-framework editions it relates to, declared in
[R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md](R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md). It is last-position
and optional, so every existing catalog validates exactly as before — which the unchanged
`control_def_resolves` arm proves, since it carries no `frameworks`.

A link says a control is RELATED to an edition. It is not a determination, it never says a control
is sufficient for any criterion, and a resolver MUST NOT vary a decision on it. **Nothing is
populated:** no shipped catalog has a `frameworks` block, because the identifier spaces of the
frameworks are not cleared for publication. NOT LEGAL ADVICE.

## References

- r14n `spec/RLPS-v0.1.md` §2 — control keys, domains, and the published pack form
- [R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md) — the packs that reference these keys
