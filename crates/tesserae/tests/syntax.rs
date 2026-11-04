mod common;

use std::path::Path;

use common::{err, ok};

#[test]
fn empty_and_comment_only_files_have_no_keys() {
    for src in ["", "\n\n", "// nothing\n   // here\n"] {
        assert!(ok(src).keys().is_empty(), "{src:?}");
    }
}

#[test]
fn trailing_comments_are_stripped() {
    assert_eq!(ok("k = 5 // five").int("k"), Ok(5));
    assert_eq!(ok("k = 5\t// five").int("k"), Ok(5));
    assert_eq!(ok("k = 5 //").int("k"), Ok(5));
    assert_eq!(ok("k = 5 // \"quote in comment").int("k"), Ok(5));
}

#[test]
fn comment_needs_preceding_whitespace() {
    assert_eq!(err("k = 5// five"), "line 1: unknown value: '5// five'");
}

#[test]
fn slashes_inside_strings_are_not_comments() {
    assert_eq!(ok(r#"k = "http://x // y""#).str("k"), Ok("http://x // y"));
    assert_eq!(ok(r#"k = "a\" // b" // real"#).str("k"), Ok(r#"a" // b"#));
}

#[test]
fn whitespace_around_equals_is_optional() {
    assert_eq!(ok("k=5").int("k"), Ok(5));
    assert_eq!(ok("k   =   5").int("k"), Ok(5));
}

#[test]
fn crlf_line_endings_work() {
    let config = ok("a = 1\r\n:: s\r\nb = \"x\" // c\r\n");
    assert_eq!(config.int("a"), Ok(1));
    assert_eq!(config.str("s/b"), Ok("x"));
}

#[test]
fn key_names_allow_alphanumerics_underscores_and_dashes() {
    assert_eq!(ok("aZ_0-9 = 1").int("aZ_0-9"), Ok(1));
}

#[test]
fn invalid_keys_are_rejected() {
    assert_eq!(err("= 1"), "line 1: key can't be empty: '= 1'");
    for key in ["a b", "a.b", "/a", "a/", "a//b", "\"a\""] {
        let src = format!("{key} = 1");
        assert_eq!(err(&src), format!("line 1: key name is invalid: '{src}'"));
    }
}

#[test]
fn unparseable_lines_are_rejected() {
    assert_eq!(err("nonsense"), r#"line 1: can't parse "nonsense""#);
}

#[test]
fn errors_report_the_real_line_number() {
    let src = "// comment\n\na = 1\n\n  // another\nb = nope\n";
    assert_eq!(err(src), "line 6: unknown value: 'nope'");
}

#[test]
fn sections_prefix_the_keys_below_them() {
    let config = ok("a = 1\n:: db\nb = 2\nc/d = 3\n:: x/y\ne = 4\n");
    assert_eq!(config.int("a"), Ok(1));
    assert_eq!(config.int("db/b"), Ok(2));
    assert_eq!(config.int("db/c/d"), Ok(3));
    assert_eq!(config.int("x/y/e"), Ok(4));
}

#[test]
fn sections_replace_rather_than_nest() {
    let config = ok(":: a\n:: b\nk = 1\n");
    assert_eq!(config.int("b/k"), Ok(1));
    assert!(config.int("a/b/k").is_err());
}

#[test]
fn bare_section_returns_to_root() {
    assert_eq!(ok(":: a\n::\nk = 1\n").int("k"), Ok(1));
}

#[test]
fn entering_or_leaving_a_tessera_returns_to_root() {
    let src = "a/k = 1\nb = 2\nc = 3\n:: a\n@e\nb = 4\n:: a\nk = 5\n@\nc2 = 6\n";
    let config = common::ok_env(src, "e");
    assert_eq!(config.int("b"), Ok(4));
    assert_eq!(config.int("a/k"), Ok(5));
    assert_eq!(config.int("c2"), Ok(6));
}

#[test]
fn invalid_section_and_tessera_names_are_rejected() {
    for src in [":: a b", ":: /a", ":: a/", ":: a = 1"] {
        assert_eq!(err(src), format!("line 1: path name is invalid: '{src}'"));
    }
    for src in ["@a b", "@a/b", "@a=1"] {
        assert_eq!(
            err(src),
            format!("line 1: tessera name is invalid: '{src}'")
        );
    }
}

#[test]
fn duplicates_are_detected_across_section_and_path_spellings() {
    assert_eq!(
        err("db/host = 1\n:: db\nhost = 2"),
        "line 3: 'db/host' already defined on line 1"
    );
}

#[test]
fn missing_file_is_reported_with_its_path() {
    let path = Path::new("/definitely/not/here.tess");
    let msg = match tesserae::load(path, None) {
        Ok(_) => panic!("expected a read error"),
        Err(e) => e,
    };
    assert!(
        msg.starts_with("couldn't read /definitely/not/here.tess: "),
        "{msg}"
    );
}
