# Pre-flight name check — `r14n` / RLPS / RegLoc

**Date:** 2026-07-06. **Method:** live registry API calls (crates.io, npm registry, PyPI JSON API,
GitHub REST), DNS NS/SOA lookups, and web search. **This is a best-effort engineering pre-flight,
NOT a formal trademark clearance** — formal USPTO TESS / EUIPO / WIPO searches are assigned to
counsel (see `docs/counsel-brief.md` §5). All findings below are framed as
**"no dominant prior claim"** — absence of hits is not clearance.

> **⚠ NOT LEGAL ADVICE.** See `/docs/not-legal-advice.md`.

## 1. Package-registry squat check (all AVAILABLE)

| Name | crates.io | npm | PyPI |
|---|---|---|---|
| `r14n` | available (`does not exist`) | available (404) | available (404) |
| `rlps` | available (`does not exist`) | available (404) | available (404) |
| `regloc` | available (`does not exist`) | available (404) | available (404) |

**Action when counsel clears publication:** reserve `r14n` on crates.io immediately (the crate
name in `resolver/Cargo.toml`); consider `rlps` + `regloc` defensively on crates.io, and `r14n`
on npm/PyPI for future bindings.

## 2. GitHub namespace

| Handle | Status | Assessment |
|---|---|---|
| `r14n` | **TAKEN** — personal user "Richard", registered 2013-06-13, 12 public repos | Organic pre-existing handle, not a squat. Bare-org `r14n` unavailable. |
| `rlps` | **TAKEN** — personal user "rafael-pacheco", registered 2018-01-20 | Organic; not a squat. |
| `regloc` | available | — |
| `squillo` | **OWNED** (our org, created 2021-02-12) | Canonical home: **`github.com/squillo/r14n`** — matches `resolver/Cargo.toml` `repository`. |

GitHub repo search for `r14n` (2026-07-06): 5 results, all personal-profile repos (usernames that
happen to contain R14N); **no software project named r14n exists on GitHub**.

## 3. Domains (DNS NS/SOA probe — confirm at registrar before purchase)

| Domain | Status |
|---|---|
| `r14n.org`, `r14n.dev`, `r14n.io` | no NS/SOA — likely available |
| `regloc.org` | no NS/SOA — likely available |
| `r14n.com` | **REGISTERED** (third party) |
| `rlps.org` | **REGISTERED** (third party) |

*Recommendation:* `r14n.org` + `r14n.dev` are the natural spec-home domains if wanted; not
blocking (the repo can live under `squillo.com` / GitHub Pages).

## 4. Prior-art / collision search (web, 2026-07-06)

- **"RLPS" (acronym):** existing users, none in software-standards / compliance / legal-tech:
  - **RLPS Architects** (Reese, Lower, Patrick & Scott, Ltd.) — US architecture/interior-design
    firm operating since 1954 (`rlps.com`, holder of `rlps.org`). The most prominent brand user.
  - **RLPS Technology** (`rlpstech.com`) — renewable-packaging technology.
  - **R.L.P.S. Limited** — UK registered company 05735773.
  - Descriptive uses: "real-time local positioning system" (RFID/IoT); "Real-Life
    Problem-Solving" (education). GSM "Radio Link Protocol" is RLP, not RLPS.
  - **Assessment:** collisions exist but in unrelated fields; *no dominant prior claim* in the
    classes RLPS would occupy (software / SaaS / standards, Nice 9/42). Formal knock-out search
    → counsel.
- **"r14n" (numeronym):** not a documented numeronym anywhere found; the numeronym *pattern*
  (i18n, l10n, c14n, i14y, k8s) is established practice and unowned. *No prior claim found.*
- **"regulatory localization" (phrase):** only generic/descriptive usage (adapting a product to
  local regulation). **Known adjacent collision (reconfirmed):** some
  industry usage of "localization + regulatory compliance" means *making compliance content
  multilingual* — we deliberately do NOT use "compliance localization" and lead with the full
  "Regulatory Localization Pack Specification" expansion to avoid that reading.
- **"Regulatory Localization Pack" (literal string):** zero web hits as of 2026-07-06 — recorded
  as *no dominant prior claim*, not as "confirmed clean."

## 5. What remains for counsel (blocking public release)

1. Formal trademark knock-out search: **USPTO TESS** + **EUIPO** + **WIPO Global Brand DB** for
   `RLPS`, `r14n`, `RegLoc`, "Regulatory Localization" (word marks, Nice classes 9, 42, 45).
   The public tmsearch API was not scriptable at pre-flight time; this needs a human/counsel run.
2. Opinion on descriptive-mark weakness of "Regulatory Localization" (likely unregistrable as a
   word mark; fine as a generic discipline name — which is how we use it).
3. Confirm no common-law conflict with RLPS Architects' service marks (unrelated classes, but
   they are a 70-year brand).

## 6. Naming posture (unchanged)

Lead with **RLPS / Regulatory Localization Pack Specification** for formal/SDO/legal audiences;
`r14n` is dev shorthand only. De-emphasize any "localizes law" phrasing (UPL profile — see
`docs/counsel-brief.md` §3). Never claim the name-space is "confirmed clean / zero hits"; the operative claim is **no
dominant prior claim as of 2026-07-06**.
