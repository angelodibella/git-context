use crate::config::{Config, Context};
use anyhow::anyhow;
use anyhow::{Context as _, Result, bail};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

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
