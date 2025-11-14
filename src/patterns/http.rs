//! HTTP API for pattern queries

use super::{PatternDatabase, PatternQuery, PatternId, Result, PatternError};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// HTTP API configuration
#[derive(Debug, Clone)]
pub struct PatternApiConfig {
    pub host: String,
    pub port: u16,
    pub max_results: usize,
}

impl Default for PatternApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_results: 100,
        }
    }
}

/// Pattern HTTP server
pub struct PatternServer {
    config: PatternApiConfig,
    database: Arc<Mutex<PatternDatabase>>,
}

impl PatternServer {
    /// Create a new pattern server
    pub fn new(config: PatternApiConfig, database: PatternDatabase) -> Self {
        Self {
            config,
            database: Arc::new(Mutex::new(database)),
        }
    }

    /// Get server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    /// Start the server (mock implementation)
    pub async fn start(&self) -> Result<()> {
        println!("Pattern API server starting on {}", self.address());
        println!("Available endpoints:");
        println!("  GET  /patterns - List all patterns");
        println!("  GET  /patterns/:id - Get pattern by ID");
        println!("  POST /patterns/query - Query patterns");
        println!("  GET  /patterns/categories - List categories");
        println!("  GET  /health - Health check");

        // In a real implementation, this would use actix-web, axum, or similar
        // For now, we just document the API structure
        Ok(())
    }
}

/// API request/response types

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub category: Option<String>,
    pub stack_effect: Option<String>,
    pub performance_class: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl From<QueryRequest> for PatternQuery {
    fn from(req: QueryRequest) -> Self {
        Self {
            category: req.category,
            stack_effect: req.stack_effect,
            performance_class: req.performance_class,
            tags: req.tags,
            limit: req.limit,
            offset: req.offset,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub pattern_count: usize,
    pub version: String,
}

/// Mock HTTP handlers (would be actual endpoints in production)
pub mod handlers {
    use super::*;

    /// List all patterns
    pub async fn list_patterns(db: Arc<Mutex<PatternDatabase>>) -> Result<PatternResponse> {
        let db = db.lock().unwrap();
        let patterns = db.list_all()?;

        Ok(PatternResponse {
            success: true,
            data: serde_json::to_value(patterns)?,
            error: None,
        })
    }

    /// Get pattern by ID
    pub async fn get_pattern(
        db: Arc<Mutex<PatternDatabase>>,
        id: PatternId
    ) -> Result<PatternResponse> {
        let db = db.lock().unwrap();

        match db.get(&id)? {
            Some(pattern) => Ok(PatternResponse {
                success: true,
                data: serde_json::to_value(pattern)?,
                error: None,
            }),
            None => Ok(PatternResponse {
                success: false,
                data: serde_json::Value::Null,
                error: Some(format!("Pattern not found: {}", id)),
            }),
        }
    }

    /// Query patterns
    pub async fn query_patterns(
        db: Arc<Mutex<PatternDatabase>>,
        query: PatternQuery
    ) -> Result<PatternResponse> {
        let db = db.lock().unwrap();
        let patterns = db.query(&query)?;

        Ok(PatternResponse {
            success: true,
            data: serde_json::to_value(patterns)?,
            error: None,
        })
    }

    /// Health check
    pub async fn health_check(db: Arc<Mutex<PatternDatabase>>) -> Result<HealthResponse> {
        let db = db.lock().unwrap();
        let count = db.count()?;

        Ok(HealthResponse {
            status: "healthy".to_string(),
            pattern_count: count,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::database::PatternDatabase;

    #[test]
    fn test_server_creation() {
        let config = PatternApiConfig::default();
        let db = PatternDatabase::open("test.db").unwrap();
        let server = PatternServer::new(config, db);

        assert_eq!(server.address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_query_request_conversion() {
        let req = QueryRequest {
            category: Some("recursive".to_string()),
            stack_effect: None,
            performance_class: None,
            tags: vec![],
            limit: Some(10),
            offset: None,
        };

        let query: PatternQuery = req.into();
        assert_eq!(query.category, Some("recursive".to_string()));
        assert_eq!(query.limit, Some(10));
    }
}
