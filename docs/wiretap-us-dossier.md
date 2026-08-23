# `wiretap/us` — secondary-source dossier for the future attorney-of-record

A starting research memo for **roadmap item 8** (the counsel-gated `wiretap/us` jurisdiction
pack). **NOT LEGAL ADVICE.** This document contains **no pack, no control prescriptions, and no
statement of current law** — it summarizes what published secondary sources say, so the engagement
with the attorney-of-record starts from organized material instead of a blank page. The pack
itself MUST NOT be authored until counsel signs off (GOVERNANCE.md hard gate 2; counsel-brief §6), and
everything below must be **re-verified against current statutes by the attorney-of-record** — the
principal source predates this memo by eight years.

## 1. Why this pack is the flagship

The US recording-consent matrix is the planned first jurisdiction pack (counsel-brief §6) because
it is the cleanest demonstration of the RLPS thesis: a single, well-bounded regulated act
(recording a communication) whose required controls differ *by state*, with a documented
choice-of-law rule for the cross-jurisdiction case, and no realistic prospect of harmonization.

## 2. What the secondary sources establish

Principal source: **Rauvin Johl, "Reassessing Wiretap and Eavesdropping Statutes: Making One-Party
Consent the Default," 12 Harvard Law & Policy Review 177 (2018)**
(<https://journals.law.harvard.edu/lpr/wp-content/uploads/sites/89/2018/03/Johl.pdf>). Full
annotation in [`secondary-sources.md`](secondary-sources.md) № 2.

### 2.1 The split (as of the article's 2018 writing — attorney must re-verify)

> "With the exception of Vermont, every state has a wiretap and surveillance statute. Thirty-eight
> states (and the District of Columbia) have one-party consent regimes, just like the federal
> statute. Eleven states have all-party consent schemes."

Three regime classes, not two: one-party (38 + DC + federal), all-party (11), and **no statute**
(Vermont) — the last being exactly the case the resolver's fail-closed floor (spec §5) exists for.

### 2.2 Cross-jurisdiction choice of law

> "If a recording or interception is made across state lines between a one-party state and an
> all-party state, the law of the all-party state controls."

Citing *Kearney v. Salomon Smith Barney*, 137 P.3d 914, 937 (Cal. 2006). This is independent
external validation of the spec's §4.2 **most-restrictive-wins** merge rule: the resolver's
behavior for a multi-jurisdiction participant set mirrors how a US court actually resolved the
conflict. (An informative note at spec §4.2 records this.)

### 2.3 Divergence *within* the all-party category

> "Some all-party consent states make it illegal to covertly record another individual or
> individuals. For example, Montana requires that all parties be aware that a recording is being
> made, and Massachusetts outlaws secret recordings. Oth[ers hinge on expectation of privacy…]"

Per the article: Montana turns on *awareness*, Massachusetts on *secrecy*, Florida on
*expectation of privacy*, and Illinois combines both. Consequence for pack design: "all-party
consent" is **not one control** — the attorney will need to decide, state by state, which controls
from the catalog (attestation, indicator, announcement, notice) actually satisfy each statute's
theory of the offense.

### 2.4 New recording surfaces have unsettled status

The article documents that wearable cameras, smart doorbells, dash cams and similar devices have
unclear legality under existing state statutes, and that vendors do not disclose the wiretap
implications. This is the "hardcoded legal assumption" failure mode in the wild — product features
shipping with an implicit, unreviewed legal posture.

### 2.5 The fragmentation is durable

> "Both suggestions for reform require action from state legislatures on a state-by-state basis,
> which can be difficult and time-consuming."

Harmonization would require 50 separate legislatures; the article treats it as unlikely. The
patchwork must be *managed*, not awaited away — the pack model is the management mechanism.

## 3. Questions for the attorney-of-record (framing only — not answers)

1. **Current matrix:** what is the one-party/all-party/no-statute split *today*, and which states
   have amended since 2018? (The 38/11/1 figures above are point-in-time.)
2. **Per-state control mapping:** for each all-party state, which catalog controls satisfy the
   statute's theory (awareness vs. secrecy vs. expectation-of-privacy vs. combined)?
3. **Choice of law:** is the *Kearney* strictest-state rule the correct general posture for the
   pack's cross-jurisdiction guidance, and are there states where a different conflicts rule
   applies?
4. **Federal overlay:** how should the pack express the federal one-party baseline (18 U.S.C.
   § 2511, per the article) relative to stricter state law?
5. **Scope boundaries:** oral vs. wire vs. electronic communications; in-person vs. remote;
   business-extension and service-provider exceptions — which are in scope for the
   `recording_consent` domain, and which are out?
6. **Temporal envelope:** what `last_reviewed_against_guidance` cadence does counsel consider
   defensible for a 51-jurisdiction matrix (spec §7)?

## 4. Implications already visible for conformance vectors

Without authoring any pack, the sources imply test shapes the conformance suite should eventually
carry (language-neutral vectors, `conformance/`):

- **Mixed participant set** (one-party state + all-party state) → resolved controls equal the
  all-party profile's (most-restrictive, §4.2) — the *Kearney* shape.
- **No-statute jurisdiction** (the Vermont shape) → fail-closed floor, not "no controls" (§5).
- **Same posture, different theory** (Montana-shape vs. Florida-shape all-party profiles) →
  *different control sets* despite the same nominal category — guards against collapsing
  "all-party" into a single boolean.
- **Stale pack** (matrix older than its review cadence) → staleness surfaced per §7.

These become runnable only when a counsel-attested pack exists; they are listed so the vectors can
be drafted alongside the pack during the item-8 engagement.

## 5. Related material

- [`secondary-sources.md`](secondary-sources.md) — the full 28-source evidence annex (esp. № 1–2).
- [`counsel-brief.md`](counsel-brief.md) §6 — the publication sequence this dossier feeds.
- `spec/RLPS-v0.1.md` §4.1–4.2, §5, §7 — the normative machinery the sources externally validate.
- [`not-legal-advice.md`](not-legal-advice.md) — governs this document like every other artifact.
