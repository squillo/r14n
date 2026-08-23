// SPDX-License-Identifier: Apache-2.0
// Builds the publishable @squillo/r14n npm package from the Rust resolver.
//
// wasm-pack emits a package.json whose `name` is the CRATE name (r14n-wasm) and
// carries none of the registry metadata we publish under, so this script runs
// the build and then rewrites exactly those fields — the generated glue, the
// .wasm, and the .d.ts are wasm-pack's output, untouched. Zero npm deps on
// purpose: the published package has no runtime dependencies, and neither does
// the thing that builds it.
//
// Usage: node bindings/npm/build.mjs [--out <dir>]
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, copyFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const outFlag = process.argv.indexOf("--out");
const outDir = outFlag === -1 ? resolve(repoRoot, "dist/npm") : resolve(process.argv[outFlag + 1]);

// The package identity we publish under. The bare name `r14n` is permanently
// unavailable on npm (its similarity guard rejects it against i18n/y18n), so
// the scoped name is the only option — see docs/preflight-name-check.md.
const PACKAGE_NAME = "@squillo/r14n";

execFileSync(
  "wasm-pack",
  ["build", "--target", "nodejs", "--out-dir", outDir, "--out-name", "r14n"],
  { cwd: resolve(repoRoot, "bindings/wasm"), stdio: "inherit" },
);

const manifestPath = resolve(outDir, "package.json");
const generated = JSON.parse(readFileSync(manifestPath, "utf8"));

writeFileSync(
  manifestPath,
  `${JSON.stringify(
    {
      ...generated,
      name: PACKAGE_NAME,
      description:
        "RLPS reference resolver for JS/TS — WebAssembly bindings generated from the Rust core. Fail-closed regulatory-control resolution with legal-review provenance. NOT LEGAL ADVICE.",
      homepage: "https://r14n.squillo.com",
      keywords: ["compliance", "regulatory", "policy-as-code", "wasm", "rlps", "r14n"],
      // wasm-pack lists only the artifacts it generated; the docs and license
      // we add below have to be declared or npm would drop them from the tarball.
      files: [...generated.files, "README.md", "LICENSE"],
    },
    null,
    2,
  )}\n`,
);

copyFileSync(resolve(repoRoot, "bindings/npm/README.md"), resolve(outDir, "README.md"));
copyFileSync(resolve(repoRoot, "LICENSE-APACHE"), resolve(outDir, "LICENSE"));

console.log(`\n${PACKAGE_NAME}@${generated.version} staged at ${outDir}`);
