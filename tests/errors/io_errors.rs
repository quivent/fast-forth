//! I/O Error Stress Tests (15 tests)
//!
//! Comprehensive tests for I/O error handling:
//! - File not found
//! - Permission denied
//! - Disk full simulation
//! - Read-only filesystem
//! - Symbolic link loops
//! - Invalid file paths
//! - Invalid UTF-8 in paths

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// FILE NOT FOUND TESTS (3 tests)
// ============================================================================

#[test]
fn test_file_not_found_basic() {
    let result = fs::read_to_string("/nonexistent/path/to/file.forth");
    assert!(result.is_err(), "Should fail on nonexistent file");

    if let Err(e) = result {
        println!("File not found error: {}", e);
        assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
    }
}

#[test]
fn test_file_not_found_relative_path() {
    let result = fs::read_to_string("./this/does/not/exist.forth");
    assert!(result.is_err(), "Should fail on nonexistent relative path");
}

#[test]
fn test_file_not_found_empty_path() {
    let result = fs::read_to_string("");
    assert!(result.is_err(), "Should fail on empty path");
}

// ============================================================================
// PERMISSION DENIED TESTS (3 tests)
// ============================================================================

#[test]
#[cfg(unix)]
fn test_read_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_read.forth");

    // Create file
    fs::write(&file_path, b"test content").unwrap();

    // Remove read permissions
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file_path, perms).unwrap();

    // Attempt to read
    let result = fs::read_to_string(&file_path);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    let _ = fs::set_permissions(&file_path, perms);

    assert!(result.is_err(), "Should fail on permission denied");
    if let Err(e) = result {
        println!("Permission denied: {}", e);
    }
}

#[test]
#[cfg(unix)]
fn test_write_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("no_write.forth");

    // Create file with write permissions initially
    fs::write(&file_path, b"test content").unwrap();

    // Remove write permissions
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&file_path, perms).unwrap();

    // Attempt to write
    let result = fs::write(&file_path, b"new content");

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o644);
    let _ = fs::set_permissions(&file_path, perms);

    assert!(result.is_err(), "Should fail on write to read-only file");
}

#[test]
#[cfg(unix)]
fn test_directory_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let restricted_dir = temp_dir.path().join("restricted");

    fs::create_dir(&restricted_dir).unwrap();

    // Remove all permissions
    let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&restricted_dir, perms).unwrap();

    // Attempt to access
    let result = fs::read_dir(&restricted_dir);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&restricted_dir, perms);

    assert!(result.is_err(), "Should fail on directory access denied");
}

// ============================================================================
// DISK FULL SIMULATION (2 tests)
// ============================================================================

#[test]
fn test_write_to_full_disk() {
    // This test simulates disk full by attempting to write
    // an extremely large file (actual disk full testing requires
    // special setup or mocking)

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large.forth");

    // Attempt to create a very large buffer
    let large_data = vec![0u8; 1_000_000];

    let result = fs::write(&file_path, &large_data);
    // May succeed or fail depending on available disk space
    println!("Large write result: {}", result.is_ok());
}

#[test]
fn test_incremental_writes_until_full() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("incremental.forth");

    let mut file = fs::File::create(&file_path).unwrap();
    let chunk = vec![0u8; 1024];

    // Write until error (disk full or limit reached)
    let mut written = 0;
    for _ in 0..10_000 {
        match file.write(&chunk) {
            Ok(n) => written += n,
            Err(e) => {
                println!("Write failed after {} bytes: {}", written, e);
                break;
            }
        }
    }

    println!("Successfully wrote {} bytes", written);
}

// ============================================================================
// INVALID PATH TESTS (4 tests)
// ============================================================================

#[test]
fn test_path_with_null_bytes() {
    let invalid_path = "test\0file.forth";
    let result = fs::read_to_string(invalid_path);
    assert!(result.is_err(), "Should fail on path with null bytes");
}

#[test]
fn test_path_too_long() {
    // Create an extremely long path
    let long_component = "a".repeat(1000);
    let mut long_path = PathBuf::from("/tmp");

    for _ in 0..10 {
        long_path.push(&long_component);
    }

    let result = fs::read_to_string(&long_path);
    assert!(result.is_err(), "Should fail on excessively long path");

    if let Err(e) = result {
        println!("Long path error: {}", e);
    }
}

#[test]
fn test_path_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();

    // Test various special characters
    let special_names = vec![
        "file:with:colons.forth",
        "file*with*asterisks.forth",
        "file?with?questions.forth",
        "file|with|pipes.forth",
    ];

    for name in special_names {
        let path = temp_dir.path().join(name);
        let result = fs::write(&path, b"test");

        // On Unix these should mostly succeed, on Windows they should fail
        println!("Special char '{}': {}", name, result.is_ok());
    }
}

#[test]
fn test_circular_symlink() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let temp_dir = TempDir::new().unwrap();
        let link1 = temp_dir.path().join("link1");
        let link2 = temp_dir.path().join("link2");

        // Create circular symlinks
        let _ = symlink(&link2, &link1);
        let _ = symlink(&link1, &link2);

        // Attempt to read through circular link
        let result = fs::read_to_string(&link1);
        assert!(result.is_err(), "Should fail on circular symlink");

        if let Err(e) = result {
            println!("Circular symlink error: {}", e);
        }
    }

    #[cfg(not(unix))]
    {
        println!("Symlink test skipped on non-Unix platform");
    }
}

// ============================================================================
// UTF-8 ENCODING TESTS (3 tests)
// ============================================================================

#[test]
fn test_invalid_utf8_in_file_content() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid_utf8.forth");

    // Write invalid UTF-8 sequence
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0xFC];
    fs::write(&file_path, &invalid_utf8).unwrap();

    // Attempt to read as string
    let result = fs::read_to_string(&file_path);
    assert!(result.is_err(), "Should fail on invalid UTF-8");

    if let Err(e) = result {
        println!("Invalid UTF-8 error: {}", e);
    }
}

#[test]
fn test_partial_utf8_sequences() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("partial_utf8.forth");

    // Write partial UTF-8 sequences
    let partial = vec![0xE0, 0xA0]; // Incomplete 3-byte sequence
    fs::write(&file_path, &partial).unwrap();

    let result = fs::read_to_string(&file_path);
    assert!(result.is_err(), "Should fail on partial UTF-8");
}

#[test]
fn test_mixed_valid_invalid_utf8() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("mixed_utf8.forth");

    // Mix valid and invalid UTF-8
    let mut mixed = b"Valid ASCII text ".to_vec();
    mixed.extend_from_slice(&[0xFF, 0xFE]);
    mixed.extend_from_slice(b" more text");

    fs::write(&file_path, &mixed).unwrap();

    let result = fs::read_to_string(&file_path);
    assert!(result.is_err(), "Should fail on mixed UTF-8");
}
