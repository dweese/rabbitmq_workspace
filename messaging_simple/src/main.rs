use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Simple messaging CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Connect {
        #[arg(short, long)]
        protocol: String,
        #[arg(long, default_value = "localhost")]  
        host: String,
        #[arg(short = 'P', long)]
        port: u16,
    },
    Reload {
        #[arg(long, default_value = "config.json")]
        config: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Connect { protocol, host, port } => {
            println!("Connecting to {}://{}:{}", protocol, host, port);
        }
        Commands::Reload { config } => {
            println!("Reloading config from {}", config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connect_parsing() {
        let cli = Cli::try_parse_from([
            "app", "connect", "--protocol", "amqp", "-P", "5672"
        ]).unwrap();
        
        if let Commands::Connect { protocol, port, host } = cli.command {
            assert_eq!(protocol, "amqp");
            assert_eq!(port, 5672);
            assert_eq!(host, "localhost");
        } else {
            panic!("Expected Connect command");
        }
    }
}
