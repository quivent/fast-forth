/// Tests for server feature
///
/// These tests only compile when the 'server' feature is enabled.
/// Run with: cargo test --features server

#[test]
fn test_server_feature_enabled() {
    assert!(
        cfg!(feature = "server"),
        "server feature should be enabled"
    );
}

#[test]
fn test_server_module_available() {
    // Verify server module is accessible when feature is enabled
    let _ = fastforth::server::PatternServer::default();
}

#[test]
fn test_server_depends_on_inference() {
    // Server feature should enable inference feature
    assert!(
        cfg!(feature = "inference"),
        "server feature should enable inference feature"
    );
}

#[test]
fn test_server_config_creation() {
    use fastforth::server::PatternApiConfig;

    let config = PatternApiConfig::default();

    // Verify default configuration
    assert!(config.host.contains("127.0.0.1") || config.host.contains("localhost"));
    assert!(config.port > 0 && config.port < 65536);
}

#[test]
fn test_pattern_server_initialization() {
    use fastforth::server::PatternServer;

    let server = PatternServer::default();

    // Server should initialize without panic
    println!("Pattern server initialized successfully");
}

#[test]
#[cfg(not(feature = "server"))]
fn test_server_feature_disabled_compilation_should_fail() {
    // This test should never compile when server feature is disabled
    // If this compiles, the cfg gate is broken
    compile_error!("This test should only compile with server feature enabled");
}
