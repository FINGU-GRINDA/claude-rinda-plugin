mod commands;
mod credentials;
mod oauth;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rinda", about = "Rinda AI CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Authentication commands
    Auth(commands::auth::AuthArgs),
    /// Configuration commands
    Config(commands::config::ConfigArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Config(args) => commands::config::run(args).await,
    }
}
