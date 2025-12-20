use crate::config::{Config, Context};
use anyhow::{Context as _, Result, anyhow, bail};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Scans upwards to find the project root (marked by .contexts or .git)
fn find_root() -> Result<(PathBuf, PathBuf)> {
    let original_dir = env::current_dir().context("Failed to get current directory")?;
    let mut ancestor = original_dir.as_path();

    loop {
        if ancestor.join(".contexts").exists() || ancestor.join(".git").exists() {
            return Ok((ancestor.to_path_buf(), original_dir));
        }

        match ancestor.parent() {
            Some(parent) => ancestor = parent,
            None => bail!(
                "Not inside a git-context managed repository (could not find .contexts or .git)"
            ),
        }
    }
}

/// Jump to the project root
fn setup_worktree() -> Result<PathBuf> {
    let (root, original_dir) = find_root()?;

    let relative_offset = original_dir
        .strip_prefix(&root)
        .unwrap_or(Path::new("."))
        .to_path_buf();

    env::set_current_dir(&root).context("Failed to change to project root")?;

    Ok(relative_offset)
}

/// Ensures project is managed by git-context.
fn ensure_managed() -> Result<()> {
    let metadata = fs::symlink_metadata(".git").context("Failed to read '.git' metadata")?;
    if !metadata.file_type().is_symlink() {
        bail!("The '.git' directory is not a symlink. Is this repo managed by git-context?");
    }
    // TODO: Add more checks, a symlink doesn't cut it.

    Ok(())
}

/// Ignore git-context-specific files
///
/// Uses .git/info/exclude to bypass the need for a .gitignore, keeping the contexts clean.
fn add_default_ignores(git_dir: &Path) -> Result<()> {
    let exclude_path = git_dir.join("info").join("exclude");

    if let Some(parent) = exclude_path.parent() {
        fs::create_dir_all(parent).context("Failed to create git info directory")?;
    }

    // Read existing content or start empty
    let mut content = if exclude_path.exists() {
        fs::read_to_string(&exclude_path).context("Failed to read git exclude file")?
    } else {
        String::new()
    };

    let patterns = [".contexts", ".git-*"];
    let mut changed = false;

    for pattern in patterns {
        if !content.contains(pattern) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(pattern);
            content.push('\n');
            changed = true;
        }
    }

    if changed {
        fs::write(&exclude_path, content).context("Failed to update git exclude file")?;
        println!("Added default ignores to {:?}", exclude_path);
    }

    Ok(())
}

/// Returns the path of kept files.
fn get_storage_path(git_dir_path: &Path, file_path: &Path) -> PathBuf {
    let mut storage = git_dir_path.join("info").join("context-managed");
    if let Some(parent) = file_path.parent() {
        storage.push(parent);
    }

    storage.join(file_path.file_name().unwrap())
}

/// Initializes git-context.
///
/// Turn the current git repository into a context, given that the repository is already
/// git-initialized and that it is not yet managed by git-context.
pub fn init(name: &str) -> Result<()> {
    if !Path::new(".git").exists() {
        bail!("Git repository not found. Run 'git init' first.");
    }

    let new_git_dir = format!(".git-{}", name);
    fs::rename(".git", &new_git_dir).context("Failed to rename existing '.git' directory")?;

    add_default_ignores(Path::new(&new_git_dir))?;

    // Use symlinks for compatibility with git tools (prompts, editors, etc.)
    symlink(&new_git_dir, ".git").context("Failed to create '.git' symlink")?;

    // Construct new blank context
    let mut contexts_map = HashMap::new();
    contexts_map.insert(
        name.to_string(),
        Context {
            path: PathBuf::from(&new_git_dir),
            managed_files: Vec::new(),
            // TODO: Add automatic keeping of duplicates across contexts
        },
    );

    // Add context to the config
    let config = Config {
        active_context: name.to_string(),
        contexts: contexts_map,
    };

    config.save()?;

    println!("Initialized context '{}'", name);
    Ok(())
}

pub fn switch(name: &str) -> Result<()> {
    ensure_managed()?;
    setup_worktree()?;

    let mut config = Config::load()?;
    if config.active_context == name {
        println!("Already in the context '{}'", name);
        return Ok(());
    }

    let old_context_name = config.active_context.clone();
    let old_context = config
        .contexts
        .get(&old_context_name)
        .ok_or_else(|| anyhow!("Current context '{}' data missing", old_context_name))?;

    let new_context = config
        .contexts
        .get(name)
        .ok_or_else(|| anyhow!("Target context '{}' not found", name))?;
    let new_context_path = new_context.path.clone(); // Clone path to use after borrow ends

    for file_path in &old_context.managed_files {
        if file_path.exists() {
            let storage_path = get_storage_path(&old_context.path, file_path);

            if let Some(parent) = storage_path.parent() {
                fs::create_dir_all(parent).context("Failed to create storage directory")?;
            }

            fs::rename(file_path, &storage_path)
                .context(format!("Failed to stash managed file '{:?}'", file_path))?;

            println!("Stashed {:?}", file_path);
        }
    }

    fs::remove_file(".git").context("Failed to remove old '.git' symlink")?;
    symlink(&new_context_path, ".git").context("Failed to switch '.git' symlink")?;

    for file_path in &new_context.managed_files {
        let storage_path = get_storage_path(&new_context_path, file_path);

        if file_path.exists() {
            println!("Warning: Overwriting existing file {:?}", file_path);
        }

        if storage_path.exists() {
            if let Some(parent) = file_path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)?;
                }
            }

            fs::rename(&storage_path, file_path)
                .context(format!("Failed to restore managed file '{:?}'", file_path))?;

            println!("Restored {:?}", file_path);
        }
    }

    config.active_context = name.to_string();
    config.save()?;

    println!("Switched to context '{}'", name);
    Ok(())
}

