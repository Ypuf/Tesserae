mod common;

use std::process::{Command, Output};

use common::TempFile;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tesserae"))
        .args(args)
        .output()
        .expect("couldn't run the binary")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn prints_keys_sorted_with_values_and_lines() {
    let file = TempFile::new("b = 2\na = :80\n");
    let out = run(&[file.path().to_str().unwrap()]);

    assert!(out.status.success());
    assert_eq!(
        stdout(&out),
        "a = Port(80)  (line 2)\nb = Int(2)  (line 1)\n"
    );
}

#[test]
fn second_argument_selects_the_env() {
    let file = TempFile::new("a = 1\n@prod\na = 2\n");
    let out = run(&[file.path().to_str().unwrap(), "prod"]);

    assert!(out.status.success());
    assert_eq!(stdout(&out), "a = Int(2)  (line 3)\n");
}

#[test]
fn load_errors_go_to_stderr_with_exit_code_one() {
    let file = TempFile::new("a = nope\n");
    let out = run(&[file.path().to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), "line 1: unknown value: 'nope'\n");
}

#[test]
fn missing_path_argument_fails_with_usage() {
    let out = run(&[]);

    assert!(!out.status.success());
    assert!(stderr(&out).contains("usage: <path> [env]"));
}
