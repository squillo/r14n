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
@deps "r14n@0.1.0-alpha.1" {
  use { $::* }
  from      = "../../snapp/r14n@0.1.0-alpha.1.json"
  integrity = "sha256-6d11342a0c9e6cc16978a598ca0b9487e7c076fd318bf6e61165ad8c8a49e14d"
}

policies "HEALTH_AGGRESSIVE" {
  meta {
    strictness = "aggressive"
    description    = "Fail-closed baseline — the full HealthKit control universe."
    legal_review { status = "not_required" }
  }
  subject {
    read       { controls = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit"] }
    write      { controls = ["consent_prompt", "usage_description", "on_device_only", "audit_log"] }
    // `background` was missing here while the smoke-test copy of this same pack carried it —
    // two sources of truth for one baseline, already disagreeing. Under `aggressive` the
    // omission changed nothing (that posture requires the caller's whole universe), which is
    // why it survived; under `as_configured` a background-delivery query would have found no
    // row and fallen through to the unknown-subject path. Restored to match, and the parity
    // test below the Snapps now fails if the two copies diverge again.
    background { controls = ["consent_prompt", "usage_description", "on_device_only", "audit_log"] }
  }
  legally_required { controls = ["consent_prompt", "usage_description", "on_device_only", "read_scope_limit", "audit_log"] }
}

// ── recording_consent ────────────────────────────────────────────────────────
//
// The two packs below mirror the PUBLISHED baselines at
// /packs/{aggressive,minimal}/recording_consent.r14n.toml, control for control. They
// are here as the convergence proof: recording_consent is the only domain this
// repository ships as normative TOML, so it is the only one where the typed source and
// the published artifact can be diffed rather than merely asserted to agree.
//
// Keep them in step. If one changes without the other, the two disagree about what
// Squillo's own baseline is, which is the drift this whole exercise exists to end.
//
// NOT LEGAL ADVICE, and `aggressive` != `compliant`.

policies "RECORDING_CONSENT_AGGRESSIVE" {
  meta {
    strictness = "aggressive"
    description    = "Fail-closed baseline — the full per-rail control universe, every jurisdiction."
    legal_review { status = "not_required" }
  }
  subject {
    local_piggyback { controls = ["attestation", "aph_mandate_principal_signed"] }
    telepresence    { controls = ["signal_notice", "indicator_mount", "aph_mandate_principal_signed"] }
    twin_attend     { controls = ["announcement", "aph_mandate_principal_signed"] }
    import          { controls = ["attestation", "aph_mandate_principal_signed"] }
  }
  legally_required { controls = ["attestation", "signal_notice", "indicator_mount", "announcement", "aph_mandate_principal_signed"] }
}

// The floor is ONE control and must never become none: a `minimal` pack with a missing
// or empty floor does not resolve to "nothing required", it fails closed to the
// caller's entire universe and discards this pack whole.
policies "RECORDING_CONSENT_MINIMAL" {
  meta {
    strictness = "minimal"
    description    = "Least-restrictive floor — only legally-required controls are enforced."
    legal_review { status = "requires_signoff" counsel = "TBD" }
  }
  subject {
    local_piggyback { controls = ["attestation"] }
    telepresence    { controls = ["signal_notice"] }
    twin_attend     { controls = ["announcement"] }
    import          { controls = ["attestation"] }
  }
  legally_required { controls = ["attestation"] }
}

// Node-validation suite. Declared LAST so load order of everything above is unchanged —
// declaration order IS load order, and an undeclared sibling file never loads at all, so
// without this line `policy_pack_tests.n` would sit in the tree compiled by nothing.
#[cfg(test)]
mod policy_pack_tests {};
```
