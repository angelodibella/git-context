use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "git-context")]
#[command(about = "A Git extension for managing multiple repositories...")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init { name: String },
    New { name: String },
    Switch { name: String },
    Keep { path: String },
    Unkeep { path: String },
    Exec { context: String, args: Vec },
    Refresh,
    Status,
}
