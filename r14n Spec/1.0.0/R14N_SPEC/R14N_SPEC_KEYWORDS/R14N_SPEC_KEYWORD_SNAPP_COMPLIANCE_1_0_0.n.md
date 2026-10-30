---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "snapp_compliance"
version: "1_0_0"
---

```nlang
@name: "r14n_snapp_compliance"
@author: "Scott Wyatt <legal@squillo.com>"
@license: "SEE LICENSE IN LICENSE"
```

# r14n Specification: Reserved Block Predicate Keywords: `snapp_compliance` [DRAFT 1_0_0]

> **⚠ NOT LEGAL ADVICE.** A `SnappCompliance` is a DECLARATION of what a Snapp says it touches:
> which `(domain, subject)` pairs it exercises and which data classes it handles. It is not a
> determination. It does not say that anything the Snapp does is lawful, certified or compliant, and
> `declared` is not `verified`.

## Purpose

r14n resolves `(profile × domain × subject)` to a set of required controls. A Snapp that wants to
be read against that resolution first has to say WHICH `(domain, subject)` pairs it exercises, and
WHAT KIND of data it handles. This block is that statement, and it is deliberately the smallest one
that says both.

**Minimal on purpose.** Two props. A Snapp does not restate packs, controls, profiles or
postures here: those belong to the pack, the catalog and the resolver, and a second copy of any of
them is a place for the two to disagree.

## Shape, and why it is a keyed table

```nlang,ignore
snapp_compliance "MY_SNAPP" {
  exercises {
    recording_consent { subjects = ["telepresence"] }
    health            { subjects = ["read", "write"] }
  }
  data_classes = ["example_class"]
}
```

- **`exercises`** is a KEYED table: each KEY is a domain, each value an `ExercisedDomain` carrying
  the `subjects` of that domain the Snapp exercises. A `(domain, subject)` pair is therefore the
  key plus one item of its `subjects`.
- **`data_classes`** is a list of SUBJECT tokens. **Never profiles.** A profile names WHOSE law
  (a `/`-delimited tag such as the fictional `example/region`); a subject names WHAT THE ACTIVITY
  IS. A data class is a kind of thing a Snapp handles, so it is spelled as a subject token, in the
  grammar of `Subject.key`: `^[a-z][a-z0-9_]{0,63}$`. A `/` can never appear in one.
- **Both props are REQUIRED**, because absence must not read as an answer. `data_classes` may be
  EMPTY, and EMPTY IS MEANINGFUL: it says the Snapp declares that it handles none, where an absent
  list says nothing was declared.

**Why a keyed table and not a list of `{domain, subject}` rows.** List ITEMS do not enforce
required props in this toolchain (measured, and pinned by the existing `Subject` arms): a row
missing its `subject` would compile. A keyed table puts the domain in the KEY, where it cannot be
omitted, and the entry's `subjects` is a direct prop on a typed entry, where "required" IS
enforced. It also has the same keyed-table shape the catalog uses for `[control.<key>]`, so the
artifact a reader opens and the typed source share one shape.

**Why the entry is an object and not a bare list** (`health = ["read"]`). An array cannot grow a
prop; an object can. A later, additive field on an exercised domain would otherwise be a breaking
change to every Snapp that already declared one.

## What the types enforce, and what they do not (MEASURED)

Measured 2026-10-30 on `nlang 0.1.0-dev (167513cb1 2026-10-26)` in a clean-room copy of this
Snapp. Every row is a positive control paired with a negative that flips ONE value, and each
negative was also run alone as a plain positive test to read the diagnostic that refused it.

| Probe | Result | Diagnostic |
|---|---|---|
| `SnappCompliance` missing `exercises`; missing `data_classes` | ENFORCED | COMP-540 |
| unknown prop on `SnappCompliance` or `ExercisedDomain` | ENFORCED | COMP-438 |
| `ExercisedDomain` missing `subjects` (direct, and as a keyed entry) | ENFORCED | COMP-540 |
| empty `subjects` list | ENFORCED | COMP-543 |
| duplicate `subjects` item; duplicate `data_classes` item | ENFORCED | COMP-545 |
| an entry given a bare list instead of an object | ENFORCED | COMP-543 |
| `ExercisedDomains` with no entries (instantiated directly) | ENFORCED | COMP-543 |
| `exercises: {}` nested inside a `SnappCompliance` | NOT ENFORCED | accepted |
| a domain KEY outside `^[a-z][a-z0-9_]{0,63}$` | NOT ENFORCED | accepted |
| a profile-shaped token (`example/region`) in `data_classes` | NOT ENFORCED | accepted |
| a non-string item in `subjects` or `data_classes` | NOT ENFORCED | accepted |

