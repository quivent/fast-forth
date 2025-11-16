// Safety guards for destructive testing
// Prevents accidental execution outside containers

use std::env;
use std::fs;
use std::path::Path;

/// Check if running in a Docker container
pub fn is_in_container() -> bool {
    // Method 1: Check for .dockerenv file
    if Path::new("/.dockerenv").exists() {
        return true;
    }

    // Method 2: Check cgroup for docker/containerd
    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        if cgroup.contains("docker") || cgroup.contains("containerd") {
            return true;
        }
    }

    // Method 3: Check environment variable
    if env::var("DESTRUCTIVE_TESTS_ENABLED").is_ok() {
        return true;
    }

    false
}

/// Check if it's safe to run destructive tests
pub fn is_safe_to_run_destructive_tests() -> bool {
    // Must be in container
    if !is_in_container() {
        return false;
    }

    // Check for explicit opt-in
    match env::var("ALLOW_DESTRUCTIVE_TESTS") {
        Ok(val) => val == "1" || val.to_lowercase() == "true",
        Err(_) => {
            // Default to safe if running in container
            true
        }
    }
}

/// Ensure we're in a containerized environment
/// Panics if not safe to run destructive tests
pub fn ensure_containerized() {
    if !is_safe_to_run_destructive_tests() {
        panic!(
            "SAFETY: Destructive tests can only run in containerized environments.\n\
             Use: ./scripts/run_destructive_tests.sh"
        );
    }
}

/// Get memory limit in bytes from cgroup (if available)
pub fn get_memory_limit() -> Option<usize> {
    // Try cgroup v2 first
    if let Ok(limit) = fs::read_to_string("/sys/fs/cgroup/memory.max") {
        if let Ok(bytes) = limit.trim().parse::<usize>() {
            return Some(bytes);
        }
    }

    // Fall back to cgroup v1
    if let Ok(limit) = fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes") {
        if let Ok(bytes) = limit.trim().parse::<usize>() {
            return Some(bytes);
        }
    }

    None
}

/// Get available disk space for testing
pub fn get_available_disk_space(path: &str) -> Option<u64> {
    use std::process::Command;

    let output = Command::new("df")
        .arg("-B1")  // 1-byte blocks
        .arg(path)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    // Parse df output (format: Filesystem 1B-blocks Used Available Use% Mounted)
    let fields: Vec<&str> = lines[1].split_whitespace().collect();
    if fields.len() >= 4 {
        fields[3].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_checks() {
        // These tests should work in any environment
        let in_container = is_in_container();
        println!("In container: {}", in_container);

        let is_safe = is_safe_to_run_destructive_tests();
        println!("Safe for destructive tests: {}", is_safe);

        if let Some(limit) = get_memory_limit() {
            println!("Memory limit: {} bytes", limit);
        }
    }
}
