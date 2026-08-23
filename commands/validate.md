---
description: Lint an RLPS policy pack with the reference CLI and report a verdict with spec-grounded diagnosis
argument-hint: <path/to/pack.r14n.toml> [--catalog <path>]
---

Lint the RLPS policy pack given in: $ARGUMENTS

**NOT LEGAL ADVICE.** This checks a pack's *structure and internal consistency*. It says
nothing about whether the pack's controls are legally correct or sufficient anywhere.

1. **Resolve the input.** If the argument is a path, make it absolute. If it is empty, ask
   which pack to lint — the shipped baselines live at
   `${CLAUDE_PLUGIN_ROOT}/packs/{aggressive,minimal}/recording_consent.r14n.toml` and the
   fictional grammar demo at `${CLAUDE_PLUGIN_ROOT}/packs/example/region/`.
   If no `--catalog` was given, default to
   `${CLAUDE_PLUGIN_ROOT}/catalog/recording_consent.catalog.toml`.

2. **Run the reference linter.** From `${CLAUDE_PLUGIN_ROOT}`:

   ```
   cargo run -q --manifest-path tools/Cargo.toml -- validate <absolute-pack-path> --catalog <absolute-catalog-path>
   ```

   Scope: `validate` enforces the **cross-file rules a JSON Schema cannot express** —
   catalog membership of every referenced control key, attorney-of-record fields on packs
   whose `legal_review.status` is `approved`, `[prohibited]` ∩ `[legally_required]` being
   empty, ISO date shape, and the presence of a `[legally_required]` floor. It is not a
   legal review and not a schema check.

3. **Report the verdict plainly.** Exit 0 with zero warnings is a clean pack. Otherwise list
   each finding and, for each, cite the rule it violates from
   `${CLAUDE_PLUGIN_ROOT}/spec/RLPS-v0.1.md` (quote the section number). Do not soften a
   finding and do not invent a fix that changes what the pack *requires* — that is an
   authoring decision with legal consequences, so propose it, never apply it silently.

4. **Two failure modes worth calling out explicitly if you see them:**
   - A `minimal` pack with a missing or empty `[legally_required]` floor. The linter rejects
     it at author time; the resolver *also* degrades it to aggressive-over-universe at
     runtime (spec §4/§5). Say both, so the author knows it fails closed either way.
   - A control key not in the catalog. Packs may only reference declared keys; a new key
     belongs in the catalog first (an additive, reviewable change — GOVERNANCE.md
     §Change process), never inline in a pack.

5. **If the pack claims a real jurisdiction** (a `<regime>/<jurisdiction>` profile naming an
   actual place, or a `jurisdiction` field), stop and say so: those packs are **counsel-gated**
   and must not be authored or committed here without a named attorney of record. Report it
   rather than linting it into shape.
