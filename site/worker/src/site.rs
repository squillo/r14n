// SPDX-License-Identifier: Apache-2.0
//! # The routing table -- plain data in, plain data out
//!
//! Everything the r14n.squillo.com WORKER decides lives here: which `/ns` path serves what, which
//! representation a `GET /ns` negotiates, when a conditional request earns a 304. The splash page and
//! its banner are not decided here at all -- they are the static tier (`site/public`), served by the
//! platform in front of this Worker, so a request for them never reaches it.
//!
//! A request arrives as four plain strings ([`Req`]) and leaves as a status, headers and a body
//! ([`Resp`]). This module names no host type, so it runs under a bare `cargo test`, and the
//! Workers shim in `lib.rs` is left with nothing to do but convert at the edges.
//!
//! For the `/ns` routes it is a behaviour-for-behaviour port of the JavaScript Worker it replaces,
//! including its quirks: parity is the acceptance test, and a "fix" here is a divergence a client
//! could see. Exactly two things are new -- the headers in `ADDED` -- and they ride on every
//! response this Worker sends.
//!
//! **NOT LEGAL ADVICE** -- this host serves a data-interchange vocabulary and a splash page; nothing
//! it returns states what the law is.
//!
//! Revision History
//! - 2026-11-02: authored -- the JavaScript Worker's `fetch`, plus `x-content-type-options` and
//!   `x-squillo-impl` on every response.

// The vocabulary is embedded STRAIGHT from its source of record in the repository, so the served
// bytes cannot drift from it. Paths are relative to this file.
const CONTEXT_JSONLD: &[u8] = ::std::include_bytes!("../../../ns/context.jsonld");
const RLPS_TTL: &[u8] = ::std::include_bytes!("../../../ns/rlps.ttl");

// `ETAG_JSONLD` and `ETAG_TTL`, written by build.rs from the two vocabulary files above.
::std::include!(::std::concat!(::std::env!("OUT_DIR"), "/etags.rs"));

/// The only two headers this Worker adds to what the JavaScript one sent, on EVERY response
/// (404, 405 and 304 included). `nosniff` stops a browser MIME-sniffing the served vocabulary into
/// something it is not; the implementation tag lets a parity audit tell the two Workers apart.
const ADDED: [(&str, &str); 2] =
  [("x-content-type-options", "nosniff"), ("x-squillo-impl", "rust-wasm-workers-rs")];

/// The content type the Workers runtime gives a plain string body. Set explicitly here so the
/// routing table, not the host, is what fixes it.
const PLAIN: &str = "text/plain;charset=UTF-8";

/// A request reduced to the four things the routing table reads.
#[derive(Clone, Copy, Debug)]
pub struct Req<'a> {
  /// The method exactly as it arrived. A string, not an enum, on purpose: a verb this table has
  /// never heard of must land in the 405 arm, not be coerced into one that is served.
  pub method: &'a str,
  /// The URL pathname -- no query, no fragment. Compared verbatim, so `/ns/` is not `/ns`.
  pub path: &'a str,
  /// The raw `Accept` header; empty when the request sent none.
  pub accept: &'a str,
  /// The raw `If-None-Match` header; `None` when the request sent none.
  pub if_none_match: ::std::option::Option<&'a str>,
}

/// What a response carries as a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
  /// No body at all. Not the same as an empty one: a 304 must be bodiless, and the host rejects
  /// a body on it.
  Empty,
  /// Bytes embedded in the binary.
  Static(&'static [u8]),
  /// Text embedded in the binary.
  Text(&'static str),
}

impl Body {
  /// The bytes to put on the wire, or `None` for a response with no body at all.
  pub fn bytes(&self) -> ::std::option::Option<&'static [u8]> {
    match *self {
      Body::Empty => ::std::option::Option::None,
      Body::Static(bytes) => ::std::option::Option::Some(bytes),
      Body::Text(text) => ::std::option::Option::Some(text.as_bytes()),
    }
  }
}

/// A response as plain data. Header names are lowercase; every value is a compile-time constant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resp {
  pub status: u16,
  pub headers: ::std::vec::Vec<(&'static str, &'static str)>,
  pub body: Body,
}

/// One negotiable vocabulary representation.
struct Vocab {
  content_type: &'static str,
  etag: &'static str,
  body: &'static [u8],
}

const JSONLD: Vocab = Vocab {
  content_type: "application/ld+json; charset=utf-8",
  etag: ETAG_JSONLD,
  body: CONTEXT_JSONLD,
};
const TURTLE: Vocab =
  Vocab { content_type: "text/turtle; charset=utf-8", etag: ETAG_TTL, body: RLPS_TTL };

/// Route one request. Total: every input has a defined answer and nothing here can panic.
pub fn route(req: &Req) -> Resp {
  let Resp { status, headers, body } = dispatch(req);
  Resp {
    status,
    headers: headers.into_iter().chain(ADDED).collect(),
    // HEAD is answered once, here, for every route -- including the 404 -- rather than being
    // re-decided in each arm.
    body: if req.method == "HEAD" { Body::Empty } else { body },
  }
}

fn dispatch(req: &Req) -> Resp {
  // Method check first, before the path is looked at: `POST /anything` is a 405, never a 404.
  if !::std::matches!(req.method, "GET" | "HEAD") {
    return text(405, "method not allowed", &[("allow", "GET, HEAD")]);
  }
  match req.path {
    // Content negotiation; JSON-LD is the default. (A `#fragment` never reaches a server.)
    "/ns" => vocab(req, if wants_turtle(req.accept) { &TURTLE } else { &JSONLD }, true),
    "/ns/context.jsonld" => vocab(req, &JSONLD, false),
    "/ns/rlps.ttl" => vocab(req, &TURTLE, false),
    // `/` and `/assets/*` are the static tier: the platform answers them before this Worker runs, so
    // reaching here with one means the static tier is missing -- a plain 404, visible, not a guess.
    _ => text(404, "not found", &[]),
  }
}

/// Turtle only when the client names it and names neither JSON flavour. A plain substring test --
/// q-values and case are ignored -- exactly as the JavaScript Worker did it.
fn wants_turtle(accept: &str) -> bool {
  accept.contains("text/turtle")
    && !accept.contains("application/ld+json")
    && !accept.contains("application/json")
}

/// A vocabulary representation. `vary` is set only where the URL itself negotiates (`/ns`); the
/// direct artifact URLs are one representation each. The tag is per representation, and the
/// comparison is a strict string equality: no lists, no `*`, no weak validators.
fn vocab(req: &Req, v: &Vocab, vary: bool) -> Resp {
  let headers: ::std::vec::Vec<(&'static str, &'static str)> = [
    ("content-type", v.content_type),
    ("access-control-allow-origin", "*"),
    ("cache-control", "public, max-age=3600"),
    ("etag", v.etag),
  ]
  .into_iter()
  .chain(vary.then_some(("vary", "Accept")))
  .collect();
  if req.if_none_match == ::std::option::Option::Some(v.etag) {
    Resp { status: 304, headers, body: Body::Empty }
  } else {
    Resp { status: 200, headers, body: Body::Static(v.body) }
  }
}

/// A plain-text error body, with any extra headers the status calls for.
fn text(status: u16, body: &'static str, extra: &[(&'static str, &'static str)]) -> Resp {
  Resp {
    status,
    headers: ::std::iter::once(("content-type", PLAIN)).chain(extra.iter().copied()).collect(),
    body: Body::Text(body),
  }
}
