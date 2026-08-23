---
description: Run the RLPS conformance vectors against the reference resolver and report level claims honestly
argument-hint: [level-1 | level-2 | level-3 | all]
---

Run the RLPS conformance suite and report what it does and does not prove: $ARGUMENTS

**NOT LEGAL ADVICE.** Conformance is a statement about an implementation's behavior against
published vectors — never about legal compliance.

1. **Run the suite.** From `${CLAUDE_PLUGIN_ROOT}`:

   ```
   cargo test -q --manifest-path resolver/Cargo.toml --test conformance -- --nocapture
   ```

   The runner reads `${CLAUDE_PLUGIN_ROOT}/conformance/level-*.json` and pins **exact run and
   skip counts**, so any drift in the vector set fails the test rather than passing quietly.

2. **Read the skips correctly — this is the part people get wrong.** Some vectors are
   authored *ahead* of the reference implementation and are skipped because the resolver does
   not yet advertise their capability. That is deliberate, not a failure. A skipped vector is
   still **normative**: it defines required behavior for any resolver that claims that
   capability. Report skips as "authored-ahead, not yet implemented here", never as "passing".

3. **Report level claims in the governed wording** (`${CLAUDE_PLUGIN_ROOT}/TRADEMARKS.md`).
   To claim **Level N**, an implementation must pass **every** vector for **all** capabilities
   that level requires. Correct forms:
   - "conforms to RLPS Level N, rlps-conformance/0.1" — only when every vector passes.
   - "implements Level N; Level N+1 in progress" — partial support.

   A bare "RLPS conformant" or "RLPS certified" is never correct and is a trademark-policy
   violation. If asked to write marketing copy making such a claim, decline and offer the
   governed wording.

4. **On a genuine failure, apply the precedence rule** (spec is normative; vectors and schemas
   are not):
   - Implementation disagrees with a vector, and the **spec is clear** → the implementation is
     wrong.
   - Vector contradicts the spec → the **vector** is the defect; file an Erratum.
   - Spec is **silent** on the disputed behavior → that silence is itself a spec defect; file
     an Erratum or a Conformance-disagreement issue rather than picking a behavior.

5. **Suite versioning** (`${CLAUDE_PLUGIN_ROOT}/conformance/VERSIONING.md`): a MINOR bump is
   append-only, and level claims do **not** carry across a MAJOR bump. When reporting a claim,
   always name the suite version alongside the level — a level without a suite version is not
   a checkable statement.
