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

    // List with verbose - shows sources section
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
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

// ============================================================================
// CLI エラー系テスト
// ============================================================================

#[test]
fn test_no_subcommand() {
    // サブコマンドなしで実行するとエラー
    amu_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_invalid_subcommand() {
    // 無効なサブコマンド
    amu_cmd()
        .arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_add_missing_source() {
    // add で source 引数がない
    amu_cmd()
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("<SOURCE>").or(predicate::str::contains("source")));
}

#[test]
fn test_remove_missing_source() {
    // remove で source 引数がない
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

    // 無効なオプション
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
// target デフォルト（現在ディレクトリ）系テスト
// ============================================================================

#[test]
fn test_add_without_target() {
    // target を省略した場合、カレントディレクトリが使用される
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

    // カレントディレクトリ（target）にシンボリックリンクが作成される
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

    // まず add
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("add")
        .arg(&source)
        .assert()
        .success();

    // target 省略で remove
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

    // target 省略で list（カレントディレクトリの情報を表示）
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

    // target 省略で status
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

    // target 省略で update（カレントディレクトリを更新）
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

    // リンクを手動で削除
    fs::remove_file(target.join("test.txt")).unwrap();

    // target 省略で restore
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

    // target 省略で clear
    amu_with_config(&config_path)
        .current_dir(&target)
        .arg("clear")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared:"));

    assert!(!target.join("test.txt").exists());
}

// ============================================================================
// フラグ組み合わせ系テスト
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

    // 2つのソースを追加
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

    // --all で全ターゲットを更新
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

    // 2つのソースを追加
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

    // リンクを手動で削除
    fs::remove_file(target1.join("file1.txt")).unwrap();
    fs::remove_file(target2.join("file2.txt")).unwrap();

    // --all で全ターゲットを復元
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

    // -v（短いオプション）で verbose
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("sources:"));
}

#[test]
fn test_update_short_source() {
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

    // -s（短いオプション）で source 指定
    amu_with_config(&config_path)
        .arg("update")
        .arg("-s")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("target(s) updated"));
}

// ============================================================================
// 競合・排他系テスト
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

    // target と --all を両方指定（--all が優先される）
    amu_with_config(&config_path)
        .arg("list")
        .arg(&target)
        .arg("--all")
        .assert()
        .success();
}

#[test]
fn test_update_source_empty_result() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let source = temp.path().join("source");

    fs::create_dir(&source).unwrap();

    // 登録されていないソースを指定
    amu_with_config(&config_path)
        .arg("update")
        .arg("--source")
        .arg(&source)
        .assert()
        .success()
        .stdout(predicate::str::contains("No targets found"));
}

#[test]
fn test_list_not_registered_target() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.yaml");
    let target = temp.path().join("target");

    fs::create_dir(&target).unwrap();

    // 登録されていないターゲットを指定
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

    // 登録されていないターゲットを指定
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

    // 登録されていないターゲットを指定
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

    // 登録されていないターゲットを指定
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

    // 登録されていないターゲットを指定
    // 設定が空の場合は "No targets registered." が出力される
    // 設定はあるが指定ターゲットがない場合は "Target not registered" が出力される
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
// dry-run テスト
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

    // dry-run でリンクが作成されないことを確認
    amu_with_config(&config_path)
        .arg("add")
        .arg("--dry-run")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // リンクが作成されていないことを確認
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

    // -n オプションが動作することを確認
    amu_with_config(&config_path)
        .arg("add")
        .arg("-n")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // リンクが作成されていないことを確認
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

    // まず add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // dry-run でリンクが削除されないことを確認
    amu_with_config(&config_path)
        .arg("remove")
        .arg("--dry-run")
        .arg(&source)
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // リンクがまだ存在することを確認
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

    // まず add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    assert!(target.join("test.txt").exists());

    // dry-run でリンクが削除されないことを確認
    amu_with_config(&config_path)
        .arg("clear")
        .arg("--dry-run")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // リンクがまだ存在することを確認
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

    // まず add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // dry-run で update
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

    // まず add
    amu_with_config(&config_path)
        .arg("add")
        .arg(&source)
        .arg(&target)
        .assert()
        .success();

    // リンクを手動で削除
    fs::remove_file(target.join("test.txt")).unwrap();
    assert!(!target.join("test.txt").exists());

    // dry-run で restore
    amu_with_config(&config_path)
        .arg("restore")
        .arg("--dry-run")
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    // リンクが復元されていないことを確認
    assert!(!target.join("test.txt").exists());
}
