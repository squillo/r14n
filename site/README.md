# site/ — r14n.squillo.com

The Cloudflare Worker behind `r14n.squillo.com`: the RLPS splash page at `/` and the vocabulary
namespace host at `/ns` (content-negotiated JSON-LD / Turtle, plus the direct artifacts
`/ns/context.jsonld` and `/ns/rlps.ttl`). **NOT LEGAL ADVICE** — see `/docs/not-legal-advice.md`.

The vocabulary is imported **directly from [`/ns`](../ns)** at bundle time — this directory holds
no copy, so the served bytes match the repo's source of record by construction. If `ns/` changes,
update the two ETag constants in `src/index.js` (first 32 hex chars of each file's sha256) and
redeploy.

Deploy (requires Cloudflare access to the squillo.com zone):

```console
$ cd site && wrangler deploy
```
