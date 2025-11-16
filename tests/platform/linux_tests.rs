/// Linux-specific tests
///
/// Tests for Linux-specific behavior in C runtime and Rust FFI.
/// These tests only compile on Linux.

#[test]
#[cfg(target_os = "linux")]
fn test_linux_platform_detected() {
    assert!(cfg!(target_os = "linux"));
    println!("Running on Linux");
}

#[test]
#[cfg(not(target_os = "linux"))]
fn test_not_on_linux() {
    // This test ensures the module compiles on other platforms
    // but doesn't run Linux-specific tests
}

// Add Linux-specific tests here as needed
// Examples:
// - /proc filesystem tests
// - Linux-specific syscalls (mmap, etc.)
// - ELF binary format tests
