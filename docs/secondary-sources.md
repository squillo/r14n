# Secondary sources — the problem RLPS addresses, and its solution lineage

An annotated bibliography of **28 verified secondary sources** documenting (a) the scale and cost
of cross-jurisdictional regulatory fragmentation for software vendors, (b) how the AI era
amplifies it, and (c) the institutional precedent for expressing compliance controls as
machine-readable, authored-once artifacts. **NOT LEGAL ADVICE** — these are summaries of what
published secondary sources say, not statements of law (see
[`not-legal-advice.md`](not-legal-advice.md)). Prepared 2026-07-31 as the evidence annex to
[`counsel-brief.md`](counsel-brief.md); the `wiretap/us` sources are expanded in
[`wiretap-us-dossier.md`](wiretap-us-dossier.md).

**Verification status:** every URL below was retrieved live on 2026-07-31 and its content read;
quotations are verbatim extractions from the retrieved page. Two entries (№ 1, № 3) additionally
passed independent 3-vote adversarial fact-checking. Where a figure is a projection or model
estimate rather than measured spend, the entry says so. **Before citing any figure in a filing,
confirm it against the source page** — standard practice for any secondary-source brief.

Credibility classes used below: *peer-reviewed / academic*, *government / national regulator*,
*intergovernmental / standard-setter*, *think tank*, *industry / professional association*,
*law firm / bar association*.

---

## I. Legal scholarship — fragmentation, the consent/privacy patchwork, law-as-code

**1. Anupam Chander, Margot E. Kaminski & William McGeveran, "Catalyzing Privacy Law," 105
Minnesota Law Review 1733 (2021).** *Peer-reviewed law review.*
<https://minnesotalawreview.org/article/catalyzing-privacy-law/>
The leading academic treatment of the US state-privacy patchwork: California — not Brussels — is
catalyzing divergent state-by-state privacy legislation absent federal preemption, and in a single
year over half the states proposed broad privacy bills or task forces. Their close CCPA/GDPR
comparison concludes the California regime is *not* "GDPR-lite":

> "Our close comparison of the GDPR and California's privacy law reveals that the California law
> is not GDPR-lite: it retains a fundamentally American approach to information privacy."

*Supports:* compliance with one major regime does not confer compliance with another —
jurisdiction-scoped policy handling is structurally unavoidable.

**2. Rauvin Johl, "Reassessing Wiretap and Eavesdropping Statutes: Making One-Party Consent the
Default," 12 Harvard Law & Policy Review 177 (2018).** *Law review.*
<https://journals.law.harvard.edu/lpr/wp-content/uploads/sites/89/2018/03/Johl.pdf>
Squarely on the one-party vs. all-party consent divergence in US wiretap law: as of writing, 38
states + DC one-party, 11 states all-party, Vermont no wiretap statute; scope diverges even
*within* the all-party category; and for cross-state recordings the stricter state's law controls
(*Kearney v. Salomon Smith Barney*, 137 P.3d 914 (Cal. 2006)). Expanded in
[`wiretap-us-dossier.md`](wiretap-us-dossier.md).
*Supports:* the flagship `wiretap/us` pack family, and the spec §4.2 most-restrictive-wins merge
rule as consistent with actual choice-of-law practice.

