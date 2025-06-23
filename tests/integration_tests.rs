use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_example_command() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("example")
        .assert()
        .success()
        .stdout(predicate::str::contains("Example command executed successfully!"));
}

#[test]
fn test_example_verbose() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("example")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running in verbose mode"));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("The next gen rm command"));
}