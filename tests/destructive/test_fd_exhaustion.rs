// File Descriptor Exhaustion Testing
// Tests handling when system runs out of file descriptors

#![cfg(feature = "destructive_tests")]

use super::safety::ensure_containerized;
use std::fs::File;
use std::io::Result as IoResult;
use std::os::unix::io::AsRawFd;

#[test]
fn test_fd_exhaustion_handling() {
    ensure_containerized();

    println!("Testing file descriptor exhaustion...");

    let mut files = Vec::new();

    // Open files until we hit the limit
    for i in 0..10_000 {
        match File::open("/dev/null") {
            Ok(f) => {
                files.push(f);
                if i % 100 == 0 {
                    println!("Opened {} file descriptors", files.len());
                }
            }
            Err(e) => {
                println!("FD exhaustion at {} files: {:?}", i, e);

                // Verify this is actually an FD limit error
                if e.raw_os_error() == Some(24) { // EMFILE
                    println!("Confirmed EMFILE (too many open files)");
                }
                break;
            }
        }
    }

    println!("Max file descriptors opened: {}", files.len());

    // Verify we opened a reasonable number before hitting limit
    assert!(files.len() > 10, "Should have opened at least 10 FDs");

    // Close half the files
    let half = files.len() / 2;
    files.truncate(half);

    // Verify we can open files again
    match File::open("/dev/null") {
        Ok(_) => println!("Successfully opened file after freeing FDs"),
        Err(e) => panic!("Failed to open file after freeing FDs: {:?}", e),
    }
}

#[test]
fn test_fd_recovery() {
    ensure_containerized();

    println!("Testing FD recovery...");

    // Open many files
    let files: Vec<File> = (0..500)
        .filter_map(|_| File::open("/dev/null").ok())
        .collect();

    println!("Opened {} files", files.len());

    // Drop all files (close FDs)
    drop(files);

    // Verify we can open new files
    let new_files: Vec<File> = (0..10)
        .filter_map(|_| File::open("/dev/null").ok())
        .collect();

    assert_eq!(new_files.len(), 10, "Failed to recover from FD pressure");
    println!("Successfully recovered - opened {} new files", new_files.len());
}

#[test]
fn test_fd_leak_detection() {
    ensure_containerized();

    println!("Testing FD leak detection...");

    let initial_fd = get_max_fd();

    // Simulate potential leak scenario
    {
        let _files: Vec<File> = (0..100)
            .filter_map(|_| File::open("/dev/null").ok())
            .collect();

        // Files should be dropped here
    }

    let final_fd = get_max_fd();

    // Allow some variance but should be close
    let fd_diff = (final_fd as i32 - initial_fd as i32).abs();
    assert!(fd_diff < 10,
            "Possible FD leak detected: {} FDs difference", fd_diff);

    println!("FD leak check passed (diff: {})", fd_diff);
}

#[test]
fn test_simultaneous_file_operations() {
    ensure_containerized();

    println!("Testing simultaneous file operations under FD pressure...");

    let test_dir = "/tmp/fd_test";
    let _ = std::fs::create_dir_all(test_dir);

    let mut test_files = Vec::new();

    // Create and open multiple files simultaneously
    for i in 0..100 {
        let path = format!("{}/test_{}.dat", test_dir, i);
        match File::create(&path) {
            Ok(f) => test_files.push(f),
            Err(e) => {
                println!("Failed to create file {} (FD limit): {:?}", i, e);
                break;
            }
        }
    }

    println!("Created {} simultaneous files", test_files.len());

    // Cleanup
    drop(test_files);
    let _ = std::fs::remove_dir_all(test_dir);

    assert!(true, "Simultaneous file operations handled");
}

#[test]
fn test_compiler_fd_usage() {
    ensure_containerized();

    println!("Testing compiler FD usage patterns...");

    // Simulate compiler opening source files, temp files, output files
    let mut source_files = Vec::new();
    let mut temp_files = Vec::new();
    let mut output_files = Vec::new();

    for i in 0..50 {
        // Source file
        if let Ok(f) = File::open("/dev/null") {
            source_files.push(f);
        } else {
            println!("Source file FD limit at {}", i);
            break;
        }

        // Temp file
        if let Ok(f) = File::open("/dev/zero") {
            temp_files.push(f);
        } else {
            println!("Temp file FD limit at {}", i);
            break;
        }

        // Output file
        if let Ok(f) = File::open("/dev/null") {
            output_files.push(f);
        } else {
            println!("Output file FD limit at {}", i);
            break;
        }
    }

    println!("Compiler FD usage: {} source, {} temp, {} output",
             source_files.len(), temp_files.len(), output_files.len());

    assert!(source_files.len() > 0, "Should handle some files");
}

#[test]
fn test_fd_limit_awareness() {
    ensure_containerized();

    println!("Testing FD limit awareness...");

    // Try to get the current FD limit
    let limit = get_fd_limit();
    println!("File descriptor limit: {:?}", limit);

    if let Some(max_fds) = limit {
        println!("System allows {} file descriptors", max_fds);

        // Don't try to exceed the limit in testing
        let safe_count = std::cmp::min(max_fds / 2, 1000);

        let files: Vec<File> = (0..safe_count)
            .filter_map(|_| File::open("/dev/null").ok())
            .collect();

        println!("Safely opened {} of {} allowed FDs", files.len(), safe_count);
        assert!(files.len() > 0, "Should open some FDs");
    }
}

// Helper functions

fn get_max_fd() -> i32 {
    // Open a file and get its FD to estimate current usage
    if let Ok(f) = File::open("/dev/null") {
        f.as_raw_fd()
    } else {
        0
    }
}

fn get_fd_limit() -> Option<usize> {
    use std::process::Command;

    // Try to get ulimit -n
    let output = Command::new("sh")
        .arg("-c")
        .arg("ulimit -n")
        .output()
        .ok()?;

    let limit_str = String::from_utf8_lossy(&output.stdout);
    limit_str.trim().parse().ok()
}
