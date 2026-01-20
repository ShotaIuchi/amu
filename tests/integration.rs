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
        .stdout(predicate::str::contains("Merge multiple source directories into one target with symlinks"));
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
fn test_list_all_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    amu_with_config(&config_path)
        .arg("list")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets registered"));
}

#[test]
fn test_status_all_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    amu_with_config(&config_path)
        .arg("status")
        .arg("--all")
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
        .arg("--all")
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
        .arg("--all")
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

    // Restore specific target
    amu_with_config(&config_path)
        .arg("restore")
        .arg(&target)
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

    // List with verbose - shows sources section and links section
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("sources:"))
        .stdout(predicate::str::contains("links:"))
        .stdout(predicate::str::contains("test.txt"));
}

#[test]
fn test_list_verbose_shows_all_links() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();

    // Create multiple files and subdirectories
    fs::write(source.join("file1.txt"), "1").unwrap();
    fs::write(source.join("file2.txt"), "2").unwrap();
    fs::create_dir(source.join("subdir")).unwrap();
    fs::write(source.join("subdir").join("file3.txt"), "3").unwrap();

    // Add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // List with verbose - verify all links are displayed
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("links:"))
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"))
        .stdout(predicate::str::contains("file3.txt"));
}

// sync command is interactive, so only basic help test
#[test]
fn test_sync_help() {
    amu_cmd()
        .arg("sync")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync targets from a source directory"));
}

// ============================================================================
// CLI error tests
// ============================================================================

#[test]
fn test_no_subcommand() {
    // Error when running without subcommand
    amu_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_invalid_subcommand() {
    // Invalid subcommand
    amu_cmd()
        .arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_add_missing_source() {
    // add without source argument
    amu_cmd()
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("<SOURCE>").or(predicate::str::contains("source")));
}

#[test]
fn test_remove_missing_source() {
    // remove without source argument
    amu_cmd()
        .arg("remove")
        .assert()
        .failure()
        .stderr(predicate::str::contains("<SOURCE>").or(predicate::str::contains("source")));
}

#[test]
fn test_invalid_option() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    // Invalid option
    amu_cmd()
        .arg("add")
        .arg("--invalid-option")
        .arg(&source)
        .arg(&target)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

// ============================================================================
// Target default (current directory) tests
// ============================================================================

#[test]
fn test_add_without_target() {
    // When target is omitted, current directory is used
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added:"));

    // Symlink is created in current directory (target)
    assert!(target.join("test.txt").exists());
}

#[test]
fn test_remove_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // remove with target omitted
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("remove")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed:"));

    assert!(!target.join("test.txt").exists());
}

#[test]
fn test_list_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // list with target omitted (shows current directory info)
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("source"));
}

#[test]
fn test_status_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // status with target omitted
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_update_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // update with target omitted (updates current directory)
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating"));
}

#[test]
fn test_restore_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // Manually delete links
    fs::remove_file(target.join("test.txt")).unwrap();

    // restore with target omitted
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("restore")
        .assert()
        .success()
        .stdout(predicate::str::contains("succeeded"));

    assert!(target.join("test.txt").exists());
}

#[test]
fn test_clear_without_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // clear with target omitted
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("clear")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared:"));

    assert!(!target.join("test.txt").exists());
}

// ============================================================================
// Flag combination tests
// ============================================================================

#[test]
fn test_update_all() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source1 = temp.path().join("source1");
    let source2 = temp.path().join("source2");
    let target1 = temp.path().join("target1");
    let target2 = temp.path().join("target2");

    fs::create_dir(&source1).unwrap();
    fs::create_dir(&source2).unwrap();
    fs::create_dir(&target1).unwrap();
    fs::create_dir(&target2).unwrap();
    fs::write(source1.join("file1.txt"), "hello1").unwrap();
    fs::write(source2.join("file2.txt"), "hello2").unwrap();

    // Add two sources
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source1)
        .arg(&target1)
        .assert()
        .success();

    amu_with_config(&config_path)
        .arg("add")
        .arg(&source2)
        .arg(&target2)
        .assert()
        .success();

    // Update all targets with --all
    amu_with_config(&config_path)
        .arg("update")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updating"));
}

#[test]
fn test_restore_all() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source1 = temp.path().join("source1");
    let source2 = temp.path().join("source2");
    let target1 = temp.path().join("target1");
    let target2 = temp.path().join("target2");

    fs::create_dir(&source1).unwrap();
    fs::create_dir(&source2).unwrap();
    fs::create_dir(&target1).unwrap();
    fs::create_dir(&target2).unwrap();
    fs::write(source1.join("file1.txt"), "hello1").unwrap();
    fs::write(source2.join("file2.txt"), "hello2").unwrap();

    // Add two sources
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source1)
        .arg(&target1)
        .assert()
        .success();

    amu_with_config(&config_path)
        .arg("add")
        .arg(&source2)
        .arg(&target2)
        .assert()
        .success();

    // Manually delete links
    fs::remove_file(target1.join("file1.txt")).unwrap();
    fs::remove_file(target2.join("file2.txt")).unwrap();

    // Restore all targets with --all
    amu_with_config(&config_path)
        .arg("restore")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("succeeded"));

    assert!(target1.join("file1.txt").exists());
    assert!(target2.join("file2.txt").exists());
}

