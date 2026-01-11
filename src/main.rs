mod cli;
mod config;
mod error;
mod stow;

use std::path::{Path, PathBuf};

use clap::Parser;

use cli::{Cli, Commands};
use config::{normalize_path, resolve_target, Config};
use error::{DotlinkError, Result};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    stow::check_installed()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Add { source, target, dry_run } => cmd_add(source, target, dry_run),
        Commands::Remove { source, target, dry_run } => cmd_remove(source, target, dry_run),
        Commands::Update { target, all, source, dry_run } => cmd_update(target, all, source, dry_run),
        Commands::Restore { target, all, dry_run } => cmd_restore(target, all, dry_run),
        Commands::List { target, all, verbose } => cmd_list(target, all, verbose),
        Commands::Status { target, all, json } => cmd_status(target, all, json),
        Commands::Clear { target, all, dry_run } => cmd_clear(target, all, dry_run),
    }
}

fn cmd_add(source: PathBuf, target: Option<PathBuf>, dry_run: bool) -> Result<()> {
    let source = normalize_path(&source)?;
    let target = resolve_target(target)?;

    if !source.is_dir() {
        return Err(DotlinkError::SourceNotFound(source));
    }
    if !target.is_dir() {
        return Err(DotlinkError::TargetNotFound(target));
    }

    // dry-run モード: プレビューのみ
    if dry_run {
        println!("[dry-run] add {} -> {}", abbreviate_path(&source), abbreviate_path(&target));
        let output = stow::dry_run(&source, &target)?;
        let links = stow::parse_dry_run_output(&output);
        if links.is_empty() {
            println!("  No changes would be made.");
        } else {
            for link in links {
                println!("  {}", link);
            }
        }
        return Ok(());
    }

    let mut config = Config::load()?;
    config.add_source(target.clone(), source.clone())?;

    stow::stow(&source, &target)?;
    config.save()?;

    println!("Added: {} -> {}", source.display(), target.display());
    Ok(())
}

fn cmd_remove(source: PathBuf, target: Option<PathBuf>, dry_run: bool) -> Result<()> {
    let source = config::expand_path(&source);
    let target = resolve_target(target)?;

    let source = if source.exists() {
        source.canonicalize()?
    } else {
        source
    };

    // dry-run モード: プレビューのみ
    if dry_run {
        println!("[dry-run] remove {} -> {}", abbreviate_path(&source), abbreviate_path(&target));
        if source.exists() {
            let output = stow::dry_run_unstow(&source, &target)?;
            let links = stow::parse_dry_run_output(&output);
            if links.is_empty() {
                println!("  No changes would be made.");
            } else {
                for link in links {
                    println!("  {}", link);
                }
            }
        } else {
            println!("  Source not found, would only remove from config.");
        }
        return Ok(());
    }

    let mut config = Config::load()?;

    if source.exists() {
        stow::unstow(&source, &target)?;
    }

    config.remove_source(&target, &source)?;
    config.save()?;

    println!("Removed: {} -> {}", source.display(), target.display());
    Ok(())
}

