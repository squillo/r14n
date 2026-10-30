---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "frameworks"
version: "1_0_0"
---

```nlang
@name: "r14n_frameworks"
@author: "Scott Wyatt <legal@squillo.com>"
@license: "SEE LICENSE IN LICENSE"
```

# r14n Specification: Reserved Block Predicate Keywords: `frameworks` [DRAFT 1_0_0]

> **⚠ NOT LEGAL ADVICE.** A framework link says a control is RELATED to an edition of a
> compliance framework. It is a pointer for a human reviewer, never a determination. It does not
> say what any law or framework requires, it does not say a control is sufficient for anything,
> and nothing here states that a configuration is lawful or certified. A resolver MUST NOT vary
> a decision on it.

## Purpose

A control in a Control Catalog (see `catalog`) may carry an OPTIONAL typed link to the
compliance-framework editions it relates to. This document declares the three types behind that
link. They are not a keyword in the `policies` / `catalog` sense — there is no `frameworks`
block collection — they are the types of one OPTIONAL prop, `frameworks`, on `ControlDef`.

**The contract ships AHEAD of its content.** No shipped catalog carries a `frameworks` block, and
the `criteria` list ships EMPTY OF REAL IDENTIFIERS: licensing for the identifier spaces of the
frameworks below is not cleared. Fixtures use obviously fictional placeholders (`EXAMPLE-1`).

## The vocabulary

**Editions are a CLOSED set of snake_case ids, one OPTIONAL direct prop each:**
`soc2_tsc_2017`, `iso27001_2022_a1_2024`, `hipaa_45cfr164_2013`, `hecvat_4_1_6`.

- One id names one PUBLISHED EDITION. A new edition of the same framework is a NEW id, so a link
  keeps pointing at the text it was reviewed against.
- Adding an edition is a LAST-POSITION add: append a prop with the next `priority`. Never reorder,
  rename or remove one — an edition id outlives the control that cites it as audit evidence
  (invariant 10, the same reasoning as a control key).
- Listing an edition is a vocabulary entry. It is not a claim about anyone's certification scope.
- **One spelling.** The wire spelling is the snake_case id everywhere in r14n: the N Lang prop name
  here, the key in `schema/catalog.schema.json`, and the TOML key in a catalog are the same string.
  There is no alias table in code. Hyphenated spellings (`soc2-tsc-2017`) are informal aliases for
  prose only, and the type refuses one (`framework_links_reject_a_hyphenated_alias`).

**`relation` is a CLOSED set: `informs` | `contributes` | `necessary`.** There is deliberately no
`satisfies`, `meets` or `complies`: no control is sufficient for a framework criterion, and a
closed set makes that unsayable here instead of merely discouraged.

| Relation | Reads as |
|---|---|
| `informs` | background a reviewer may find useful |
| `contributes` | one part of what an organization might do about a criterion |
| `necessary` | the criterion cannot be addressed without it — NOT "sufficient" |

**`criteria`** is an optional list of non-empty identifier strings, spelled as the framework's own
publisher spells them. Free-form on purpose: the identifier space belongs to the publisher, not to
this vocabulary. EMPTY OF REAL IDS until licensing clears.

**`review`** is optional: a `status` from the closed set `draft` | `requires_signoff` | `approved`
and an optional `counsel` string. `approved` records that a review HAPPENED. It is not a finding
that the link, or anything it names, is lawful, sufficient or current. It is deliberately NOT
`LegalReview` (whose set also carries `not_required`, a state with no meaning for a link: either
somebody has looked or they have not).

**Three rules cross fields**, and are stated here because a reader of the type needs them:

1. `relation = "necessary"` REQUIRES a `review` whose `status` is `approved`. `necessary` is the one
   relation that reads as a claim about someone else's compliance posture, so it does not ship
   unreviewed.
2. A link to the LEGAL-TEXT edition `hipaa_45cfr164_2013` REQUIRES a `review` block, whatever its
   relation. A voluntary framework is a convention; a legal-text edition is a claim about the law, and
   an unreviewed claim about the law must not ship.
3. A `review` whose `status` is `approved` REQUIRES `counsel`. An approval with nobody standing behind
   it is the hazard a pack's `legal_review` block already closes by requiring an attorney of record
   before `approved`; without this, rule 1 could be met by writing one word. `draft` and
   `requires_signoff` need no name, so unfinished work stays expressible.

## What the types enforce, and what they do not (MEASURED)

Measured 2026-10-30 on `nlang 0.1.0-dev (167513cb1 2026-10-26)` in a clean-room copy of this
Snapp, every probe a positive control (the type resolves) paired with a negative that flips ONE
value. Each negative was also run alone as a plain positive test to read the diagnostic that
refused it, so none passes for an accidental reason.

| Probe | Result | Diagnostic |
|---|---|---|
| unknown edition prop on `FrameworkLinks` | ENFORCED | COMP-438 |
| hyphenated edition spelling | ENFORCED | COMP-438 |
| unknown prop inside a link or a review | ENFORCED | COMP-438 |
| `relation` outside the set (`satisfies`, `Informs`) | ENFORCED | COMP-546 |
| `relation` missing | ENFORCED | COMP-540 |
| `review.status` outside the set (`rubber_stamped`, `not_required`) | ENFORCED | COMP-546 |
| `review.status` missing | ENFORCED | COMP-540 |
| empty `criteria` list; duplicate `criteria` | ENFORCED | COMP-543, COMP-545 |
| an edition given a bare string instead of a link | ENFORCED | COMP-540 |
| bad `relation` reached THROUGH `ControlDef.frameworks` | ENFORCED | COMP-546 |
| `necessary` without a review; with a `draft` review | NOT ENFORCED | accepted |
| `hipaa_45cfr164_2013` without a review | NOT ENFORCED | accepted |
| empty `frameworks: {}` nested in a `ControlDef` | NOT ENFORCED | accepted |

