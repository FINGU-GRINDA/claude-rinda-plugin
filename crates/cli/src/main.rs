mod api_helper;
mod commands;
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
    /// Buyer (lead) discovery and enrichment commands
    Buyer(commands::buyer::BuyerArgs),
    /// Customer group CRUD and member management commands
    Group(commands::group::GroupArgs),
    /// Campaign statistics commands
    Campaign(commands::campaign::CampaignArgs),
    /// Configuration commands
    Config(commands::config::ConfigArgs),
    /// Email sending commands
    Email(commands::email::EmailArgs),
    /// Lead management commands (CRUD, search, status, assessment)
    Lead(commands::lead::LeadArgs),
    /// Order history commands (uses leads/search; no orders API exists)
    Order(commands::order::OrderArgs),
    /// Email reply commands
    Reply(commands::reply::ReplyArgs),
    /// Sequence management commands
    Sequence(commands::sequence::SequenceArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Buyer(args) => commands::buyer::run(args).await,
        Commands::Group(args) => commands::group::run(args).await,
        Commands::Campaign(args) => commands::campaign::run(args).await,
        Commands::Config(args) => commands::config::run(args).await,
        Commands::Email(args) => commands::email::run(args).await,
        Commands::Lead(args) => commands::lead::run(args).await,
        Commands::Order(args) => commands::order::run(args).await,
        Commands::Reply(args) => commands::reply::run(args).await,
        Commands::Sequence(args) => commands::sequence::run(args).await,
    }
}