fn cmd_update(target: Option<PathBuf>, all: bool, source: Option<PathBuf>, dry_run: bool) -> Result<()> {
    let config = Config::load()?;

    // --source モード: 指定ソースを参照している全ターゲットを更新
    if let Some(src) = source {
        let src = normalize_path(&src)?;
        let mut updated = 0;

        let prefix = if dry_run { "[dry-run] " } else { "" };
        println!("{}Updating targets that reference {}:", prefix, abbreviate_path(&src));

        for (target, sources) in &config.targets {
            if sources.contains(&src) {
                if src.exists() && target.exists() {
                    if dry_run {
                        let output = stow::dry_run_restow(&src, target)?;
                        let links = stow::parse_dry_run_output(&output);
                        println!("  {} (would restow {} links)", abbreviate_path(target), links.len());
                    } else {
                        stow::restow(&src, target)?;
                        println!("  \u{2713} {}", abbreviate_path(target));
                    }
                    updated += 1;
                } else if !target.exists() {
                    println!("  \u{2717} {} (target not found)", abbreviate_path(target));
                }
            }
        }

        if updated == 0 {
            println!("No targets found for this source.");
        } else {
            println!("\nDone: {} target(s) {}", updated, if dry_run { "would be updated" } else { "updated" });
        }
        return Ok(());
    }

    // ターゲットを決定
    let targets: Vec<PathBuf> = if all {
        config.targets.keys().cloned().collect()
    } else {
        let t = resolve_target(target)?;
        if !config.targets.contains_key(&t) {
            println!("Target not registered: {}", abbreviate_path(&t));
            return Ok(());
        }
        vec![t]
    };

    if targets.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    let prefix = if dry_run { "[dry-run] " } else { "" };
    for target in targets {
        if let Some(sources) = config.get_sources(&target) {
            println!("{}Updating {}:", prefix, abbreviate_path(&target));
            for source in sources {
                if source.exists() {
                    if dry_run {
                        let output = stow::dry_run_restow(source, &target)?;
                        let links = stow::parse_dry_run_output(&output);
                        if links.is_empty() {
                            println!("  Would restow: {} (no changes)", abbreviate_path(source));
                        } else {
                            println!("  Would restow: {} ({} links)", abbreviate_path(source), links.len());
                        }
                    } else {
                        stow::restow(source, &target)?;
                        println!("  Restowed: {}", abbreviate_path(source));
                    }
                } else {
                    println!("  Skipped (not found): {}", abbreviate_path(source));
                }
            }
        }
    }

    Ok(())
}

fn cmd_list(target: Option<PathBuf>, all: bool, verbose: bool) -> Result<()> {
    let config = Config::load()?;

    // ターゲットを決定
    let target_list: Vec<PathBuf> = if all {
        config.targets.keys().cloned().collect()
    } else {
        let t = resolve_target(target)?;
        if config.targets.contains_key(&t) {
            vec![t]
        } else {
            println!("Target not registered: {}", abbreviate_path(&t));
            return Ok(());
        }
    };

    if target_list.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    for target in &target_list {
        println!("{}:", abbreviate_path(target));
        if let Some(sources) = config.get_sources(target) {
            if verbose {
                println!("  sources:");
                for source in sources {
                    println!("    - {}", abbreviate_path(source));
                }
                let links = collect_symlinks(target, sources);
                if !links.is_empty() {
                    println!("  links:");
                    for (link_path, link_target) in links {
                        println!("    {} -> {}", abbreviate_path(&link_path), abbreviate_path(&link_target));
                    }
                }
            } else {
                for source in sources {
                    println!("  - {}", abbreviate_path(source));
                }
            }
        }
        println!();
    }

    Ok(())
}

fn collect_symlinks(target: &Path, sources: &[PathBuf]) -> Vec<(PathBuf, PathBuf)> {
    let mut links = Vec::new();
    collect_symlinks_recursive(target, sources, target, &mut links);
    links
}

fn collect_symlinks_recursive(base_target: &Path, sources: &[PathBuf], current: &Path, links: &mut Vec<(PathBuf, PathBuf)>) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_symlink() {
                if let Ok(link_target) = std::fs::read_link(&path) {
                    // Check if this symlink points to one of our sources
                    let abs_target = if link_target.is_absolute() {
                        link_target.clone()
                    } else {
                        path.parent().unwrap_or(current).join(&link_target)
                    };
                    for source in sources {
                        if abs_target.starts_with(source) {
                            links.push((path.clone(), abs_target));
                            break;
                        }
                    }
                }
            } else if path.is_dir() {
                collect_symlinks_recursive(base_target, sources, &path, links);
            }
        }
    }
}

