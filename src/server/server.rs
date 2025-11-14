//! Async verification server implementation

use crate::inference::InferenceAPI;
use std::net::SocketAddr;
use std::sync::Arc;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: num_cpus::get(),
        }
    }
}

/// Real-time verification server
pub struct VerificationServer {
    config: ServerConfig,
    api: Arc<InferenceAPI>,
}

impl VerificationServer {
    /// Create a new verification server
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            api: Arc::new(InferenceAPI::new()),
        }
    }

    /// Start the server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Invalid server address");

        println!("Fast Forth Verification Server starting...");
        println!("  Address: {}", addr);
        println!("  Workers: {}", self.config.workers);
        println!("\nEndpoints:");
        println!("  POST /verify       - Verify code against stack effect");
        println!("  POST /infer        - Infer stack effect from code");
        println!("  POST /compose      - Verify composition of words");
        println!("  GET  /health       - Health check");
        println!();

        #[cfg(feature = "server")]
        {
            use axum::{
                routing::{get, post},
                Router,
            };
            use super::routes;

            let app = Router::new()
                .route("/health", get(routes::health))
                .route("/verify", post(routes::verify))
                .route("/infer", post(routes::infer))
                .route("/compose", post(routes::compose))
                .with_state(self.api);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            println!("✓ Server listening on {}", addr);
            axum::serve(listener, app).await?;
        }

        #[cfg(not(feature = "server"))]
        {
            eprintln!("Error: Server feature not enabled. Build with --features server");
            std::process::exit(1);
        }

        Ok(())
    }

    /// Get the server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = VerificationServer::new(config);
        assert_eq!(server.address(), "127.0.0.1:8080");
    }
}