Where a row is not enforced it is pinned as an INVERTED TRIPWIRE in
`smoke_tests/type_constraint_tests.n` — green while the gap stands, red the day the compiler
starts refusing it, at which point the arm becomes the `should_fail = true` arm it holds the place
of. The unenforced rows are exactly the ones a SCHEMA can carry (`propertyNames`, `pattern`,
`items`, `minProperties`) and a type cannot, which is what a schema for this block is
for. They are not enforced by these types and must not be read as if they were.

**Why the keyed table uses a WILDCARD arm and not the catalog's regex arm.** Neither enforces the
key (both rows above are measured, and the catalog's own tripwire pins the regex case). In the lab
copy the wildcard arm was strictly stronger: `min_items` inside an entry was enforced under a
wildcard arm and not under a regex arm. That comparison concerns a type this Snapp does not
declare, so no shipped arm pins it; it is recorded here as a measurement, not a guarantee. The key
format is documented above. Nothing enforces it today: no published schema covers a
declaration and these types do not, so a consumer must check it.

## Object Blocks

```nlang
blocks "snapp_compliance" {
  atom        = list
  description = "{SnappCompliance} Object Block — what a Snapp declares it exercises and handles"
  blocks * {
    type = <SnappCompliance>
  }
}

pub mod * blocks "SnappCompliance" {
  atom             = list
  additional_props = false
  description      = "{SnappCompliance} Object Block — the (domain, subject) pairs a Snapp exercises and the data classes it handles. A declaration, not a determination. NOT LEGAL ADVICE"

  props {
    exercises {
      type        = <ExercisedDomains>
      description = "The domains this Snapp exercises, as a KEYED table: domain -> the subjects of that domain it exercises. REQUIRED. At least one entry is expected (an `exercises` table with none says nothing); that is NOT enforced when this table is nested here (see the tripwire), and is left to the block's JSON schema."
      priority    = 1
    }
    data_classes {
      atom         = list
      items        { atom = str }
      unique_items = true
      description  = "The data classes this Snapp handles, as SUBJECT tokens (`^[a-z][a-z0-9_]{0,63}$`) — never profiles. REQUIRED, and EMPTY IS MEANINGFUL: an empty list declares that the Snapp handles none, where an absent one declares nothing. Item atom kind and token shape are NOT enforced by this type (list items do not enforce; a `format` facet is not applied); they are left to the block's JSON schema."
      priority     = 2
    }
  }
}

pub mod * blocks "ExercisedDomains" {
  atom             = list
  additional_props = false
  min_props        = 1
  description      = "{ExercisedDomains} Object Block — domain names to the subjects exercised in each, keyed by the domain itself"

  props {
    // The key IS the domain, spelled as a catalog domain name: `^[a-z][a-z0-9_]{0,63}$`.
    // A WILDCARD arm rather than a regex arm: neither enforces the key, and the wildcard arm
    // enforced strictly more inside an entry when measured. The key format is NOT enforced
    // anywhere today (no published schema covers a declaration); the tripwire in the smoke suite
    // pins that this type does not.
    * {
      type        = <ExercisedDomain>
      description = "One exercised domain."
    }
  }
}

pub mod * blocks "ExercisedDomain" {
  atom             = list
  additional_props = false
  description      = "{ExercisedDomain} Object Block — the subjects of one domain a Snapp exercises"

  props {
    subjects {
      atom         = list
      items        { atom = str }
      min_items    = 1
      unique_items = true
      description  = "The subject tokens (`^[a-z][a-z0-9_]{0,63}$`, the grammar of `Subject.key`) this Snapp exercises in the domain. At least one, no duplicates. Item atom kind is NOT enforced (list items do not enforce); it is left to the block's JSON schema."
      priority     = 1
    }
  }
}
```

## References

- [R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md) — `Subject`, whose `key` grammar a subject token shares
- [R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md](R14N_SPEC_KEYWORD_CATALOG_1_0_0.n.md) — the domains a Snapp's `exercises` keys name
- r14n `spec/RLPS-v0.1.md` §2 — domains, subjects and the `(profile × domain × subject)` resolution
- `docs/not-legal-advice.md` — what a declaration does and does not claim
