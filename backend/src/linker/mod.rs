//! Linker Infrastructure
//!
//! Links object files with runtime library to create executable

use crate::error::{BackendError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Link mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Static linking
    Static,
    /// Dynamic linking
    Dynamic,
}

/// Linker configuration
pub struct LinkerConfig {
    /// Link mode
    pub mode: LinkMode,

    /// Runtime library path
    pub runtime_lib: PathBuf,

    /// Additional library search paths
    pub lib_paths: Vec<PathBuf>,

    /// Additional libraries to link
    pub libs: Vec<String>,

    /// Output executable path
    pub output: PathBuf,

    /// Optimization level
    pub optimize: bool,

    /// Strip symbols
    pub strip: bool,

    /// Generate position-independent executable
    pub pie: bool,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            mode: LinkMode::Static,
            runtime_lib: PathBuf::from("runtime/forth_runtime.c"),
            lib_paths: Vec::new(),
            libs: vec!["c".to_string(), "m".to_string()],
            output: PathBuf::from("a.out"),
            optimize: true,
            strip: false,
            pie: true,
        }
    }
}

/// Linker implementation
pub struct Linker {
    config: LinkerConfig,
}

impl Linker {
    /// Create a new linker with configuration
    pub fn new(config: LinkerConfig) -> Self {
        Self { config }
    }

    /// Link object files to create executable
    pub fn link(&self, object_files: &[PathBuf]) -> Result<PathBuf> {
        match self.detect_linker() {
            LinkerType::Gcc => self.link_with_gcc(object_files),
            LinkerType::Clang => self.link_with_clang(object_files),
            LinkerType::Ld => self.link_with_ld(object_files),
        }
    }

    /// Link with GCC
    fn link_with_gcc(&self, object_files: &[PathBuf]) -> Result<PathBuf> {
        let mut cmd = Command::new("gcc");

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Add runtime library
        if self.config.runtime_lib.exists() {
            cmd.arg(&self.config.runtime_lib);
        }

        // Add library paths
        for path in &self.config.lib_paths {
            cmd.arg(format!("-L{}", path.display()));
        }

        // Add libraries
        for lib in &self.config.libs {
            cmd.arg(format!("-l{}", lib));
        }

        // Output file
        cmd.arg("-o").arg(&self.config.output);

        // Optimization
        if self.config.optimize {
            cmd.arg("-O2");
        }

        // Strip symbols
        if self.config.strip {
            cmd.arg("-s");
        }

        // PIE
        if self.config.pie {
            cmd.arg("-pie");
        }

        // Static/dynamic linking
        match self.config.mode {
            LinkMode::Static => {
                cmd.arg("-static");
            }
            LinkMode::Dynamic => {
                // Dynamic is default
            }
        }

        // Execute linker
        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to execute gcc: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Linking failed: {}", stderr)));
        }

        Ok(self.config.output.clone())
    }

    /// Link with Clang
    fn link_with_clang(&self, object_files: &[PathBuf]) -> Result<PathBuf> {
        let mut cmd = Command::new("clang");

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Add runtime library
        if self.config.runtime_lib.exists() {
            cmd.arg(&self.config.runtime_lib);
        }

        // Add library paths
        for path in &self.config.lib_paths {
            cmd.arg(format!("-L{}", path.display()));
        }

        // Add libraries
        for lib in &self.config.libs {
            cmd.arg(format!("-l{}", lib));
        }

        // Output file
        cmd.arg("-o").arg(&self.config.output);

        // Optimization
        if self.config.optimize {
            cmd.arg("-O2");
        }

        // Strip symbols
        if self.config.strip {
            cmd.arg("-Wl,-s");
        }

        // PIE
        if self.config.pie {
            cmd.arg("-pie");
        }

        // Static/dynamic linking
        match self.config.mode {
            LinkMode::Static => {
                cmd.arg("-static");
            }
            LinkMode::Dynamic => {
                // Dynamic is default
            }
        }

        // Execute linker
        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to execute clang: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Linking failed: {}", stderr)));
        }

        Ok(self.config.output.clone())
    }

    /// Link with ld directly
    fn link_with_ld(&self, object_files: &[PathBuf]) -> Result<PathBuf> {
        let mut cmd = Command::new("ld");

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Output file
        cmd.arg("-o").arg(&self.config.output);

        // Add library paths
        for path in &self.config.lib_paths {
            cmd.arg(format!("-L{}", path.display()));
        }

        // Add libraries
        for lib in &self.config.libs {
            cmd.arg(format!("-l{}", lib));
        }

        // PIE
        if self.config.pie {
            cmd.arg("-pie");
        }

        // Execute linker
        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to execute ld: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Linking failed: {}", stderr)));
        }

        Ok(self.config.output.clone())
    }

    /// Detect available linker
    fn detect_linker(&self) -> LinkerType {
        // Try clang first (better on macOS)
        if Command::new("clang").arg("--version").output().is_ok() {
            return LinkerType::Clang;
        }

        // Try gcc
        if Command::new("gcc").arg("--version").output().is_ok() {
            return LinkerType::Gcc;
        }

        // Fall back to ld
        LinkerType::Ld
    }

    /// Link runtime library separately
    pub fn compile_runtime(&self) -> Result<PathBuf> {
        let runtime_src = &self.config.runtime_lib;
        let runtime_obj = runtime_src.with_extension("o");

        let mut cmd = Command::new("gcc");
        cmd.arg("-c")
            .arg(runtime_src)
            .arg("-o")
            .arg(&runtime_obj)
            .arg("-O2")
            .arg("-fPIC");

        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to compile runtime: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Runtime compilation failed: {}", stderr)));
        }

        Ok(runtime_obj)
    }

    /// Create static library archive
    pub fn create_archive(&self, object_files: &[PathBuf], archive_name: &Path) -> Result<()> {
        let mut cmd = Command::new("ar");
        cmd.arg("rcs")
            .arg(archive_name);

        for obj in object_files {
            cmd.arg(obj);
        }

        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to create archive: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Archive creation failed: {}", stderr)));
        }

        Ok(())
    }

    /// Create shared library
    pub fn create_shared_library(&self, object_files: &[PathBuf], lib_name: &Path) -> Result<()> {
        let mut cmd = Command::new("gcc");
        cmd.arg("-shared")
            .arg("-fPIC")
            .arg("-o")
            .arg(lib_name);

        for obj in object_files {
            cmd.arg(obj);
        }

        let output = cmd.output()
            .map_err(|e| BackendError::LinkingFailed(format!("Failed to create shared library: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::LinkingFailed(format!("Shared library creation failed: {}", stderr)));
        }

        Ok(())
    }
}

/// Linker type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkerType {
    Gcc,
    Clang,
    Ld,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_config_default() {
        let config = LinkerConfig::default();
        assert_eq!(config.mode, LinkMode::Static);
        assert!(config.libs.contains(&"c".to_string()));
    }

    #[test]
    fn test_linker_creation() {
        let config = LinkerConfig::default();
        let _linker = Linker::new(config);
    }
}
