mod commands;
mod config;

use clap::Parser;
use commands::{Cli, Commands};

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init { name } => println!("Init called with name: {}", name),
        Commands::Switch { name } => println!("Switch called with name: {}", name),

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
}
