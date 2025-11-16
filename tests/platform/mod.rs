//! Platform-specific and feature flag test modules
//!
//! This module contains tests for:
//! - Platform-specific code paths (OS, architecture)
//! - Feature flag combinations (cranelift, llvm, server, inference)
//! - Backend selection logic

// Feature flag tests (always available, conditional compilation inside)
mod backend_selection_tests;

#[cfg(feature = "cranelift")]
mod cranelift_tests;

#[cfg(feature = "llvm")]
mod llvm_tests;

#[cfg(feature = "server")]
mod server_tests;

#[cfg(feature = "inference")]
mod inference_tests;

// Platform-specific tests (conditional compilation based on target)
// Note: These test platform-specific behavior in C runtime and Rust FFI

#[cfg(target_os = "linux")]
pub mod linux_tests;

#[cfg(target_os = "macos")]
pub mod macos_tests;

#[cfg(target_os = "windows")]
pub mod windows_tests;

#[cfg(target_arch = "x86_64")]
pub mod x86_64_tests;

#[cfg(target_arch = "aarch64")]
pub mod aarch64_tests;
