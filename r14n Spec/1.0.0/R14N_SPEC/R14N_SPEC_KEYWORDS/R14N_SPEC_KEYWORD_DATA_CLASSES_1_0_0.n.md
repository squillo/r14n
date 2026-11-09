---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "data_classes"
version: "1_0_0"
---

```nlang
@name: "r14n_data_classes"
@author: "Scott Wyatt <legal@squillo.com>"
@license: "SEE LICENSE IN LICENSE"
```

# r14n Specification: Reserved Block Predicate Keywords: `data_classes` [DRAFT 1_0_0]

> **⚠ NOT LEGAL ADVICE.** A data class names a KIND of data a Snapp says it handles. It is not a
> determination. It does not say that any law applies to that data, that any law is satisfied, or
> that a configuration is lawful, certified or compliant. The family names below (a health-privacy
> rule, a student-records rule, a financial-privacy rule) say which kind of regime a reviewer
> might look at first; they are pointers for a human, never a mapping a resolver may act on.

**STATUS: MEASURED (2026-11-08).** This is the closed-set form. The first design named each class
as a keyed instance a dependent Snapp would reach by path. It was measured and does not compile: a
keyed instance cannot be written `pub`, so a dependent Snapp could never reach a member (see "What the
types enforce, and what they do not", below). Every row in that section's table was run.

## Purpose

`snapp_compliance` lets a Snapp declare the data classes it handles as a list of SUBJECT tokens.
Until now the tokens were an open string: any `^[a-z][a-z0-9_]{0,63}$` spelling compiled, a typo
(`phii`) read as a different class, and two Snapps that meant the same thing could spell it two
ways. A consumer that has to compare a Snapp's declaration against an organization's grant
(attested within declared within granted) needs a vocabulary both sides can point at.

This document is that vocabulary: a CLOSED, ORDERED set of data classes, declared as the props of
one type, `DataClassSet`. A Snapp names the classes it handles as the KEYS of a `DataClassSet`, so a
misspelled class is a prop the type does not declare. In the Snapp that declares the type the
compiler refuses it, and it does not become a new class. This is the idiom the `frameworks` document
uses for its closed edition set, and the refusal is the same measured one. **Across a dependency the
refusal does not hold yet** (measured, below): a dependent Snapp's misspelled class still compiles, so
a consumer that reads a declaration checks every key against the nine itself.

**Minimal on purpose.** Nine classes, a `title` and a `description` each. The set is the kinds of
data the shipped catalogs and the platform's own surfaces already distinguish, and no more. It is
not a taxonomy of the world and it does not try to be one: a class nobody has needed yet is a
LAST-position add later, not a guess now.

## The vocabulary

| Class | A kind of data | Where the shipped vocabulary already distinguishes it | Regime family a reviewer might look at first |
|---|---|---|---|
| `phi` | Health information about an individual | the `health` catalog | health-privacy rules |
| `pii` | Information that identifies, or can identify, a person | the `contact_import` and `data_residency` catalogs | general data-protection and privacy rules |
| `financial` | Account, payment, transaction and tax information | the `export_control` catalog | financial-privacy and payment-card rules |
| `biometric` | Measurements of a body or behavior used to recognize a person | the `recording_consent` and `credential_use` catalogs | biometric-privacy rules |
| `student_record` | Records directly related to a student at an educational institution | no shipped catalog yet; the `hecvat_4_1_6` edition id is the nearest | student-records rules |
| `credentials` | Secrets that prove identity or authority | the `credential_use` catalog | security-control frameworks |
| `location` | Where a person or a device is, or was | the `location` and `photo_library` catalogs | location-privacy rules |
| `communications` | The content or the metadata of messages, calls and recordings between people | the `recording_consent` and `contact_import` catalogs | communications-privacy and interception rules |
| `telemetry` | Usage, diagnostic and performance data about a product or a device | the `analytics_consent` and `diagnostics_upload` catalogs | data-protection rules on analytics and diagnostics |

How to read the table:

- **No criterion text.** The last column names a FAMILY of rules, not a section, a clause or a
  control identifier. The identifier spaces of the frameworks are licensed by their publishers and
  not cleared (see `frameworks`); a data class carries none. The gate that exists is narrower than
  that sentence might suggest: `tools/tests/framework_links.rs`
  (`no_shipped_catalog_populates_framework_links_before_the_licensing_gate_clears`) refuses a
  catalog CONTROL that carries a populated `frameworks` block while
  `FRAMEWORK_LINK_LICENSING_CLEARED` is false. It does not scan free text, and it does not read
  this document. Keeping this document identifier-free is held by the denylist-of-shapes test in
  `snapp_data_classes.rs` and by review.
- **A class is a kind, not a verdict.** Declaring `phi` says the Snapp says it handles health
  information. It does not say a health-privacy rule applies to the operator, or that it does not.
- **Classes overlap, and that is allowed.** A recorded clinical call is `phi`, `biometric` and
  `communications` at once. A Snapp lists every class it handles; the list is a set.
- **Order is part of the contract.** The order above is the declaration order and the
  `priority` order of the props of `DataClassSet`. It never changes (see Growth, below).

## Rules

1. **A data class is a SUBJECT token, never a profile.** Its key has the grammar of `Subject.key`:
   `^[a-z][a-z0-9_]{0,63}$`. A `/` can never appear in one. A profile names WHOSE law; a subject
   names WHAT THE ACTIVITY IS; a data class names WHAT KIND OF DATA IS HANDLED. The existing str
   form of `data_classes` keeps this rule, and so does every key of `DataClassSet`.
2. **The set is CLOSED where it is declared, and a consumer checks it across a dependency.**
   `DataClassSet` declares exactly the nine classes, with `additional_props = false`, so in this
   Snapp a key it does not declare does not compile. That is measured, in a `DataClassSet` nested in a
   `SnappCompliance` declaration, the shape a Snapp writes. In a DEPENDENT Snapp, which is every Snapp
   that declares its data classes, the same misspelling compiles: this toolchain does not carry
   `additional_props = false` across a dependency (measured, below, the same boundary
   `baselines/1.0.0/policy_pack_tests.n` records for scalar atom kind). Until it does, a consumer that
   reads a declaration MUST refuse a `data_class_set` key outside the nine itself, exactly as it must
   check the str form. The str form stays open everywhere: a misspelled string compiles, and the closed
   form is what lets a consumer tell a typo from a class.
3. **Growth is LAST-position.** A new class is appended with the next `priority`. A class is never
   reordered, renamed or removed: a class key outlives the Snapp that declared it as audit
   evidence (invariant 10, the same reasoning as a control key and an edition id). A rename is a new
   class; the old one stays.
4. **No domain join.** This vocabulary does NOT say which r14n domains a class implicates. Joining
   `phi` to the `health` domain would put a resolution decision in a vocabulary, and a resolver
   MUST NOT vary a decision on a data class. A consumer that wants a join keeps it in its own
   policy and cites this file for the names.
5. **Declared is not verified.** As with the rest of `snapp_compliance`, a class a Snapp lists is
   what its author says. A toolchain that attests the facts, and an installer that refuses when
   the attested facts exceed the declaration, belong to the consuming platform; this file declares
   names.

## Object Blocks

Two types. `DataClass` is one class as a Snapp declares it: a `title` and a `description`, both
REQUIRED, and nothing else. `DataClassSet` is the closed set: one OPTIONAL `DataClass` prop per
class, in the contract order, with `additional_props = false`. The prop descriptions below carry the
title and the sentence of the vocabulary table, so the closed set and the table read the same.

Both are declared in `*` (`pub mod * blocks`), as `SnappCompliance` and the sibling documents' types
are. This is measured, not style: a dependent Snapp imports this Snapp's types with `use { $::* }`, and
a type declared at the root (`pub blocks`) was NOT reached that way. In the dependent Snapp a
`SnappCompliance` missing a REQUIRED prop compiled, because the type behind it had not resolved.

`DataClassSet` has NO `min_props`. An empty set declares that the Snapp handles none, which is what
an empty `data_classes` list says, so the two forms agree. A `min_props = 1` would also say less than
it seems: this repository has MEASURED that `min_props` is not enforced on a nested empty table (the
`frameworks` document, `tripwire_empty_frameworks_nested_in_a_control_def_is_unenforced`), and a set
is always nested in a `SnappCompliance` declaration.

```nlang
pub mod * blocks "DataClass" {
  atom             = list
  additional_props = false
  description      = "{DataClass} Object Block — one kind of data a Snapp declares it handles. A KIND, never a statute and never a determination. NOT LEGAL ADVICE"

  props {
    title {
      atom        = str
      description = "A short human title for the class. REQUIRED."
      priority    = 1
    }
    description {
      atom        = str
      description = "What kind of data the class names, in a sentence, with no criterion identifier and no claim about any law. REQUIRED."
      priority    = 2
    }
  }
}

pub mod * blocks "DataClassSet" {
  atom             = list
  additional_props = false
  description      = "{DataClassSet} Object Block — the CLOSED set of data classes a Snapp declares it handles, keyed by class. A key this type does not declare is refused. An empty set declares none. A KIND of data, never a statute and never a determination. NOT LEGAL ADVICE"

  props {
    // The class set is CLOSED and ORDERED: a LAST-POSITION add only. Append the next
    // priority; never reorder, rename or remove (a class key is audit evidence).
    phi? {
      type        = <DataClass>
      description = "Health information: information about an individual's health, health care or payment for health care."
      priority    = 1
    }
    pii? {
      type        = <DataClass>
      description = "Personal information: information that identifies a person, or that can be combined with other information to identify one."
      priority    = 2
    }
    financial? {
      type        = <DataClass>
      description = "Financial information: account, payment, transaction and tax information about a person or an organization."
      priority    = 3
    }
    biometric? {
      type        = <DataClass>
      description = "Biometric information: measurements of a body or of behavior used to recognize a person, such as a voiceprint, a face geometry or a fingerprint."
      priority    = 4
    }
    student_record? {
      type        = <DataClass>
      description = "Student records: records directly related to a student that an educational institution maintains."
      priority    = 5
    }
    credentials? {
      type        = <DataClass>
      description = "Credentials: secrets that prove identity or authority, such as passwords, tokens and key material."
      priority    = 6
    }
    location? {
      type        = <DataClass>
      description = "Location: where a person or a device is, or was, at any precision."
      priority    = 7
    }
    communications? {
      type        = <DataClass>
      description = "Communications: the content or the metadata of messages, calls and recordings between people."
      priority    = 8
    }
    telemetry? {
      type        = <DataClass>
      description = "Telemetry: usage, diagnostic and performance data about a product or a device."
      priority    = 9
    }
  }
}
```

## How a Snapp uses it

The str form is unchanged and stays REQUIRED (see `snapp_compliance`). The closed form is an
additional, OPTIONAL prop, `data_class_set`, appended LAST to `SnappCompliance`:

```nlang,ignore
snapp_compliance "MY_SNAPP" {
  exercises {
    recording_consent { subjects = ["telepresence"] }
  }
  data_classes   = ["communications", "biometric"]
  data_class_set {
    communications { title = "Communications"  description = "Call audio and its metadata." }
    biometric      { title = "Biometric information"  description = "A voiceprint used to tell speakers apart." }
  }
}
```

Each value is a `DataClass`, so the Snapp says in its own words what it handles of that class. The
KEY is what the closure checks. A consumer reads a class from the key, never from the title.

When both forms are present they SHOULD agree. A consumer that finds them disagreeing reads the
UNION: the wider declaration is the one that was made. That rule belongs to the consumer's
determination, not to these types, and is stated beside the prop in `snapp_compliance`.

## What the types enforce, and what they do not

Measured 2026-11-08 on `nlang 0.1.0-dev (12c4c7fea 2026-11-01)` (binary sha256 `a7665e6b4c7e914f…`)
in this Snapp's own tree: `nlang check`, then `nlang test`, of the spec root (86 arms passed, 0 failed) and of `baselines/1.0.0` (25 passed, the 2 RED arms named below, 1 ignored).
Every positive is paired with a negative that flips ONE value, and the deciding negative was also run
once alone as a plain positive test to read the diagnostic that refused it. The arms live in
`smoke_tests/type_constraint_tests.n`, and for the rows marked ACROSS A DEPENDENCY in
`baselines/1.0.0/data_class_consumer_tests.n`, the one Snapp in this repository that depends on r14n.

| Probe | Result | Diagnostic, and the arm |
|---|---|---|
| a `DataClass` with a title and a description | resolves | `data_class_resolves` |
| a `DataClass` with no `title` | ENFORCED | refused (`data_class_requires_a_title`) |
| a prop `DataClass` does not declare (`regime`) | ENFORCED | refused (`data_class_rejects_a_prop_it_does_not_declare`) |
| all nine classes as the keys of one `DataClassSet` | resolves | `data_class_set_resolves_for_all_nine_classes` |
| a `data_class_set` beside `data_classes` in a `SnappCompliance` | resolves | `snapp_compliance_resolves_with_a_data_class_set` |
| a class key the set does not declare (`communicatoins`), nested in a `SnappCompliance` | ENFORCED | COMP-438, "Unexpected prop `%communicatoins` not defined in type `data_class_set`", which lists the nine and suggests `%communications` (`snapp_compliance_rejects_a_data_class_set_key_it_does_not_declare`) |
| an empty `data_class_set` | NOT REFUSED, by design | accepted (`snapp_compliance_accepts_an_empty_data_class_set`) |
| a `SnappCompliance` that omits `data_class_set` | NOT REFUSED, by design | accepted: every earlier positive `SnappCompliance` arm omits it and stays green |
| ACROSS A DEPENDENCY (`baselines/1.0.0`, `use { $::* }`): a `SnappCompliance` with no `data_classes` | ENFORCED | refused, which proves the type resolved there (`consumer_snapp_compliance_still_requires_data_classes`) |
| across a dependency: a `data_class_set` beside `data_classes` | resolves | `consumer_resolves_a_data_class_set` |
| across a dependency: a class key the set does not declare (`communicatoins`) | NOT ENFORCED | compiles. The arm is a `should_fail` arm and is RED until the compiler closes it (`consumer_rejects_a_data_class_set_key_the_vocabulary_does_not_declare`) |
| across a dependency: a prop `SnappCompliance` does not declare (`regime`) | NOT ENFORCED | compiles; RED for the same reason (`consumer_rejects_a_prop_snapp_compliance_does_not_declare`) |
| across a dependency: the same types declared at the root (`pub blocks`) instead of in `*` | NOT RESOLVED | a missing REQUIRED prop compiled, while a type declared in `*` (`FrameworkLink`) refused it. Measured with two throwaway arms that were not kept |

Two rules are not in the table because no type can express them, so no arm can measure them: the two
forms SHOULD agree (a cross-field rule), and a consumer reads a class from its KEY, never from the
`title` a Snapp wrote under it (a title is free text). Both belong to the consumer.

**Why the first design was dropped (measured the same day).** The first design declared each class as
a keyed instance, `pub DataClass "phi" { ... }`, in a module `data_classes`, for a dependent Snapp to
reach by path. `nlang check` refused the document at parse: SYN-9052, "Expected
mod/blocks/props/fn/trait/macro/use after pub at document level", at the first member. The N Language
Book shows `pub` before a value node and never before a keyed instance, and an unmarked node is
reachable only inside its own ledger scope, so no member could ever have been reached from a dependent
Snapp. The closed set reaches the same goal, a misspelled class that does not compile, through a
refusal this repository had already measured for `FrameworkLinks`.

## References

- [R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0.n.md](R14N_SPEC_KEYWORD_SNAPP_COMPLIANCE_1_0_0.n.md) — `SnappCompliance`, whose `data_classes` list takes these tokens and whose LAST optional prop `data_class_set` takes a `DataClassSet`
- [R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md](R14N_SPEC_KEYWORD_POLICIES_1_0_0.n.md) — `Subject`, whose `key` grammar a class key shares
- [R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md](R14N_SPEC_KEYWORD_FRAMEWORKS_1_0_0.n.md) — why this vocabulary carries no criterion identifier, and the closed-set idiom `DataClassSet` reuses
- r14n `spec/RLPS-v0.1.md` §2 — the data model: domains and subjects; §3 — posture, including the fail-closed absent subject; §5 — fail-closed
- `docs/not-legal-advice.md` — what a declaration does and does not claim
