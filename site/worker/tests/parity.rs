// SPDX-License-Identifier: Apache-2.0
//! Parity tests: the Rust Worker's routing table against the recorded behaviour of the live
//! JavaScript Worker it replaces. Run by a bare `cargo test` -- the table names no host type.
//!
//! The acceptance bar is "a client cannot tell the two apart", with exactly two additive headers
//! allowed (tested on their own). The recorded matrix is `fixtures/live_parity.toml`; the tests
//! after it pin behaviour that matrix cannot reach (negotiation corners, conditional-request
//! quirks, HEAD, totality) and the one thing the JavaScript Worker did by hand -- its ETags.
//!
//! **NOT LEGAL ADVICE** -- see `docs/not-legal-advice.md`.
//!
//! Revision History
//! - 2026-11-02: authored.

/// The recorded live matrix; its header explains the format.
const FIXTURE: &str = ::std::include_str!("fixtures/live_parity.toml");

/// The response headers the parity audit compares.
const COMPARED: [&str; 6] =
  ["content-type", "cache-control", "etag", "vary", "allow", "access-control-allow-origin"];

/// The two headers allowed to differ from the live Worker, with their exact values. Written out
/// here, not read from the crate, so the test is an independent statement of the contract.
const ADDED: [(&str, &str); 2] =
  [("x-content-type-options", "nosniff"), ("x-squillo-impl", "rust-wasm-workers-rs")];

const METHODS: [&str; 11] =
  ["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "CONNECT", "TRACE", "REPORT", "PROPFIND"];

/// Every path the table serves.
const KNOWN_PATHS: [&str; 3] = ["/ns", "/ns/context.jsonld", "/ns/rlps.ttl"];

/// Near misses -- each differs from a served path by one character of slash, case or encoding --
/// plus the static tier's own paths, which the platform answers before this Worker runs and which
/// must therefore be a plain 404 here, never served from a second copy.
const UNKNOWN_PATHS: [&str; 10] = [
  "/ns/", "/assets/", "/nope", "/NS", "/ns/context.jsonld/", "/index.html", "//", "/n%73", "/",
  "/assets/r14n-banner.webp",
];

/// A repo-relative file read FRESH from disk -- never through the crate's own `include_bytes!` --
/// so a wrong embed path shows up as a mismatch instead of the embed being compared to itself.
fn repo_bytes(rel: &str) -> ::std::vec::Vec<u8> {
  let path = ::std::path::Path::new(::std::env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
  ::std::fs::read(&path).unwrap_or_else(|e| ::std::panic!("read {}: {e}", path.display()))
}

/// Full lowercase hex sha256.
fn sha256_hex(bytes: &[u8]) -> ::std::string::String {
  <::sha2::Sha256 as ::sha2::Digest>::digest(bytes)
    .iter()
    .map(|b| ::std::format!("{b:02x}"))
    .collect()
}

/// The ETag format the live Worker serves: `"` + first 32 hex of the sha256 + `"`.
fn etag_of(bytes: &[u8]) -> ::std::string::String {
  ::std::format!("\"{}\"", &sha256_hex(bytes)[..32])
}

fn request<'a>(method: &'a str, path: &'a str) -> ::r14n_ns::Req<'a> {
  ::r14n_ns::Req { method, path, accept: "", if_none_match: ::std::option::Option::None }
}

/// The value of the first header called `name`.
fn header(resp: &::r14n_ns::Resp, name: &str) -> ::std::option::Option<&'static str> {
  resp.headers.iter().find(|h| h.0 == name).map(|h| h.1)
}

/// How many headers called `name` the response carries.
fn count(resp: &::r14n_ns::Resp, name: &str) -> usize {
  resp.headers.iter().filter(|h| h.0 == name).count()
}

fn fixture() -> ::toml::Value {
  ::toml::from_str(FIXTURE).expect("fixtures/live_parity.toml parses")
}

fn cases(doc: &::toml::Value) -> &[::toml::Value] {
  doc.get("case").and_then(::toml::Value::as_array).expect("the fixture has [[case]] tables")
}

fn text<'a>(case: &'a ::toml::Value, key: &str) -> ::std::option::Option<&'a str> {
  case.get(key).and_then(::toml::Value::as_str)
}

fn case_name(case: &::toml::Value) -> &str {
  text(case, "name").expect("every case has a name")
}

/// The request a case records.
fn req_of(case: &::toml::Value) -> ::r14n_ns::Req<'_> {
  ::r14n_ns::Req {
    method: text(case, "method").expect("case.method"),
    path: text(case, "path").expect("case.path"),
    accept: text(case, "accept").unwrap_or(""),
    if_none_match: text(case, "if_none_match"),
  }
}

