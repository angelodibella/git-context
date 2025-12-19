mod commands;
mod config;
mod ops;

use anyhow::{Ok, Result};
use clap::Parser;
use commands::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init { name } => ops::init(&name)?,
        Commands::Switch { name } => ops::switch(&name)?,
        Commands::New { name } => ops::new(&name)?,
        Commands::Keep { path } => ops::keep(&path)?,
        Commands::Unkeep { path } => ops::unkeep(&path)?,
        Commands::Exec { context, args } => ops::exec(&context, args)?,
        Commands::Refresh => println!("Switch called with name: "),
        Commands::Status => ops::status()?,
    }

    Ok(())
}
