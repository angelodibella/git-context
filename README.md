### :construction: **This project is in very early development and is not currently usable. Once releases are rolled out, it will be safe to use.** :construction:

# Git Context

A Git extension for managing multiple repositories and developer personas within a single working directory.

Stop juggling `--git-dir`, complex aliases, or separate directories. Keep your public dotfiles, private keys, and work projects all in one place, seamlessly.

## The Problem

You have a `~/dotfiles` repository for sharing your shell configuration with the world. But you also have sensitive files you want to track in a *separate*, private repository, like `rclone` configs, `git` credentials, or SSH keys. More generally, you have a singe directory in which you want contents from different repositories to be there, but track them seamlessly.

You can try to set up symlinks from different directories—storing the different repos—in the target directory; however, this also means handling VCS in two separate directories, and no information at all from what is tracked and by what in the target directory. This makes tracking and development teadious and inefficient.

Furthermore, trying to directly merge differently tracked content is out of the question: different Git directories, especially called anything but `.git/` in a worktree, cause various problems including
- Duplicate READMEs, gitignores etc.
- Terminal prompts not synced with correct tracker (or none).
- Editors in the worktree confused as to what files are tracked, ignored, etc.
- Managing the `--git-dir` flags and aliases including them is inefficient and does not sacale to work generally.

## The Solution

`git-context` manages multiple Git repositories within one working tree by cleverly swapping what the `.git` directory points to. It uses a symbolic link (`.git`) which can be instantly pointed to different underlying Git directories (e.g., `.git-public`, `.git-private`).

This means all your tools that expect a single `.git` directory—your terminal prompt (Starship, Powerlevel10k), your IDE (VS Code), and `git` itself—work perfectly, no matter which context is active.

### How It Looks

<!-- *(some cool terminal recording or something idk)* -->

Here's a conceptual demo:

```sh
# You are in your dotfiles directory, the prompt shows the "public" context
~/dotfiles ❯ git context switch private
✔ Switched context to "private"

# The prompt immediately updates to show the new context!
# Your git commands now operate on the private repo.
~/dotfiles [private] ❯ git status
On branch main
Your branch is up to date with 'origin/main'.

Untracked files:
  (use "git add <file>..." to include in what will be committed)
        new-secret.key

# Switch back just as easily
~/dotfiles [private] ❯ git context switch public
✔ Switched context to "public"
```

## Core Features

  * **Seamless Context Switching:** Instantly switch the active repository with `git context switch <name>`.
  * **Automatic Ignore Management:** When you `git context track` a file in one context, it's automatically ignored by all other contexts. No more `.gitignore` headaches\!
  * **Command Passthrough:** Run a command on any context without switching. Perfect for a quick push: `git context exec private -- git push`.
  * **Shell Integration:** Display the active context name directly in your shell prompt.
  * **Safe and Fast:** Written in Rust for performance and memory safety.
  * **Simple Configuration:** A single, human-readable `.gitcontexts.toml` file manages your workspace.

## Installation

Once the project is on `crates.io`, you can install it via `cargo`:

```sh
cargo install git-context
```

Pre-compiled binaries will also be available from the GitHub Releases page for Windows, macOS, and Linux.

## Quick Start

1.  **Initialize your first context.**
    Navigate to an existing Git repository:

    ```sh
    cd ~/dotfiles
    git context init public
    ```

    This will rename your `.git` to `.git-public`, create a `.git` symlink pointing to it, and set up your configuration file.

2.  **Create a new, empty context.**

    ```sh
    git context new private
    ```

    This creates a new, empty Git repository at `.git-private` and adds it to your configuration. Now you can switch between `public` and `private`.

3.  **Track a file.**
    Add a new file that should only belong to your `private` repository:

    ```sh
    # Make sure you are in the right context
    git context switch private

    # Track the file using the special command
    git context track secrets.txt
    ```

    This will `git add secrets.txt` to the `private` repo and simultaneously add `secrets.txt` to a global ignore file, so your `public` repo won't see it.

## Contributing

This project is open-source and contributions are welcome\! Feel free to open an issue or submit a pull request.