/// The body a case requires, as the bytes of the repository file (or literal) it names; `None`
/// means "no body at all".
fn recorded_body(case: &::toml::Value) -> ::std::option::Option<::std::vec::Vec<u8>> {
  match text(case, "body").expect("case.body") {
    "none" => ::std::option::Option::None,
    "jsonld" => ::std::option::Option::Some(repo_bytes("ns/context.jsonld")),
    "ttl" => ::std::option::Option::Some(repo_bytes("ns/rlps.ttl")),
    "text" => ::std::option::Option::Some(
      text(case, "body_text").expect("case.body_text").as_bytes().to_vec(),
    ),
    other => ::std::panic!("{}: unknown body kind {other:?}", case_name(case)),
  }
}

/// Why: this is the acceptance test. The live Worker's recorded answers -- status, the six compared
/// headers (and no others), and the body bytes -- are the contract the cutover must not change; a
/// header present here but absent live (or the reverse) is a visible difference, so the header
/// set is compared whole. The recorded digests make the literal 404 and 405 texts exact (they are
/// lowercase live, and a capitalised one fails here); equality with the repository file does the
/// same for the vocabulary bytes.
#[test]
fn route_matches_the_recorded_live_matrix() {
  let doc = fixture();
  for case in cases(&doc) {
    let label = case_name(case);
    let resp = ::r14n_ns::route(&req_of(case));

    ::std::assert_eq!(
      ::std::option::Option::Some(i64::from(resp.status)),
      case.get("status").and_then(::toml::Value::as_integer),
      "{label}: status"
    );

    let got: ::std::collections::BTreeMap<&str, &str> = resp
      .headers
      .iter()
      .copied()
      .filter(|(name, _)| !ADDED.iter().any(|(added, _)| added == name))
      .collect();
    ::std::assert_eq!(
      resp.headers.len(),
      got.len() + ADDED.len(),
      "{label}: a header name repeats, or an additive header is missing"
    );
    let want: ::std::collections::BTreeMap<&str, &str> = case
      .get("headers")
      .and_then(::toml::Value::as_table)
      .expect("case.headers")
      .iter()
      .map(|(k, v)| (k.as_str(), v.as_str().expect("header values are strings")))
      .collect();
    ::std::assert!(
      want.keys().all(|name| COMPARED.contains(name)),
      "{label}: the fixture names a header outside the compared set"
    );
    ::std::assert_eq!(got, want, "{label}: headers");

    let served = resp.body.bytes();
    ::std::assert!(
      served.map(<[u8]>::to_vec) == recorded_body(case),
      "{label}: body differs from the repository file it names (None means NO body)"
    );
    // Length and digest pin only the LITERAL texts (404, 405): their bytes exist nowhere else, so a
    // transcription slip in `body_text` would otherwise go unseen. A vocabulary or page body is
    // pinned by equality with the repository file above, so an intentional edit to `ns/` or the
    // splash never fails against a snapshot of the old live bytes.
    if text(case, "body") == ::std::option::Option::Some("text") {
      let bytes = served.unwrap_or(&[]);
      if let ::std::option::Option::Some(len) =
        case.get("body_bytes").and_then(::toml::Value::as_integer)
      {
        ::std::assert_eq!(i64::try_from(bytes.len()).expect("fits"), len, "{label}: body length");
      }
      if let ::std::option::Option::Some(digest) = text(case, "body_sha256_16") {
        ::std::assert_eq!(&sha256_hex(bytes)[..16], digest, "{label}: body sha256");
      }
    }
  }
}

/// Why: a fixture that silently lost cases would let the parity test pass vacuously. Twenty live
/// requests were recorded; the four the static tier now answers moved to the cache-policy test, so
/// sixteen remain, and a duplicate name would hide one behind another.
#[test]
fn the_recorded_matrix_is_complete_and_unambiguous() {
  let doc = fixture();
  let names: ::std::collections::BTreeSet<&str> = cases(&doc).iter().map(case_name).collect();
  ::std::assert_eq!(cases(&doc).len(), 16, "replayed live cases");
  ::std::assert_eq!(names.len(), 16, "case names are unique");
}

