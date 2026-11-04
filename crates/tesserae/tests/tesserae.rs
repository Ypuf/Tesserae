mod common;

use common::{err, err_env, ok_env};

const TWO_ENVS: &str = r#"
    a = 1
    b = 2
    @prod
    a = 10
    @staging
    a = 20
    b = 21
"#;

#[test]
fn env_applies_only_its_own_overrides() {
    let prod = ok_env(TWO_ENVS, "prod");
    assert_eq!(prod.int("a"), Ok(10));
    assert_eq!(prod.int("b"), Ok(2));

    let staging = ok_env(TWO_ENVS, "staging");
    assert_eq!(staging.int("a"), Ok(20));
    assert_eq!(staging.int("b"), Ok(21));
}

#[test]
fn overridden_keys_report_the_override_line() {
    let src = "a = 1\nb = 2\n@e\na = 3\n";
    let config = ok_env(src, "e");
    assert_eq!(config.line("a"), Ok(4));
    assert_eq!(config.line("b"), Ok(2));
}

#[test]
fn a_tessera_can_be_reopened() {
    let src = "a = 1\nb = 2\n@e\na = 3\n@\nc = 4\n@e\nb = 5\n";
    let config = ok_env(src, "e");
    assert_eq!(config.int("a"), Ok(3));
    assert_eq!(config.int("b"), Ok(5));
    assert_eq!(config.int("c"), Ok(4));
}

#[test]
fn an_empty_tessera_equals_the_base() {
    assert_eq!(ok_env("a = 1\n@e\n", "e").int("a"), Ok(1));
}

#[test]
fn overrides_of_the_same_type_may_change_units() {
    let src = "t = 30sec\ns = 1GiB\n@e\nt = 1min\ns = 1B\n";
    let config = ok_env(src, "e");
    assert_eq!(config.duration("t"), Ok(std::time::Duration::from_secs(60)));
    assert_eq!(config.size("s"), Ok(1));
}

#[test]
fn unknown_env_suggests_the_closest_tessera() {
    let src = "a = 1\n@prod\n@staging\n";
    assert_eq!(
        err_env(src, "prdo"),
        "no tessera matching 'prdo'. did you mean 'prod'?"
    );
    assert_eq!(
        err_env(src, "stagin"),
        "no tessera matching 'stagin'. did you mean 'staging'?"
    );
}

#[test]
fn suggestions_stop_after_edit_distance_two() {
    let src = "a = 1\n@prod\n";
    assert_eq!(
        err_env(src, "pr"),
        "no tessera matching 'pr'. did you mean 'prod'?"
    );
    assert_eq!(err_env(src, "p"), "no tessera matching 'p'.");
    assert_eq!(
        err_env(src, "production"),
        "no tessera matching 'production'."
    );
}

#[test]
fn suggestion_ties_break_alphabetically() {
    assert_eq!(
        err_env("a = 1\n@prom\n@prod\n", "prox"),
        "no tessera matching 'prox'. did you mean 'prod'?"
    );
}

#[test]
fn env_names_are_case_sensitive() {
    assert_eq!(
        err_env("a = 1\n@prod\n", "PROD"),
        "no tessera matching 'PROD'."
    );
}

#[test]
fn overriding_with_a_different_type_is_rejected() {
    for (base, over) in [
        ("1", "1.5"),
        ("1", "\"1\""),
        ("1", "true"),
        ("\"x\"", ":80"),
        (":80", "80"),
        ("30sec", "1B"),
        ("#FF6611", "\"#FF6611\""),
    ] {
        assert_eq!(
            err_env(&format!("k = {base}\n@e\nk = {over}"), "e"),
            "line 3: @e sets 'k' to a different type than base (line 1)",
            "{base} -> {over}"
        );
    }
}

#[test]
fn type_changes_through_references_are_rejected() {
    assert_eq!(
        err_env("p = :80\nk = 1\n@e\nk = \"{p}\"", "e"),
        "line 4: @e resolves 'k' to a different type than base (line 2)"
    );
    assert_eq!(
        err_env("p = :80\nk = \"{p}\"\n@e\nk = 1", "e"),
        "line 4: @e resolves 'k' to a different type than base (line 2)"
    );
}

#[test]
fn same_resolved_type_through_a_reference_is_allowed() {
    let src = "p = :80\nq = :81\nk = \"{p}\"\n@e\nk = \"{q}\"";
    assert_eq!(ok_env(src, "e").port("k"), Ok(81));
}

#[test]
fn duplicate_override_across_reopened_blocks_is_rejected() {
    assert_eq!(
        err_env("a = 1\n@e\na = 2\n@\n@e\na = 3", "e"),
        "line 6: 'a' already set in @e on line 3"
    );
}

#[test]
fn every_tessera_is_checked_even_when_not_requested() {
    let src = "a = 1\n@good\na = 2\n@bad\nnope = 3";
    let want = "line 5: @bad overrides 'nope', which isn't in base";
    assert_eq!(err(src), want);
    assert_eq!(err_env(src, "good"), want);
}

#[test]
fn bad_sizes_and_durations_in_any_tessera_fail_every_load() {
    let huge = format!("1{}yr", "0".repeat(30));
    for (src, want) in [
        (
            "a = 1B\n@good\na = 2B\n@bad\na = 1.5B\n".to_string(),
            "@bad: line 5: 'a' isn't a whole number of bytes that fits in u64",
        ),
        (
            "a = [1B]\n@good\na = [2B]\n@bad\na = [1.5B]\n".to_string(),
            "@bad: line 5: 'a[0]' isn't a whole number of bytes that fits in u64",
        ),
        (
            format!("a = 1sec\n@good\na = 2sec\n@bad\na = {huge}\n"),
            "@bad: line 5: duration is out of range: 'a'",
        ),
    ] {
        assert_eq!(err(&src), want);
        assert_eq!(err_env(&src, "good"), want);
        assert_eq!(err_env(&src, "bad"), want);
    }
}

#[test]
fn requested_tessera_errors_win_then_others_alphabetically() {
    let src = "a = 1\n@zed\nz = 1\n@alpha\ny = 1\n@mid\nx = 1\n";
    let zed = "line 3: @zed overrides 'z', which isn't in base";
    let alpha = "line 5: @alpha overrides 'y', which isn't in base";
    let mid = "line 7: @mid overrides 'x', which isn't in base";

    assert_eq!(err(src), alpha);
    assert_eq!(err_env(src, "zed"), zed);
    assert_eq!(err_env(src, "mid"), mid);
}

#[test]
fn base_errors_win_over_tessera_errors() {
    assert_eq!(
        err_env("a = \"{nope}\"\n@e\nb = 1", "e"),
        "line 1: reference to 'nope', which doesn't exist"
    );
}
