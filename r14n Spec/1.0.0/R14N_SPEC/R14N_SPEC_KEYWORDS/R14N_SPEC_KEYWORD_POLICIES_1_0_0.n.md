---
section: "r14n Specification"
subsection: "Reserved Block Predicate Keywords"
name: "policies"
version: "1_0_0"
---

```nlang
@name: "r14n_policies"
@author: "Scott Wyatt <legal@squillo.com>"
@license: "SEE LICENSE IN LICENSE"
```

# r14n Specification: Reserved Block Predicate Keywords: `policies` [DRAFT 1_0_0]

The keywords "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 RFC2119 RFC8174 when, and only when, they appear in all capitals.

N Language Spec reserved block keyword `policies` is part of the r14n Grammar. Any N Lang Parser/Interpreter that speaks the r14n framework should implement this keyword in accordance with its Draft version.

> **NOT LEGAL ADVICE** — see `R14N.n.md`. A pack is a control-prescription resolved fail-closed, never a statement of law.

---

## Summary

This document covers the reserved predicate keyword `policies`. A `PolicyPack` is a complete regulatory control-prescription in miniature — it declares the strictness dial, a per-`subject` required-control table, and a `legally_required` floor for a single `(profile × domain)`. One keyword block expresses all of it at author time; `R14N.n` fills in everything omitted. The same keyword declares NEW packs and REFINES existing ones — ledger position decides. A base entry is the floor; the same keyword in a Snapp contributor document is a **tighten-only** overlay.

---

## What is a Policy Pack?

A Policy Pack is a typed N Lang document declaring the required compliance controls for one `(profile × domain)`, resolved against a `subject` facet. The consuming resolver (`RegulatoryPolicyPort`, the Squillo OS regulatory-policy engine) folds the pack's ledger into an effective decision and BLOCKs a capability whose declared controls do not satisfy the pack — fail-closed.

**How Policy Packs Work:**
1. **Domain + Profile**: the domain (`health`, `smart_home`, `recording_consent`, …) is NOT a field of the pack. RLPS §2.2 puts it in the PATH — on disk a pack is `<domain>.r14n.toml` under its profile directory, and in an emitted bundle the pack's declaration name is its key. Carrying it in the body too made the emitted node fail the published schema, which forbids unknown top-level keys, for data every reader already had. `strictness` is the dial (`aggressive` / `as_configured` / `minimal`).
2. **Subject Table**: `subject` is the per-facet required-control table. The keyword form emits it as a KEYED MAP (`subject { read { controls = [...] } }`), which is exactly the published pack schema's `subject` object-of-`control_list` — singular and map-shaped on purpose, so the emitted bytes need no projection to validate. `key` is the facet (`read`, `write`, `control`, …); `controls` is the required opaque control keys for that facet.
3. **Legally-Required Floor**: `legally_required` is the control list enforced under `minimal` strictness / when no subject row matches.
4. **Defaults**: `strictness` and `legal_review_status` are inherited from `R14N.n` when omitted.
5. **Fold Position**: base declaration vs tighten-only overlay is decided by ledger position, not syntax.

**Strictness semantics** (shared with the Squillo OS consumer, which reads the same model over a different transport):
- `aggressive` — the caller's FULL declared control universe is required regardless of the subject table (fail-closed install default; the resolver degrades to exactly this when no pack matches).
- `as_configured` — the per-`subject` table is honored exactly.
- `minimal` — only the `legally_required` floor is enforced; the subject table is documentary. USE WITH CARE.

---

## Object Blocks

The keyword and its types are defined as follows. Per the N Language Book, `3. Reference/8. Items/8. Blocks, Props.md`: a `blocks` definition with `props` is a complete type; block references use `type = <T>`; closed string sets use `values`; a list-of-blocks prop uses `atom = list` + `items { type = <T> }`; `min_items` bounds a list. This shape mirrors the Taxa `ClassifierSpec` gold standard (scalars + lists-of-blocks — the compilable wire form; NO map-of-blocks).

