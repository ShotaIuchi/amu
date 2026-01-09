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
        Commands::Add { source, target } => cmd_add(source, target),
        Commands::Remove { source, target } => cmd_remove(source, target),
        Commands::Update { target } => cmd_update(target),
        Commands::List { target } => cmd_list(target),
        Commands::Status => cmd_status(),
    }
}

fn cmd_add(source: PathBuf, target: Option<PathBuf>) -> Result<()> {
    let source = normalize_path(&source)?;
    let target = resolve_target(target)?;

    if !source.is_dir() {
        return Err(DotlinkError::SourceNotFound(source));
    }
    if !target.is_dir() {
        return Err(DotlinkError::TargetNotFound(target));
    }

    let mut config = Config::load()?;
    config.add_source(target.clone(), source.clone())?;

    stow::stow(&source, &target)?;
    config.save()?;

    println!("Added: {} -> {}", source.display(), target.display());
    Ok(())
}

fn cmd_remove(source: PathBuf, target: Option<PathBuf>) -> Result<()> {
    let source = config::expand_path(&source);
    let target = resolve_target(target)?;

    let source = if source.exists() {
        source.canonicalize()?
    } else {
        source
    };

    let mut config = Config::load()?;

    if source.exists() {
        stow::unstow(&source, &target)?;
    }

    config.remove_source(&target, &source)?;
    config.save()?;

    println!("Removed: {} -> {}", source.display(), target.display());
    Ok(())
}

fn cmd_update(target: Option<PathBuf>) -> Result<()> {
    let config = Config::load()?;

    let targets: Vec<PathBuf> = match target {
        Some(t) => {
            let normalized = resolve_target(Some(t))?;
            vec![normalized]
        }
        None => config.targets.keys().cloned().collect(),
    };

    if targets.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    for target in targets {
        if let Some(sources) = config.get_sources(&target) {
            println!("Updating {}:", target.display());
            for source in sources {
                if source.exists() {
                    stow::restow(source, &target)?;
                    println!("  Restowed: {}", source.display());
                } else {
                    println!("  Skipped (not found): {}", source.display());
                }
            }
        }
    }

    Ok(())
}

fn cmd_list(target: Option<PathBuf>) -> Result<()> {
    let config = Config::load()?;

    let targets: Vec<&PathBuf> = match &target {
        Some(t) => {
            let normalized = resolve_target(Some(t.clone()))?;
            config
                .targets
                .keys()
                .filter(|k| **k == normalized)
                .collect()
        }
        None => config.targets.keys().collect(),
    };

    if targets.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    for target in targets {
        println!("{}:", abbreviate_path(target));
        if let Some(sources) = config.get_sources(target) {
            for source in sources {
                println!("  - {}", abbreviate_path(source));
            }
        }
        println!();
    }

    Ok(())
}

fn cmd_status() -> Result<()> {
    let config = Config::load()?;

    if config.targets.is_empty() {
        println!("No targets registered.");
        return Ok(());
    }

    let mut has_issues = false;

    for (target, sources) in &config.targets {
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

    if has_issues {
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