/// Create new context.
///
/// Assume git-context is already managing the current git project.
pub fn new(name: &str) -> Result<()> {
    setup_worktree()?;
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

    add_default_ignores(Path::new(&target_path_str))?;

    config.contexts.insert(
        name.to_string(),
        Context {
            path: PathBuf::from(&target_path_str),
            managed_files: Vec::new(),
        },
    );

    fs::remove_file(".git").context("Failed to remove old '.git' symlink")?;
    symlink(&target_path, ".git").context("Failed to switch '.git' symlink")?;

    config.save()?;
    switch(name)?;

    println!("Created new context '{}'", name);
    Ok(())
}

pub fn clone(url: &str) -> Result<()> {
    todo!()
}

pub fn keep(path: &str) -> Result<()> {
    let offset = setup_worktree()?;
    ensure_managed()?;

    let mut config = Config::load()?;

    let raw_path = Path::new(path);
    let target_path = if raw_path.is_absolute() {
        bail!("Please use relative paths");
    } else {
        offset.join(raw_path)
    };

    if !target_path.exists() {
        bail!("Target '{:?}' not found", target_path);
    }

    // Mutable references are cool!
    let context = config
        .contexts
        .get_mut(&config.active_context)
        .ok_or_else(|| anyhow::anyhow!("Active context not found"))?;

    if context.managed_files.contains(&target_path) {
        println!(
            "Target is already managed by the context '{}'",
            config.active_context
        );
        return Ok(());
    }

    context.managed_files.push(target_path);
    config.save()?;

    Ok(())
}

pub fn unkeep(path: &str) -> Result<()> {
    let offset = setup_worktree()?;
    ensure_managed()?;

    let mut config =
        Config::load().context("Could not load contexts. Have you run 'git context init'?")?;

    // Mutable references are cool!
    let context = config
        .contexts
        .get_mut(&config.active_context)
        .ok_or_else(|| anyhow::anyhow!("Active context not found"))?;

    let target_path = offset.join(path);

    let initial_len = context.managed_files.len();
    context.managed_files.retain(|x| x != &target_path);

    if context.managed_files.len() == initial_len {
        println!(
            "File '{:?}' was not being managed by context '{}'",
            target_path, config.active_context
        );
    } else {
        println!(
            "Stopped managing '{:?}' in context '{}'",
            target_path, config.active_context
        );
        config.save()?;
    }

    Ok(())
}
pub fn exec(context_name: &str, args: Vec<String>) -> Result<()> {
    ensure_managed()?;
    let (root, _) = find_root()?;

    let config_path = root.join(".contexts");
    let content = fs::read_to_string(&config_path)
        .context("Could not load contexts. Have you run 'git context init'?")?;
    let config: Config = toml::from_str(&content)?;

    let context = config
        .contexts
        .get(context_name)
        .ok_or_else(|| anyhow!("Context '{}' not found", context_name))?;

    let absolute_git_dir = root.join(&context.path);

    if args.is_empty() {
        bail!("No command specified");
    }

    let program = &args[0];
    let program_args = &args[1..];

    Command::new(program)
        .args(program_args)
        .env("GIT_DIR", absolute_git_dir)
        .env("GIT_WORK_TREE", root)
        .status()
        .context(format!("Failed to execute '{}'", program))?;

    Ok(())
}

pub fn refresh() -> Result<()> {
    todo!()
}

pub fn status() -> Result<()> {
    setup_worktree()?;
    ensure_managed()?;

    let config = Config::load()
        .context("Could not load contexts. Is this repository managed by git-context?")?;

    // Unwrap since we're sure here
    let context = config.contexts.get(&config.active_context).unwrap();

    println!("Active context: {}", config.active_context);
    println!("----------------");

    println!("\nManaged Files:");
    if context.managed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &context.managed_files {
            let exists = file.exists();
            let status_text = if exists { "Present" } else { "Stashed/Missing" };
            println!("  - {:?} [{}]", file, status_text);
        }
    }

    println!("\nAvailable Contexts:");
    for (name, _ctx) in &config.contexts {
        let arrow = if name == &config.active_context {
            "(active)"
        } else {
            ""
        };
        println!("  - {}  {}", name, arrow);
    }

    Ok(())
}
