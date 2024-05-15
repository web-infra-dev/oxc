//! Parsing utilities for converting Javascript numbers to Rust f64
//! code copied from [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/master/crates/parser/src/numeric_value.rs)

use num_bigint::BigInt;
use num_traits::Num as _;
use std::borrow::Cow;

use super::kind::Kind;

pub fn parse_int(s: &str, kind: Kind, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    parse_int_without_underscores(&s, kind)
}

pub fn parse_float(s: &str, has_sep: bool) -> Result<f64, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));

    parse_float_without_underscores(&s)
}

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_int_without_underscores(s: &str, kind: Kind) -> Result<f64, &'static str> {
    if kind == Kind::Decimal {
        return parse_float_without_underscores(s);
    }
    match kind {
        Kind::Binary => Ok(parse_binary(&s[2..])),
        Kind::Octal => {
            let s = if s.starts_with("0o") || s.starts_with("0O") {
                &s[2..]
            } else {
                s // legacy octal
            };
            Ok(parse_octal(s))
        }
        Kind::Hex => Ok(parse_hex(&s[2..])),
        _ => unreachable!(),
    }
}

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_float_without_underscores(s: &str) -> Result<f64, &'static str> {
    s.parse::<f64>().map_err(|_| "invalid float")
}

#[allow(clippy::cast_precision_loss,clippy::cast_possible_truncation)]
fn parse_binary(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_u64;

    for c in s.as_bytes() {
        debug_assert!(c != &b'_');
        #[allow(clippy::cast_lossless)]
        let value = (c - b'0') as u64;
        result <<= 1;
        result |= value;
    }

    result as f64
}

#[allow(clippy::cast_precision_loss)]
fn parse_octal(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_u64;

    for c in s.as_bytes() {
        debug_assert!(c != &b'_');
        #[allow(clippy::cast_lossless)]
        let value = (c - b'0') as u64;
        result <<= 3;
        result |= value;
    }

    result as f64
}

#[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
fn parse_hex(s: &str) -> f64 {
    debug_assert!(!s.is_empty());

    let mut result = 0_u64;

    for c in s.as_bytes() {
        debug_assert!(c != &b'_');
        let value = match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'F' => c - b'A' + 10,
            b'a'..=b'f' => c - b'a' + 10,
            _ => unreachable!("invalid hex syntax {}", s),
        };
        result <<= 4;
        result |= value as u64;
    }

    result as f64
}

pub fn parse_big_int(s: &str, kind: Kind, has_sep: bool) -> Result<BigInt, &'static str> {
    let s = if has_sep { Cow::Owned(s.replace('_', "")) } else { Cow::Borrowed(s) };
    debug_assert!(!s.contains('_'));
    parse_big_int_without_underscores(&s, kind)
}

/// This function assumes `s` has had all numeric separators (`_`) removed.
/// Parsing will fail if this assumption is violated.
fn parse_big_int_without_underscores(s: &str, kind: Kind) -> Result<BigInt, &'static str> {
    let s = match kind {
        Kind::Decimal => s,
        Kind::Binary | Kind::Octal | Kind::Hex => &s[2..],
        _ => unreachable!(),
    };
    let radix = match kind {
        Kind::Decimal => 10,
        Kind::Binary => 2,
        Kind::Octal => 8,
        Kind::Hex => 16,
        _ => unreachable!(),
    };
    // NOTE: BigInt::from_bytes does a utf8 check, then uses from_str_radix
    // under the hood. We already have a string, so we can just use that
    // directly.
    BigInt::from_str_radix(s, radix).map_err(|_| "invalid bigint")
}

#[cfg(test)]
#[allow(clippy::unreadable_literal,clippy::mixed_case_hex_literals)]
mod test {

    use super::{parse_float, parse_int, Kind};

    /// Assert that all (&str, number) items in an [`Iterator`] parse to the
    /// expected number. `number` will be casted to an [`f64`].
    macro_rules! assert_all_eq {
        // parser fn shorthands
        (float $test_cases:expr, $has_sep:expr) => {
            assert_all_eq!($test_cases, nokind, $has_sep, parser = parse_float);
        };
        (bigint $test_cases:expr, $kind:expr, $has_sep:expr) => {
            assert_all_eq!($test_cases, $kind, $has_sep, parser = parse_big_int);
        };
        (int $test_cases:expr, $kind:expr, $has_sep:expr) => {
            assert_all_eq!($test_cases, $kind, $has_sep, parser = parse_int);
        };

        // impl
        ($test_cases:expr, nokind, $has_sep:expr, parser = $parse_fn:tt) => {
            for (s, expected) in $test_cases.into_iter() {
                let parsed = $parse_fn(s, $has_sep);
                assert_eq!(
                    parsed,
                    Ok(expected as f64),
                    "expected {s} to parse to {expected}, but got {parsed:?}"
                );
            }
        };
        ($test_cases:expr, $kind:expr, $has_sep:expr, parser = $parse_fn:tt) => {
            for (s, expected) in $test_cases.into_iter() {
                let parsed = $parse_fn(s, $kind, $has_sep);
                assert_eq!(
                    parsed,
                    Ok(expected as f64),
                    "expected {s} to parse to {expected}, but got {parsed:?}"
                );
            }
        };
    }

