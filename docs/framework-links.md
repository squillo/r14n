# Framework links — relating a control to a compliance-framework edition

> **⚠ NOT LEGAL ADVICE.** RLPS is a data-interchange and configuration standard. Nothing on this
> page states what any law, standard or questionnaire requires, and no framework link, relation or
> review status is a finding that any control, application or organization is sufficient for any
> framework or certification. **"aggressive" ≠ "compliant," and "linked" ≠ "compliant" either.**
> See [`not-legal-advice.md`](not-legal-advice.md).

*Status: informative.* This page explains the optional `frameworks` facet of a catalog control
(`catalog/*.catalog.toml`; shape in [`schema/catalog.schema.json`](../schema/catalog.schema.json);
resolution in [`spec/RLPS-v0.1.md`](../spec/RLPS-v0.1.md)). Where this page disagrees with the
schema or the spec, this page is the defect. **This build delivers the contract only: no catalog
shipped in this repository carries a `frameworks` block.**

## 1. What a framework link is, and is not

Like everything in this project, a framework link is not legal advice and not a compliance
finding. It is **navigation and audit metadata**: it records that a control the catalog already
names is related to an edition of a compliance framework, at a stated strength, with an optional
record of who reviewed that statement. It has the same standing as `legal_basis_hint` and the
`authorities` array of spec §6 — a pointer for a human reader. The contract is that a resolver MUST
NOT vary a decision on it, and that listing a framework is not a representation of what that
framework requires.

| Part of a link | Shape | Present |
|---|---|---|
| `frameworks.<edition>` | one optional direct property per edition (closed set, section 3) | optional |
| `relation` | `informs` \| `contributes` \| `necessary` (section 2) | always, on a link |
| `criteria` | list of non-empty strings | optional; empty of real identifiers today (section 6) |
| `review` | `status` (`draft` \| `requires_signoff` \| `approved`) plus a `counsel` string, which is required when `status` is `approved` | required by two rules (section 2) |

```text
[control.<key>.frameworks.<edition>]
relation = "informs"
```

A link is **not**:

