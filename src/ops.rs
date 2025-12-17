use crate::config::{Config, Context};
use anyhow::anyhow;
use anyhow::{Context as _, Result, bail};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ensure_managed() -> Result<()> {
    let metadata = fs::symlink_metadata(".git").context("Failed to read '.git' metadata")?;
    if !metadata.file_type().is_symlink() {
        bail!("The '.git' directory is not a symlink. Is this repo managed by git-context?");
    }

    Ok(())
}

pub fn init(name: &str) -> Result<()> {
    if !Path::new(".git").exists() {
        bail!("Git repository not found. Run 'git init' first.");
    }

    ensure_managed()?;

    let new_git_dir = format!(".git-{}", name);
    fs::rename(".git", &new_git_dir).context("Failed to rename existing '.git' directory")?;

    symlink(&new_git_dir, ".git").context("Failed to create '.git' symlink")?;

    let mut contexts_map = HashMap::new();
    contexts_map.insert(
        name.to_string(),
        Context {
            path: PathBuf::from(&new_git_dir),
        },
    );

    let config = Config {
        active_context: name.to_string(),
        managed_files: Vec::new(),
        contexts: contexts_map,
    };

    config.save()?;

    println!("Initialized context '{}'", name);
    Ok(())
}

pub fn switch(name: &str) -> Result<()> {
    let mut config = Config::load()?;
    if config.active_context == name {
        println!("Already in the context '{}'", name);
        return Ok(());
    }

    let target_path = config
        .contexts
        .get(name)
        .ok_or_else(|| anyhow!("Context '{}' not found in '.contexts'", name))?
        .path
        .clone();

    ensure_managed()?;

    fs::remove_file(".git").context("Failed to remove old '.git' symlink")?;
    symlink(&target_path, ".git").context("Failed to switch '.git' symlink")?;

    config.active_context = name.to_string();
    config.save()?;

    println!("Switched to context '{}'", name);
    Ok(())
}

pub fn new(name: &str) -> Result<()> {
    ensure_managed()?;

    let mut config = Config::load()?;
    if config.contexts.contains_key(name) {
        bail!("Context '{}' already exists", name);
    }

    let target_path_str = format!(".git-{}", name);
    let target_path = Path::new(&target_path_str);
    if target_path.exists() {
        bail!("Directory '{}' already exists", target_path_str);
    }

    println!("Creating git repository at {}...", target_path_str);

    let status = Command::new("git")
        .args(&["init", "--bare", &target_path_str])
        .output()
        .context("Failed to run git init")?;

    if !status.status.success() {
        bail!("'git init' failed");
    }

    Command::new("git")
        .args(&[
            "--git-dir",
            &target_path_str,
            "config",
            "core.bare",
            "false",
        ])
        .status()?;

    Command::new("git")
        .args(&[
            "--git-dir",
            &target_path_str,
            "config",
            "core.logallrefupdates",
            "true",
        ])
        .status()?;

    config.contexts.insert(
        name.to_string(),
        Context {
            path: PathBuf::from(&target_path_str),
        },
    );

    fs::remove_file(".git").context("Failed to remove old '.git' symlink")?;
    symlink(&target_path, ".git").context("Failed to switch '.git' symlink")?;

    config.active_context = name.to_string();
    config.save()?;

    println!("Created new context '{}'", name);
    Ok(())
}

pub fn keep(path: &str) -> Result<()> {
    let mut config =
        Config::load().context("Could not load contexts. Have you run 'git context init'?")?;

    if !Path::new(path).exists() {
        bail!("Target to keep not found")
    }

    let target_path = PathBuf::from(path);
    if config.managed_files.contains(&target_path) {
        println!(
            "Target is already managed by the context '{}'",
            config.active_context
        );
        return Ok(());
    }

    config.managed_files.push(target_path);
    config.save()?;

    Ok(())
}

pub fn unkeep(path: &str) -> Result<()> {
    let mut config =
        Config::load().context("Could not load contexts. Have you run 'git context init'?")?;

    let target_path = PathBuf::from(path);
    config.managed_files.retain(|x| x != &target_path);
    config.save()?;

    Ok(())
}

pub fn exec(context_name: &str, args: Vec<String>) -> Result<()> {
    let config =
        Config::load().context("Could not load contexts. Have you run 'git context init'?")?;

    let context = config
        .contexts
        .get(context_name)
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", context_name))?;

    let git_dir = context
        .path
        .to_str()
        .context("Path contains invalid characters")?;

    if args.is_empty() {
        bail!("No command specified");
    }

    ensure_managed()?;
    let program = &args[0];
    let program_args = &args[1..];
    let status = Command::new(program)
        .args(program_args)
        .env("GIT_DIR", git_dir)
        .env("GIT_WORK_TREE", ".")
        .status()
        .context(format!("Failed to execute '{}'", program))?;

    // TODO: Propagate the exit code
    Ok(())
}

pub fn refresh() -> Result<()> {
    todo!()
}
pub fn status() -> Result<()> {
    let config = Config::load()
        .context("Could not load contexts. Is this repository managed by git-context?")?;

    println!("Active context: {}", config.active_context);
    println!("\nKept files:");
    for file in config.managed_files {
        print!("{}\t", file.to_string_lossy());
    }

    println!("\n\nContexts:");
    for context in config.contexts.keys() {
        print!("{}\t", context);
    }

    println!();

    Ok(())
}
