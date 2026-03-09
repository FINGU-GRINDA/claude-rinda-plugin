// CLI commands for campaign statistics.

use std::process;

use clap::{Args, Subcommand};

use crate::api_helper::{exit_api_error, get_authenticated_client, print_json};

#[derive(Debug, Args)]
pub struct CampaignArgs {
    #[command(subcommand)]
    pub command: CampaignCommands,
}

#[derive(Debug, Subcommand)]
pub enum CampaignCommands {
    /// Retrieve campaign statistics for a period
    Stats {
        /// Period to query (e.g. "7d", "30d", "90d"). Defaults to "30d".
        #[arg(long, default_value = "30d")]
        period: String,
    },
}

/// Parse a period string like "7d", "30d", "90d" into a number of days.
/// Returns `None` if the string cannot be parsed.
pub fn parse_period_days(period: &str) -> Option<i64> {
    let period = period.trim().to_lowercase();
    if let Some(stripped) = period.strip_suffix('d') {
        stripped.parse::<i64>().ok()
    } else if let Some(stripped) = period.strip_suffix('w') {
        stripped.parse::<i64>().ok().map(|w| w * 7)
    } else if let Some(stripped) = period.strip_suffix('m') {
        stripped.parse::<i64>().ok().map(|m| m * 30)
    } else {
        // Try bare number (treat as days).
        period.parse::<i64>().ok()
    }
}

pub async fn run(args: CampaignArgs) {
    let (client, _creds) = get_authenticated_client().await;

    match args.command {
        CampaignCommands::Stats { period } => {
            let days = parse_period_days(&period).unwrap_or(30);

            // Compute start_date and end_date in ISO 8601 format (date only).
            let now = chrono::Utc::now();
            let end_date = now.format("%Y-%m-%d").to_string();
            let start_date = (now - chrono::Duration::days(days))
                .format("%Y-%m-%d")
                .to_string();

            match client
                .get_api_v1_dashboard_stats(
                    Some(&end_date),
                    Some(&start_date),
                    None, // workspace_id
                )
                .await
            {
                Ok(resp) => print_json(&resp.into_inner()),
                Err(e) => exit_api_error("campaign stats failed", e),
            }
        }
    }

    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_period_days_suffix_d() {
        assert_eq!(parse_period_days("7d"), Some(7));
        assert_eq!(parse_period_days("30d"), Some(30));
        assert_eq!(parse_period_days("90d"), Some(90));
    }

    #[test]
    fn parse_period_days_suffix_w() {
        assert_eq!(parse_period_days("2w"), Some(14));
        assert_eq!(parse_period_days("4w"), Some(28));
    }

    #[test]
    fn parse_period_days_suffix_m() {
        assert_eq!(parse_period_days("1m"), Some(30));
        assert_eq!(parse_period_days("3m"), Some(90));
    }

    #[test]
    fn parse_period_days_bare_number() {
        assert_eq!(parse_period_days("14"), Some(14));
    }

    #[test]
    fn parse_period_days_invalid() {
        assert_eq!(parse_period_days(""), None);
        assert_eq!(parse_period_days("xyz"), None);
        assert_eq!(parse_period_days("dd"), None);
    }

    #[test]
    fn parse_period_days_uppercase() {
        // Should work case-insensitively.
        assert_eq!(parse_period_days("7D"), Some(7));
    }
}
