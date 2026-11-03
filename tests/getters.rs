mod common;

use std::time::Duration;

use common::ok;
use tesserae::Config;

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
fn oversized_duration_errors_on_access_not_load() {
    let config = ok(&format!("\nk = 1{}yr", "0".repeat(30)));
    assert_eq!(
        config.duration("k"),
        Err("line 2: duration is out of range: 'k'".to_string())
    );
}

#[test]
fn sizes_are_returned_in_bytes() {
    const KI: u64 = 1024;
    for (raw, bytes) in [
        ("0b", 0),
        ("8b", 1),
        ("0B", 0),
        ("1B", 1),
        ("1kB", 1000),
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
    for raw in ["1b", "1.5b", "0.5B", "0.1B"] {
        assert_eq!(
            ok(&format!("k = {raw}")).size("k"),
            Err("line 1: 'k' isn't a whole number of bytes that fits in u64".to_string()),
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
        ok("k = 16777216TiB").size("k"),
        Err("line 1: 'k' isn't a whole number of bytes that fits in u64".to_string())
    );
}

#[test]
fn missing_key_is_an_error() {
    assert_eq!(
        ok("a = 1").int("b"),
        Err("no key 'b' in config".to_string())
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

    type Getter = fn(&Config, &str) -> Result<(), String>;
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
            assert_eq!(get(&config, key), expected, "{want} getter on {key}");
        }
    }
}
