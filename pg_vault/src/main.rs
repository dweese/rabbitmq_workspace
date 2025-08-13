//! pg_vault CLI
//!
//! A command-line interface for interacting with the secure PostgreSQL vault.

use clap::{Parser, Subcommand};
use log::{error, info};
use pg_vault::prelude::*;

#[derive(Parser)]
#[command(name = "pg-vault-cli", version, about)]
#[command(
    long_about = "A command-line tool to interact with a pg_vault secure database connection."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for the presence of a hardware token.
    Check,
    /// Get information about the connected hardware token.
    Info,
    /// Test the database connection through the vault.
    TestConnection {
        /// The PIN for the hardware token.
        #[arg(long, env = "PG_VAULT_PIN")]
        pin: String,
    },
    /// Execute a SQL query through a secure vault connection.
    Query {
        /// The PIN for the hardware token.
        #[arg(long, env = "PG_VAULT_PIN")]
        pin: String,
        /// The SQL query to execute.
        #[arg()]
        sql: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // For simplicity, use default configs. In a real app, these would be loaded from a file.
    let db_config = DatabaseConfig::default();
    let auth_config = AuthConfig::default();
    let vault_config = VaultConfig::default();

    // Create the vault. This will use a real YubiKey if present and the feature is enabled,
    // otherwise it will fall back to the mock provider.
    let vault = Vault::new_with_hardware(auth_config, db_config, vault_config)?;

    match cli.command {
        Commands::Check => {
            if vault.is_token_present() {
                println!("✅ Hardware token is present.");
            } else {
                println!("❌ No hardware token found.");
            }
        }
        Commands::Info => {
            if let Some(info) = vault.auth_info() {
                println!("✅ Token Information:");
                println!("  Model: {}", info.model);
                if let Some(serial) = info.serial {
                    println!("  Serial: {}", serial);
                }
                if let Some(version) = info.version {
                    println!("  Version: {}", version);
                }
                println!("  Touch Required: {}", info.touch_required);
            } else {
                println!("❌ Could not retrieve token information. Is a token present?");
            }
        }
        Commands::TestConnection { pin } => {
            info!("Attempting to establish a secure connection...");
            // Use `?` to propagate errors cleanly.
            let _conn = vault.connect(&pin).await?;
            println!("✅ Secure connection successful!");
            println!("  Session created and will be automatically cleaned up.");
        }
        Commands::Query { pin, sql } => {
            info!("Attempting to execute query: \"{}\"", sql);
            let secure_conn = vault.connect(&pin).await?;
            let client = secure_conn.client();

            let rows = client.query(&sql, &[]).await?;
            println!("✅ Query successful. {} rows returned.", rows.len());
            // A real CLI would format this nicely (e.g., as a table)
            for row in rows {
                println!("{:?}", row);
            }
        }
    }

    Ok(())
}