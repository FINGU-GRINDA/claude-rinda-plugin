use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommands,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Log in with Google OAuth
    Login,
    /// Check authentication status
    Status,
    /// Log out and clear credentials
    Logout,
}

pub async fn run(args: AuthArgs) {
    match args.command {
        AuthCommands::Login => {
            println!("Login not yet implemented");
        }
        AuthCommands::Status => {
            println!("Status not yet implemented");
        }
        AuthCommands::Logout => {
            println!("Logout not yet implemented");
        }
    }
}
