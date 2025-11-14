//! Integration tests for type specialization

use fastforth_optimizer::{
    ConcreteType, ForthIR, Instruction, OptimizationLevel, Optimizer, SpecializationStats,
    TypeInferenceResults, TypeSignature, TypeSpecializer, WordDef,
};

#[test]
fn test_basic_type_specialization() {
    // Create a simple word: SQUARE ( n -- n² ) = DUP *
    let mut ir = ForthIR::new();

    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

    // Create type inference results
    let mut type_info = TypeInferenceResults::new();

    // INT version
    type_info.add_word_signature(
        "square".to_string(),
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );

    // FLOAT version
    type_info.add_word_signature(
        "square".to_string(),
        TypeSignature::new(vec![ConcreteType::Float], vec![ConcreteType::Float]),
    );

    // Add call sites
    type_info.add_call_site(
        0,
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );

    type_info.add_call_site(
        1,
        TypeSignature::new(vec![ConcreteType::Float], vec![ConcreteType::Float]),
    );

    // Run specialization
    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info).unwrap();

    // Verify results
    assert!(stats.specializations_created > 0, "Should create specializations");
    assert!(stats.words_analyzed > 0, "Should analyze words");

    println!("{}", stats);
}

#[test]
fn test_polymorphic_detection() {
    let mut ir = ForthIR::new();

    // Create DUP word
    let dup_word = WordDef::new("dup".to_string(), vec![Instruction::Dup]);
    ir.add_word(dup_word);

    let mut type_info = TypeInferenceResults::new();

    // Use DUP with different types
    type_info.add_word_signature(
        "dup".to_string(),
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int, ConcreteType::Int]),
    );

    type_info.add_word_signature(
        "dup".to_string(),
        TypeSignature::new(
            vec![ConcreteType::Float],
            vec![ConcreteType::Float, ConcreteType::Float],
        ),
    );

    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info).unwrap();

    assert!(stats.polymorphic_words > 0, "Should detect polymorphic words");
}

#[test]
fn test_name_mangling() {
    let sig_int = TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]);

    let mangled = sig_int.mangle_name("square");
    assert!(mangled.contains("INT"), "Should contain type suffix");

    let sig_float = TypeSignature::new(vec![ConcreteType::Float], vec![ConcreteType::Float]);

    let mangled_float = sig_float.mangle_name("square");
    assert!(mangled_float.contains("FLOAT"), "Should contain type suffix");
    assert_ne!(mangled, mangled_float, "Different types should have different names");
}

#[test]
fn test_specialization_stats_calculation() {
    let mut stats = SpecializationStats {
        words_analyzed: 10,
        polymorphic_words: 3,
        specializations_created: 5,
        call_sites_rewritten: 15,
        estimated_speedup_percent: 0.0,
    };

    stats.calculate_speedup();

    // Should estimate significant speedup
    assert!(
        stats.estimated_speedup_percent >= 10.0,
        "Should estimate at least 10% speedup, got {}%",
        stats.estimated_speedup_percent
    );
    assert!(
        stats.estimated_speedup_percent <= 25.0,
        "Should not overestimate speedup, got {}%",
        stats.estimated_speedup_percent
    );
}

#[test]
fn test_optimizer_integration() {
    let mut ir = ForthIR::new();

    // Create a word that will benefit from specialization
    let add_square = WordDef::new(
        "add_square".to_string(),
        vec![
            Instruction::Add,
            Instruction::Dup,
            Instruction::Mul,
        ],
    );
    ir.add_word(add_square);

    let mut type_info = TypeInferenceResults::new();
    type_info.add_word_signature(
        "add_square".to_string(),
        TypeSignature::new(
            vec![ConcreteType::Int, ConcreteType::Int],
            vec![ConcreteType::Int],
        ),
    );

    // Run full optimization with type specialization
    let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);
    let optimized = optimizer.optimize_with_types(ir, &type_info).unwrap();

    // Verify optimization occurred
    let stats = optimizer.specialization_stats();
    println!("Optimization stats: {}", stats);

    assert!(
        stats.words_analyzed > 0,
        "Should have analyzed words"
    );
}

#[test]
fn test_type_inference_results_builder() {
    let mut results = TypeInferenceResults::new();

    // Add multiple signatures
    results.add_word_signature(
        "test".to_string(),
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );

    results.add_call_site(
        0,
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );

    assert_eq!(results.word_signatures.len(), 1);
    assert_eq!(results.call_site_signatures.len(), 1);
}

#[test]
fn test_example_type_results() {
    let results = TypeInferenceResults::example();

    assert!(
        !results.word_signatures.is_empty(),
        "Example should have word signatures"
    );
    assert!(
        !results.call_site_signatures.is_empty(),
        "Example should have call site signatures"
    );
}

#[test]
fn test_concrete_type_specialization_checks() {
    assert!(ConcreteType::Int.needs_specialization());
    assert!(ConcreteType::Float.needs_specialization());
    assert!(!ConcreteType::String.needs_specialization());
}

#[test]
fn test_multiple_specializations_per_word() {
    let mut ir = ForthIR::new();

    let math_op = WordDef::new(
        "math".to_string(),
        vec![Instruction::Dup, Instruction::Add],
    );
    ir.add_word(math_op);

    let mut type_info = TypeInferenceResults::new();

    // Add three different type signatures
    for concrete_type in &[ConcreteType::Int, ConcreteType::Float, ConcreteType::Addr] {
        type_info.add_word_signature(
            "math".to_string(),
            TypeSignature::new(vec![concrete_type.clone()], vec![concrete_type.clone()]),
        );
    }

    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info).unwrap();

    // Should create multiple specializations
    assert!(
        stats.specializations_created >= 2,
        "Should create multiple specializations, got {}",
        stats.specializations_created
    );
}

#[test]
fn test_empty_ir_specialization() {
    let mut ir = ForthIR::new();
    let type_info = TypeInferenceResults::new();

    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info).unwrap();

    assert_eq!(stats.specializations_created, 0);
    assert_eq!(stats.words_analyzed, 0);
}

#[test]
fn test_call_site_rewriting() {
    let mut ir = ForthIR::new();

    // Create main sequence with calls
    ir.main = vec![
        Instruction::Literal(5),
        Instruction::Call("square".to_string()),
        Instruction::Literal(3),
        Instruction::Call("square".to_string()),
    ];

    let square = WordDef::new(
        "square".to_string(),
        vec![Instruction::Dup, Instruction::Mul],
    );
    ir.add_word(square);

    let mut type_info = TypeInferenceResults::new();

    // Both calls use Int
    type_info.add_call_site(
        1,
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );
    type_info.add_call_site(
        3,
        TypeSignature::new(vec![ConcreteType::Int], vec![ConcreteType::Int]),
    );

    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info).unwrap();

    assert!(
        stats.call_sites_rewritten > 0,
        "Should rewrite call sites"
    );
}
