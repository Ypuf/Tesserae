mod common;

use std::error::Error as _;
use std::io;
use std::path::Path;

use common::{TempFile, load_err, load_err_env, ok};
use tesserae::{Error, ErrorKind, ValueKind, load};

macro_rules! assert_kind {
    ($err:expr, $line:expr, $($pattern:tt)+) => {{
        let err = $err;
        assert!(matches!(err.kind(), $($pattern)+), "{err:?}");
        assert_eq!(err.line(), $line, "{err:?}");
    }};
}

#[test]
fn line_syntax_errors() {
    assert_kind!(load_err("@b d"), Some(1), ErrorKind::InvalidTesseraName { text } if text == "@b d");
    assert_kind!(load_err(":: a b"), Some(1), ErrorKind::InvalidPath { text } if text == ":: a b");
    assert_kind!(load_err("= 1"), Some(1), ErrorKind::EmptyKey { text } if text == "= 1");
    assert_kind!(load_err("a b = 1"), Some(1), ErrorKind::InvalidKey { text } if text == "a b = 1");
    assert_kind!(load_err("\nnope"), Some(2), ErrorKind::InvalidLine { text } if text == "nope");
}

#[test]
fn string_errors() {
    assert_kind!(load_err(r#"k = "\n""#), Some(1), ErrorKind::BadEscape { string } if string == r"\n");
    assert_kind!(load_err(r#"k = "{a""#), Some(1), ErrorKind::UnclosedReference { string } if string == "{a");
    assert_kind!(load_err(r#"k = "{}""#), Some(1), ErrorKind::EmptyReference { string } if string == "{}");
    assert_kind!(load_err(r#"k = "a}""#), Some(1), ErrorKind::UnmatchedBrace { string } if string == "a}");
    assert_kind!(load_err(r#"k = "a"b""#), Some(1), ErrorKind::UnescapedQuote { string } if string == r#"a"b"#);
    assert_kind!(load_err(r#"k = "a"#), Some(1), ErrorKind::UnterminatedString { value } if value == r#""a"#);
}

#[test]
fn list_errors() {
    assert_kind!(load_err("k = [1"), Some(1), ErrorKind::UnterminatedList { list } if list == "[1");
    assert_kind!(load_err("k = [1,]"), Some(1), ErrorKind::EmptyListItem { list } if list == "[1,]");
    assert_kind!(
        load_err("k = [[1]]"),
        Some(1),
        ErrorKind::NestedList { list: Some(list) } if list == "[[1]]"
    );
    assert_kind!(
        load_err("a = [1]\nb = [\"{a}\"]"),
        Some(2),
        ErrorKind::NestedList { list: None }
    );
    assert_kind!(
        load_err("k = [1, 1.5]"),
        Some(1),
        ErrorKind::MixedList {
            first: ValueKind::Int,
            other: ValueKind::Float
        }
    );
}

#[test]
fn literal_errors() {
    assert_kind!(load_err("k = :99999"), Some(1), ErrorKind::InvalidPort { value } if value == "99999");
    assert_kind!(load_err("k = #xyz"), Some(1), ErrorKind::InvalidColour { value } if value == "#xyz");
    assert_kind!(load_err("k = nope"), Some(1), ErrorKind::UnknownValue { value } if value == "nope");
    assert_kind!(load_err("k = +1"), Some(1), ErrorKind::UnknownValue { value } if value == "+1");

    let huge = format!("1{}.0", "0".repeat(400));
    assert_kind!(load_err(&format!("k = {huge}")), Some(1), ErrorKind::FloatOutOfRange { value } if *value == huge);
}

#[test]
fn reference_errors() {
    assert_kind!(
        load_err("a = \"{nope}\""),
        Some(1),
        ErrorKind::UnknownReference { reference } if reference == "nope"
    );
    assert_kind!(
        load_err("a = \"{b}\"\nb = \"{a}\""),
        Some(2),
        ErrorKind::ReferenceCycle { cycle } if *cycle == ["a", "b", "a"]
    );
    assert_kind!(
        load_err("a = [1]\nb = \"x{a}\""),
        Some(2),
        ErrorKind::ListInString { reference } if reference == "a"
    );
}

#[test]
fn unknown_tessera_suggests_a_close_name() {
    let src = "a = 1\n@prod\na = 2";

    assert_kind!(
        load_err_env(src, "prd"),
        None,
        ErrorKind::UnknownTessera { name, suggestion: Some(suggestion) }
            if name == "prd" && suggestion == "prod"
    );
    assert_kind!(
        load_err_env(src, "staging"),
        None,
        ErrorKind::UnknownTessera { name, suggestion: None } if name == "staging"
    );
}

#[test]
fn duplicate_keys() {
    assert_kind!(
        load_err("a = 1\na = 2"),
        Some(2),
        ErrorKind::DuplicateKey { key, first_line: 1 } if key == "a"
    );
    assert_kind!(
        load_err("a = 1\n@prod\na = 2\na = 3"),
        Some(4),
        ErrorKind::DuplicateOverride { tessera, key, first_line: 3 }
            if tessera == "prod" && key == "a"
    );
}

#[test]
fn bad_overrides_name_their_tessera() {
    let not_in_base = load_err("a = 1\n@prod\nb = 2");
    assert_eq!(not_in_base.tessera(), Some("prod"));
    assert_kind!(
        not_in_base,
        Some(3),
        ErrorKind::OverrideNotInBase { tessera, key } if tessera == "prod" && key == "b"
    );

    let literal = load_err("a = 1\n@prod\na = \"x\"");
    assert_eq!(literal.tessera(), Some("prod"));
    assert_kind!(
        literal,
        Some(3),
        ErrorKind::OverrideTypeMismatch { tessera, key, base_line: 1 }
            if tessera == "prod" && key == "a"
    );

    let resolved = load_err("a = 1\nb = \"{a}\"\ns = \"x\"\n@prod\nb = \"{s}\"");
    assert_eq!(resolved.tessera(), Some("prod"));
    assert_kind!(
        resolved,
        Some(5),
        ErrorKind::ResolvedTypeMismatch { tessera, key, base_line: 2 }
            if tessera == "prod" && key == "b"
    );
}

#[test]
fn parse_errors_inside_a_tessera_carry_its_name() {
    for (bad, want) in [
        ("a = nope", "@prod: line 3: unknown value: 'nope'"),
        ("b c = 1", "@prod: line 3: key name is invalid: 'b c = 1'"),
    ] {
        let err = load_err(&format!("a = 1\n@prod\n{bad}"));
        assert_eq!(err.tessera(), Some("prod"), "{bad}");
        assert_eq!(err.to_string(), want, "{bad}");
    }
}

#[test]
fn errors_are_blamed_on_the_most_recent_tessera_header() {
    for (src, want) in [
        ("a = nope", None),
        ("a = 1\n@prod\na = 2\n@staging\na = nope", Some("staging")),
        ("a = 1\n@prod\na = 2\n@\nb = nope", None),
        ("a = 1\n@prod\n@b d", None),
    ] {
        assert_eq!(load_err(src).tessera(), want, "{src:?}");
    }
}

#[test]
fn conversion_errors() {
    let long = format!("k = 1{}yr", "0".repeat(30));
    assert_kind!(load_err(&long), Some(1), ErrorKind::DurationOutOfRange { key } if key == "k");
    assert_kind!(load_err("k = 0.5B"), Some(1), ErrorKind::InvalidSize { key } if key == "k");
    assert_kind!(load_err("k = [1B, 0.5B]"), Some(1), ErrorKind::InvalidSize { key } if key == "k[1]");
}

#[test]
fn getter_errors() {
    let config = ok("\nnum = 1\nmix = [\"a\"]");

    assert_kind!(
        config.int("nope").unwrap_err(),
        None,
        ErrorKind::MissingKey { key } if key == "nope"
    );
    assert_kind!(
        config.str("num").unwrap_err(),
        Some(2),
        ErrorKind::WrongType { key, expected: ValueKind::Str, found: ValueKind::Int } if key == "num"
    );
}

#[test]
fn read_errors_keep_the_path_and_the_io_error() {
    let path = Path::new("/definitely/not/here.tess");
    let err = match load(path, None) {
        Ok(_) => panic!("expected a read error"),
        Err(e) => e,
    };

    assert_kind!(
        &err,
        None,
        ErrorKind::Read {
            error: io::ErrorKind::NotFound
        }
    );
    assert_eq!(err.path(), Some(path));
    assert_eq!(
        err.to_string(),
        "/definitely/not/here.tess: couldn't read file"
    );

    let source = err.source().expect("read errors have a source");
    assert!(source.downcast_ref::<io::Error>().is_some());

    assert!(load_err("a = nope").source().is_none());
}

#[test]
fn every_stage_of_a_file_load_reports_the_path() {
    let sources = [
        ("nonsense", None),
        ("a = nope", None),
        ("a = 1\n@prod\na = nope", None),
        ("a = 1\na = 2", None),
        ("a = 1\n@prod\nb = 2", None),
        ("a = \"{nope}\"", None),
        ("a = 0.5B", None),
        ("a = 1B\n@prod\na = 0.5B", None),
        ("a = 1\nb = \"{a}\"\ns = \"x\"\n@prod\nb = \"{s}\"", None),
        ("a = 1\n@prod\na = 2", Some("nope")),
    ];

    for (src, env) in sources {
        let file = TempFile::new(src);
        let err = match load(file.path(), env) {
            Ok(_) => panic!("expected {src:?} to fail to load"),
            Err(e) => e,
        };
        assert_eq!(err.path(), Some(file.path()), "{src:?}");
        assert!(
            err.to_string()
                .starts_with(&format!("{}: ", file.path().display())),
            "{err}"
        );
    }
}

#[test]
fn getter_errors_from_a_file_report_the_path() {
    let file = TempFile::new("num = 1");
    let config = match load(file.path(), None) {
        Ok(config) => config,
        Err(e) => panic!("expected it to load, got: {e}"),
    };

    assert_eq!(config.int("nope").unwrap_err().path(), Some(file.path()));
    assert_eq!(
        config.str("num").unwrap_err().to_string(),
        format!(
            "{}: line 1: 'num' is an int, not a string",
            file.path().display()
        )
    );
}

#[test]
fn errors_from_a_string_have_no_path() {
    assert_eq!(load_err("a = nope").path(), None);
    assert_eq!(ok("a = 1").int("b").unwrap_err().path(), None);
    assert_eq!(ok("a = 1").str("a").unwrap_err().path(), None);
}

#[test]
fn errors_compare_by_kind_line_and_tessera() {
    assert_eq!(load_err("a = nope"), load_err("a = nope"));
    assert_ne!(load_err("a = nope"), load_err("a = nah"));
    assert_ne!(load_err("a = nope"), load_err("\na = nope"));
    assert_ne!(
        load_err("a = 1B\n@x\na = 0.5B"),
        load_err("a = 1B\n@y\na = 0.5B")
    );
}

#[test]
fn error_works_with_generic_error_handling() {
    fn assert_bounds<E: std::error::Error + Send + Sync + 'static>() {}
    assert_bounds::<Error>();
}
