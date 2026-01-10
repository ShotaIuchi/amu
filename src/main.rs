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
        Commands::Status { target, all } => cmd_status(target, all),
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

fn cmd_status(target: Option<PathBuf>, all: bool) -> Result<()> {
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

    let mut has_issues = false;

    for target in &target_list {
        if let Some(sources) = config.get_sources(target) {
            println!("{}:", abbreviate_path(target));

            for source in sources {
                let status = check_source_status(source, target);
                match status {
                    SourceStatus::Ok => {
                        println!("  \u{2713} {} (OK)", abbreviate_path(source));
                    }
                    SourceStatus::SourceNotFound => {
                        println!("  \u{2717} {} (source not found)", abbreviate_path(source));
                        has_issues = true;
                    }
                    SourceStatus::TargetNotFound => {
                        println!("  \u{2717} {} (target not found)", abbreviate_path(source));
                        has_issues = true;
                    }
                    SourceStatus::BrokenLinks(links) => {
                        println!("  ! {} (broken links)", abbreviate_path(source));
                        for link in links {
                            println!("    - {}", link);
                        }
                        has_issues = true;
                    }
                    SourceStatus::Conflicts(msg) => {
                        println!("  ! {} (conflicts detected)", abbreviate_path(source));
                        for line in msg.lines().take(5) {
                            if !line.trim().is_empty() {
                                println!("    {}", line.trim());
                            }
                        }
                        has_issues = true;
                    }
                }
            }
            println!();
        }
    }

    if has_issues {
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

enum SourceStatus {
    Ok,
    SourceNotFound,
    TargetNotFound,
    BrokenLinks(Vec<String>),
    Conflicts(String),
}

fn check_source_status(source: &Path, target: &Path) -> SourceStatus {
    if !source.exists() {
        return SourceStatus::SourceNotFound;
    }
    if !target.exists() {
        return SourceStatus::TargetNotFound;
    }

    let broken_links = find_broken_links(source, target);
    if !broken_links.is_empty() {
        return SourceStatus::BrokenLinks(broken_links);
    }

    if let Ok(output) = stow::dry_run(source, target) {
        if output.contains("CONFLICT") || output.contains("existing target") {
            return SourceStatus::Conflicts(output);
        }
    }

    SourceStatus::Ok
}

fn find_broken_links(source: &Path, target: &Path) -> Vec<String> {
    let mut broken = Vec::new();

    if let Ok(entries) = std::fs::read_dir(source) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let target_path = target.join(&file_name);

            if target_path.is_symlink() {
                if let Ok(link_target) = std::fs::read_link(&target_path) {
                    if !link_target.exists() && !target_path.exists() {
                        broken.push(file_name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    broken
}

fn abbreviate_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
