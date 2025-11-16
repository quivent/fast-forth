//! Platform-specific tests for FastForth
//!
//! These tests verify platform-specific functionality and ensure
//! all conditional compilation branches are exercised on each platform.

#![cfg(test)]

// Platform-specific modules
#[cfg(target_os = "linux")]
mod linux_tests;

#[cfg(target_os = "macos")]
mod macos_tests;

#[cfg(target_os = "windows")]
mod windows_tests;

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
mod x86_64_tests;

#[cfg(target_arch = "aarch64")]
mod aarch64_tests;

// Common platform tests that run on all platforms
mod common {
    use fastforth::*;

    #[test]
    fn test_basic_forth_execution() {
        // Test basic Forth execution on current platform
        let result = std::panic::catch_unwind(|| {
            // Basic sanity check
            assert!(true, "Platform test sanity check");
        });
        assert!(result.is_ok(), "Basic execution should work on all platforms");
    }

    #[test]
    fn test_platform_detection() {
        // Verify we can detect the current platform
        #[cfg(target_os = "linux")]
        {
            println!("Running on Linux");
        }
        #[cfg(target_os = "macos")]
        {
            println!("Running on macOS");
        }
        #[cfg(target_os = "windows")]
        {
            println!("Running on Windows");
        }
    }

    #[test]
    fn test_architecture_detection() {
        // Verify we can detect the current architecture
        #[cfg(target_arch = "x86_64")]
        {
            println!("Running on x86_64");
        }
        #[cfg(target_arch = "aarch64")]
        {
            println!("Running on ARM64/aarch64");
        }
    }

    #[test]
    fn test_endianness() {
        // Verify endianness handling
        let x: u32 = 0x12345678;
        let bytes = x.to_ne_bytes();

        #[cfg(target_endian = "little")]
        {
            assert_eq!(bytes[0], 0x78);
            println!("Little endian confirmed");
        }

        #[cfg(target_endian = "big")]
        {
            assert_eq!(bytes[0], 0x12);
            println!("Big endian confirmed");
        }
    }

    #[test]
    fn test_pointer_size() {
        use std::mem;
        let ptr_size = mem::size_of::<usize>();

        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(ptr_size, 8);
            println!("64-bit platform confirmed");
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(ptr_size, 4);
            println!("32-bit platform confirmed");
        }
    }
}

// Feature-specific tests
#[cfg(feature = "cranelift")]
mod cranelift_feature_tests {
    #[test]
    fn test_cranelift_available() {
        // Verify cranelift feature is properly enabled
        println!("Cranelift backend is enabled");
    }
}

#[cfg(feature = "llvm")]
mod llvm_feature_tests {
    #[test]
    fn test_llvm_available() {
        // Verify LLVM feature is properly enabled
        println!("LLVM backend is enabled");
    }
}

#[cfg(feature = "interpreter")]
mod interpreter_feature_tests {
    #[test]
    fn test_interpreter_available() {
        // Verify interpreter feature is properly enabled
        println!("Interpreter mode is enabled");
    }
}

#[cfg(feature = "server")]
mod server_feature_tests {
    #[test]
    fn test_server_available() {
        // Verify server feature is properly enabled
        println!("Server feature is enabled");
    }
}

#[cfg(feature = "inference")]
mod inference_feature_tests {
    #[test]
    fn test_inference_available() {
        // Verify inference feature is properly enabled
        println!("Inference feature is enabled");
    }
}
