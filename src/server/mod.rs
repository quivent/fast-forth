//! Real-Time Verification Server
//!
//! High-performance async server for stack effect verification.
//! Target: <1ms latency, 10,000+ requests/sec

pub mod routes;
pub mod server;

pub use server::{VerificationServer, ServerConfig};
