// SPDX-License-Identifier: Apache-2.0
// Smoke tests against the BUILT npm package — the artifact a consumer installs,
// not the Rust source. The Rust-level contract is pinned by `cargo test` in
// bindings/wasm; what these prove is that the generated glue, the .wasm, and
// the published package metadata actually work together under Node.
//
// Usage: node --test bindings/npm/  (after `node bindings/npm/build.mjs`)
import { test } from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const outDir = process.env.R14N_NPM_DIR
  ? resolve(process.env.R14N_NPM_DIR)
  : resolve(repoRoot, "dist/npm");

const require = createRequire(import.meta.url);
const r14n = require(resolve(outDir, "r14n.js"));
const manifest = JSON.parse(readFileSync(resolve(outDir, "package.json"), "utf8"));

const PACK =
  '[meta]\nstrictness = "minimal"\n[subject.telepresence]\ncontrols = ["user_attestation"]\n[legally_required]\ncontrols = ["user_attestation"]\n';
const packs = JSON.stringify({ "minimal/recording_consent.r14n.toml": PACK });
const query = JSON.stringify({
  domain: "recording_consent",
  profile: "minimal",
  jurisdiction: "all_party",
  subject: "telepresence",
  universe: ["user_attestation", "signal_notice"],
});

// Why: the package is published under the scoped name (bare `r14n` is blocked by
// npm's similarity guard) and must carry zero runtime dependencies — the whole
// point of shipping the core as wasm rather than a hand-written port.
test("published metadata: scoped name, no runtime dependencies", () => {
  assert.equal(manifest.name, "@squillo/r14n");
  assert.equal(manifest.dependencies, undefined);
  assert.ok(manifest.files.includes("LICENSE"), "LICENSE ships in the tarball");
});

// Why: the happy path across the real wasm ABI — verdict, the sorted required
// set, and the map key as provenance source.
test("resolve returns the core decision through the wasm boundary", () => {
  const d = JSON.parse(r14n.resolve(packs, query));
  assert.equal(d.verdict, "permit");
  assert.deepEqual(d.required, ["user_attestation"]);
  assert.equal(d.provenance.source, "minimal/recording_consent.r14n.toml");
  assert.equal(d.provenance.fell_back, false);
  assert.match(d.disclaimer, /^NOT LEGAL ADVICE/);
});

// Why: fail-closed is the property the whole standard rests on (spec §4/§5) —
// a missing pack must demand the caller's full universe, loudly, never resolve
// empty and never throw.
test("a missing pack fails closed to the full universe", () => {
  const d = JSON.parse(r14n.resolve("{}", query));
  assert.equal(d.provenance.fell_back, true);
  assert.deepEqual(d.required, ["signal_notice", "user_attestation"]);
});

// Why: caller error must be a thrown Error, distinct from a pack-level
// fallback — a JS consumer has to be able to tell "I sent junk" from
// "no reviewed pack governed this".
test("malformed input throws rather than silently falling back", () => {
  assert.throws(() => r14n.resolve(packs, "{not json"), /query_json/);
});

// Why: the strictness dial must reach the core over the boundary.
test("the strictness override dial reaches the core", () => {
  const d = JSON.parse(r14n.resolve(packs, query, "aggressive"));
  assert.equal(d.provenance.strictness, "aggressive");
  assert.deepEqual(d.required, ["signal_notice", "user_attestation"]);
});

// Why: receipts are the auditable artifact (spec §8); both forms must serialize
// and stay advisory-only, since the reference resolver verifies no provenance.
test("resolve_with_receipts emits both receipt forms, advisory-only", () => {
  const ctx = JSON.stringify({
    record_id: "urn:test:1",
    issued_at: "2026-08-23T12:00:00Z",
    issued_at_unix: 1787832000,
    language: "en",
    pii_principal_id: "user-1",
    pii_controller: "Example Operator",
    domain: "recording_consent",
    subject: "telepresence",
    attribution_source: "operator_declared",
  });
  const out = JSON.parse(r14n.resolve_with_receipts(packs, query, ctx));
  assert.equal(out.decision.verdict, "permit");
  assert.equal(out.receipt_dpv27560.provenance.advisory_only, true);
  assert.equal(typeof out.receipt_kantara_cr_v1_1, "object");
});

// Why: the vocabulary IRI in receipts must match the live namespace host.
test("rlps_ns matches the published namespace", () => {
  assert.equal(r14n.rlps_ns(), "https://r14n.squillo.com/ns#");
});
