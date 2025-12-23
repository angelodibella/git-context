mod commands;
mod config;
mod ops;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init { context } => ops::init(&context)?,
        Commands::Switch { context } => ops::switch(&context)?,
        Commands::New { context } => ops::new(&context)?,
        Commands::Keep { path } => ops::keep(&path)?,
        Commands::Unkeep { path } => ops::unkeep(&path)?,
        Commands::Exec { context, args } => ops::exec(&context, args)?,
        Commands::Refresh => todo!(),
        Commands::Status => ops::status()?,
    }

    Ok(())
}
