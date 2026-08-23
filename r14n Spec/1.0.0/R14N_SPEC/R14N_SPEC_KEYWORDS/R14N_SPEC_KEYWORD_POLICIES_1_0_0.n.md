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
1. **Domain + Profile**: `domain` names a capability family (`health`, `smart_home`, `photo_library`, `location`, `nfc`, `recording_consent`, …). `strictness` is the dial (`aggressive` / `as_configured` / `minimal`).
2. **Subject Table**: `subjects` is a LIST of `Subject` rows, each a `{ key, controls }` pair. `key` is the facet (`read`, `write`, `control`, …); `controls` is the required opaque control keys for that facet.
3. **Legally-Required Floor**: `legally_required` is the control list enforced under `minimal` strictness / when no subject row matches.
4. **Defaults**: `strictness` and `legal_review_status` are inherited from `R14N.n` when omitted.
5. **Fold Position**: base declaration vs tighten-only overlay is decided by ledger position, not syntax.

**Strictness semantics** (mirrors the resolver twin in Squillo OS):
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
    id {
      atom     = str
      format   = "^[a-z][a-z0-9_]{0,63}$"
      priority = 1
    }
    domain {
      atom        = str
      format      = "^[a-z][a-z0-9_]{0,63}$"
      description = "Capability family this pack governs (e.g. health, smart_home, nfc)"
      priority    = 2
    }
    strictness {
      atom     = str
      values   = ["aggressive", "as_configured", "minimal"]
      default  = "aggressive"
      priority = 3
    }
    summary? {
      atom        = str
      description = "Human-readable pack description"
      priority    = 4
    }
    legal_review_status {
      atom     = str
      values   = ["draft", "requires_signoff", "not_required"]
      default  = "not_required"
      priority = 5
    }
    legal_review_counsel? {
      atom        = str
      description = "Reviewing counsel of record (\"TBD\" until assigned); REQUIRED before a real-jurisdiction pack ships"
      priority    = 6
    }
    subjects {
      atom        = list
      items       { type = <Subject> }
      min_items   = 1
      description = "Per-subject required-control table (list of {key, controls} rows)"
      priority    = 7
    }
    legally_required {
      atom        = list
      items       { atom = str }
      description = "Controls required under minimal strictness / when no subject row matches"
      priority    = 8
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
```

### Fixed Fields — `PolicyPack`

| Field | Type | Default (from R14N.n) | Description |
|---|---|---|---|
| `id` | `string` | — | REQUIRED: pack id; lowercase `^[a-z][a-z0-9_]{0,63}$` |
| `domain` | `string` | — | REQUIRED: capability family |
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
```nlang,ignore
@deps {
  r14n { use: ["$::*"], from: "registry:@com::squillo/r14n@1.0.0", integrity: "sha256-<hash>" }
}
mod policies {
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
