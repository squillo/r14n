// SPDX-License-Identifier: Apache-2.0
//! # r14n.squillo.com -- the RLPS splash page and the `/ns` vocabulary host, in Rust
//!
//! Replaces `site/src/index.js`. All routing lives in [`site`] as a pure function over plain data;
//! this file is the Workers shim and does one job: turn the platform's request into a [`Req`],
//! call [`route`], and turn the [`Resp`] back into a platform response. There is no decision in it
//! to get wrong, which is why the tests drive [`route`] directly.
//!
//! **NOT LEGAL ADVICE** -- see `docs/not-legal-advice.md`.
//!
//! Revision History
//! - 2026-11-02: authored -- Rust port of the JavaScript Worker, behind a parity test of the
//!   recorded live behaviour.

#[forbid(unsafe_code)]
pub mod site;

pub use crate::site::{route, Body, Req, Resp};

#[::worker::event(fetch)]
async fn fetch(
  req: ::worker::Request,
  _env: ::worker::Env,
  _ctx: ::worker::Context,
) -> ::worker::Result<::worker::Response> {
  let accept = req.headers().get("accept")?.unwrap_or_default();
  let if_none_match = req.headers().get("if-none-match")?;
  // The raw method string, not `req.method()`: that enum maps every verb it does not recognise to
  // GET, which would turn the JavaScript Worker's 405 for such a verb into a served 200.
  let method = req.inner().method();
  let path = req.path();
  let resp = route(&Req {
    method: &method,
    path: &path,
    accept: &accept,
    if_none_match: if_none_match.as_deref(),
  });
  let builder = resp.headers.iter().try_fold(
    ::worker::ResponseBuilder::new().with_status(resp.status),
    |builder, &(name, value)| builder.with_header(name, value),
  )?;
  ::std::result::Result::Ok(match resp.body.bytes() {
    ::std::option::Option::Some(bytes) => builder.fixed(bytes.to_vec()),
    ::std::option::Option::None => builder.empty(),
  })
}