/// Why: the two additive headers are the ONLY sanctioned difference from the live Worker, and they
/// must ride on every response -- 404, 405 and 304 included, where a hand-built early return
/// is most likely to skip them. `nosniff` stops a browser MIME-sniffing the served vocabulary into
/// something it is not; `x-squillo-impl` lets the parity audit tell old from new. Each must appear
/// exactly once with its exact value. The status check keeps the claim from being vacuous: the
/// matrix really does reach a 200, a 304, a 404 and a 405.
#[test]
fn every_response_carries_exactly_the_two_additive_headers() {
  let doc = fixture();
  let statuses: ::std::collections::BTreeSet<u16> = cases(&doc)
    .iter()
    .map(|case| {
      let resp = ::r14n_ns::route(&req_of(case));
      for (name, value) in ADDED {
        ::std::assert_eq!(count(&resp, name), 1, "{}: {name} count", case_name(case));
        ::std::assert_eq!(header(&resp, name), ::std::option::Option::Some(value));
      }
      resp.status
    })
    .collect();
  ::std::assert_eq!(statuses.into_iter().collect::<::std::vec::Vec<_>>(), [200u16, 304, 404, 405]);
}

/// Why: the JavaScript Worker pinned its two ETags by hand and a README sentence was the only thing
/// keeping them honest. They are now derived at build time; this recomputes each from the file on
/// disk, independently of build.rs, so a wrong constant (stale, swapped between the two files, or
/// hashing the wrong path) fails here instead of shipping. The one-bit tamper proves the oracle is
/// sensitive -- otherwise the equality could hold for any constant -- and the final inequality
/// catches both representations being given the same tag.
#[test]
fn etags_are_the_sha256_prefix_of_the_bytes_actually_served() {
  let tags: ::std::vec::Vec<&str> =
    [("/ns/context.jsonld", "ns/context.jsonld"), ("/ns/rlps.ttl", "ns/rlps.ttl")]
      .iter()
      .map(|(path, rel)| {
        let resp = ::r14n_ns::route(&request("GET", path));
        let on_disk = repo_bytes(rel);
        let tag = header(&resp, "etag").expect("a vocabulary response carries an etag");
        ::std::assert!(
          resp.body.bytes() == ::std::option::Option::Some(on_disk.as_slice()),
          "{path}: served bytes are not the repository file"
        );
        ::std::assert_eq!(tag, etag_of(&on_disk), "{path}: etag is not the digest of the bytes");
        let mut tampered = on_disk.clone();
        tampered[0] ^= 1;
        ::std::assert_ne!(tag, etag_of(&tampered), "{path}: the oracle ignores the content");
        ::std::assert!(
          tag.len() == 34
            && tag.starts_with('"')
            && tag.ends_with('"')
            && tag[1..33].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
          "{path}: etag {tag} is not a quoted 32-char lowercase hex string"
        );
        tag
      })
      .collect();
  ::std::assert_ne!(tags[0], tags[1], "both representations share one etag");
}

/// Why: totality. Every method against every near-miss path must come back with a defined status
/// and never panic: 405 (with `allow: GET, HEAD`) for anything but GET and HEAD -- checked before
/// the path, so a bad verb on a bad path is still 405 -- else 200 for a served path, else 404.
/// `PROPFIND` is the unknown verb that a method enum would have turned into GET and served.
#[test]
fn every_method_and_path_gets_a_defined_answer() {
  for method in METHODS {
    for path in KNOWN_PATHS.iter().chain(UNKNOWN_PATHS.iter()) {
      let resp = ::r14n_ns::route(&request(method, path));
      let served = ::std::matches!(method, "GET" | "HEAD");
      let want = if !served {
        405
      } else if KNOWN_PATHS.contains(path) {
        200
      } else {
        404
      };
      ::std::assert_eq!(resp.status, want, "{method} {path}");
      ::std::assert_eq!(
        header(&resp, "allow"),
        if served { ::std::option::Option::None } else { ::std::option::Option::Some("GET, HEAD") },
        "{method} {path}: allow"
      );
    }
  }
}

/// Why: HEAD must be exactly GET minus the body -- same status, same headers -- on every route
/// and for every conditional/negotiated variant, including the 404, which carries a body in the
/// GET case. A HEAD that carried bytes (or a different status) is wrong on the wire. The GET-side
/// assertion keeps the test honest: a GET that is not a 304 does have a body to strip.
#[test]
fn head_is_get_without_a_body() {
  let json_tag = header(&::r14n_ns::route(&request("GET", "/ns")), "etag").expect("etag");
  for path in KNOWN_PATHS.iter().chain(UNKNOWN_PATHS.iter()) {
    for accept in ["", "text/turtle"] {
      for if_none_match in [::std::option::Option::None, ::std::option::Option::Some(json_tag)] {
        let get_req = ::r14n_ns::Req { accept, if_none_match, ..request("GET", path) };
        let get = ::r14n_ns::route(&get_req);
        let head = ::r14n_ns::route(&::r14n_ns::Req { method: "HEAD", ..get_req });
        ::std::assert_eq!(head.status, get.status, "HEAD {path}: status");
        ::std::assert_eq!(head.headers, get.headers, "HEAD {path}: headers");
        ::std::assert!(head.body.bytes().is_none(), "HEAD {path} carried a body");
        ::std::assert!(
          get.status == 304 || get.body.bytes().is_some(),
          "GET {path}: no body to strip"
        );
      }
    }
  }
}

