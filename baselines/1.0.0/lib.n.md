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
    integrity: "sha256-d51ec114946115f37fd238b719251b6986a5bdb07ae0da519fe943d90e75d631"
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