```nlang
blocks "policies" {
  atom        = list
  description = "{Policies} Object Block — regulatory policy pack ledger"
  blocks * {
    type = <PolicyPack>
  }
}

pub mod * blocks "PolicyPack" {
  atom        = list
  description = "{PolicyPack} Object Block — one (profile × domain) control-prescription"

  props {
    id? {
      atom        = str
      format      = "^[a-z][a-z0-9_]{0,63}$"
      description = "OPTIONAL pack id. On disk the profile and domain ARE the path, and in an emitted bundle the pack's declaration name is its key — so an id is only useful when a reader needs a stable handle that is neither."
      priority    = 1
    }
    meta {
      type        = <PackMeta>
      description = "Posture + review provenance. NESTED rather than flattened so the typed source mirrors the published pack's `[meta]` / `[meta.legal_review]` tables: a flat `legal_review_status` cannot round-trip to `[meta.legal_review] status` without a projection rule, and a projection rule is a place for the two to disagree."
      priority    = 3
    }
    subject {
      atom        = list
      items       { type = <Subject> }
      min_items   = 1
      description = "Per-subject required-control table. SINGULAR to match the published pack schema, whose `subject` is an object of `control_list` — which is exactly what the keyword form already emits. It was `subjects` until 2026-10-16, and that one letter was the whole reason a consumer had to project the emitted bundle before anything could validate it."
      priority    = 7
    }
    legally_required {
      type        = <ControlList>
      description = "Controls required under minimal strictness / when no subject row matches. WRAPPED in a ControlList rather than a bare list so the emitted bytes match the published pack's `[legally_required] controls = [...]` table and validate against schema/pack.schema.json, whose `legally_required` is a `$ref` to `control_list`. A bare list emitted a bare array, and the gap between the two is what a consumer had to project around."
      priority    = 8
    }
    prohibited? {
      type        = <ControlList>
      description = "Controls this profile FORBIDS — the data-minimization guard input: no posture may ADD a control listed here, and a resolver subtracts it from every resolved set. MUST NOT intersect legally_required (a pack that both requires and forbids a control is not satisfiable). OPTIONAL: an absent list forbids nothing, which is why it is not the same as an empty one in review."
      priority    = 9
    }
  }
}

pub mod * blocks "PackMeta" {
  atom        = list
  description = "{PackMeta} Object Block — the pack's posture and its review provenance"

  props {
    strictness {
      atom     = str
      values   = ["aggressive", "as_configured", "minimal"]
      default  = "aggressive"
      priority = 1
    }
    description? {
      atom        = str
      description = "Human-readable pack description. Named `description` to match the published pack's `[meta] description`: this field was called `summary` here until 2026-10-15, which meant the typed source and the artifact a reviewer reads used different names for the same sentence."
      priority    = 2
    }
    legal_review {
      type        = <LegalReview>
      description = "Who reviewed this pack and how far that review went"
      priority    = 3
    }
    // ── The temporal envelope. OPTIONAL, and absent means UNBOUNDED, not expired. ──
    //
    // These are load-bearing: a resolver that is given a decision date and finds it
    // outside this window fails closed, because no in-effect pack governs. A typed pack
    // could not express the window at all until 2026-10-15, so a pack authored here could
    // not be time-bounded even though the published schema and the resolver both support
    // it — and an author who needed that had to leave the typed path.
    effective_from? {
      atom        = date
      description = "ISO-8601 (YYYY-MM-DD) date this pack's text takes effect. A decision date before this means no in-effect pack governs."
      priority    = 4
    }
    effective_until? {
      atom        = date
      description = "ISO-8601 (YYYY-MM-DD) date this pack's text stops being in effect. A decision date after this means no in-effect pack governs."
      priority    = 5
    }
    last_reviewed_against_guidance? {
      atom        = date
      description = "Interpretation-currency date — when this pack was last read against case law and regulator guidance, which is NOT the same as the statute's effective date. REQUIRED by the published schema once `legal_review.status` is `approved`: an approval nobody can age is indistinguishable from one that never expires."
      priority    = 6
    }
    inherits? {
      atom        = str
      description = "Parent profile this pack delta-inherits from, as a `/`-delimited profile tag. Declaring it does not make a resolver honour it — progressive-truncation negotiation and delta-merge are specified but unimplemented in the reference resolver, so treat this as authored-ahead."
      priority    = 7
    }
  }
}

pub mod * blocks "LegalReview" {
  atom        = list
  description = "{LegalReview} Object Block — counsel-review provenance for one pack"

  props {
    status {
      atom     = str
      // `approved` was ABSENT from this closed set until 2026-10-15, while the published
      // pack schema carried all four. A typed pack therefore could not say the one thing a
      // counsel gate exists to recognise — a consuming gate governs only when the status is
      // `approved` or `not_required`, so a pack authored here could never reach the cleared
      // state at all. The typed path must be able to express everything the published schema
      // allows, or authors quietly drop back to hand-written artifacts to say it.
      values   = ["draft", "requires_signoff", "approved", "not_required"]
      default  = "not_required"
      priority = 1
    }
    counsel? {
      atom        = str
      description = "Reviewing counsel of record (\"TBD\" until assigned); REQUIRED before a real-jurisdiction pack ships"
      priority    = 2
    }
    reviewed? {
      atom        = date
      description = "ISO-8601 (YYYY-MM-DD) date the pack was last reviewed. A quoted YYYY-MM-DD literal is shape-typed `date`, not `str`. An `approved` real-jurisdiction pack must state this: an approval with no date cannot be aged, and a review nobody can age is indistinguishable from a review that never expires."
      priority    = 3
    }
  }
}

pub mod * blocks "Subject" {
  atom        = list
  description = "{Subject} Object Block — one capability facet and its required controls"

  props {
    key {
      atom        = str
      format      = "^[a-z][a-z0-9_]{0,63}$"
      description = "Gate subject key (capability facet: read, write, control, enumerate, …)"
    }
    controls {
      atom        = list
      items       { atom = str }
      min_items   = 1
      description = "Required opaque control keys for this subject (the gate owns polarity)"
    }
  }
}

pub mod * blocks "ControlList" {
  atom        = list
  description = "{ControlList} Object Block — a named list of control keys, as a TABLE"

  props {
    controls {
      atom        = list
      items       { atom = str }
      description = "Opaque control keys. EMPTY IS MEANINGFUL and is not the same as absent: an empty list under `prohibited` forbids nothing explicitly, where an absent `prohibited` was never reviewed. The resolver's reader carries a serde default for exactly this reason."
    }
  }
}
```

