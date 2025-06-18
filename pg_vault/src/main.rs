use pg_vault::prelude::*;
use pg_vault::vault::{DatabaseConfig, Vault, SslMode};
use std::sync::Arc;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("pg_vault - Secure PostgreSQL with Hardware Token Authentication");
    
    // Get password from environment variable or prompt
    let password = std::env::var("DB_PASSWORD").ok();
    
    if password.is_none() {
        println!("⚠️  No DB_PASSWORD environment variable set. Run with:");
        println!("   DB_PASSWORD=\"your_password\" cargo run --bin pg_vault");
        println!("   Or set the password in your environment:");
        println!("   export DB_PASSWORD=\"your_password\"");
        return Ok(());
    }
    
    // Test direct connection first
    println!("🧪 Testing direct PostgreSQL connection...");
    let direct_conn_string = format!(

        "host=localhost port=5432 user=dweese dbname=frodo sslmode=prefer{}",
        password.as_ref().map(|p| format!(" password={}", p)).unwrap_or_default()
    );
    
    match tokio_postgres::connect(&direct_conn_string, NoTls).await {
        Ok((client, connection)) => {
            println!("✅ Direct connection successful!");
            
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });
            
            match client.query("SELECT version()", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let version: String = row.get(0);
                        println!("PostgreSQL Version: {}", version);
                    }
                }
                Err(e) => println!("Query failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Direct connection failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test hardware authentication
    let auth_provider: Arc<dyn YubikeyAuth> = Arc::new(MockYubikey::new());
    
    println!("\n🔐 Testing Hardware Token Authentication:");
    println!("Hardware token present: {}", auth_provider.is_present());
    println!("Requires touch: {}", auth_provider.requires_touch());
    if let Some(serial) = auth_provider.serial_number() {
        println!("Token serial: {}", serial);
    }
    
    let challenge = b"test_challenge";
    match auth_provider.challenge_response(challenge) {
        Ok(response) => {
            println!("Challenge-response successful: {} bytes", response.len());
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Create database configuration with password
    let db_config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "frodo".to_string(),
        username: "dweese".to_string(),
        password: password.clone(), // ← This was missing!
        connect_timeout: 30,
        query_timeout: 60,
        ssl_mode: SslMode::Prefer,
        application_name: Some("pg_vault".to_string()),
    };
    
    let vault = Vault::new_with_mock(db_config);
    
    println!("\n🔐 Attempting secure vault connection...");
    
    match vault.connect().await {
        Ok(secure_conn) => {
            println!("✅ Secure connection established!");
            println!("Session ID: {}", secure_conn.session_id());
            
            let client = secure_conn.client();
            match client.query("SELECT current_user, current_database()", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let user: String = row.get(0);
                        let db: String = row.get(1);
                        println!("Connected as: {} to database: {}", user, db);
                    }
                }
                Err(e) => println!("Query failed: {}", e),
            }
            
            // Test a timestamp query with string format
            match client.query("SELECT current_timestamp::text", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let timestamp: String = row.get(0);
                        println!("Connection time: {}", timestamp);
                    }
                }
                Err(e) => println!("Timestamp query failed: {}", e),
            }
            
            println!("\n🎯 pg_vault is ready for secure operations!");
            
            // Show vault capabilities
            println!("\n📊 Vault Status:");
            println!("Token present: {}", vault.is_token_present());
            println!("Active sessions: {}", vault.session_count().await);
            
            if let Some(token_info) = vault.auth_info() {
                println!("Token model: {}", token_info.model);
                println!("Touch required: {}", token_info.touch_required);
            }
            
            // Demonstrate session
        }
        Err(e) => {
            println!("❌ Vault connection failed: {}", e);
        }
    }
    
    Ok(())
}