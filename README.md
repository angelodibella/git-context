# `git-context`

A Git extension for managing multiple repositories within a single working directory.

## The Problem

You have a `~/dotfiles` repository for sharing your shell configuration with the world. But you also have sensitive files you want to track in a *separate*, private repository, like `rclone` configs or SSH keys. More generally, you have a single directory where you want content from different repositories to coexist and be tracked seamlessly.

Trying to merge differently-tracked content is fraught with problems:

  * Standard Git commands get confused by multiple git directories.
  * Terminal prompts and IDE integrations break.
  * Managing `--git-dir` flags or complex aliases is tedious and doesn't scale.
  * Files like `README.md` or `.gitignore` that exist in both repos create conflicts in the working directory.
  * Being bound by usint `--separate-git-dir` which uses (fragile) absolute paths.

## The Solution

`git-context` orchestrates multiple Git repositories within one working tree. It uses a symbolic link named `.git` that it can instantly point to different underlying repositories (e.g., `.git-public`, `.git-private`).

This makes all your tools (your terminal prompt, your text editor/IDE, and `git` itself) work perfectly, no matter which context is active. It goes a step further by actively managing files that are shared between contexts, giving you a branch-like switching experience.

## Core Features

  * **Seamless Context Switching:** Instantly switch the active repository with `git context switch <name>`.
  * **Context-Dependent Files:** Designate files like `README.md` or `.gitignore` as "managed" with `git context keep <file>`. `git-context` will then automatically show the correct version of the file for the active context, hiding the others.
  * **Automatic Ignore Management:** Use standard `git add` and `git commit`. When you're done, run `git context refresh`, and the tool will ensure files from one context are automatically ignored by all others. No more cluttered `.gitignore` files.
  * **Command Passthrough:** Run a command on an inactive context without a full switch. Perfect for a quick push: `git context exec private -- git push`.
  * **Shell Integration:** Display the active context name directly in your shell prompt for immediate clarity.
  * **Safe and Fast:** Written in Rust for performance, safety, and reliability.
  * **Simple Configuration:** A single, human-readable `.contexts` TOML file manages your entire workspace.

## Installation

You can install it via `cargo`:

```sh
cargo install git-context
```

Pre-compiled binaries will also be available from the [GitHub Releases](https://github.com/angelodibella/git-context/releases) page.

## Quick Start

1.  **Initialize your first context from an existing repo.**

    ```sh
    cd ~/dotfiles
    git context init public
    ```

    This creates your `.contexts` config and prepares the workspace.

2.  **Tell `git-context` which files to manage.**
    If your `public` repo has a `README.md` and `.gitignore` that are unique to it, register them:

    ```sh
    git context keep README.md
    git context keep .gitignore
    ```

3.  **Create and switch to a new context.**

    ```sh
    git context new private
    ```

    At this point, the `README.md` and `.gitignore` from the `public` context will have vanished from your working directory, ready for you to create new, private versions.

4.  **Work as usual, then refresh.**
    Add and commit files to your `private` context using normal git commands.

    ```sh
    echo "SECRET_KEY=123" > api.key
    git add api.key
    git commit -m "Add secret key"
    ```

    Now, your `public` context will correctly ignore `api.key`.

## Future Features

* Add `git context clone <url> <name>` with optional `<name>` to merge directly from remote into a context.
* Automatic managing of duplicate files on creationg of new context.
