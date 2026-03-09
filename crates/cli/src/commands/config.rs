use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
}

pub async fn run(args: ConfigArgs) {
    match args.command {
        ConfigCommands::Show => {
            println!("Config show not yet implemented");
        }
    }
}
