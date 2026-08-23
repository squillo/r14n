// SPDX-License-Identifier: Apache-2.0
// r14n.squillo.com — the RLPS splash page and the vocabulary namespace host
// (https://r14n.squillo.com/ns#). The vocabulary artifacts are imported STRAIGHT
// from /ns at bundle time — one source of truth, byte-identical by construction.
import INDEX_HTML from "./index.html";
import CONTEXT_JSONLD from "../../ns/context.jsonld";
import RLPS_TTL from "../../ns/rlps.ttl";

// ETags: first 32 hex chars of each artifact's sha256, pinned at deploy time.
const ETAG_JSONLD = '"f61fd666dc4b97d128122187215a4719"';
const ETAG_TTL = '"fc94b11047ca028515610517a4ea82ed"';

const CT_JSONLD = "application/ld+json; charset=utf-8";
const CT_TTL = "text/turtle; charset=utf-8";

function artifact(request, body, contentType, etag, vary) {
  const headers = {
    "content-type": contentType,
    "access-control-allow-origin": "*",
    "cache-control": "public, max-age=3600",
    etag,
  };
  if (vary) headers.vary = "Accept";
  if (request.headers.get("if-none-match") === etag) {
    return new Response(null, { status: 304, headers });
  }
  return new Response(request.method === "HEAD" ? null : body, { status: 200, headers });
}

export default {
  async fetch(request) {
    if (request.method !== "GET" && request.method !== "HEAD") {
      return new Response("method not allowed", {
        status: 405,
        headers: { allow: "GET, HEAD" },
      });
    }
    const { pathname } = new URL(request.url);

    if (pathname === "/ns") {
      // Content negotiation; JSON-LD is the default (fragments like /ns# never reach the server).
      const accept = request.headers.get("accept") || "";
      const wantsTurtle =
        accept.includes("text/turtle") &&
        !accept.includes("application/ld+json") &&
        !accept.includes("application/json");
      return wantsTurtle
        ? artifact(request, RLPS_TTL, CT_TTL, ETAG_TTL, true)
        : artifact(request, CONTEXT_JSONLD, CT_JSONLD, ETAG_JSONLD, true);
    }
    if (pathname === "/ns/context.jsonld") {
      return artifact(request, CONTEXT_JSONLD, CT_JSONLD, ETAG_JSONLD, false);
    }
    if (pathname === "/ns/rlps.ttl") {
      return artifact(request, RLPS_TTL, CT_TTL, ETAG_TTL, false);
    }
    if (pathname === "/") {
      return new Response(request.method === "HEAD" ? null : INDEX_HTML, {
        status: 200,
        headers: {
          "content-type": "text/html; charset=utf-8",
          "cache-control": "public, max-age=600",
        },
      });
    }
    return new Response("not found", { status: 404 });
  },
};
