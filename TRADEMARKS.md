# Trademark & conformance-claim usage policy

**NOT LEGAL ADVICE.** This is a usage policy for the project's names and conformance claims, not a
grant of trademark rights and not a legal opinion. See [`docs/not-legal-advice.md`](docs/not-legal-advice.md).

> Formal trademark clearance for **RLPS** / **r14n** / **RegLoc** is pending counsel
> (`docs/preflight-name-check.md` records the engineering pre-flight; a formal USPTO/EUIPO/WIPO
> knock-out search is counsel's task). This policy applies once the names are cleared and the repo
> is public.

## The names

- **RLPS** — Regulatory Localization Pack Specification (formal name).
- **RegLoc** — the long-form discipline name.
- **r14n** — the dev shorthand and the reference crate/CLI name.

Use the names to refer to the specification, the format, and this project. Do **not** use them in a
way that implies endorsement, certification, or authorship by the project of a product that is not
this project — e.g. do not name a fork or a competing implementation `r14n` or "RLPS", and do not
brand a product "RLPS Certified".

## Conformance claims

A resolver or tool MAY state that it **"conforms to RLPS Level N"** (1 minimal-viable / 2 configured
/ 3 comprehensive-with-provenance) **only if** it passes every vector in the corresponding
`/conformance/level-N.json` for **all** capabilities that level requires (see
`conformance/README.md`). State the suite version you tested against (e.g. "RLPS Level 2,
rlps-conformance/0.1"). Partial or self-asserted conformance MUST be described as such
("implements RLPS Level 1; Level 2 in progress"), never as bare "RLPS conformant".

A conformance claim is an **engineering** claim about behavior. It is **never** a claim that any
pack, decision, or receipt is lawful — "aggressive" ≠ "compliant" (spec §Posture).

## Attribution

Attribution of the spec/vocabulary/vectors follows CC-BY-4.0 (`LICENSE-CC-BY`); attribution of the
code follows Apache-2.0 (`LICENSE-APACHE` + `NOTICE`).
