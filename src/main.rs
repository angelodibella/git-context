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
        Commands::New { name } => println!("New context with name: {}", name),
        Commands::Keep { path } => println!("Kept file at path: {}", path),
        Commands::Unkeep { path } => println!("Unkept file at path: {}", path),
        Commands::Exec { context, args } => {
            println!(
                "Executed command '{}' in context: {}",
                args.join(" "),
                context
            )
        }
        Commands::Refresh => println!("Switch called with name: "),
        Commands::Status => println!("Switch called with name:"),
    }

    Ok(())
}
