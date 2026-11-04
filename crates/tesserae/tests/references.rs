mod common;

use common::{err, err_env, ok, ok_env};

#[test]
fn whole_string_reference_keeps_the_target_type() {
    let config = ok(r#"
        port = :80
        int = 1
        dur = 30sec
        s = "text"
        a = "{port}"
        b = "{int}"
        c = "{dur}"
        d = "{s}"
    "#);
    assert_eq!(config.port("a"), Ok(80));
    assert_eq!(config.int("b"), Ok(1));
    assert_eq!(config.duration("c"), Ok(std::time::Duration::from_secs(30)));
    assert_eq!(config.str("d"), Ok("text"));
    assert_eq!(
        config.str("a"),
        Err("line 6: 'a' is a port, not a string".to_string())
    );
}

#[test]
fn interpolation_stringifies_every_type() {
    for (raw, want) in [
        ("true", "true"),
        ("42", "42"),
        ("1.5", "1.5"),
        ("2.0", "2.0"),
        (r#""text""#, "text"),
        (":8080", "8080"),
        ("30sec", "30sec"),
        ("1.5min", "1.5min"),
        ("4GiB", "4GiB"),
        ("#ff6611", "#FF6611"),
    ] {
        let config = ok(&format!("v = {raw}\nk = \"<{{v}}>\""));
        assert_eq!(config.str("k"), Ok(format!("<{want}>").as_str()), "{raw}");
    }
}

#[test]
fn multiple_references_in_one_string() {
    let config = ok(r#"
        proc = "http"
        host = "localhost"
        port = :80
        url = "{proc}://{host}:{port}/api"
        joined = "{proc}{host}"
    "#);
    assert_eq!(config.str("url"), Ok("http://localhost:80/api"));
    assert_eq!(config.str("joined"), Ok("httplocalhost"));
}

#[test]
fn references_chain() {
    let config = ok(r#"
        a = "{b}"
        b = "x{c}"
        c = "{d}"
        d = 1
    "#);
    assert_eq!(config.str("a"), Ok("x1"));
    assert_eq!(config.str("b"), Ok("x1"));
    assert_eq!(config.int("c"), Ok(1));
}

#[test]
fn references_are_always_from_root() {
    let config = ok(r#"
        host = "root"
        :: db
        host = "db"
        a = "{host}"
        b = "{db/host}"
    "#);
    assert_eq!(config.str("db/a"), Ok("root"));
    assert_eq!(config.str("db/b"), Ok("db"));
}

#[test]
fn escaped_braces_are_literal() {
    let config = ok(r#"a = 1
        k = "\{a\} {a}"
        only = "\{a\}"
    "#);
    assert_eq!(config.str("k"), Ok("{a} 1"));
    assert_eq!(config.str("only"), Ok("{a}"));
}

#[test]
fn reusing_a_key_twice_is_not_a_cycle() {
    let config = ok(r#"
        a = "{b}{b}{c}"
        b = "{c}"
        c = "x"
    "#);
    assert_eq!(config.str("a"), Ok("xxx"));
}

#[test]
fn missing_references_are_rejected() {
    assert_eq!(
        err("a = 1\nb = \"{nope}\""),
        "line 2: reference to 'nope', which doesn't exist"
    );
    assert_eq!(
        err("b = \"x{nope}\""),
        "line 1: reference to 'nope', which doesn't exist"
    );
    assert_eq!(
        err("c = \"{ a }\"\na = 1"),
        "line 1: reference to ' a ', which doesn't exist"
    );
}

#[test]
fn cycles_are_rejected() {
    assert_eq!(err(r#"a = "{a}""#), "line 1: reference cycle: a -> a");
    assert_eq!(
        err("a = \"{b}\"\nb = \"{a}\""),
        "line 2: reference cycle: a -> b -> a"
    );
    assert_eq!(
        err("a = \"x{b}\"\nb = \"y{c}\"\nc = \"z{a}\""),
        "line 3: reference cycle: a -> b -> c -> a"
    );
}

#[test]
fn cycle_message_only_lists_the_cycle_itself() {
    assert_eq!(
        err("a = \"{b}\"\nb = \"{c}\"\nc = \"{b}\""),
        "line 3: reference cycle: b -> c -> b"
    );
}

#[test]
fn overrides_flow_through_references() {
    let src = r#"
        host = "localhost"
        url = "http://{host}/api"
        alias = "{host}"
        @prod
        host = "ypuf.xyz"
    "#;
    assert_eq!(ok(src).str("url"), Ok("http://localhost/api"));
    assert_eq!(ok_env(src, "prod").str("url"), Ok("http://ypuf.xyz/api"));
    assert_eq!(ok_env(src, "prod").str("alias"), Ok("ypuf.xyz"));
}

#[test]
fn a_tessera_can_replace_a_reference_or_a_value_with_a_reference() {
    let src = r#"
        a = "x"
        b = "{a}"
        c = "y"
        @e
        b = "literal"
        c = "{a}"
    "#;
    let config = ok_env(src, "e");
    assert_eq!(config.str("b"), Ok("literal"));
    assert_eq!(config.str("c"), Ok("x"));
}

#[test]
fn tessera_reference_errors_name_the_tessera() {
    let missing = "a = 1\n@prod\na = \"{nope}\"";
    assert_eq!(
        err(missing),
        "@prod: line 3: reference to 'nope', which doesn't exist"
    );

    let cycle = "a = 1\nb = 2\n@prod\na = \"{b}\"\nb = \"{a}\"";
    assert_eq!(
        err_env(cycle, "prod"),
        "@prod: line 5: reference cycle: a -> b -> a"
    );
}
