---
section: "r14n Baselines"
name: "r14n Baseline Policy Packs"
---
# r14n Baseline Policy Packs

Squillo's own regulatory-policy baselines (`aggressive` / `minimal`), authored as a
consumer Snapp that `@deps` the r14n framework (its `policies` keyword + `PolicyPack`
/ `Subject` type blocks fold in through the dep). Emitted via `nlang export -f json`
to an ABI-consumable dist that the Squillo OS regulatory-policy engine reads through the N Lang ABI.

**NOT LEGAL ADVICE** — Squillo's own `aggressive`/`minimal` posture only; jurisdiction
packs are counsel-gated and held out.

```nlang
@deps {
  r14n {
    use { $::* }
    from: "../../snapp/r14n@0.1.0-alpha.1.json"
    integrity: "sha256-38b50faca609b73945041b2b060ae8372e1aaba74250f19c1a8fa45a87159358"
  }
}

policies "HEALTH_AGGRESSIVE" {
  domain     = "health"
  strictness = "aggressive"
  summary    = "Fail-closed baseline — the full HealthKit control universe."
  subjects {
    read  { controls = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit"] }
    write { controls = ["consent_prompt", "usage_description", "on_device_only", "audit_log"] }
  }
  legally_required = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit", "audit_log"]
}
```