Closed edition sets and closed relation sets are real here, because both are direct props with
`values` and no wildcard arm. The cross-field rules are NOT expressible: a rule conditioned on the
VALUE of another prop needs a union that discriminates, and unions do not discriminate in this
toolchain. Each unenforced row is pinned as an INVERTED TRIPWIRE in
`smoke_tests/type_constraint_tests.n`: green while the gap stands, red the day the compiler starts
refusing it, at which point the arm is replaced by the `should_fail = true` arm it holds the place
of. Until then the three cross-field rules are carried by `schema/catalog.schema.json` and must not
be read as enforced by these types.

`additional_props = false` is the DEFAULT here (COMP-438 fires with and without the flag). It is
stated explicitly on every type below anyway: closure of the edition set is load-bearing, and an
unstated default is a place for a future toolchain to widen it silently.

## Object Blocks

```nlang
pub mod * blocks "FrameworkReview" {
  atom             = list
  additional_props = false
  description      = "{FrameworkReview} Object Block — where the review of one framework link stands"

  props {
    status {
      atom        = str
      values      = ["draft", "requires_signoff", "approved"]
      description = "Where the review stands. `draft` — being written. `requires_signoff` — awaiting a named reviewer. `approved` — a named reviewer has signed off. `approved` records that a review HAPPENED; it is not a finding that the link, or anything it names, is lawful, sufficient or current. A closed set, and NOT LegalReview's: that set also carries `not_required`, which has no meaning for a link."
      priority    = 1
    }
    counsel? {
      atom        = str
      description = "Who reviewed, as a free-form string. Not verified here, not a legal credential (nothing checks that it names a lawyer, a firm or a person), and not an attorney-of-record claim: that contract belongs to a pack's `legal_review`, which this field neither replaces nor mirrors."
      priority    = 2
    }
  }
}

pub mod * blocks "FrameworkLink" {
  atom             = list
  additional_props = false
  description      = "{FrameworkLink} Object Block — one control's link to one framework edition: HOW it relates, WHERE, and who has looked"

  props {
    relation {
      atom        = str
      values      = ["informs", "contributes", "necessary"]
      description = "How the control relates to the edition. `informs` — background. `contributes` — one part of what an organization might do about a criterion. `necessary` — a reviewer has recorded that the criterion cannot be addressed without it (a recorded claim, never verified here); NOT `sufficient`. Closed, and deliberately without `satisfies`, `meets` or `complies`. `necessary` REQUIRES a `review` with status `approved` — a rule these types cannot express (see the tripwires), carried by the published catalog schema."
      priority    = 1
    }
    criteria? {
      atom         = list
      items        { atom = str }
      min_items    = 1
      unique_items = true
      description  = "Identifiers of the edition's criteria this link bears on, spelled as the framework's publisher spells them. Optional, and an EMPTY list is refused: it reads as answered, which is worse than absent. Ships EMPTY OF REAL IDENTIFIERS until licensing is cleared; fixtures use `EXAMPLE-1`. Item atom kind and non-emptiness are NOT enforced by these types (list items do not enforce); the published catalog schema carries them."
      priority     = 2
    }
    review? {
      type        = <FrameworkReview>
      description = "Who has looked at this link, and where it stands. Optional in general; REQUIRED when `relation` is `necessary` (with status `approved`) and for ANY link to `hipaa_45cfr164_2013`. Neither requirement is enforced by these types."
      priority    = 3
    }
  }
}

pub mod * blocks "FrameworkLinks" {
  atom             = list
  additional_props = false
  min_props        = 1
  description      = "{FrameworkLinks} Object Block — OPTIONAL typed links from one control to the framework EDITIONS it relates to. A pointer for a human, never a determination: it does not say a control is sufficient for, or satisfies, any criterion, and a resolver MUST NOT vary a decision on it. NOT LEGAL ADVICE"

  props {
    // The edition set is CLOSED and ORDERED: a LAST-POSITION add only. Append the next
    // priority; never reorder, rename or remove (an edition id is audit evidence).
    soc2_tsc_2017? {
      type        = <FrameworkLink>
      description = "Link to the 2017 edition of a SOC 2 trust-services criteria set."
      priority    = 1
    }
    iso27001_2022_a1_2024? {
      type        = <FrameworkLink>
      description = "Link to the 2022 edition of an ISO 27001 information-security standard as amended in 2024."
      priority    = 2
    }
    hipaa_45cfr164_2013? {
      type        = <FrameworkLink>
      description = "Link to the 2013 edition of the HIPAA 45 CFR part 164 text. A legal-text edition (a regulation): a link to it REQUIRES a `review` block (not enforced by this type)."
      priority    = 3
    }
    hecvat_4_1_6? {
      type        = <FrameworkLink>
      description = "Link to version 4.1.6 of the HECVAT higher-education vendor assessment."
      priority    = 4
    }
  }
}
```

## References

- [R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md](R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md) — `ControlDef`, whose OPTIONAL `frameworks` prop takes `FrameworkLinks`
- `schema/catalog.schema.json` — the published schema, which carries the three cross-field rules
- `docs/not-legal-advice.md` — what a control, a link and a pack do and do not claim
