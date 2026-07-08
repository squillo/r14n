// SPDX-License-Identifier: Apache-2.0
//! JSON Canonicalization Scheme (RFC 8785) — the deterministic byte form the
//! steward signs a reviewer-key directory over (spec §6 / council-audit D1).
//!
//! Correct for the data RLPS directories carry: strings, integers, booleans,
//! null, arrays, and objects (sorted keys). RFC 8785's full number
//! canonicalization (ECMAScript `Number.prototype.toString` for non-integer
//! floats) is NOT implemented — directories contain no floats, and a non-integer
//! number here is rejected rather than mis-canonicalized (so we never silently
//! produce a non-8785 form).
//!
//! Revision History
//! - 2026-07-07: authored — council-audit D1 (steward directory signing).

/// Canonicalize a JSON value to its RFC-8785 byte form (for the data classes
/// listed in the module doc). Returns Err on a non-integer number (unsupported).
pub fn canonicalize(value: &::serde_json::Value) -> ::std::result::Result<::std::string::String, ::std::string::String> {
  let mut out = ::std::string::String::new();
  write_value(value, &mut out)?;
  ::std::result::Result::Ok(out)
}

fn write_value(
  value: &::serde_json::Value,
  out: &mut ::std::string::String,
) -> ::std::result::Result<(), ::std::string::String> {
  match value {
    ::serde_json::Value::Null => out.push_str("null"),
    ::serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
    ::serde_json::Value::Number(n) => {
      if let ::std::option::Option::Some(i) = n.as_i64() {
        out.push_str(&i.to_string());
      } else if let ::std::option::Option::Some(u) = n.as_u64() {
        out.push_str(&u.to_string());
      } else {
        return ::std::result::Result::Err(::std::format!(
          "non-integer number {n} — RFC-8785 float canonicalization is not implemented (RLPS directories carry no floats)"
        ));
      }
    }
    ::serde_json::Value::String(s) => write_string(s, out),
    ::serde_json::Value::Array(arr) => {
      out.push('[');
      for (i, v) in arr.iter().enumerate() {
        if i > 0 {
          out.push(',');
        }
        write_value(v, out)?;
      }
      out.push(']');
    }
    ::serde_json::Value::Object(map) => {
      // RFC 8785: object members sorted by key (UTF-16 code units). Our keys are
      // ASCII, so byte order == UTF-16 order; a BTreeMap sorts them.
      let sorted: ::std::collections::BTreeMap<&::std::string::String, &::serde_json::Value> =
        map.iter().collect();
      out.push('{');
      for (i, (k, v)) in sorted.iter().enumerate() {
        if i > 0 {
          out.push(',');
        }
        write_string(k, out);
        out.push(':');
        write_value(v, out)?;
      }
      out.push('}');
    }
  }
  ::std::result::Result::Ok(())
}

/// RFC 8785 §3.2.2.2 string escaping: minimal escapes, lowercase `\uXXXX` for
/// control chars, otherwise the raw UTF-8 character.
fn write_string(s: &str, out: &mut ::std::string::String) {
  out.push('"');
  for c in s.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\u{08}' => out.push_str("\\b"),
      '\u{0C}' => out.push_str("\\f"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if (c as u32) < 0x20 => out.push_str(&::std::format!("\\u{:04x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
}

#[cfg(test)]
mod tests {
  /// Why: the steward signature is over the canonical bytes; if key ordering or
  /// whitespace were not normalized, two byte-equal directories could produce
  /// different signatures (or a re-serialization would break a valid one). Pins
  /// sorted keys + no insignificant whitespace against the RFC-8785 shape.
  #[test]
  fn sorts_keys_and_strips_whitespace() {
    let v: ::serde_json::Value =
      ::serde_json::from_str("  { \"b\": 1 , \"a\": [ 2, 3 ] , \"c\": \"x\" }  ").expect("json");
    ::std::assert_eq!(super::canonicalize(&v).expect("jcs"), "{\"a\":[2,3],\"b\":1,\"c\":\"x\"}");
  }

  /// Why: canonicalization must be idempotent + insensitive to input key order —
  /// two directories that differ only in serialization must sign identically.
  #[test]
  fn is_order_insensitive_and_idempotent() {
    let a: ::serde_json::Value = ::serde_json::from_str("{\"y\":true,\"x\":null}").expect("a");
    let b: ::serde_json::Value = ::serde_json::from_str("{\"x\":null,\"y\":true}").expect("b");
    let ca = super::canonicalize(&a).expect("ca");
    ::std::assert_eq!(ca, super::canonicalize(&b).expect("cb"));
    let reparsed: ::serde_json::Value = ::serde_json::from_str(&ca).expect("reparse");
    ::std::assert_eq!(ca, super::canonicalize(&reparsed).expect("idempotent"));
  }

  /// Why: control-character escaping is where naive canonicalizers diverge from
  /// RFC 8785; a wrong escape would make our signatures non-interoperable.
  #[test]
  fn escapes_controls_per_rfc8785() {
    let v = ::serde_json::Value::String(::std::string::String::from("a\tb\nc\u{01}"));
    ::std::assert_eq!(super::canonicalize(&v).expect("jcs"), "\"a\\tb\\nc\\u0001\"");
  }

  /// Why: silently mis-canonicalizing a float would produce a non-8785 signature
  /// that other implementations reject — better to refuse than to lie.
  #[test]
  fn rejects_non_integer_numbers() {
    let v: ::serde_json::Value = ::serde_json::from_str("{\"n\": 1.5}").expect("json");
    ::std::assert!(super::canonicalize(&v).is_err());
  }
}
