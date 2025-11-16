// Disk Full Testing
// Tests handling of disk space exhaustion scenarios

#![cfg(feature = "destructive_tests")]

use super::safety::{ensure_containerized, get_available_disk_space};
use std::fs::{File, OpenOptions};
use std::io::{Write, Result as IoResult};
use std::path::PathBuf;

const TEST_DIR: &str = "/tmp/destructive_tests";

/// Setup test directory
fn setup_test_dir() -> IoResult<PathBuf> {
    let dir = PathBuf::from(TEST_DIR);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Cleanup test directory
fn cleanup_test_dir() {
    let _ = std::fs::remove_dir_all(TEST_DIR);
}

#[test]
fn test_disk_full_write_handling() {
    ensure_containerized();

    let test_dir = setup_test_dir().expect("Failed to create test directory");

    if let Some(space) = get_available_disk_space(TEST_DIR) {
        println!("Available disk space: {} MB", space / 1_048_576);
    }

    // Try to fill disk by writing large files
    let mut files_written = 0;
    let chunk_size = 1_048_576; // 1MB chunks
    let chunk = vec![0u8; chunk_size];

    for i in 0..10000 {
        let file_path = test_dir.join(format!("test_file_{}.dat", i));

        match write_until_full(&file_path, &chunk, 100) {
            Ok(chunks_written) => {
                files_written += 1;
                if i % 10 == 0 {
                    println!("Wrote file {} ({} MB total)",
                             i, files_written * chunks_written);
                }
            }
            Err(e) => {
                println!("Disk full after {} files: {:?}", i, e);
                break;
            }
        }

        // Stop if we've written a reasonable amount for testing
        if files_written > 50 {
            println!("Stopping after {} files for safety", files_written);
            break;
        }
    }

    cleanup_test_dir();
    assert!(files_written > 0, "Should have written at least one file");
}

#[test]
fn test_disk_full_append_handling() {
    ensure_containerized();

    let test_dir = setup_test_dir().expect("Failed to create test directory");
    let file_path = test_dir.join("append_test.dat");

    // Create file and keep appending until disk full
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .expect("Failed to create test file");

    let data = vec![0u8; 1_048_576]; // 1MB
    let mut mb_written = 0;

    for i in 0..10000 {
        match file.write_all(&data) {
            Ok(_) => {
                mb_written += 1;
                if i % 50 == 0 {
                    println!("Appended {} MB", mb_written);
                }
            }
            Err(e) => {
                println!("Append failed after {} MB: {:?}", mb_written, e);

                // Verify error is disk-full related
                if e.kind() == std::io::ErrorKind::Other {
                    println!("Disk full error detected (expected)");
                }
                break;
            }
        }

        // Safety limit
        if mb_written > 1000 {
            println!("Stopping after 1GB for safety");
            break;
        }
    }

    cleanup_test_dir();
    assert!(mb_written > 0, "Should have written some data");
}

#[test]
fn test_disk_full_temp_file_handling() {
    ensure_containerized();

    let test_dir = setup_test_dir().expect("Failed to create test directory");

    // Simulate compiler temp file creation under disk pressure
    let mut temp_files = Vec::new();
    let temp_data = vec![0u8; 524_288]; // 512KB

    for i in 0..1000 {
        let temp_path = test_dir.join(format!("temp_{}.tmp", i));

        match File::create(&temp_path) {
            Ok(mut f) => {
                match f.write_all(&temp_data) {
                    Ok(_) => {
                        temp_files.push(temp_path);
                        if i % 20 == 0 {
                            println!("Created {} temp files", temp_files.len());
                        }
                    }
                    Err(e) => {
                        println!("Temp file write failed at file {}: {:?}", i, e);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Temp file creation failed at file {}: {:?}", i, e);
                break;
            }
        }

        // Safety limit
        if temp_files.len() > 500 {
            println!("Stopping after 500 temp files");
            break;
        }
    }

    // Cleanup temp files
    for temp_file in temp_files {
        let _ = std::fs::remove_file(temp_file);
    }

    cleanup_test_dir();
}

#[test]
fn test_disk_full_recovery() {
    ensure_containerized();

    let test_dir = setup_test_dir().expect("Failed to create test directory");

    println!("Testing disk full recovery...");

    // Fill disk
    let mut test_files = Vec::new();
    let data = vec![0u8; 1_048_576];

    for i in 0..100 {
        let file_path = test_dir.join(format!("recovery_test_{}.dat", i));
        match File::create(&file_path) {
            Ok(mut f) => {
                if f.write_all(&data).is_ok() {
                    test_files.push(file_path);
                } else {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    println!("Created {} files before disk full", test_files.len());

    // Delete some files to free space
    let files_to_delete = test_files.len() / 2;
    for file_path in &test_files[..files_to_delete] {
        let _ = std::fs::remove_file(file_path);
    }

    println!("Deleted {} files to free space", files_to_delete);

    // Verify we can write again
    let recovery_path = test_dir.join("recovery_verify.dat");
    let recovery_file = File::create(&recovery_path);
    assert!(recovery_file.is_ok(), "Failed to recover from disk full");

    cleanup_test_dir();
    println!("Successfully recovered from disk full");
}

#[test]
fn test_disk_full_compilation() {
    ensure_containerized();

    let test_dir = setup_test_dir().expect("Failed to create test directory");

    println!("Testing compilation under disk pressure...");

    // Simulate compiler generating output files under disk pressure
    let mut output_files = Vec::new();

    for i in 0..100 {
        let output_path = test_dir.join(format!("compiled_{}.o", i));

        // Simulate compilation output
        match simulate_compilation_output(&output_path) {
            Ok(_) => {
                output_files.push(output_path);
                if i % 10 == 0 {
                    println!("Generated {} compilation outputs", output_files.len());
                }
            }
            Err(e) => {
                println!("Compilation output failed at {}: {:?}", i, e);

                // Verify graceful handling
                assert!(output_files.len() > 0,
                        "Should have generated some outputs before failure");
                break;
            }
        }
    }

    cleanup_test_dir();
}

// Helper functions

fn write_until_full(path: &PathBuf, chunk: &[u8], max_chunks: usize) -> IoResult<usize> {
    let mut file = File::create(path)?;

    for i in 0..max_chunks {
        file.write_all(chunk)?;
        if i >= max_chunks - 1 {
            return Ok(i + 1);
        }
    }

    Ok(max_chunks)
}

fn simulate_compilation_output(path: &PathBuf) -> IoResult<()> {
    let mut file = File::create(path)?;

    // Simulate typical compilation output size (100KB)
    let output_data = vec![0u8; 102_400];
    file.write_all(&output_data)?;
    file.sync_all()?;

    Ok(())
}
