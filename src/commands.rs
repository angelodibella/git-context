use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    #[arg(default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initializes new context from a present Git repository
    Init {
        context: String,
    },

    /// Creates a clean Git repository and adds it as a context
    New {
        context: String,
    },

    /// Switches context
    Switch {
        context: String,
    },

    /// Keep files or directories unique to the current context
    Keep {
        path: PathBuf,
    },

    /// Discard files or directories from being unique to the current context
    Unkeep {
        path: PathBuf,
    },

    /// Execute commands passed through to an available context
    Exec {
        context: String,
        args: Vec<String>,
    },

    Refresh,
    Status,
    // TODO: Add manual entry in Git API for '--help' flag, since only 'help' and '-h' work
}