| A link is not… | Because |
|---|---|
| a finding that any framework is addressed | No single control is sufficient for a criterion (section 2). |
| a statement of what a law requires | Catalogs are jurisdiction-neutral; a control is a stable name, not legal text (spec §2.1). |
| evidence that the control operates | RLPS answers *which* controls are required, never whether one is satisfied (see the header of [`credential_use`](../catalog/credential_use.catalog.toml)). |
| an input to resolution | The reference resolver never reads a catalog. A consumer joins links to a decision *after* resolving (section 7). |
| a pack requirement | Packs reference control keys. A link adds no control to any set and removes none. |
| an accepted mapping | Whether an auditor, institution or regulator accepts a mapping is theirs to say. |
| a claim of affiliation | Framework names identify editions only. No review, endorsement or affiliation by their owners exists or is implied. (This repository's own terms, [`TRADEMARKS.md`](../TRADEMARKS.md), cover the RLPS names and conformance wording, not theirs.) |

## 2. The relation vocabulary

The set is closed. Claim strength rises down the table; that is an ordering of what the link
*asserts*, not of how much of any criterion is addressed, and (like posture, spec §3) not a legal
ordering.

| Relation | What it says |
|---|---|
| `informs` | The control's subject matter is relevant background to the cited criteria or edition: a person reasoning about them would want to know this control exists, and nothing further is claimed. |
| `contributes` | Operating the control is one of several practices that can contribute to addressing the cited criteria; the link does not say how much, and it expects other controls and practices to be part of the answer. |
| `necessary` | A reviewer has recorded that the cited criteria cannot be addressed without this control: the link states a precondition (no control, no addressing), never an outcome (control, therefore addressed). |

**Why there is no `satisfies`.** No single control is sufficient for a criterion. A criterion is
addressed by a system of controls, operated over time, inside a defined scope, and judged by
someone other than the catalog: a service auditor for an attestation, a certification body for a
management-system standard, the vendor's own respondent and then the reviewing institution for a
questionnaire, a regulator or court for a legal text. A relation named for sufficiency would let a
catalog publish the one conclusion that only that judgement can reach. It would also claim
something this layer cannot know: a key is indistinguishable here whether its satisfier is wired or
not. The relations do not accumulate, either: any number of `contributes` links do not add up to
`necessary`, and no number of links of any kind adds up to coverage. The closed set has no spelling
for the missing word on purpose; adding one would change the contract, not a catalog.

**`necessary` needs an approved review.** `necessary` is the strongest statement a link can make
and the one nearest to a claim about what a framework demands, so the contract refuses it unless
the link carries a `review` whose `status` is `approved`. What `approved` proves is what spec §6
says of a pack's review: that someone recorded an approval, not that the reviewer was qualified or
the review correct ("provenance proves signing, not correctness",
[`not-legal-advice.md`](not-legal-advice.md)). This contract defines no signature over a link's
`review` block, and `counsel` is free text: not verified, not a legal credential (nothing checks that the name is a lawyer, a firm or a person), and not an attorney-of-record claim (that
contract belongs to a pack's `legal_review` block). Read `approved` as an assertion by whoever
published the catalog.

**`approved` must name its reviewer.** An `approved` review with no `counsel` is refused: an approval
with nobody standing behind it is the same hazard a pack's `legal_review` block already closes by
requiring an attorney of record before `approved`, and without this rule `necessary` (which needs
`approved`) could be satisfied by writing one word. The rule binds `approved` only, so an
unfinished `draft` or `requires_signoff` review stays expressible without a name.

| `review.status` | Meaning |
|---|---|
| `draft` | The statement is being prepared; nobody has signed off. |
| `requires_signoff` | Prepared and waiting on a named reviewer. |
| `approved` | A reviewer recorded approval of this link as written. |

These are three of the four values a pack's `[meta.legal_review].status` takes, without
`not_required`: on a link the `review` block is either absent or records a review at some stage.

**The rules at a glance**

| If the link… | then it must also… |
|---|---|
| has relation `necessary` | carry `review` with `status = "approved"` |
| is to `hipaa_45cfr164_2013`, with any relation | carry a `review` block (any status) |
| is both | carry `review.status = "approved"` (the stricter rule governs) |
| names `criteria` | name no real framework identifier until section 6 clears |
| is anything else | nothing further |

## 3. The four editions

The set is closed; each edition is one optional direct property of `frameworks`. An edition id pins
a *specific edition* of the framework, so a link made against one text is never silently read
against another.

| Edition id | The edition | Kind | Who judges |
|---|---|---|---|
| `soc2_tsc_2017` | AICPA Trust Services Criteria, 2017 edition (the criteria evaluated in a SOC 2 examination) | Attestation framework | An independent practitioner (a CPA firm) examining a service organization, who issues a report |
| `iso27001_2022_a1_2024` | ISO/IEC 27001:2022 as amended by Amendment 1 (2024) | Certifiable management-system standard | An accredited certification body auditing an organization's management system within its stated scope |
| `hipaa_45cfr164_2013` | Title 45 CFR Part 164, as amended in 2013 | Legal text (a federal regulation) | Regulators and courts |
| `hecvat_4_1_6` | EDUCAUSE Higher Education Community Vendor Assessment Toolkit, version 4.1.6 | Questionnaire | The vendor answers; each institution reads the answers for itself |

What a link to each can honestly say, and what it cannot:

- **`soc2_tsc_2017`.** The control is related to criteria an auditor evaluates. A SOC 2 Type 2
  report speaks to how an organization's controls *operated over a period*, as examined by an
  auditor; a link speaks to neither operation nor period, and a catalog entry is not evidence that
  the control ran.
- **`iso27001_2022_a1_2024`.** The control relates to the standard's requirements or its Annex A
  reference controls. Which controls apply to an organization, and within what scope, is that
  organization's own risk-based decision recorded in its own documents; a link says nothing about
  applicability or scope.
- **`hipaa_45cfr164_2013`.** Linking to a legal text is closer to a legal claim than linking to a
  framework: "this control relates to this provision" reads as an interpretation of what the
  provision demands. So *any* link to this edition — `informs` included, `draft` included — must
  carry a `review` block, so the claim arrives with its provenance attached. The id pins the text as
  amended in 2013; later amendments are not covered and need their own id (section 5). Whether the
  regulation applies to any organization is a legal question outside this page and this project.
- **`hecvat_4_1_6`.** HECVAT is a **questionnaire, not a certification**: nothing is passed, and no
  one but the vendor attests to the answers. A link says the control is *relevant to answering
  questions* in that edition, not that any answer follows. The answer depends on the facts of a
  deployment, which a catalog does not hold, and on a respondent's judgement.

## 4. Wire spelling

The edition id **is** the spelling: the N Lang property name, the JSON Schema key and the TOML key
are the same snake_case string (`soc2_tsc_2017`). There is no alias table in code. A second
spelling for one edition would be two rows no query unites — the same reasoning the catalog schema
gives for control keys, which are matched verbatim and are long-lived audit evidence.

A tool that meets a hyphenated or display spelling in a catalog is meeting an unrecognised key
(section 5); it does not translate it. The table below is **non-normative**, for people writing
documentation and talking to colleagues:

| Edition id (the only wire spelling) | Informal aliases (documentation only; never parsed) |
|---|---|
| `soc2_tsc_2017` | `soc2-tsc-2017`, "SOC 2 (2017 TSC)" |
| `iso27001_2022_a1_2024` | `iso27001-2022-a1-2024`, "ISO/IEC 27001:2022 + Amd 1:2024" |
| `hipaa_45cfr164_2013` | `hipaa-45cfr164-2013`, "45 CFR 164 (2013)" |
| `hecvat_4_1_6` | `hecvat-4-1-6`, "HECVAT 4.1.6" |

## 5. How the set grows

- **Last-position additions only.** A new edition is a new optional property appended after the
  existing ones. Existing ids are never reordered, renamed or repurposed, because they are cited
  from outside the catalog — by compiled bundles, by readers of old documents, by auditors — and
  changing one silently changes what a past citation means.
- **A new version of a framework is a new id.** A link to `hecvat_4_1_6` was made against that
  edition. A later edition gets its own id; the old id and its links remain. Retiring an id would
  be a deprecation by RFC, never a deletion (the catalog is additive by default,
  [`GOVERNANCE.md`](../GOVERNANCE.md) §Change process).
- **Closed sets stay closed.** New relation or review-status values are a change to the contract
  and need an RFC; `satisfies` is not a candidate (section 2).

**How a reader treats what it does not know.** A tool built before an edition was added can meet a
catalog that names it:

| The reader meets… | It does |
|---|---|
| a known edition | Reads the link by the rules above. |
| an edition id it does not know | **Preserves** it byte-for-byte on any round trip; **takes no action** on it; never reads it as satisfied, covered or informed; never maps it to the nearest known edition. A tool may surface that an unrecognised block exists. |
| a relation or review status it does not know | The same, by extension: preserved, no action, never read as `necessary` or `approved`. |
| no `frameworks` block | Nothing. Absence is "no link recorded" — not "not relevant", and not "unmapped, therefore safe". |

A *validator* built from an older schema may refuse a newer edition: that is the closed set doing
its job, and the remedy is to update the validator, not to strip the block.

## 6. The licensing gate

`criteria` holds identifiers into a framework's own numbering — requirement numbers, control
numbers, question numbers, provision citations. Those identifiers belong to their publishers, and
the terms on which this repository may republish them have **not been cleared**. Whether and on
what terms an identifier may be republished is a licensing question for the maintainers and
counsel; this page does not answer it. So:

- **No real criterion or question identifier is published** — not in a catalog, not in this page,
  not in a fixture. Placeholders are obviously fictional (`EXAMPLE-1`).
- **The field exists so the contract can expand** without a schema change when licensing for a
  framework's identifiers clears. It is deliberately not absent.
- **The repository enforces the gate with a guard test**, which fails if any `catalog/*.catalog.toml`
  control carries a `frameworks` block at all (stricter than `criteria` alone: an edition-level link
  is still a public claim about someone else's certification scope). Its scope is those catalogs and
  nothing else: it does not read packs, the Snapp bundles under `snapp/`, or this page (the worked
  example below is checked separately for fictional-only values). The switch is the constant
  `FRAMEWORK_LINK_LICENSING_CLEARED` in `tools/tests/framework_links.rs`. It is a constant in a test
  file, so the guard holds only as long as review of a change to it does: a maintainer flips it
  deliberately — in a reviewed change with a `CHANGELOG.md` entry — once licensing for the
  identifiers concerned is cleared.
- **Until then a link is edition-level:** "this control is related, somewhere, to this edition."
  Scoping a link to a part of an edition is exactly what `criteria` is for, and stays unavailable
  until the gate opens. In a shipped catalog, omit the field: an empty list is refused because it
  reads as answered, which is worse than absent (and a `frameworks` table holding no links is
  refused for the same reason).

## 7. How an application declares what it exercises, and how an installer can resolve it

This section describes how the pieces compose. It is not a requirement on any tool, and this page
does not claim that any tool in this repository performs the steps below.

### 7.1 The declaration

A consuming application states what it exercises. A declaration has two kinds of entry:

- **(domain, subject) pairs** — the dimensions a resolver query already takes (spec §2, items 2
  and 3).
- **Data classes**, each written as a **subject** within a domain — **never a profile**. A profile
  (spec §2, item 4) is a `<regime>/<jurisdiction>/<subjurisdiction>` tag that selects which pack
  governs. Spelled as a profile, a data class would choose a law-shaped pack instead of asking
  which controls apply to that class of data.

A declaration contains no jurisdiction or attribution source (inputs to the resolve, spec §4.1),
no posture (§3) and no framework. Frameworks are read *out of* the result, never put *into* the
query. This page fixes what a declaration says, not how it is serialized. A declaration is the
author's claim; nothing here verifies that the application exercises only what it declares.

### 7.2 Resolving a declaration

1. **Resolve each declared pair on its own,** through the resolver, with the caller-supplied
   universe, jurisdictions with their attribution source (§4.1), and posture.
2. **Resolution is per domain.** The merge operates within one domain only (§4.2, item 4), and a
   resolver never merges control sets across domains (§3).
3. **Merge across declarations most-restrictively.** Within one domain, take the union of the
   required sets; the merged verdict is `block` if any resolution blocks; a required/prohibited
   conflict blocks and is never silently dropped (§4.2). The spec states this merge for
   jurisdictions; applying it across several declarations of one domain is this page's guidance,
   not spec text.
4. **Join to the catalog.** For each resolved control, keyed by *(domain, control key)*, read its
   `frameworks` block. The **informed frameworks** are the union of those links. "Informed" is
   used loosely here: it means linked at *any* relation, not only `informs`.
5. **Carry the provenance through.** Keep domain, control, edition, relation, criteria and review
   beside every link; do not collapse links to a set of edition names, which would lose the
   relation and the review. Carry each resolution's verdict and `fell_back` into the report.

Limits of the result, stated so they are not rediscovered:

- It lists frameworks the *resolved controls* are related to. It is not a list of frameworks the
  application is subject to, ready for, or covered by.
- Links on controls that were not resolved are absent, and a link to an edition a reader does not
  know is preserved but not listed (section 5).
- A prohibition's links never appear. A prohibition key is not in the caller's universe, so no
  resolved decision names it (see the `data_protection` catalog header). Read such links from the
  catalog directly.
- A resolution that blocked or fell back produces an over-approximation — the caller's whole
  universe — and the report should say so.

### 7.3 Two open decisions

These are **undecided**. This page does not choose between the options.

**A. What an undeclared application means at install.**

| Option | Behaviour | Consequence |
|---|---|---|
| Most-restrictive | An application that declares nothing is treated as exercising everything the installer's catalogs can describe. | The gap is loud and no application escapes the report. Every application that predates declarations is reported with maximal links, and may be blocked. |
| Warn-only | The install proceeds; the installer warns that nothing was declared and that no informed-frameworks report could be produced. | Nothing is blocked. The application is absent from any rollup, and a warning can go unread. |

Spec §5 governs what a *resolver* returns on absence, ambiguity or error; it does not settle what
an *installer* does about an application that never declared anything.

**B. Where the install-time resolve runs.**
Candidates: the installer on the target machine, a service the installer calls, a build or
publishing pipeline before distribution, or the application's own first run. What bears on the
choice:

- **Trust.** Whoever runs the resolve controls its inputs; the report is only as trustworthy as
  that party.
- **Inputs.** Jurisdiction attribution (§4.1) may be unknown at install time. It must then be
  reported as unknown to fail closed; leaving it out opts out of the check (section 8, variant 1).
- **Integrity of what is consumed.** Packs are content-addressed (§2) and signatures bind a
  pack's identity (§6); a catalog read for links needs the same care.
- **Capability is not the constraint.** The resolver ships as a Rust crate, a wasm package and a
  Python package, and its wire form is byte-stable across hosts and runs, so the same resolve can
  sit on either side of a frontend/backend boundary. The choice is about trust and inputs, not
  feasibility.

## 8. A worked example

Every value is fictional: the domain is `example_domain`, the criterion is `EXAMPLE-1` (not an
identifier of any framework), the counsel is invented, and the resolved sets are illustrative
rather than output from a real pack. The catalog is written to be a complete, valid catalog under
a schema that carries the facet. A shipped catalog would omit `criteria` today (section 6); it is
shown here so the shape is visible.

**The catalog** (`example_domain`; a documentation snippet, not a shipped file):

```toml
[domain]
name = "example_domain"
description = "Fictional domain used only to illustrate framework links."

[control.example_session_logged]
kind = "obligation"
title = "Session activity logged"

[control.example_session_logged.frameworks.soc2_tsc_2017]
relation = "contributes"
criteria = ["EXAMPLE-1"]

[control.example_session_logged.frameworks.hecvat_4_1_6]
relation = "informs"

[control.example_record_sealed]
kind = "obligation"
title = "Record sealed at rest"

[control.example_record_sealed.frameworks.hipaa_45cfr164_2013]
relation = "contributes"

[control.example_record_sealed.frameworks.hipaa_45cfr164_2013.review]
status = "requires_signoff"

[control.example_key_rotated]
kind = "obligation"
title = "Keys rotated on a schedule"

[control.example_key_rotated.frameworks.iso27001_2022_a1_2024]
relation = "necessary"

[control.example_key_rotated.frameworks.iso27001_2022_a1_2024.review]
status = "approved"
counsel = "Example Counsel LLP (fictional)"

[control.example_export_gated]
kind = "obligation"
title = "Export gated"

[control.example_export_gated.frameworks.soc2_tsc_2017]
relation = "contributes"
```

The HIPAA link is `contributes`, which would need no review on its own; it carries one because
the edition is a legal text, and `requires_signoff` is enough. The ISO link is `necessary`, so its
review must be `approved`.

**The declaration** (illustrative shape only):

```toml
[[exercises]]
domain = "example_domain"
subject = "example_subject_a"

[[exercises]]
domain = "example_domain"
subject = "example_data_class"   # a data class: a SUBJECT, never a profile
```

**Resolution** (section 7.2, steps 1 to 3), against a fictional pack and profile:

| Resolve | `required` | Verdict | `fell_back` |
|---|---|---|---|
| (`example_domain`, `example_subject_a`) | `example_record_sealed`, `example_session_logged` | permit | false |
| (`example_domain`, `example_data_class`) | `example_key_rotated`, `example_record_sealed` | permit | false |
| Merged, one domain, most-restrictively | `example_key_rotated`, `example_record_sealed`, `example_session_logged` | permit | false |

**The informed frameworks** (steps 4 and 5): the join over the merged set. `example_export_gated`
was not resolved, so its SOC 2 link is absent.

| Control | Edition | Relation | Criteria | Review |
|---|---|---|---|---|
| `example_key_rotated` | `iso27001_2022_a1_2024` | `necessary` | none | `approved` |
| `example_record_sealed` | `hipaa_45cfr164_2013` | `contributes` | none | `requires_signoff` |
| `example_session_logged` | `hecvat_4_1_6` | `informs` | none | none |
| `example_session_logged` | `soc2_tsc_2017` | `contributes` | `EXAMPLE-1` | none |

**How to read it.** The result says the controls resolved for this declaration are linked to four
editions at these strengths. It does **not** say that `EXAMPLE-1` or any criterion is addressed;
that any questionnaire answer follows; that the HIPAA link has been reviewed (`requires_signoff`
means it has not); or that the ISO row means more than "a reviewer recorded this control as a
precondition". An edition missing from the table is not thereby irrelevant.

**Variant 1: attribution is unknown.** If the deployment cannot supply jurisdictions (spec §4.1),
it must SAY so, by marking the attribution source `unknown`. Attribution is then ambiguous and the
resolution fails closed: it blocks or, at an infallible interface, returns the caller's whole
universe with `fell_back = true`. Merely omitting the attribution source does not do this: the
reference resolver treats an absent source as an opt-out of the §4.1 check and resolves normally,
so a gate that must fail closed here has to supply it. With all four controls in
the universe, the table gains the `example_export_gated` SOC 2 link. That is an over-approximation
by construction, flagged `fell_back = true`, and the error runs in the safe direction: too many
links reported rather than too few (subject to the limits listed in section 7.2).

**Variant 2: what the contract refuses.**

| Link | Refused because |
|---|---|
| `relation = "necessary"` with no `review` | `necessary` requires an approved review. |
| `relation = "necessary"` with `review.status = "requires_signoff"` | The status must be `approved`. |
| a link to `hipaa_45cfr164_2013` with no `review`, even `informs` | A legal-text edition requires a review block on any link. |
| `relation = "satisfies"` | Not in the closed set, by design (section 2). |
| a key `soc2-tsc-2017` | A validator refuses it; a reader treats it as an unrecognised edition (section 5). It is never translated. |

## See also

[`not-legal-advice.md`](not-legal-advice.md) · [`namespace.md`](namespace.md) (the `rlps:` receipt
vocabulary; control keys do not live there) · [`../spec/RLPS-v0.1.md`](../spec/RLPS-v0.1.md) ·
[`../GOVERNANCE.md`](../GOVERNANCE.md) (the RFC process a change to this contract follows) ·
[`../TRADEMARKS.md`](../TRADEMARKS.md)
