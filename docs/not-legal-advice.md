# NOT LEGAL ADVICE

**RLPS (regloc) is a data-interchange and configuration standard. It is not legal advice, and no
part of this project — the specification, the resolver, the schemas, the control catalog, or any
pack — constitutes legal advice or creates an attorney–client relationship.**

## What this means

- **A pack is a template of controls, not a statement of law.** A pack expresses *which controls an
  author chose to require* for a profile. It does not tell you what any jurisdiction's law actually
  is, nor guarantee that following it makes you compliant.
- **"aggressive" ≠ "compliant."** The `aggressive` posture over-restricts by design — it can require
  more than the law demands (and may itself raise data-minimization concerns). The `minimal` posture
  under-restricts to a floor. Neither is a compliance guarantee.
- **Liability rests with the consuming operator.** If an application sets a posture that under-
  restricts a jurisdiction's law, responsibility for that choice lies with the operator who deployed
  it — not with RLPS, the resolver, or the pack author. The specification cannot and does not
  allocate legal liability.
- **Provenance proves signing, not correctness.** An Ed25519 signature or a `legal_review: approved`
  status proves *who* attested a pack, not that the reviewer was qualified or that the review was
  correct. Decisions from unattested or self-attested packs are **advisory-only**.

## Before relying on any jurisdiction pack

Real jurisdiction packs (e.g. US wiretap, GDPR) are **held out of this repository until reviewed by
licensed counsel** and are not present in this counsel-safe subset. Any such pack, when published,
MUST name a reviewing attorney of record and bar credential. Consult qualified counsel in the
relevant jurisdiction before relying on RLPS for any real compliance decision.

## Unauthorized practice of law (UPL)

RLPS is positioned as a *specification and configuration format*, not a tool that "localizes law" or
produces legal conclusions. Contributors and adopters must not represent RLPS output as legal advice.
