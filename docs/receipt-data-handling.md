# Receipt data-handling guidance

RLPS decision **receipts carry personal data** — a data-subject identifier (`pii_principal.id`), a
controller identity, jurisdictions, and processing operations. Handle them as records containing
PII. **NOT LEGAL ADVICE.**

## What a receipt contains

- `pii_principal.id` — the data subject (may be a pseudonymous handle; may be directly identifying,
  depending on what the caller supplies).
- `pii_controller.id`, `record.record_id`, `record.issued_at`, `language`.
- `processing` (domain, subject, operations), `jurisdiction` (ISO-3166 + attribution source),
  `decision` (posture, required controls, escalation), `provenance` (pack source, review status,
  taint), and any `ai_disclosure` (timestamp + method).

## Handling

- **Minimize the principal id.** Prefer a pseudonymous, per-session identifier over a directly
  identifying one where the use case allows.
- **Store + transmit as sensitive data.** Receipts are audit evidence; protect them at rest and in
  transit like other consent/processing records. They may themselves be subject to
  retention/erasure obligations under the very regimes RLPS helps localize.
- **Signing exposes bytes, not extra data.** A signed receipt is content-addressable; the signature
  adds no PII, but publishing a receipt publishes everything in it.
- **The receipt is a control-prescription, not consent evidence.** It records which controls were
  *required*, never that consent was obtained (the Kantara shim carries a top-level `notice` to that
  effect). Do not treat a receipt as proof a data subject consented.

## Retention

RLPS does not prescribe a retention period for receipts — that is the deploying operator's decision
under applicable law. Record why a receipt is retained and for how long, and include receipts in the
same erasure/retention tooling as your other processing records.
