# APH dependency report — what r14n depends on, and when that changes

A standing, dated answer to the question APH's contribution rules make load-bearing. APH carries
a **pre-production exception**: while nobody outside that repository depends on the wire format,
defect corrections land in place in the 0.1 draft instead of forking a version.

r14n is the candidate for *someone*. This file exists so APH's maintainer never has to guess,
and so the answer has a date on it rather than living in a conversation.

**The trigger, as APH now states it** (sharpened in response to this report — APH `CONTRIBUTING.md`,
commit `5932f2a`; do not paraphrase it here, cite it): the test is whether a wire change would
**break** a consumer or merely **cost them a documentation edit**. A consumer that defines its
terms by citation and mints, parses, verifies, and asserts nothing is documentation-only. The
exception expires when someone outside ships an artifact that **asserts wire facts** — code that
mints, parses, or verifies an envelope; a schema or receipt carrying `aphVersion`,
`attestationMode`, `bodySha256`, or an `APH_E*` code **as data**; or conformance vectors
containing real envelope bytes.

That list is the checklist this report answers against.

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

Checked against APH's enumerated list: r14n ships **no** minting/parsing/verifying code, **no**
schema or receipt carrying `aphVersion` / `attestationMode` / `bodySha256` / an `APH_E*` code as
data, and **no** vectors containing envelope bytes. Every APH term in this repository is a
citation in prose or an opaque token — a wire change costs us an edit.

So: **the pre-production exception is still in force as far as r14n is concerned**, and APH
recorded it as such on 2026-08-24 ([squillo/aph#1](https://github.com/squillo/aph/issues/1)). If
a normative change would improve the protocol, now is still cheap.

One item on that list deserves watching, because it is the nearest thing to a crossing:
[`aph-integration.md`](aph-integration.md) tells consuming gates to record `attestationMode`,
`bodySha256`, and `APH_E*` codes. That is **guidance about** wire facts, not an artifact carrying
them, so it stays on the safe side of the test — but the moment this repository ships a receipt
*profile* rather than a paragraph, it crosses. See the RFC discussion below.

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

## Normative changes requested from APH

**Our own design still needs nothing.** The two-control split requires a verifier to *demand* an
attestation mode as policy rather than discover it after the fact, and APH already specifies
exactly that (§8.3.1 step 1a, refusing with `APH_E012`).

**One request, raised 2026-08-24 as [squillo/aph#2](https://github.com/squillo/aph/issues/2):
recording that a status check happened.** APH's maintainer
identified this gap first and deliberately declined to file it, on the reasoning that an RFC
written before anyone has hit the problem is speculation with a number on it. That reasoning is
right about the *evidence-record design* and, we think, wrong about one narrow piece — because
the drift it warns of has already started here: `aph-integration.md` had to tell gates what to
record, and inventing that vocabulary locally in the first downstream repository is precisely the
failure mode. The RFC therefore asks for the minimum that prevents divergence (naming the status
dispositions APH's §6.3.3.4 already enumerates) and explicitly does **not** ask for the evidence
schema, which stays deferred until a real gate exists.

**On citation.** APH asked whether it may cite this project's sentence — *"a key whose strength
varies invisibly by payload cannot be audited"* — in its security considerations. It may, and no
permission was needed: all prose in this repository is **CC-BY-4.0** (see `LICENSE`), so the only
condition is attribution.

One deliberate divergence from APH's integration guidance, recorded here so it is visible rather
than discovered: APH recommended linking `aph-core` directly. **The r14n reference resolver links
no APH code**, because the resolver prescribes and does not adjudicate; a component that did both
could be audited as neither. Consuming enforcement gates link `aph-core` (Rust/wasm) and pin the
crate version and the protocol `aphVersion` separately.

## Revision history

- **2026-08-24** — initial report. Status (a): documentation-only consumer, no wire dependency.
- **2026-08-24b** — APH answered ([#1](https://github.com/squillo/aph/issues/1)) and sharpened
  the exception's trigger in response (`5932f2a`); this file now cites the corrected test instead
  of the superseded phrase, and checks r14n against APH's enumerated list of wire-asserting
  artifacts. Status unchanged: **(a)**, exception in force, recorded on both sides. Filed
  [#2](https://github.com/squillo/aph/issues/2) — the one normative request, scoped to naming the
  passing status dispositions and explicitly not to designing the evidence record.