**3. James Grimmelmann, "Programming Languages and Law: A Research Agenda," Proceedings of the ACM
Symposium on Computer Science and Law (CSLAW '22), 2022.** *Peer-reviewed (ACM); open access
arXiv:2206.14879.* <https://dl.acm.org/doi/10.1145/3511265.3550447>
Research agenda by a leading internet-law scholar surveying law-as-code. Documents a real failure
of ad-hoc hardcoded legal logic: France's tax authority computes every taxpayer's bill from a
~3,500-page code via a fragile legacy language ("M", 1990), now being re-engineered into a
formally specified language whose implementation can be proven correct.

> "If there is a lesson to take from the history of law and programming languages' engagement, it
> is that principled methods are often superior to ad hoc ones."

*Supports:* the core RLPS premise — a principled, authored-once substrate over per-feature ad-hoc
encodings of legal assumptions.

**4. Michael Genesereth, "Computational Law: The Cop in the Backseat," CodeX — The Stanford Center
for Legal Informatics (2016).** *Foundational academic (pre-2020).*
<https://law.stanford.edu/2016/01/13/michael-genesereths-computational-law-the-cop-in-the-backseat/>
The seminal CodeX piece defining computational law and envisioning compliance guidance embedded in
everyday systems — rules checked at the moment of action rather than after the fact.
*Supports:* the resolve-at-act-time model (controls resolved before the regulated act proceeds).

## II. Quantitative compliance cost & burden

**5. Daniel Castro et al., "The Looming Cost of a Patchwork of State Privacy Laws," ITIF, Jan.
2022.** *Think tank; entered the Congressional record.*
<https://itif.org/publications/2022/01/24/looming-cost-patchwork-state-privacy-laws/>
The most-cited quantitative estimate of the state-privacy-patchwork cost — **projections, not
measured spend**: $98–112B/year in out-of-state compliance costs (> $1T over 10 years; $20–23B/yr
on small businesses), with the cost driver being fragmentation itself ("multiple, duplicative
rules" imposed across state lines).
*Supports:* the headline cost-of-fragmentation figure; externalizing the duplicated interpretation
is precisely the RLPS mechanism.

**6. IAPP & FTI Consulting, "Privacy Governance Report 2024."** *Industry association.*
<https://iapp.org/resources/article/privacy-governance-report>
99% of surveyed privacy professionals face compliance challenges (55% report five or more); only
43% are fully confident they stay informed of new laws; **69% of chief privacy officers have
absorbed AI governance** on top of privacy, and >80% of privacy teams carry responsibilities
beyond privacy.
*Supports:* manual, human-tracked compliance is already over capacity, and AI-era duties are
landing on the same stretched functions.

**7. Thomson Reuters Regulatory Intelligence, "Cost of Compliance 2023."** *Industry (14th annual;
350+ practitioners).*
<https://www.thomsonreuters.com/en-us/posts/investigation-fraud-and-risk/2023-cost-of-compliance-report/>
Practitioners expect regulatory-change volume to keep rising and rate it a key board-level
challenge — while 45% of firms do not monitor their own cost of compliance at all.
*Supports:* the velocity argument (change outpaces manual tracking) and the missing
instrumentation an authored, diffable pack layer provides.

**8. Ponemon Institute (Globalscape), "The True Cost of Compliance with Data Protection
Regulations," 2017.** *Industry benchmark (seminal, pre-2020).*
<https://dynamic.globalscape.com/files/Whitepaper-The-True-Cost-of-Compliance-with-Data-Protection-Regulations.pdf>
Across 53 multinationals: average compliance cost $5.47M vs. non-compliance cost $14.82M —
**2.71×** — with technology/software organizations showing the second-fastest compliance-cost
growth of any industry (+99% 2011→2017), driven by "multiple and overlapping" regimes (GDPR, PCI
DSS, HIPAA, state privacy laws, SOX).
*Supports:* the economics of systematic, authored-once compliance vs. ad-hoc per-feature failure.

**9. Carl Benedikt Frey & Giorgio Presidente, "Privacy Regulation and Firm Performance: Estimating
the GDPR Effect Globally," Economic Inquiry 62(3), 1074–1089 (2024).** *Peer-reviewed economics;
OA via ora.ox.ac.uk.* <https://onlinelibrary.wiley.com/doi/10.1111/ecin.13213>
Global difference-in-differences estimate: GDPR reduced the profitability of firms targeting
European consumers **through the cost channel** — technology firms saw a 2.1% profit decline with
no sales decline, rising operating expenses and wage bills, and R&D diverted into
compliance-related patenting; the burden fell hardest on smaller firms.
*Supports:* a peer-reviewed anchor (not vendor numbers) that the cost of a regime is engineering
effort diverted to compliance — the effort RLPS externalizes.

**10. Scott Ikeda, "Global 500 Faces GDPR Compliance Costs of $7.8 Billion," CPO Magazine
(reporting the IAPP-EY Annual Privacy Governance survey).** *Trade press — cite the underlying
IAPP-EY report as primary.*
<https://www.cpomagazine.com/data-protection/global-500-faces-gdpr-compliance-costs-of-7-8-billion/>
Headline dollar figure for a single regime: Global 500 firms projected a combined $7.8B GDPR spend
(~$15.8M average per company), largely hiring and one-time product modifications.
*Supports:* an accessible single-regime cost figure; flagged secondary.

## III. The exploding AI regulatory patchwork

**11. Stanford HAI, "The 2025 AI Index Report — Chapter 6: Policy and Governance."** *Academic
institute.* <https://hai.stanford.edu/ai-index/2025-ai-index-report/policy-and-governance>
US state AI laws: 1 (2016) → 49 (2023) → **131 (2024)**; 59 federal AI regulations in 2024 (vs. 25
in 2023) from 42 unique agencies — fragmentation *within* the federal government; across 75
countries, legislative AI mentions rose 21.3% in 2024 and >9× since 2016; election-deepfake laws
reached 24 states by 2024.
*Supports:* the AI-amplification thesis with citable volume/velocity data at state, federal, and
international levels simultaneously.

**12. MultiState, "Artificial Intelligence (AI) Legislation" — 50-state tracker (2026).**
*Government-affairs intelligence firm.* <https://www.multistate.ai/artificial-intelligence-ai-legislation>
1,208 AI bills introduced across all 50 states in 2025 (145 enacted), up from 635 in 2024 and
<200 in 2023 — >6× in two years; as of March 2026, 1,561 bills already introduced in 45 states,
exceeding the full 2025 total. Nonconsensual-deepfake laws passed in 22 states.
*Supports:* raw scale and acceleration — beyond what any per-feature engineering process can track.

**13. IAPP, "US State AI Governance Legislation Tracker."** *Industry association.*
<https://iapp.org/resources/article/us-state-ai-governance-legislation-tracker>
The canonical tracker of cross-sectoral state AI-governance bills imposing private-sector
obligations, categorized by AI system type and **developer-vs-deployer role**; notes legislatures
moved within months of generative AI's emergence where responses historically took decades.
*Supports:* obligations are jurisdiction-scoped *and* role-scoped — both axes a pack must encode.

**14. IAPP, "US state AI legislation: Reviewing the 2025 session."** *Industry analysis.*
<https://iapp.org/news/a/us-state-ai-legislation-reviewing-the-2025-session>
Analytical retrospective: obligations fragment by sector (healthcare, employment, price-fixing,
intimate deepfakes) and single bills increasingly bundle separate guardrails for multiple AI
technologies in different provisions.
*Supports:* per-jurisdiction obligation sets are internally heterogeneous — a reviewable pack per
profile beats hand-parsed statutes per feature.

**15. Future of Privacy Forum, "The State of State AI: Legislative Approaches to AI in 2025," Oct.
2025.** *Think tank.* <https://fpf.org/wp-content/uploads/2025/10/The-State-of-State-AI-2025.pdf>
Using a deliberately narrow industry-facing methodology: 210 bills across 42 states (only eight
states introduced none). Identifies **inconsistent state definitions** of "AI," "generative AI,"
"frontier model," and "chatbot" as a central compliance problem — the definition determines which
systems fall under which duty. Eight 2025 laws impose AI-interaction disclosure duties whose form
varies by state; flags that risk-assessment frameworks in current law may be ill-suited to agents.
*Supports:* divergence is also *definitional* — jurisdiction packs must carry each jurisdiction's
scoping, not assume a shared vocabulary.

**16. Brookings Institution, "How different states are approaching AI" (2025).** *Think tank.*
<https://www.brookings.edu/articles/how-different-states-are-approaching-ai/>
Contrasts structurally different models: Colorado's comprehensive high-risk regime vs.
California's deliberate patchwork of many narrow targeted laws; 47 states introduced AI
legislation in the 2025 session (260 measures), yielding "a patchwork of uneven mandates."
*Supports:* even the *shape* of regulation differs by state — identical features face incompatible
governance models depending on where they run.

## IV. Why AI agents make jurisdictional compliance harder

**17. OECD, "The Agentic AI Landscape and its Conceptual Foundations," OECD Artificial
Intelligence Papers No. 56, Feb. 2026.** *Intergovernmental.*
<https://www.oecd.org/content/dam/oecd/en/publications/reports/2026/02/the-agentic-ai-landscape-and-its-conceptual-foundations_a9d4b451/396cf758-en.pdf>
Defines agentic AI as coordinated agents pursuing complex objectives autonomously over extended
periods in open-ended environments with minimal supervision — the properties that undermine
oversight-based per-feature compliance. Documents a 920% increase in agentic-framework
repositories (early 2023 → mid 2025); flags accountability, transparency, and sovereignty as
unresolved.
*Supports:* governance must be expressed in precise machine-actionable policy the agent consults
at act time (the APH lane).

**18. Hogan Lovells, "AI Agents Under Antitrust and Consumer Protection Scrutiny: Risks and
Opportunities for Compliance" (2026).** *Global law firm client alert.*
<https://www.hoganlovells.com/en/publications/ai-agents-under-antitrust-and-consumer-protection-scrutiny-risks-and-opportunities-for-compliance>
The same agent functionality is classified and enforced differently across regimes: EU AI Act
Art. 50(1) disclosure duties; Italian Competition Authority interim measures over Meta's exclusion
of third-party chatbots from WhatsApp with a parallel European Commission inquiry; regulators
disagreeing whether an agent's search feature makes it a "search engine" under the DSA.
*Supports:* cross-border agents face divergent, simultaneous multi-regulator scrutiny — the
patchwork packs absorb.

**19. Squire Patton Boggs, "The Agentic AI Revolution: Managing Legal Risks" (2026).** *Global law
firm client alert.*
<https://www.squirepattonboggs.com/insights/publications/the-agentic-ai-revolution-managing-legal-risks/>
AI regulation is jurisdictionally fragmented (EU overarching legislation; US "fragmented patchwork
of state-level regulation and federal laws"; UK no centralized AI statute); traditional frameworks
assume a human decision-maker, creating an accountability gap when an agent's output is remote
from any human instruction. Cites *Moffatt v. Air Canada* (2024, company liable for its chatbot)
and the EU Product Liability Directive (implementation deadline 2026-12-09) classifying software
and AI as "products."
*Supports:* liability for agentic action is unsettled and jurisdiction-dependent — hard-coded
legal assumptions are untenable.

**20. American Bar Association, "The Hands-Free Future: Compliance, Consent, and Control in
Agentic AI," SciTech Lawyer (Spring 2026).** *Bar-association scholarship.*
<https://www.americanbar.org/groups/science_technology/resources/scitech-lawyer/2026-spring/hands-free-future-compliance-consent-control-agentic-ai/>
A single consumer-facing agentic transaction simultaneously implicates multiple federal regimes
(FTC Act; potentially GLBA's security-program requirement; CAN-SPAM) layered on state statutes
that may be stricter. Its recommended controls map directly to pack vocabulary: mandatory AI-agent
disclosure, data-retention/minimization limits, heightened parental-consent safeguards for minors.
*Supports:* the `ai_disclosure`, retention, and minor-protection pack families as counsel-authored
controls one act must satisfy at once.

**21. Future of Privacy Forum, "AI Governance Behind the Scenes: Emerging Practices for AI Impact
Assessments," 2025 Update.** *Think tank (empirical; 60+ companies consulted).*
<https://fpf.org/wp-content/uploads/2025/04/FPF_AI_Governance_Behind_the_Scenes_Digital_-_2025_Update.pdf>
Assessment obligations are non-uniform across jurisdictions (EU, Colorado, Canada, China);
organizations already treat **deployment geography as a distinct compliance variable**, compiling
per-use-case jurisdiction lists by hand — manual, ad-hoc, non-scalable.
*Supports:* practitioners already localize compliance by geography informally; RLPS is the
formalization of that existing practice.

**22. U.S. Federal Trade Commission, "FTC Announces Crackdown on Deceptive AI Claims and Schemes"
(Operation AI Comply), Sept. 25, 2024.** *US federal regulator (primary).*
<https://www.ftc.gov/news-events/news/press-releases/2024/09/ftc-announces-crackdown-deceptive-ai-claims-schemes>
Five enforcement actions applying existing Section 5 authority to AI conduct with no new statute —
including DoNotPay's "robot lawyer" that purported to scan a business for "hundreds of federal and
state law violations."

> "Using AI tools to trick, mislead, or defraud people is illegal… there is no AI exemption from
> the laws on the books." — FTC Chair Lina M. Khan

*Supports:* obligations arrive via enforcement posture, not only legislation — compliance logic
must be counsel-updatable, not frozen in code. (The DoNotPay action is also a cautionary tale this
project's not-legal-advice posture is designed around.)

## V. Hard-coded-compliance failure & the case for policy-as-code

**23. Jamie Mohun & Alex Roberts, "Cracking the Code: Rulemaking for Humans and Machines," OECD
Working Papers on Public Governance No. 42 (2020).** *Intergovernmental.*
<https://oecd-opsi.org/publications/cracking-the-code/>
The canonical Rules-as-Code primer: publish an authoritative machine-consumable version of rules
alongside the natural-language text, so implementers consume one official version instead of each
re-interpreting the law.

> "The current system creates the need for repeated interpretation and translation, from natural
> language rules into the machine-consumable rules almost ubiquitously required for modern service
> delivery."

Documents live programs in New Zealand, France, Australia, Canada, Germany and Jersey.
*Supports:* the exact architectural pattern RLPS applies one layer up (control prescriptions
rather than the rules themselves) — and that machine-consumable rules lower compliance burdens and
drive consistent application.

**24. NIST, "Open Security Controls Assessment Language (OSCAL)."** *US government standard.*
<https://csrc.nist.gov/projects/open-security-controls-assessment-language>
The shipping existence proof of author-once/inherit-everywhere in compliance: machine-readable
control formats (XML/JSON/YAML) reused across frameworks (FedRAMP, SP 800-53, SOC 2, international
adopters). NIST: automation "can reduce audit timelines from months to just minutes, decrease the
likelihood of human error, and help organizations adapt more quickly to the changing regulatory
requirements."
*Supports:* feasibility — a standards body already ships this model at scale (RLPS already lists
OSCAL as interop, spec §8/§11; this entry adds the *why*).

**25. UK Financial Conduct Authority (with the Bank of England), "Digital Regulatory Reporting."**
*Government regulator.* <https://www.fca.org.uk/innovation/regtech/digital-regulatory-reporting>
Two major regulators piloted Machine Readable / Machine Executable Regulation (MRR/MER) with seven
banks — the regulator itself converting rulebook text into code. Quantifies the burden (regulatory
reporting estimated at £1.5–4B/yr for UK firms; ~500,000 scheduled reports/yr received) and the
failure mode: submitted data "not always consistent, timely or of a sufficient standard to be
useful."
*Supports:* regulators treat machine-executable rules as the fix for hand-interpreted compliance.

**26. FCA & Bank of England, "Digital Regulatory Reporting: Pilot Phase 2 Viability Assessment"
(2020).** *Government regulator (quantified pilot evaluation).*
<https://www.fca.org.uk/publication/discussion/digital-regulatory-reporting-pilot-phase-2-viability-assessment.pdf>
The empirical backbone: the **single biggest cost driver** of the current approach is
"inconsistent interpretation of regulations (and its disambiguation)"; cross-jurisdictional
fragmentation multiplies cost ~6× (derivatives reporting to ~7 jurisdictions vs. mortgage
reporting to 2 UK regulators); modeled 33% reduction in a large firm's annual reporting costs
under machine-executable reporting, with interpretation costs alone falling 65%.
*Supports:* hand interpretation is the measured cost driver; machine-readable rules cut it — with
fragmentation as the multiplier.

**27. Financial Stability Board, "The Use of Supervisory and Regulatory Technology by Authorities
and Regulated Institutions," 9 Oct. 2020.** *International standard-setter (G20-mandated).*
<https://www.fsb.org/wp-content/uploads/P091020.pdf>
Regulatory complexity and volume have outgrown ad-hoc human interpretation: large compliance-spend
increases post-2008; reporting "increasingly complex and expensive"; authorities across many
jurisdictions piloting data-native approaches; **lack of common data standards** cited as a
significant barrier.
*Supports:* the burden has structurally exceeded manual interpretation, and the missing piece is a
common machine-readable standard — the layer RLPS proposes.

**28. Raphael Auer, "Embedded Supervision: How to Build Regulation into Blockchain Finance," BIS
Working Papers No. 811 (2019).** *Central-bank research (seminal, pre-2020).*
<https://www.bis.org/publ/work811.pdf>
The strongest institutional articulation of compliance-by-architecture: supervision "embedded" —
verified from a system's native data rather than bolted on through manual reporting; the BIS line
of work notes machine-readable regulation helps narrow the gap between regulatory intent and
interpretation.
*Supports:* verifying compliance from the system's own data against authored rules — the
decision-receipt lane (spec §8, `resolver/src/receipt.rs`) is RLPS's version of this.

---

## Mapping: sources → planned pack families

| Pack family (roadmap item 8) | Primary sources above | Concrete statutory anchors the sources identify |
|---|---|---|
| `recording_consent` (shipped catalog) / `wiretap/us` (counsel-gated) | 1, 2 | One-party/all-party split; *Kearney* cross-border rule; within-category divergence — see [`wiretap-us-dossier.md`](wiretap-us-dossier.md) |
| `ai_disclosure` | 11, 15, 18, 20, 22 | EU AI Act Art. 50(1) interaction disclosure; Utah SB 452 chatbot notice; Colorado AI Act (SB 205) annual impact re-assessment triggers; 24-state election-deepfake and 22-state NCII patchworks |
| `minor_protection` | 20 | Parental-consent and enhanced-safeguard duties for minors' data in agentic transactions |
| `data_retention` / minimization | 20, 21 | Retention/minimization limits recommended as structural agentic-AI controls |
| `data_residency` / `export_control` | 21, 27 | Deployment geography as an explicit assessment variable; cross-border reporting divergence |
| Temporal envelope (spec §7) test data | 15, 19, 21 | Dated obligations: EU Product Liability Directive 2026-12-09; Colorado AI Act effective Feb 2026 — realistic `valid_from`/staleness fixtures |

Definitional divergence (source 15) is a standing design consideration for `ai_disclosure`
scoping: which systems a duty attaches to differs by jurisdiction *by definition*, so jurisdiction
packs must carry the reviewing attorney's scoping notes rather than assume a shared vocabulary.

## Provenance

Collected via a fan-out research workflow (5 parallel search sweeps → 25 full-source fetches with
verbatim quote extraction → adversarial verification), then hand-verified 2026-07-31. A formatted,
shareable rendering of this bibliography was delivered to counsel separately; **this file is the
source of record for the repo.**