#[test]
fn test_list_short_verbose() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // verbose with -v (short option)
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("sources:"));
}

// Note: test_update_short_source was removed as it moved to sync command

// ============================================================================
// Conflict and mutual exclusion tests
// ============================================================================

#[test]
fn test_list_target_with_all() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Specify both target and --all (--all takes precedence)
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("--all")
        .assert()
        .success();
}

// Note: test_update_source_empty_result moved to sync command (skipped due to interactivity)

#[test]
fn test_list_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // Specify unregistered target
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target not registered"));
}

#[test]
fn test_status_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // Specify unregistered target
    amu_with_config(&config_path)
        .arg("status")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target not registered"));
}

#[test]
fn test_update_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // Specify unregistered target
    amu_with_config(&config_path)
        .arg("update")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target not registered"));
}

#[test]
fn test_restore_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // Specify unregistered target
    amu_with_config(&config_path)
        .arg("restore")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target not registered"));
}

#[test]
fn test_clear_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // Specify unregistered target
    // If config is empty, "No targets registered." is output
    // If config exists but specified target is not registered, "Target not registered" is output
    amu_with_config(&config_path)
        .arg("clear")
        .arg(&target)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Target not registered")
                .or(predicate::str::contains("No targets registered"))
        );
}

// ============================================================================
// dry-run tests
// ============================================================================

#[test]
fn test_add_dry_run() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Verify links are not created with dry-run
    amu_with_config(&config_path)
        .arg("add")
        .arg("--dry-run")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify links are not created
    assert!(!target.join("test.txt").exists());
}

#[test]
fn test_add_dry_run_short_option() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // Verify -n option works
    amu_with_config(&config_path)
        .arg("add")
        .arg("-n")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify links are not created
    assert!(!target.join("test.txt").exists());
}

#[test]
fn test_remove_dry_run() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // Verify links are not deleted with dry-run
    amu_with_config(&config_path)
        .arg("remove")
        .arg("--dry-run")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify links still exist
    assert!(target.join("test.txt").exists());
}

#[test]
fn test_clear_dry_run() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // Verify links are not deleted with dry-run
    amu_with_config(&config_path)
        .arg("clear")
        .arg("--dry-run")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify links still exist
    assert!(target.join("test.txt").exists());
}

#[test]
fn test_update_dry_run() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // update with dry-run
    amu_with_config(&config_path)
        .arg("update")
        .arg("--dry-run")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));
}

#[test]
fn test_restore_dry_run() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Manually delete links
    fs::remove_file(target.join("test.txt")).unwrap();
    assert!(!target.join("test.txt").exists());

    // restore with dry-run
    amu_with_config(&config_path)
        .arg("restore")
        .arg("--dry-run")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // Verify links are not restored
    assert!(!target.join("test.txt").exists());
}

// === status command extended tests ===

#[test]
fn test_status_json_output() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Verify JSON output
    amu_with_config(&config_path)
        .arg("status")
        .arg("--json")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""))
        .stdout(predicate::str::contains("\"link_count\""))
        .stdout(predicate::str::contains("\"summary\""));
}

#[test]
fn test_status_link_count() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("file1.txt"), "1").unwrap();
    fs::write(source.join("file2.txt"), "2").unwrap();
    fs::write(source.join("file3.txt"), "3").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Verify link count is displayed
    amu_with_config(&config_path)
        .arg("status")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 links"));
}

#[test]
fn test_status_real_files_detection() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "source").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Delete link and replace with real file
    fs::remove_file(target.join("test.txt")).unwrap();
    fs::write(target.join("test.txt"), "real file").unwrap();

    // Verify real files are detected by status
    amu_with_config(&config_path)
        .arg("status")
        .arg(&target)
        .assert()
        .failure()
        .stdout(predicate::str::contains("real files found"));
}

#[test]
fn test_status_summary() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");
    let target = temp.path().join("target");

    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::write(source.join("test.txt"), "hello").unwrap();

    // First add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // Verify Summary is displayed
    amu_with_config(&config_path)
        .arg("status")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"))
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn test_status_json_empty() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");

    // JSON output with empty config
    amu_with_config(&config_path)
        .arg("status")
        .arg("--all")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"targets\": []"));
}