fn cmd_status(target: Option<PathBuf>, all: bool, json: bool) -> Result<()> {
    let config = Config::load()?;

    // ターゲットを決定
    let target_list: Vec<PathBuf> = if all {
        config.targets.keys().cloned().collect()
    } else {
        let t = resolve_target(target)?;
        if config.targets.contains_key(&t) {
            vec![t]
        } else {
            if json {
                println!("{{\"error\": \"Target not registered\"}}");
            } else {
                println!("Target not registered: {}", abbreviate_path(&t));
            }
            return Ok(());
        }
    };

    if target_list.is_empty() {
        if json {
            println!("{{\"targets\": [], \"summary\": {{\"ok\": 0, \"warning\": 0, \"error\": 0}}}}");
        } else {
            println!("No targets registered.");
        }
        return Ok(());
    }

    let mut ok_count = 0;
    let mut warning_count = 0;
    let mut error_count = 0;
    let mut json_targets: Vec<String> = Vec::new();

    for target in &target_list {
        if let Some(sources) = config.get_sources(target) {
            if !json {
                println!("{}:", abbreviate_path(target));
            }
            let mut json_sources: Vec<String> = Vec::new();

            for source in sources {
                let status = check_source_status(source, target);
                match &status {
                    SourceStatus::Ok { link_count } => {
                        if json {
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"ok\", \"link_count\": {}}}",
                                abbreviate_path(source), link_count
                            ));
                        } else {
                            println!("  \u{2713} {} ({} links)", abbreviate_path(source), link_count);
                        }
                        ok_count += 1;
                    }
                    SourceStatus::SourceNotFound => {
                        if json {
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"error\", \"message\": \"source not found\"}}",
                                abbreviate_path(source)
                            ));
                        } else {
                            println!("  \u{2717} {} (source not found)", abbreviate_path(source));
                        }
                        error_count += 1;
                    }
                    SourceStatus::TargetNotFound => {
                        if json {
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"error\", \"message\": \"target not found\"}}",
                                abbreviate_path(source)
                            ));
                        } else {
                            println!("  \u{2717} {} (target not found)", abbreviate_path(source));
                        }
                        error_count += 1;
                    }
                    SourceStatus::BrokenLinks(links) => {
                        if json {
                            let links_json: Vec<String> = links.iter().map(|l| format!("\"{}\"", l)).collect();
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"warning\", \"message\": \"broken links\", \"details\": [{}]}}",
                                abbreviate_path(source), links_json.join(", ")
                            ));
                        } else {
                            println!("  ! {} (broken links)", abbreviate_path(source));
                            for link in links {
                                println!("    - {}", link);
                            }
                        }
                        warning_count += 1;
                    }
                    SourceStatus::Conflicts(msg) => {
                        if json {
                            let escaped_msg = msg.replace('\"', "\\\"").replace('\n', "\\n");
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"warning\", \"message\": \"conflicts\", \"details\": \"{}\"}}",
                                abbreviate_path(source), escaped_msg
                            ));
                        } else {
                            println!("  ! {} (conflicts detected)", abbreviate_path(source));
                            for line in msg.lines().take(5) {
                                if !line.trim().is_empty() {
                                    println!("    {}", line.trim());
                                }
                            }
                        }
                        warning_count += 1;
                    }
                    SourceStatus::RealFiles(files) => {
                        if json {
                            let files_json: Vec<String> = files.iter().map(|f| format!("\"{}\"", f)).collect();
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"warning\", \"message\": \"real files (expected symlinks)\", \"details\": [{}]}}",
                                abbreviate_path(source), files_json.join(", ")
                            ));
                        } else {
                            println!("  ! {} (real files found)", abbreviate_path(source));
                            for file in files {
                                println!("    - {} (expected symlink)", file);
                            }
                        }
                        warning_count += 1;
                    }
                    SourceStatus::PermissionDenied(msg) => {
                        if json {
                            json_sources.push(format!(
                                "{{\"path\": \"{}\", \"status\": \"error\", \"message\": \"permission denied: {}\"}}",
                                abbreviate_path(source), msg
                            ));
                        } else {
                            println!("  \u{2717} {} (permission denied: {})", abbreviate_path(source), msg);
                        }
                        error_count += 1;
                    }
                }
            }

            if json {
                json_targets.push(format!(
                    "{{\"path\": \"{}\", \"sources\": [{}]}}",
                    abbreviate_path(target), json_sources.join(", ")
                ));
            } else {
                println!();
            }
        }
    }

    if json {
        println!(
            "{{\"targets\": [{}], \"summary\": {{\"ok\": {}, \"warning\": {}, \"error\": {}}}}}",
            json_targets.join(", "), ok_count, warning_count, error_count
        );
    } else {
        println!("Summary: {} OK, {} warning, {} error", ok_count, warning_count, error_count);
    }

    if error_count > 0 || warning_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_clear(target: Option<PathBuf>, all: bool, dry_run: bool) -> Result<()> {
    let mut config = Config::load()?;

    if config.targets.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    // Determine which targets to clear
    let targets_to_clear: Vec<PathBuf> = if all {
        config.targets.keys().cloned().collect()
    } else {
        let t = resolve_target(target)?;
        if !config.targets.contains_key(&t) {
            println!("Target not registered: {}", abbreviate_path(&t));
            return Ok(());
        }
        vec![t]
    };

    // dry-run モード: プレビューのみ
    if dry_run {
        println!("[dry-run] Would clear:");
        for target in &targets_to_clear {
            println!("  {}", abbreviate_path(target));
            if let Some(sources) = config.targets.get(target) {
                for source in sources {
                    if source.exists() && target.exists() {
                        let output = stow::dry_run_unstow(source, target)?;
                        let links = stow::parse_dry_run_output(&output);
                        println!("    {} ({} links)", abbreviate_path(source), links.len());
                    }
                }
            }
        }
        return Ok(());
    }

    for target in &targets_to_clear {
        if let Some(sources) = config.targets.get(target) {
            for source in sources {
                if source.exists() && target.exists() {
                    if let Err(e) = stow::unstow(source, target) {
                        eprintln!("Warning: Failed to unstow {} -> {}: {}", source.display(), target.display(), e);
                    }
                }
            }
        }
        config.targets.remove(target);
    }

    config.save()?;

    if all {
        println!("Cleared all registered sources.");
    } else {
        println!("Cleared: {}", abbreviate_path(&targets_to_clear[0]));
    }
    Ok(())
}

