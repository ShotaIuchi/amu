use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

fn amu_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("amu"))
}

fn amu_with_config(config_path: &std::path::Path) -> Command {
    let mut cmd = amu_cmd();
    cmd.env("AMU_CONFIG", config_path);
    cmd
}

#[test]
fn test_help() {
    amu_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge multiple sources into one target"));
}

#[test]
fn test_version() {
    amu_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("amu"));
}

#[test]
fn test_list_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    amu_with_config(&config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_status_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    amu_with_config(&config_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_add_nonexistent_source() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let nonexistent = temp.path().join("nonexistent");
    let target = temp.path().join("target");
    fs::create_dir(&target).unwrap();

    amu_with_config(&config_path)
        .arg("add")
        .arg(&nonexistent)
        .arg(&target)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Source directory does not exist"));
}

#[test]
fn test_add_nonexistent_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("nonexistent");
    fs::create_dir(&source).unwrap();

    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Target directory does not exist"));
}

#[test]
fn test_remove_not_registered() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();

    amu_with_config(&config_path)
        .arg("remove")
        .arg(&source)
        .arg(&target)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not registered"));
}

#[test]
fn test_add_remove_workflow() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();

    fs::write(source.join("test.txt"), "hello").unwrap();

    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added:"));

    assert!(target.join("test.txt").exists());

    amu_with_config(&config_path)
        .arg("list")
        .assert()
        .success();

    amu_with_config(&config_path)
        .arg("remove")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed:"));

    assert!(!target.join("test.txt").exists());
}

#[test]
fn test_clear_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Add first
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // Clear specific target
    amu_with_config(&config_path)
        .arg("clear")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared:"));

    // Symlink should be removed
    assert!(!target.join("test.txt").exists());

    // Config should be empty
    amu_with_config(&config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_clear_all() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Add first
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // Clear all
    amu_with_config(&config_path)
        .arg("clear")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared all registered sources"));

    // Symlink should be removed
    assert!(!target.join("test.txt").exists());

    // Config should be empty
    amu_with_config(&config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_restore() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Add first
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // Clear links but keep config
    fs::remove_file(target.join("test.txt")).unwrap();
    assert!(!target.join("test.txt").exists());

    // Restore
    amu_with_config(&config_path)
        .arg("restore")
        .assert()
        .success()
        .stdout(predicate::str::contains("succeeded"));

    // Symlink should be restored
    assert!(target.join("test.txt").exists());
}

#[test]
fn test_list_verbose() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // List with verbose - shows sources section
    amu_with_config(&config_path)
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("sources:"));
}

#[test]
fn test_update_source() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Update with --source
    amu_with_config(&config_path)
        .arg("update")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("target(s) updated"));
}
