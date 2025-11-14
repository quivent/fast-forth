//! Code generation with metadata support
//!
//! Integrates provenance metadata generation with code generation

pub mod metadata;
pub mod spec_gen;
pub mod hotpath_opt;

pub use metadata::CodegenMetadata;
pub use spec_gen::SpecCodeGenerator;
pub use hotpath_opt::{
    generate_word_definition_fast,
    generate_test_harness_fast,
    generate_provenance_fast,
};
