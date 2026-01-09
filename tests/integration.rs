use std::fs;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

fn dotlink_with_config(config_path: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("dotlink").unwrap();
    cmd.env("DOTLINK_CONFIG", config_path);
    cmd
}

#[test]
fn test_help() {
    Command::cargo_bin("dotlink")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dotfiles linker using GNU stow"));
}

#[test]
fn test_version() {
    Command::cargo_bin("dotlink")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("dotlink"));
}

#[test]
fn test_list_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    dotlink_with_config(&config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_status_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    dotlink_with_config(&config_path)
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

    dotlink_with_config(&config_path)
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

    dotlink_with_config(&config_path)
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

    dotlink_with_config(&config_path)
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

    dotlink_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added:"));

    assert!(target.join("test.txt").exists());

    dotlink_with_config(&config_path)
        .arg("list")
        .assert()
        .success();

    dotlink_with_config(&config_path)
        .arg("remove")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed:"));

    assert!(!target.join("test.txt").exists());
}
