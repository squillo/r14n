# APH integration — what `aph_mandate` means, and what satisfies it

How RLPS control keys bind to the **APH (Agent per Human)** notarization protocol
(<https://github.com/squillo/aph>, spec `spec/aph-0.1.md`).

> **⚠ NOT LEGAL ADVICE.** A control key is a stable identifier for a required action. Nothing
> here states what any jurisdiction's law requires, and satisfying an APH-backed control is not
> a finding that recording, processing, or delegation is lawful. See
> [`not-legal-advice.md`](not-legal-advice.md).

**This document cites APH; it does not restate it.** Verification rules, error semantics, and
freshness bounds live in the APH specification and are normative *there*. Copying them here
would create a second copy that silently ages into disagreement. Every rule below is a citation
plus an RLPS-side outcome. If a citation and the APH spec disagree, **APH wins and this file is
the defect** — file it as an Erratum against this repository.

## The boundary: RLPS prescribes, it does not verify

RLPS answers *which controls are required*. It never answers *whether a control is satisfied*.
That second question belongs to the **enforcement gate** in the consuming application, and APH
verification lives there.

Concretely, and deliberately: **the reference resolver does not link `aph-core`.** Not for
dependency hygiene (though `resolver/` is `serde` + `serde_json` + `toml` + `std` and stays that
way), but because linking a verifier into the resolver would make the resolver a verifier — and
a component that both prescribes and adjudicates cannot be audited as either. The resolver treats
every control key as opaque (spec §2); an APH-backed key is opaque to it exactly like every other
key.

So the integration is a **definition**, not a dependency:

```
RLPS pack        → requires control key K            (prescription — this repository)
Control catalog  → K means "an APH envelope that …"  (definition — this document)
Enforcement gate → links aph-core, verifies, decides (satisfaction — the consuming application)
```

A consuming gate links [`aph-core`](https://crates.io/crates/aph-core) directly when it is Rust
or wasm; the APH bindings exist for languages that cannot.

## The decision: two controls, because a control key *is* a badge

APH distinguishes two trust models that are **not** interchangeable (spec §7.1.7): an envelope is
`PrincipalSigned` — the human's own key signed this act — or `NotaryAttested` — a notary asserts
the human authorized it. Absent means `NotaryAttested`, deliberately: **the default is the weaker
claim**, and it must never be read as "probably the strong one."

APH's security considerations §2.6 requires that a verifier accepting both modes render them
differently to a human, because *"Alice signed this"* and *"Alice's notary says Alice approved
this"* are different sentences — and states that collapsing them into one badge is the defect
that section exists to prevent.

**In RLPS, a control key is that badge.** A single `aph_mandate` key satisfied by either mode is
exactly the collapse §2.6 forbids, expressed in this project's vocabulary — and it fails RLPS's
own rule that a control key is long-lived audit evidence. A key whose strength varies invisibly
by payload cannot be audited: a receipt naming it would record that *something* authorized the
act, while concealing whether a human ever signed anything.

So the catalog carries **two** keys, and `aph_mandate` is **deprecated**:

| Control key | Satisfied by | What it is |
|---|---|---|
| `aph_mandate_principal_signed` | An envelope that passes APH §8.3 verification with `attestationMode = "PrincipalSigned"` (§7.1.7), evaluated at decision time | **Consent, cryptographically.** The principal's own key signed this act. |
| `aph_mandate_notary_attested` | An envelope that passes APH §8.3 verification with `attestationMode = "NotaryAttested"` (or absent), evaluated at decision time | **Provenance, not consent.** A notary asserts the human authorized it; the human's key never touched this envelope. |
| `aph_mandate` | — | **DEPRECATED.** Ambiguous between the two above. Retained because a control key is long-lived audit evidence and shipped receipts reference it; MUST NOT be used in new packs. |

`PrincipalSigned` is strictly stronger, so an envelope satisfying
`aph_mandate_principal_signed` also satisfies `aph_mandate_notary_attested`. The converse is
false and a gate MUST NOT treat it as true — that is the downgrade §2.6 describes, and APH gives
the exact refusal: a verifier requiring `PrincipalSigned` reads `attestationMode` **first** and
refuses anything else with `APH_E012 AttestationModeRefused`, before doing verification work
(§8.3.1 step 1a).

**Which one does a consent-gating pack require?** `aph_mandate_principal_signed`, and it must
refuse the downgrade. A pack that requires only `aph_mandate_notary_attested` is asserting that
provenance suffices for its purpose — a deliberate, visible, reviewable choice, which is the
point of splitting the key.

### What these controls do NOT mean

An APH mandate authorizes **the agent to act for its own principal**. It says nothing about any
*other* participant's consent. In an all-party-consent jurisdiction, a principal-signed mandate
from the recording party's own human does not supply the other parties' consent — that is what
`attestation`, `all_party_consent`, `announcement`, and `signal_notice` are for. Treating a
mandate as though it discharged all-party consent would be a category error with legal weight,
and no posture in this repository does so.

## Evaluated when: at decision time, with a live status check

`aph_mandate_principal_signed` and `aph_mandate_notary_attested` mean **the authority is live at
evaluation time** — not "an envelope was validly issued at some past moment." An APH mandate is
revocable: the human can pull it, and revocation reaches verifiers through the status list the
notary publishes (APH §6.3.3), with a freshness bound that makes a stale surface fail (§6.3.3.3).

This is the same posture RLPS takes everywhere, arriving from the other side: both projects fail
closed on a surface they cannot establish.

| APH outcome | RLPS-side effect |
|---|---|
| Verifies, status bit clear | Control **satisfied** |
| `APH_E015 MandateRevoked` — the human withdrew the authority | Control **not satisfied**. A revoked mandate is never re-activated (APH §6.3.2); the remedy is to ask the human again. |
| `APH_E008 NotaryServiceUnreachable` — status could not be established | Control **not satisfied**. Unreachable is a refusal, not a warning; an attacker who can make the check fail must not thereby choose that it is skipped. |
| `APH_E016 MandateRequired` — nothing authorized this act | Control **not satisfied** |
| `APH_E011` / `APH_E006` — an authorization exists and is invalid | Control **not satisfied** |
| `APH_E012 AttestationModeRefused` — `NotaryAttested` offered where `PrincipalSigned` required | `aph_mandate_principal_signed` **not satisfied** (`aph_mandate_notary_attested` may still be) |

## What an enforcement gate must record

A verdict is a boolean; an audit trail is not. When a gate records satisfaction of an
APH-backed control, it MUST carry:

1. **The APH error code on failure**, not just "not satisfied". `APH_E016` (nothing authorized
   this) and `APH_E015` (the human revoked it) are different facts about the same failed
   control, and their remedies differ completely — mint a mandate versus ask the human again.
   An auditor needs the distinction; a boolean destroys it.
2. **The `attestationMode` actually presented**, so the receipt records which of the two
   sentences is true.
3. **The envelope's `bodySha256`.** An APH envelope commits to specific message bytes. A record
   that names an envelope without carrying its body digest holds a *reference*, not a *binding*,
   and the tamper-evidence does not transfer.
4. **The protocol version accepted** — the envelope's `aphVersion` field, currently `"0.1"`.

**Status in this repository:** RLPS decision receipts (spec §8) record *what was required*, not
*what was satisfied* — satisfaction happens downstream in the gate. So the four items above are
normative guidance for consuming gates, and are not fields the reference resolver emits today.
When an enforcement-side receipt profile is specified, it lands as an RFC against this
repository, not as a quiet addition.

## Version pinning — two versions, moving independently

- **Protocol:** the envelope's `aphVersion` field, currently `"0.1"`. This is what a satisfying
  envelope must carry, and what a gate pins in its accept policy.
- **Crate:** `aph-core` is `0.1.0-alpha.1` on crates.io. A consuming gate pins this in its own
  `Cargo.toml`.

Neither implies the other. This repository pins **neither** — it links no APH code (see the
boundary above); both pins belong to the consuming gate.

## Testing an enforcement gate

Test against APH's twelve published vectors in its `examples/` before writing local fixtures —
four carry real signatures, and `principal_signed_envelope.json` with its published body file
exercises the full path including the body-hash binding. Do **not** invent values for APH's
closed enums to make something fit: the closed sets are a published contract, and a needed value
that does not exist is an RFC against APH, never a local extension.

## Status of this integration

As of 2026-08-24, `aph_mandate*` in this repository is **a named control key with no APH wire
bytes behind it**: it appears in the catalog, in posture baselines, in conformance vectors as an
opaque set-algebra token, and in a receipt example. No code in this repository mints, parses, or
verifies an APH envelope. The reporting obligation this creates for APH's pre-production
exception is tracked in [`docs/aph-dependency-report.md`](aph-dependency-report.md).