    #[test]
    fn test_int_precision() {
        assert_eq!(parse_int("9007199254740991", Kind::Decimal, false), Ok(9007199254740991.0));
    }

    #[test]
    fn test_float_precision() {
        let cases = vec![
            ("1.7976931348623157e+308", 1.7976931348623157e+308),
            ("0.000_000_001", 0.000_000_001),
        ];
        assert_all_eq!(float cases, false);
        // assert_eq!(parse_float("1.7976931348623157e+308", false), Ok(1.7976931348623157e+308));
    }

    #[test]
    fn test_parse_int_no_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // normal
            ("0", 0),
            ("-0", 0),
            ("1", 1),
            ("-1", -1),
            ("000000000000", 0),
            ("-000000000000", 0),
            ("9007199254740991", 9007199254740991), // max safe integer, 2^53 - 1
            ("-9007199254740990", -9007199254740990), // min safe integer, -(2^53 - 1)
        ];
        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
        ];
        let octal = vec![("0o0", 0o0), ("0o1", 0o1), ("0o10", 0o10), ("0o777", 0o777)];
        let hex: Vec<(&str, i64)> = vec![
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xc", 0xc), // :)
            ("0xdeadbeef", 0xdeadbeef),
            ("0xFfEeDdCcBbAa", 0xFfEeDdCcBbAa),
        ];

        assert_all_eq!(int decimal, Kind::Decimal, false);
        assert_all_eq!(int binary, Kind::Binary, false);
        assert_all_eq!(int octal, Kind::Octal, false);
        assert_all_eq!(int hex, Kind::Hex, false);
    }

    #[test]
    fn test_parse_int_with_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0", 0),
            ("-0", 0),
            ("1", 1),
            ("-1", -1),
            ("1_000_000", 1_000_000),
            ("-1_000_000", -1_000_000),
            ("000000000000", 0),
            ("-000000000000", 0),
            ("9_007_199_254_740_991", 9_007_199_254_740_991), // max safe integer, 2^53 - 1
            ("-9_007_199_254_740_990", -9_007_199_254_740_990), // min safe integer, -(2^53 - 1)
            // still works for illegal tokens
            ("1___000_000", 1_000_000),
            ("1_", 1),
            ("_1", 1),
        ];

        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
            ("0b1_1000_1001_0001_00", 0b1_1000_1001_0001_00),
            // still works for illegal tokens
            ("0b1_0000__0000", 0b1_0000_0000),
            ("0b1_", 0b1),
            ("0b_0", 0b0),
        ];

        let octal = vec![
            ("0o0", 0o0),
            ("0o1", 0o1),
            ("0o10", 0o10),
            ("0o777", 0o777),
            ("0o7_7_7", 0o777),
            ("0o77_73_72", 0o77_73_72),
            // still works for illegal tokens
            ("0o1_0000__0000", 0o100_000_000),
            ("0o1_", 0o1),
            ("0o_0", 0o0),
        ];

        let hex: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xFF_AA_11", 0xFFAA11),
            ("0xdead_beef", 0xdead_beef),
            ("0xFf_Ee_Dd_Cc_Bb_Aa", 0xFfEe_DdCc_BbAa),
            ("0xFfEe_DdCc_BbAa", 0xFfEe_DdCc_BbAa),
            // still works for illegal tokens
            ("0x1_0000__0000", 0x100_000_000),
            ("0x1_", 0x1),
            ("0x_0", 0x0),
        ];

        assert_all_eq!(int decimal, Kind::Decimal, true);
        assert_all_eq!(int binary, Kind::Binary, true);
        assert_all_eq!(int octal, Kind::Octal, true);
        assert_all_eq!(int hex, Kind::Hex, true);
    }

    #[test]
    fn test_decimal() {
        let no_sep: Vec<(&'static str, f64)> =
            vec![("0", 0.0), ("1.0", 1.0), ("1.1", 1.1), ("25.125", 25.125)];

        let sep: Vec<(&'static str, f64)> = vec![
            ("1_000.0", 1000.0),
            ("1.5_000", 1.5),
            // works on invalid tokens
            ("_0._5", 0.5),
            ("0._5", 0.5),
            ("0.5_", 0.5),
        ];

        // parse_int() handles Kind::Decimal as a float. Should we check if
        // a '.' is encountered during lexing and pick which parser to use?
        assert_all_eq!(float no_sep.clone(), false);
        assert_all_eq!(float sep.clone(), true);
        assert_all_eq!(int no_sep, Kind::Decimal, false);
        assert_all_eq!(int sep, Kind::Decimal, true);
    }
}