/// Why: the JavaScript Worker validated with `if-none-match === etag` -- a strict string equality.
/// So a list, `*`, a weak (`W/`) validator, an unquoted or re-cased tag, or the OTHER
/// representation's tag all miss and get a full 200. RFC 9110 would match several of those; the
/// miss is the safe direction (a full response is never stale) and is kept for parity rather than
/// improved. The tag is per representation, so a Turtle request carrying the JSON-LD tag is
/// served the Turtle bytes with the Turtle tag. HEAD validates like GET.
#[test]
fn if_none_match_is_a_strict_string_equality_like_the_original() {
  let tag = header(&::r14n_ns::route(&request("GET", "/ns/context.jsonld")), "etag").expect("tag");
  let ttl_tag = header(&::r14n_ns::route(&request("GET", "/ns/rlps.ttl")), "etag").expect("tag");
  let on_ctx = |method: &str, accept: &str, inm: &str| {
    ::r14n_ns::route(&::r14n_ns::Req {
      method,
      accept,
      if_none_match: ::std::option::Option::Some(inm),
      ..request("GET", "/ns/context.jsonld")
    })
  };
  ::std::assert_eq!(on_ctx("GET", "", tag).status, 304, "the exact tag validates");
  ::std::assert_eq!(on_ctx("HEAD", "", tag).status, 304, "HEAD validates like GET");

  let weak = ::std::format!("W/{tag}");
  let list_last = ::std::format!("\"deadbeef\", {tag}");
  let list_first = ::std::format!("{tag}, \"deadbeef\"");
  let unquoted = tag.trim_matches('"').to_string();
  let upper = tag.to_uppercase();
  for (what, inm) in [
    ("weak validator", weak.as_str()),
    ("wildcard", "*"),
    ("list, match last", list_last.as_str()),
    ("list, match first", list_first.as_str()),
    ("unquoted", unquoted.as_str()),
    ("upper-cased", upper.as_str()),
    ("the other representation's tag", ttl_tag),
    ("empty", ""),
  ] {
    let resp = on_ctx("GET", "", inm);
    ::std::assert_eq!(resp.status, 200, "{what} must not validate");
    ::std::assert!(resp.body.bytes().is_some(), "{what}: a miss serves the full body");
  }

  let json_via_turtle = on_ctx("GET", "text/turtle", tag);
  ::std::assert_eq!(json_via_turtle.status, 304, "the direct URL ignores Accept");
  let negotiated = ::r14n_ns::route(&::r14n_ns::Req {
    accept: "text/turtle",
    if_none_match: ::std::option::Option::Some(tag),
    ..request("GET", "/ns")
  });
  ::std::assert_eq!(negotiated.status, 200, "a JSON-LD tag does not validate Turtle");
  ::std::assert_eq!(header(&negotiated, "etag"), ::std::option::Option::Some(ttl_tag));
}

/// Why: content negotiation on `/ns` is a plain case-sensitive substring test, not RFC 9110 Accept
/// parsing, and clients depend on its outcomes: Turtle only when `text/turtle` is named and
/// neither JSON flavour is; q-values and case are ignored; `*/*` and an empty header get JSON-LD.
/// Every negotiated answer says `Vary: Accept`.
#[test]
fn accept_negotiation_is_the_original_substring_rule() {
  let json_tag = header(&::r14n_ns::route(&request("GET", "/ns/context.jsonld")), "etag");
  let ttl_tag = header(&::r14n_ns::route(&request("GET", "/ns/rlps.ttl")), "etag");
  for (accept, turtle) in [
    ("", false),
    ("*/*", false),
    ("text/turtle", true),
    ("text/turtle, */*", true),
    ("application/json", false),
    ("text/turtle, application/json", false),
    ("application/ld+json, text/turtle", false),
    ("text/turtle;q=0.1, application/ld+json;q=0.9", false),
    ("TEXT/TURTLE", false),
  ] {
    let resp = ::r14n_ns::route(&::r14n_ns::Req { accept, ..request("GET", "/ns") });
    ::std::assert_eq!(
      header(&resp, "etag"),
      if turtle { ttl_tag } else { json_tag },
      "Accept: {accept:?}"
    );
    ::std::assert_eq!(header(&resp, "vary"), ::std::option::Option::Some("Accept"));
  }
}

