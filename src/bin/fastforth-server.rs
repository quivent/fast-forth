//! Fast Forth Verification Server Binary
//!
//! Real-time stack effect verification server with sub-millisecond latency.
//!
//! Usage:
//!   fastforth-server --port 8080
//!
//! Endpoints:
//!   POST /verify  - Verify code against expected stack effect
//!   POST /infer   - Infer stack effect from code
//!   POST /compose - Verify composition of words
//!   GET  /health  - Health check

use clap::Parser;

#[cfg(feature = "server")]
use fastforth::server::{VerificationServer, ServerConfig};

#[derive(Parser)]
#[command(name = "fastforth-server")]
#[command(about = "Fast Forth Real-Time Verification Server", long_about = None)]
#[command(version)]
struct Cli {
    /// Server host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Number of worker threads (0 = auto)
    #[arg(short, long, default_value = "0")]
    workers: usize,
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = ServerConfig {
        host: cli.host,
        port: cli.port,
        workers: if cli.workers == 0 {
            num_cpus::get()
        } else {
            cli.workers
        },
    };

    let server = VerificationServer::new(config);
    server.start().await
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("Error: Server feature not enabled.");
    eprintln!("Build with: cargo build --bin fastforth-server --features server");
    std::process::exit(1);
}
