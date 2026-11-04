mod common;

use common::{err, ok};

fn value(raw: &str) -> tesserae::Config {
    ok(&format!("k = {raw}"))
}

fn value_err(raw: &str) -> String {
    err(&format!("k = {raw}"))
}

#[test]
fn ints() {
    for (raw, want) in [
        ("42", 42),
        ("0", 0),
        ("-7", -7),
        ("9223372036854775807", i64::MAX),
        ("-9223372036854775808", i64::MIN),
    ] {
        assert_eq!(value(raw).int("k"), Ok(want), "{raw}");
    }
}

#[test]
fn int_overflow_is_rejected() {
    assert_eq!(
        value_err("9223372036854775808"),
        "line 1: unknown value: '9223372036854775808'"
    );
}

#[test]
fn floats() {
    for (raw, want) in [
        ("67.6", 67.6),
        ("-1.5", -1.5),
        ("0.0", 0.0),
        ("1.5e3", 1500.0),
        ("1.5e+3", 1500.0),
    ] {
        assert_eq!(value(raw).float("k"), Ok(want), "{raw}");
    }
}

#[test]
fn infinite_floats_are_rejected() {
    let long = format!("1{}.0", "0".repeat(400));
    for raw in ["1.0e999", "-1.0e999", long.as_str()] {
        assert_eq!(
            value_err(raw),
            format!("line 1: float is out of range: '{raw}'")
        );
    }
}

#[test]
fn leading_plus_signs_are_rejected() {
    for raw in ["+5", "+0", "+1.5", "+1.0e999", "+5sec", "+4GiB", "+"] {
        assert_eq!(
            value_err(raw),
            format!("line 1: unknown value: '{raw}'"),
            "{raw}"
        );
    }
}

#[test]
fn bools() {
    assert_eq!(value("true").bool("k"), Ok(true));
    assert_eq!(value("false").bool("k"), Ok(false));
}

#[test]
fn strings() {
    for (raw, want) in [
        (r#""text""#, "text"),
        (r#""""#, ""),
        (r#""  spaced  ""#, "  spaced  "),
        (r#""héllo ✨""#, "héllo ✨"),
        (r#""a=b""#, "a=b"),
        (r#""a\"b\\c\{d\}""#, r#"a"b\c{d}"#),
    ] {
        assert_eq!(value(raw).str("k"), Ok(want), "{raw}");
    }
}

#[test]
fn malformed_strings_are_rejected() {
    for (raw, want) in [
        (r#""abc"#, r#"line 1: unterminated string: "abc"#),
        (r#"""#, r#"line 1: unterminated string: ""#),
        (r#""a\nb""#, r#"line 1: bad escape in "a\nb""#),
        (r#""abc\""#, r#"line 1: bad escape in "abc\""#),
        (r#""a{b""#, r#"line 1: unclosed { in "a{b""#),
        (r#""{}""#, r#"line 1: empty reference in "{}""#),
        (r#""a}""#, r#"line 1: unmatched } in "a}""#),
        (r#""a"b""#, r#"line 1: unescaped quote in "a"b""#),
    ] {
        assert_eq!(value_err(raw), want, "{raw}");
    }
}

#[test]
fn ports() {
    for (raw, want) in [(":0", 0), (":8080", 8080), (":65535", 65535)] {
        assert_eq!(value(raw).port("k"), Ok(want), "{raw}");
    }
}

#[test]
fn invalid_ports_are_rejected() {
    for num in ["", "abc", "-1", "+80", "65536", "80.5"] {
        assert_eq!(
            value_err(&format!(":{num}")),
            format!("line 1: invalid port: '{num}'")
        );
    }
}

#[test]
fn colours_are_normalised_to_uppercase() {
    for (raw, want) in [
        ("#FF6611", "#FF6611"),
        ("#ff6611", "#FF6611"),
        ("#Ff661180", "#FF661180"),
    ] {
        assert_eq!(value(raw).colour("k").as_deref(), Ok(want), "{raw}");
    }
}

#[test]
fn invalid_colours_are_rejected() {
    for raw in ["#", "#FFF", "#FF661", "#FF66110", "#FF6611000", "#GGGGGG"] {
        assert_eq!(value_err(raw), format!("line 1: invalid colour: '{raw}'"));
    }
}

#[test]
fn unknown_values_are_rejected() {
    for raw in [
        "", "abc", "True", "yes", "'single'", "1_000", "1.2.3", "5 sec", "5xyz", "-5sec", "-4GiB",
        "1.2.3sec", "sec", ".sec", "GiB", "4gib", "8b",
    ] {
        assert_eq!(
            value_err(raw),
            format!("line 1: unknown value: '{raw}'"),
            "{raw}"
        );
    }
}
