use std::process;

use clap::{Args, Subcommand};

use rinda_common::config::{RindaConfig, base_url};

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set the environment (alpha or beta)
    Set {
        /// Environment name: "alpha" or "beta"
        #[arg(long)]
        env: String,
    },
}

pub async fn run(args: ConfigArgs) {
    match args.command {
        ConfigCommands::Show => {
            let config = RindaConfig::load();
            println!("Environment:  {}", config.env);
            println!("Base URL:     {}", base_url());
        }

        ConfigCommands::Set { env } => {
            if env != "alpha" && env != "beta" {
                eprintln!("Invalid env: \"{env}\". Use \"alpha\" or \"beta\".");
                process::exit(1);
            }

            let mut config = RindaConfig::load();
            config.env = env.clone();
            if let Err(e) = config.save() {
                eprintln!("Error saving config: {e}");
                process::exit(1);
            }
            let url = match env.as_str() {
                "alpha" => "https://alpha.rinda.ai",
                _ => "https://app.rinda.ai",
            };
            println!("Environment set to: {env}");
            println!("Base URL:           {url}");
        }
    }
}
