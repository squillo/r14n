---
description: Resolve required controls for a (profile × jurisdiction × subject) query against local packs and explain the decision
argument-hint: <profile> <subject> [--domain <d>] [--as-of YYYY-MM-DD]
---

Resolve an RLPS query and explain the result: $ARGUMENTS

**NOT LEGAL ADVICE.** The output is a control-prescription from a configuration file, not a
statement of what any law requires.

1. **Assemble the query.** You need a profile (pack directory, e.g. `aggressive`), a subject
   (e.g. `telepresence`), a domain (default `recording_consent`), a jurisdiction key (default
   `all_party`), and a **universe** — every control the calling application can enforce. If
   the user did not give a universe, use the catalog's full key set from
   `${CLAUDE_PLUGIN_ROOT}/catalog/recording_consent.catalog.toml` and say that you did, since
   the universe is the aggressive/fallback ceiling and changes the answer.

2. **Resolve it.** Write a small example against the reference resolver, or use the
   in-memory path via the generated bindings. From `${CLAUDE_PLUGIN_ROOT}`, the simplest
   route is a throwaway Rust example against `resolver/`, mirroring
   `${CLAUDE_PLUGIN_ROOT}/resolver/examples/ai_act_50.rs`:

   ```
   cargo run -q --manifest-path resolver/Cargo.toml --example ai_act_50
   ```

   Read that example first — it shows the full query construction, resolution, and receipt
   serialization end to end.

3. **Explain the decision, in this order:**
   - **`verdict`** — `permit` or `block`.
   - **`required`** — the controls that must be satisfied. Note that this is always a subset
     of the universe you supplied.
   - **`provenance.source`** — which pack governed. If it starts with `<missing:`,
     `<malformed:`, `<outside-effective-envelope:`, or `<minimal-missing-floor:`, the
     resolver **fell back**: no reviewed pack governed this decision, and the required set is
     the entire universe. Lead with that fact — it is the most important thing on the screen.
   - **`provenance.fell_back`** — say it plainly when true.
   - **`provenance.legal_review_status`** and `last_reviewed_against_guidance` — a pack can
     be in-envelope and still stale.

4. **Never present the result as a compliance conclusion.** "This pack requires X" is
   accurate; "you are compliant if you do X" is not, and "aggressive means compliant" is
   explicitly false (spec §3). If the user asks whether something is *legal*, decline that
   question and point them to the counsel-gating model in `GOVERNANCE.md`.
