use crate::config::{Config, Context};
use anyhow::anyhow;
use anyhow::{Context as _, Result, bail};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn init(name: &str) -> Result<()> {
    if !Path::new(".git").exists() {
        bail!("Git repository not found. Run 'git init' first.");
    }

    let metadata = fs::symlink_metadata(".git")?;
    if metadata.file_type().is_symlink() {
        bail!("Git is already managed by git-context (it is a symlink).");
    }

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

    let metadata = fs::symlink_metadata(".git").context("Failed to read '.git' metadata")?;
    if !metadata.file_type().is_symlink() {
        bail!("The '.git' directory is not a symlink. Is this repo managed by git-context?");
    }

    fs::remove_file(".git").context("Failed to remove old '.git' symlink")?;
    symlink(&target_path, ".git").context("Failed to switch '.git' symlink")?;

    config.active_context = name.to_string();
    config.save()?;

    println!("Switched to context '{}'", name);
    Ok(())
}

pub fn new(name: &str) -> Result<()> {
    if Path::new(".contexts").exists() {
        let config = Config::load()?;
        if config.contexts.contains_key(name) {
            bail!("Context '{}' already exists", name);
        }
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

    let mut config = if Path::new(".contexts").exists() {
        Config::load()?
    } else {
        Config {
            active_context: name.to_string(),
            managed_files: Vec::new(),
            contexts: HashMap::new(),
        }
    };

    config.contexts.insert(
        name.to_string(),
        Context {
            path: PathBuf::from(&target_path_str),
        },
    );
    config.active_context = name.to_string();

    config.save()?;

    println!("Created new context '{}'", name);
    Ok(())
}

pub fn keep() -> Result<()> {
    todo!()
}
pub fn unkeep() -> Result<()> {
    todo!()
}

pub fn exec(context_name: &str, args: Vec<String>) -> Result<()> {
    let config =
        Config::load().context("Could not load contexts. Have you run 'git context init'?")?;

    let context = config
        .contexts
        .get(context_name)
        .ok_or_else(|| anyhow::anyhow!("Context '{}' not found", context_name))?;

    let git_dir = context.path.to_str().unwrap();

    if args.is_empty() {
        bail!("No command specified");
    }

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
    todo!()
}
