---
section: "r14n Baselines"
name: "r14n Baseline Policy Packs"
---
# r14n Baseline Policy Packs

Squillo's own regulatory-policy baselines (`aggressive` / `minimal`), authored as a
consumer Snapp that `@deps` the r14n framework (its `policies` keyword + `PolicyPack`
/ `Subject` type blocks fold in through the dep). Emitted via `nlang export -f json`
to an ABI-consumable dist that `the Squillo OS policy engine` reads through `nlang_abi`.

**NOT LEGAL ADVICE** — Squillo's own `aggressive`/`minimal` posture only; jurisdiction
packs are counsel-gated and held out.

```nlang
@deps {
  r14n {
    use: ["$::*"]
    from: "../../snapp/r14n@0.1.0-alpha.1.json"
    integrity: "sha256-3f8569052d784f8855f1e8e89b93d3caeca8f0674887700fc757a105ae11105f"
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