/// Why: only `/ns` negotiates. The direct artifact URLs are one representation each, so they ignore
/// `Accept` entirely and send no `Vary` -- a cache keyed on them must not be split by a header
/// that cannot change the answer.
#[test]
fn the_direct_artifact_urls_ignore_accept_and_do_not_vary() {
  for (path, content_type) in [
    ("/ns/context.jsonld", "application/ld+json; charset=utf-8"),
    ("/ns/rlps.ttl", "text/turtle; charset=utf-8"),
  ] {
    for accept in ["", "text/turtle", "application/ld+json", "application/json"] {
      let resp = ::r14n_ns::route(&::r14n_ns::Req { accept, ..request("GET", path) });
      ::std::assert_eq!(header(&resp, "content-type"), ::std::option::Option::Some(content_type));
      ::std::assert_eq!(header(&resp, "vary"), ::std::option::Option::None, "{path} vs {accept:?}");
    }
  }
}

/// Why: the routing table must stay free of host types so it is testable natively and no
/// decision can hide in the Workers shim (the shim is a thin converter by design). A host type
/// leaking into it would need the WASM runtime to test, which is how logic ends up untested.
#[test]
fn the_routing_table_names_no_host_type() {
  let source = ::std::include_str!("../src/site.rs");
  for needle in ["worker::", "wasm_bindgen", "web_sys", "js_sys"] {
    ::std::assert!(!source.contains(needle), "src/site.rs mentions {needle}");
  }
}

/// `site/public/_headers` as path pattern -> (lowercase header name -> value). The file is read by
/// the platform, not the Worker, so this is the only place a Rust test can see it.
fn headers_file() -> ::std::collections::BTreeMap<
  ::std::string::String,
  ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
> {
  ::std::string::String::from_utf8(repo_bytes("site/public/_headers"))
    .expect("_headers is UTF-8")
    .lines()
    .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
    .fold((::std::collections::BTreeMap::new(), ::std::string::String::new()), |(mut rules, path), line| {
      if line.starts_with(char::is_whitespace) {
        let (name, value) = line.trim().split_once(':').expect("a header line is `Name: value`");
        rules
          .entry(path.clone())
          .or_insert_with(::std::collections::BTreeMap::new)
          .insert(name.trim().to_lowercase(), value.trim().to_string());
        (rules, path)
      } else {
        (rules, line.trim().to_string())
      }
    })
    .0
}

/// Why: the splash page and the banner moved out of the Worker into the static tier, so the cache
/// policy the Worker used to send -- ten minutes for `/`, immutable for the banner because its
/// filename changes with its content, and `Allow` on the platform's own 405 -- now lives in `site/public/_headers`, where the platform
/// reads it and nothing else would look. A dropped or mistyped rule would silently change what every
/// cache in front of the site does. Exactly three rules must exist (a stray one is a surprise), the
/// page must still point at the banner path the platform serves, and both files must exist there.
#[test]
fn the_static_tier_declares_the_cache_policy_the_worker_used_to_send() {
  let rules = headers_file();
  let value = |path: &str, name: &str| rules.get(path).and_then(|r| r.get(name)).map(String::as_str);
  ::std::assert_eq!(value("/", "cache-control"), ::std::option::Option::Some("public, max-age=600"));
  ::std::assert_eq!(
    value("/assets/*", "cache-control"),
    ::std::option::Option::Some("public, max-age=31536000, immutable")
  );
  ::std::assert_eq!(
    value("/*", "x-content-type-options"),
    ::std::option::Option::Some("nosniff")
  );
  ::std::assert_eq!(value("/*", "allow"), ::std::option::Option::Some("GET, HEAD"));
  ::std::assert_eq!(rules.keys().map(String::as_str).collect::<::std::vec::Vec<_>>(), ["/", "/*", "/assets/*"]);

  let html = ::std::string::String::from_utf8(repo_bytes("site/public/index.html")).expect("UTF-8");
  ::std::assert!(
    html.contains("src=\"/assets/r14n-banner.webp\""),
    "the page does not reference the banner at the path the static tier serves"
  );
  ::std::assert!(!repo_bytes("site/public/assets/r14n-banner.webp").is_empty(), "the banner is empty");
}
