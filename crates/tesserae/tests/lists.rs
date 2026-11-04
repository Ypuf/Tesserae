mod common;

use std::time::Duration;

use common::{err, err_env, ok, ok_env};

#[test]
fn every_type_has_a_list_getter() {
    let config = ok(r#"
        bools = [true, false]
        ints = [1, -2]
        floats = [1.5, 2.5]
        strs = ["a", "b"]
        ports = [:80, :443]
        durs = [1sec, 1min]
        sizes = [1B, 1KiB]
        colours = [#ff6611, #000000]
    "#);

    assert_eq!(config.bools("bools"), Ok(vec![true, false]));
    assert_eq!(config.ints("ints"), Ok(vec![1, -2]));
    assert_eq!(config.floats("floats"), Ok(vec![1.5, 2.5]));
    assert_eq!(config.strs("strs"), Ok(vec!["a", "b"]));
    assert_eq!(config.ports("ports"), Ok(vec![80, 443]));
    assert_eq!(
        config.durations("durs"),
        Ok(vec![Duration::from_secs(1), Duration::from_secs(60)])
    );
    assert_eq!(config.sizes("sizes"), Ok(vec![1, 1024]));
    assert_eq!(
        config.colours("colours"),
        Ok(vec!["#FF6611".to_string(), "#000000".to_string()])
    );
}

#[test]
fn whitespace_inside_lists_is_optional() {
    for src in ["k = [1,2]", "k = [ 1 , 2 ]", "k=[1,   2]"] {
        assert_eq!(ok(src).ints("k"), Ok(vec![1, 2]), "{src}");
    }
}

#[test]
fn single_item_lists_work() {
    assert_eq!(ok("k = [1]").ints("k"), Ok(vec![1]));
}

#[test]
fn empty_lists_work_with_any_list_getter() {
    for src in ["k = []", "k = [ ]"] {
        let config = ok(src);
        assert_eq!(config.ints("k"), Ok(vec![]));
        assert_eq!(config.strs("k"), Ok(vec![]));
    }
}

#[test]
fn punctuation_inside_strings_does_not_split_items() {
    let config = ok(r#"k = ["a,b", "c]", "[d", "e\",f", "g // h"] // real comment"#);
    assert_eq!(
        config.strs("k"),
        Ok(vec!["a,b", "c]", "[d", "e\",f", "g // h"])
    );
}

#[test]
fn malformed_lists_are_rejected() {
    for (raw, want) in [
        ("[", "line 1: unterminated list: ["),
        ("[1, 2", "line 1: unterminated list: [1, 2"),
        ("[1, 2] 3", "line 1: unterminated list: [1, 2] 3"),
        ("[,]", "line 1: empty item in list: [,]"),
        ("[1,]", "line 1: empty item in list: [1,]"),
        ("[,1]", "line 1: empty item in list: [,1]"),
        ("[1,,2]", "line 1: empty item in list: [1,,2]"),
        ("[1 2]", "line 1: unknown value: '1 2'"),
        ("[1, 2]]", "line 1: unknown value: '2]'"),
        ("[nope]", "line 1: unknown value: 'nope'"),
        (r#"["a, 1]"#, r#"line 1: unterminated string: "a, 1"#),
    ] {
        assert_eq!(err(&format!("k = {raw}")), want, "{raw}");
    }
}

#[test]
fn lists_cant_be_nested() {
    for raw in ["[[1]]", "[1, [2]]", "[[1, 2], [3]]"] {
        assert_eq!(
            err(&format!("k = {raw}")),
            format!("line 1: lists can't be nested: {raw}")
        );
    }
    assert_eq!(
        err("a = [1]\nb = [\"{a}\"]"),
        "line 2: lists can't be nested"
    );
}

#[test]
fn lists_cant_mix_types() {
    assert_eq!(
        err("k = [1, \"a\"]"),
        "line 1: list mixes an int and a string"
    );
    assert_eq!(err("k = [1, 1.5]"), "line 1: list mixes an int and a float");
    assert_eq!(
        err("p = :80\nk = [1, \"{p}\"]"),
        "line 2: list mixes an int and a port"
    );
}

#[test]
fn items_can_use_references() {
    let config = ok(r#"
        host = "x"
        port = :80
        hosts = ["{host}", "{host}.com"]
        ports = ["{port}", :443]
    "#);
    assert_eq!(config.strs("hosts"), Ok(vec!["x", "x.com"]));
    assert_eq!(config.ports("ports"), Ok(vec![80, 443]));
}

#[test]
fn a_reference_can_point_at_a_list() {
    let config = ok("a = [1, 2]\nb = \"{a}\"");
    assert_eq!(config.ints("b"), Ok(vec![1, 2]));
}

#[test]
fn lists_cant_be_put_inside_strings() {
    assert_eq!(
        err("a = [1]\nb = \"x{a}\""),
        "line 2: can't put list 'a' inside a string"
    );
}

#[test]
fn reference_errors_inside_lists_are_reported() {
    assert_eq!(
        err("k = [\"{nope}\"]"),
        "line 1: reference to 'nope', which doesn't exist"
    );
    assert_eq!(err("k = [\"{k}\"]"), "line 1: reference cycle: k -> k");
}

#[test]
fn getters_check_lists_and_items() {
    let config = ok("\nnum = 1\nlist = [1, 2]\nmix = [\"a\"]");
    assert_eq!(
        config.ints("num"),
        Err("line 2: 'num' is an int, not a list".to_string())
    );
    assert_eq!(
        config.int("list"),
        Err("line 3: 'list' is a list, not an int".to_string())
    );
    assert_eq!(
        config.ints("mix"),
        Err("line 4: 'mix[0]' is a string, not an int".to_string())
    );
    assert_eq!(
        config.ints("missing"),
        Err("no key 'missing' in config".to_string())
    );
}

#[test]
fn item_conversion_errors_name_the_item() {
    assert_eq!(
        ok("k = [1B, 0.5B]").sizes("k"),
        Err("line 1: 'k[1]' isn't a whole number of bytes that fits in u64".to_string())
    );
}

#[test]
fn tesserae_can_override_lists() {
    let src = "k = [1, 2]\n@prod\nk = [3]\n";
    assert_eq!(ok(src).ints("k"), Ok(vec![1, 2]));
    assert_eq!(ok_env(src, "prod").ints("k"), Ok(vec![3]));
}

#[test]
fn tesserae_cant_change_the_item_type() {
    let want = "line 3: @prod resolves 'k' to a different type than base (line 1)";
    let src = "k = [1]\n@prod\nk = [\"a\"]\n";
    assert_eq!(err_env(src, "prod"), want);
    assert_eq!(err(src), want);
}

#[test]
fn tesserae_cant_swap_a_list_for_a_scalar() {
    let want = "line 3: @prod sets 'k' to a different type than base (line 1)";
    assert_eq!(err_env("k = [1]\n@prod\nk = 1\n", "prod"), want);
    assert_eq!(err_env("k = 1\n@prod\nk = [1]\n", "prod"), want);
}

#[test]
fn empty_lists_fit_any_override() {
    assert_eq!(
        ok_env("k = []\n@prod\nk = [1]\n", "prod").ints("k"),
        Ok(vec![1])
    );
    assert_eq!(
        ok_env("k = [1]\n@prod\nk = []\n", "prod").ints("k"),
        Ok(vec![])
    );
}

#[test]
fn reference_items_follow_overrides() {
    let src = "p = :80\nk = [\"{p}\"]\n@prod\np = :443\n";
    assert_eq!(ok_env(src, "prod").ports("k"), Ok(vec![443]));
}
