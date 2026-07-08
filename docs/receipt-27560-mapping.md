# RLPS receipt → ISO/IEC TS 27560 mapping

How the RLPS decision receipt (`rlps-receipt/0.1+iso27560`,
[`schema/receipt.schema.json`](../schema/receipt.schema.json)) maps onto the ISO/IEC TS 27560:2023
consent-record **abstract structure**. **This is a structural approximation, NOT the DPVCG
"dpv-27560" JSON-LD profile** (which serializes `dpv:ConsentRecord` with `dct:conformsTo` /
`dpv:hasDataSubject` / `dpv:hasProcess`); adopting those profile properties is future work.
**NOT LEGAL ADVICE.**

| TS 27560 abstract section | RLPS receipt field | Notes |
|---|---|---|
| Record header | `record` (`schema_version`, `record_id`, `issued_at`, `language`) | Identity + issuance of the record. |
| PII principal | `pii_principal` (`@type: dpv:DataSubject`, `id`) | Uses the established DPV class. |
| PII controller | `pii_controller` (`@type: dpv:DataController`, `id`) | Uses the established DPV class. |
| Processing / purpose | `processing` (`@type: rlps:RegulatedActivity`, `domain`, `subject`, `operations`) | `operations` are RLPS-namespaced IRIs (`rlps:AudioRecording`, …). |
| Jurisdiction | `jurisdiction` (`dpv:hasJurisdiction`, `attribution_source`) | `dpv:hasJurisdiction` is a real DPV property; **values here are ISO-3166 strings** (RLPS convention), not DPV location-concept IRIs — full alignment is future work. |
| Event / status | `event[]` (`event_time`, `event_type`, `event_state`) + `decision` (`verdict`, `posture`, `required_controls`, `escalation`) | The decision + any AI-disclosure event. `event_state` carries the verdict / disclosure method. |
| Provenance / evidence | `provenance` (`profile`, `pack_source`, `legal_review_status`, `fell_back`, `provenance_verified`, `advisory_only`, optional `pack_sha256`, optional `last_reviewed_against_guidance`) | The trust-root taint lives here: `advisory_only` is `true` unless provenance is cryptographically verified. |
| — (RLPS addition) | `disclaimer` | NOT-LEGAL-ADVICE, always present. |
| — (RLPS addition) | `ai_disclosure` (`disclosed_at`, `method`) | EU-AI-Act-§50 disclosure FACTS only (no legal-basis claim). |

**What RLPS does NOT assert:** the receipt records which controls were *required*, never that consent
was *obtained*; and it names no governing statute (the resolver emits no legal conclusion). The flat
Kantara CR v1.1 shim (also emitted) carries a top-level `notice` to the same effect.

**Interop caveats (tracked):**
- Not the DPVCG dpv-27560 JSON-LD profile (structural approximation only).
- `dpv:hasJurisdiction` values are ISO-3166 strings, not location-concept IRIs.
- DPV terms confirmed present in DPV 2.x: `dpv:DataSubject`, `dpv:DataController`,
  `dpv:hasJurisdiction` (see [`namespace.md`](namespace.md)).