fn cmd_restore(target: Option<PathBuf>, all: bool, dry_run: bool) -> Result<()> {
    let config = Config::load()?;

    // ターゲットを決定
    let target_list: Vec<PathBuf> = if all {
        config.targets.keys().cloned().collect()
    } else {
        let t = resolve_target(target)?;
        if config.targets.contains_key(&t) {
            vec![t]
        } else {
            println!("Target not registered: {}", abbreviate_path(&t));
            return Ok(());
        }
    };

    if target_list.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    // dry-run モード: プレビューのみ
    if dry_run {
        println!("[dry-run] Would restore:");
        for target in &target_list {
            println!("  {}:", abbreviate_path(target));
            if let Some(sources) = config.get_sources(target) {
                for source in sources {
                    if source.exists() {
                        // ターゲットが存在しない場合も表示
                        if target.exists() {
                            let output = stow::dry_run(source, target)?;
                            let links = stow::parse_dry_run_output(&output);
                            println!("    {} ({} links)", abbreviate_path(source), links.len());
                        } else {
                            println!("    {} (target would be created)", abbreviate_path(source));
                        }
                    } else {
                        println!("    {} (source not found)", abbreviate_path(source));
                    }
                }
            }
        }
        return Ok(());
    }

    let mut success = 0;
    let mut failed = 0;

    for target in &target_list {
        if let Some(sources) = config.get_sources(target) {
            println!("{}:", abbreviate_path(target));

            // Create target directory if it doesn't exist
            if !target.exists() {
                if let Err(e) = std::fs::create_dir_all(target) {
                    eprintln!("  Failed to create target directory: {}", e);
                    failed += sources.len();
                    continue;
                }
            }

            for source in sources {
                if source.exists() {
                    match stow::stow(source, target) {
                        Ok(()) => {
                            println!("  \u{2713} {}", abbreviate_path(source));
                            success += 1;
                        }
                        Err(e) => {
                            println!("  \u{2717} {} ({})", abbreviate_path(source), e);
                            failed += 1;
                        }
                    }
                } else {
                    println!("  \u{2717} {} (source not found)", abbreviate_path(source));
                    failed += 1;
                }
            }
            println!();
        }
    }

    println!("Done: {} succeeded, {} failed", success, failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/*
 * ソースのステータスを表す列挙型
 */
enum SourceStatus {
    Ok { link_count: usize },
    SourceNotFound,
    TargetNotFound,
    BrokenLinks(Vec<String>),
    Conflicts(String),
    RealFiles(Vec<String>),
    PermissionDenied(String),
}

fn check_source_status(source: &Path, target: &Path) -> SourceStatus {
    // 権限チェック
    if let Err(e) = std::fs::read_dir(source) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            return SourceStatus::PermissionDenied(format!("source: {}", source.display()));
        }
        return SourceStatus::SourceNotFound;
    }
    if !source.exists() {
        return SourceStatus::SourceNotFound;
    }
    if !target.exists() {
        return SourceStatus::TargetNotFound;
    }

    // 壊れたリンクのチェック
    let broken_links = find_broken_links(source, target);
    if !broken_links.is_empty() {
        return SourceStatus::BrokenLinks(broken_links);
    }

    // 実ファイルのチェック（リンクであるべき場所に実ファイルがある）
    let real_files = find_real_files(source, target);
    if !real_files.is_empty() {
        return SourceStatus::RealFiles(real_files);
    }

    // コンフリクトのチェック
    if let Ok(output) = stow::dry_run(source, target) {
        if output.contains("CONFLICT") || output.contains("existing target") {
            return SourceStatus::Conflicts(output);
        }
    }

    // リンク数をカウント
    let link_count = count_links(source, target);
    SourceStatus::Ok { link_count }
}

