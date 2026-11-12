mod common;

use std::time::Duration;

use common::{err, ok};
use tesserae::{Config, Resolved};

#[test]
fn get_returns_every_type_converted() {
    let config = ok(r#"
        bool = true
        int = 1
        float = 1.5
        str = "s"
        built = "{str}!"
        port = :80
        dur = 1.5min
        size = 1KiB
        colour = #ff6611
        list = [:80, :81]
        empty = []
    "#);

    for (key, want) in [
        ("bool", Resolved::Bool(true)),
        ("int", Resolved::Int(1)),
        ("float", Resolved::Float(1.5)),
        ("str", Resolved::Str("s".to_string())),
        ("built", Resolved::Str("s!".to_string())),
        ("port", Resolved::Port(80)),
        ("dur", Resolved::Duration(Duration::from_secs(90))),
        ("size", Resolved::Size(1024)),
        ("colour", Resolved::Colour("#FF6611".to_string())),
        (
            "list",
            Resolved::List(vec![Resolved::Port(80), Resolved::Port(81)]),
        ),
        ("empty", Resolved::List(Vec::new())),
    ] {
        assert_eq!(config.get(key), Ok(want), "{key}");
    }
}

#[test]
fn keys_are_sorted() {
    assert_eq!(ok("b = 1\nab = 2\na/c = 3\n").keys(), ["a/c", "ab", "b"]);
}

#[test]
fn getters_reject_missing_keys() {
    let config = ok("a = 1");
    let missing = "no key 'b' in config";
    assert_eq!(config.get("b").unwrap_err().to_string(), missing);
    assert_eq!(config.line("b").unwrap_err().to_string(), missing);
    assert_eq!(config.int("b").unwrap_err().to_string(), missing);
    assert_eq!(config.ints("b").unwrap_err().to_string(), missing);
}

#[test]
fn durations_convert_every_unit() {
    for (raw, secs) in [
        ("0sec", 0),
        ("30sec", 30),
        ("5min", 300),
        ("2hr", 7200),
        ("1d", 86400),
        ("1wk", 7 * 86400),
        ("1mo", 30 * 86400),
        ("1yr", 365 * 86400),
        ("1.5min", 90),
    ] {
        let config = ok(&format!("k = {raw}"));
        assert_eq!(config.duration("k"), Ok(Duration::from_secs(secs)), "{raw}");
    }
}

#[test]
fn durations_keep_subsecond_precision() {
    assert_eq!(
        ok("k = 0.1sec").duration("k"),
        Ok(Duration::from_millis(100))
    );
}

#[test]
fn oversized_duration_fails_at_load() {
    assert_eq!(
        err(&format!("\nk = 1{}yr", "0".repeat(30))),
        "line 2: duration is out of range: 'k'"
    );
}

#[test]
fn sizes_are_returned_in_bytes() {
    const KI: u64 = 1024;
    for (raw, bytes) in [
        ("0B", 0),
        ("1B", 1),
        ("1kB", 1000),
        ("1KB", 1000),
        ("1MB", 1_000_000),
        ("1GB", 1_000_000_000),
        ("1TB", 1_000_000_000_000),
        ("1KiB", KI),
        ("1MiB", KI.pow(2)),
        ("4GiB", 4 * KI.pow(3)),
        ("1TiB", KI.pow(4)),
        ("1.5KiB", 1536),
        ("0.1kB", 100),
    ] {
        let config = ok(&format!("k = {raw}"));
        assert_eq!(config.size("k"), Ok(bytes), "{raw}");
    }
}

#[test]
fn fractional_bytes_are_rejected() {
    for raw in ["0.5B", "0.1B"] {
        assert_eq!(
            err(&format!("k = {raw}")),
            "line 1: 'k' isn't a whole number of bytes that fits in u64",
            "{raw}"
        );
    }
}

#[test]
fn sizes_must_fit_in_u64() {
    assert_eq!(
        ok("k = 16777215TiB").size("k"),
        Ok(16_777_215 * (1u64 << 40))
    );
    assert_eq!(
        err("k = 16777216TiB"),
        "line 1: 'k' isn't a whole number of bytes that fits in u64"
    );
}

#[test]
fn every_getter_rejects_every_other_type() {
    let config = ok(r#"
        bool = true
        int = 1
        float = 1.5
        str = "s"
        port = :80
        dur = 1sec
        size = 1B
        colour = #FF6611
    "#);

    let keys = [
        ("bool", "a bool", 2),
        ("int", "an int", 3),
        ("float", "a float", 4),
        ("str", "a string", 5),
        ("port", "a port", 6),
        ("dur", "a duration", 7),
        ("size", "a size", 8),
        ("colour", "a colour", 9),
    ];

    type Getter = fn(&Config, &str) -> Result<(), tesserae::Error>;
    let getters: [(&str, Getter); 8] = [
        ("a bool", |c, k| c.bool(k).map(drop)),
        ("an int", |c, k| c.int(k).map(drop)),
        ("a float", |c, k| c.float(k).map(drop)),
        ("a string", |c, k| c.str(k).map(drop)),
        ("a port", |c, k| c.port(k).map(drop)),
        ("a duration", |c, k| c.duration(k).map(drop)),
        ("a size", |c, k| c.size(k).map(drop)),
        ("a colour", |c, k| c.colour(k).map(drop)),
    ];

    for (key, kind, line) in keys {
        for (want, get) in getters {
            let expected = if want == kind {
                Ok(())
            } else {
                Err(format!("line {line}: '{key}' is {kind}, not {want}"))
            };
            assert_eq!(
                get(&config, key).map_err(|e| e.to_string()),
                expected,
                "{want} getter on {key}"
            );
        }
    }
}
