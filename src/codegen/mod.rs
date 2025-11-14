//! Code generation with metadata support
//!
//! Integrates provenance metadata generation with code generation

pub mod metadata;
pub mod spec_gen;

pub use metadata::CodegenMetadata;
pub use spec_gen::SpecCodeGenerator;