### Fixed Fields — `PolicyPack`

| Field | Type | Default (from R14N.n) | Description |
|---|---|---|---|
| `id` | `string` | — | REQUIRED: pack id; lowercase `^[a-z][a-z0-9_]{0,63}$` |
| `strictness` | `string` | `"aggressive"` | Closed set: `aggressive`, `as_configured`, `minimal` |
| `summary` | `string?` | — | OPTIONAL: human-readable description |
| `legal_review_status` | `string` | `"not_required"` | Closed set: `draft`, `requires_signoff`, `not_required` |
| `legal_review_counsel` | `string?` | — | OPTIONAL: reviewing counsel of record |
| `subjects` | `list<Subject>` | — | REQUIRED: min 1 subject row |
| `legally_required` | `list<string>` | — | Floor controls (minimal / no-match) |

### Fixed Fields — `Subject`

| Field | Type | Description |
|---|---|---|
| `key` | `string` | REQUIRED: facet id; lowercase snake_case |
| `controls` | `list<string>` | REQUIRED: min 1 required control key |

---

## Desugar Reference

The keyword form DESUGARS to the core-book form. They are semantically identical; the core-book form is the wire (v1 today — keyword→wire desugaring is parser Wave I.4, NOT yet in the toolchain).

**Keyword Form (the DX target; DRAFT):**
```nlang,ignore
name: "health"
version: "1.0.0"
extends: "r14n@1.0.0"
r14n: "1.0.0"

policies "HEALTH_AGGRESSIVE" {
  domain     = "health"
  strictness = "aggressive"
  summary    = "Fail-closed baseline — the full HealthKit control universe."
  subjects {
    READ  { controls = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit"] }
    WRITE { controls = ["consent_prompt", "usage_description", "on_device_only", "audit_log"] }
  }
  legally_required = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit", "audit_log"]
  // legal_review_status: R14N.n default ("not_required")
}
```

**Core-Book Form (the wire — v1 today):**

> The holding module is `packs`, never `policies`. A consumer that imports `$::*` pulls the
> `policies` block into local scope, so `mod policies` would make a module and a type share one
> canonical address — **COMP-447**, since modules and types occupy the same namespace (the N
> Language Book, Modules, "Name Collisions Across Positions"). This example read `mod policies` until 2026-10-15; anyone who copied it got a
> type-resolution error with no source location.

```nlang,ignore
@deps {
  r14n { use: ["$::*"], from: "registry:@com::squillo/r14n@1.0.0", integrity: "sha256-<hash>" }
}
mod packs {
  health_aggressive: <r14n::PolicyPack> {
    id: "health"
    domain: "health"
    strictness: "aggressive"
    summary: "Fail-closed baseline — the full HealthKit control universe."
    legal_review_status: "not_required"
    subjects: [
      { key: "read",  controls: ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit"] },
      { key: "write", controls: ["consent_prompt", "usage_description", "on_device_only", "audit_log"] }
    ]
    legally_required: ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit", "audit_log"]
  }
}
```

---

## References

- [R14N.n.md](../R14N.n.md) — global defaults document
- [mod.n.md](mod.n.md) — keyword index
- r14n `spec/RLPS-v0.1.md` — the Regulatory Localization Pack Specification
- N Language Book `3. Reference/8. Items/8. Blocks, Props.md` — blocks / props / values / items / min_items
- N Language Book `3. Reference/8. Items/15. External Snapps.md` — `@deps` + integrity + `use: ["$::*"]`
- Consuming OS twin: the Squillo OS regulatory-policy engine (`RegulatoryPolicyPort`) — fail-closed resolver
