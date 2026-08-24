# APH dependency report — what r14n depends on, and when that changes

A standing, dated answer to the question APH's contribution rules make load-bearing. APH carries
a **pre-production exception**: while nobody outside that repository depends on the wire format,
defect corrections land in place in the 0.1 draft instead of forking a version — and that
exception expires "the moment someone outside this repository depends on the wire format."

r14n is the candidate for *someone*. This file exists so APH's maintainer never has to guess,
and so the answer has a date on it rather than living in a conversation.

> **⚠ NOT LEGAL ADVICE.** See [`not-legal-advice.md`](not-legal-advice.md).

## Status as of 2026-08-24: **(a) — a named control key, no wire bytes**

r14n does **not** depend on the APH wire format today. Specifically:

| Where APH appears in r14n | What it is |
|---|---|
| `catalog/recording_consent.catalog.toml` | Two control keys defined by *citation* to APH §8.3 + §7.1.7 (`aph_mandate_principal_signed`, `aph_mandate_notary_attested`), and the deprecated `aph_mandate` |
| `packs/aggressive/`, `packs/example/region/` | Control keys named in posture baselines |
| `conformance/level-1.json`, `level-2.json` | `aph_mandate` as an **opaque set-algebra token** — the vectors exercise universe intersection, floors, and `[prohibited]` subtraction; any opaque string would serve, and the resolver treats every control key as opaque by design (RLPS spec §2) |
| `docs/examples/receipt-ai-act-50.json` | The key appears in a receipt's `required_controls` |
| `resolver/`, `tools/`, `bindings/` | **Nothing.** No APH crate is linked, no envelope is minted, parsed, or verified, and no `aphVersion` is read or asserted anywhere in this repository. |

So: **the pre-production exception is still in force as far as r14n is concerned.** Breaking
corrections to the 0.1 draft cost us a documentation edit, not a migration. If a normative change
would improve the protocol, now is still cheap.

## When that becomes (b)

Transition to a real wire dependency requires an **enforcement gate** — the component that
verifies an envelope and decides whether a control is satisfied. By design that gate is *not* in
this repository (RLPS prescribes; it does not verify — see
[`aph-integration.md`](aph-integration.md)), so the transition happens in a consuming
application, and r14n crosses into (b) only when it ships artifacts that assert wire facts:
an enforcement-side receipt profile carrying `attestationMode`, `bodySha256`, and APH error
codes; or conformance vectors that carry real envelope bytes rather than opaque tokens.

**There is no committed date.** Two gates sit in front of it, and neither is scheduled:

1. **The counsel gate.** Real-jurisdiction packs — the packs that would make delegate
   authorization operational rather than illustrative — are held out pending legal review
   (`GOVERNANCE.md` hard gates). RLPS is Stage 1 of a three-stage plan.
2. **An enforcement gate implementation**, which does not exist in any form today.

**Commitment instead of a date:** r14n will notify APH *before* shipping anything that asserts
APH wire facts — by updating this file and filing an issue against `squillo/aph` — rather than
letting the dependency arrive silently and strand the exception. Until such a notice exists,
APH's maintainer may treat this repository as a documentation-only consumer.

## Normative changes requested from APH: none at this time

r14n's needs are met by the published 0.1 draft. In particular, the two-control split this
repository adopted requires a verifier to *demand* an attestation mode as policy rather than
discover it after the fact, and APH already specifies exactly that (§8.3.1 step 1a, refusing with
`APH_E012`). Nothing about our design needs the spec to move.

One deliberate divergence from APH's integration guidance, recorded here so it is visible rather
than discovered: APH recommended linking `aph-core` directly. **The r14n reference resolver links
no APH code**, because the resolver prescribes and does not adjudicate; a component that did both
could be audited as neither. Consuming enforcement gates link `aph-core` (Rust/wasm) and pin the
crate version and the protocol `aphVersion` separately.

## Revision history

- **2026-08-24** — initial report. Status (a): documentation-only consumer, no wire dependency.