/*
 * 壊れたシンボリックリンクを検出する
 */
fn find_broken_links(source: &Path, target: &Path) -> Vec<String> {
    let mut broken = Vec::new();
    find_broken_links_recursive(source, target, source, &mut broken);
    broken
}

fn find_broken_links_recursive(source_base: &Path, target: &Path, current_source: &Path, broken: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(current_source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let relative = source_path.strip_prefix(source_base).unwrap_or(&source_path);
            let target_path = target.join(relative);

            if source_path.is_dir() && !source_path.is_symlink() {
                find_broken_links_recursive(source_base, target, &source_path, broken);
            } else if target_path.is_symlink() {
                // リンクが壊れているかチェック
                if !target_path.exists() {
                    broken.push(relative.display().to_string());
                }
            }
        }
    }
}

/*
 * リンクであるべき場所に実ファイルがあるか検出する
 */
fn find_real_files(source: &Path, target: &Path) -> Vec<String> {
    let mut real_files = Vec::new();
    find_real_files_recursive(source, target, source, &mut real_files);
    real_files
}

fn find_real_files_recursive(source_base: &Path, target: &Path, current_source: &Path, real_files: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(current_source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let relative = source_path.strip_prefix(source_base).unwrap_or(&source_path);
            let target_path = target.join(relative);

            if source_path.is_dir() && !source_path.is_symlink() {
                find_real_files_recursive(source_base, target, &source_path, real_files);
            } else if source_path.is_file() {
                // ターゲットに同名のファイルがあり、シンボリックリンクでない場合
                if target_path.exists() && !target_path.is_symlink() {
                    real_files.push(relative.display().to_string());
                }
            }
        }
    }
}

/*
 * ソースからターゲットへのリンク数をカウントする
 */
fn count_links(source: &Path, target: &Path) -> usize {
    let mut count = 0;
    count_links_recursive(source, target, source, &mut count);
    count
}

fn count_links_recursive(source_base: &Path, target: &Path, current_source: &Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(current_source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let relative = source_path.strip_prefix(source_base).unwrap_or(&source_path);
            let target_path = target.join(relative);

            if source_path.is_dir() && !source_path.is_symlink() {
                count_links_recursive(source_base, target, &source_path, count);
            } else if target_path.is_symlink() {
                *count += 1;
            }
        }
    }
}

fn abbreviate_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
