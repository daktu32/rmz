use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "modern replacement for the rm command",
        ));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_delete_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("delete")
        .arg(&test_file)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"));

    // File should still exist after dry run
    assert!(test_file.exists());
}

#[test]
fn test_status_command() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Trash Status"));
}

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("list").assert().success().stdout(
        predicate::str::contains("Trash is empty").or(predicate::str::contains("Files in trash")),
    );
}

#[test]
fn test_restore_invalid_id() {
    let mut cmd = Command::cargo_bin("rmz").unwrap();
    cmd.arg("restore")
        .arg("--id")
        .arg("invalid-id")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid UUID format"));
}
