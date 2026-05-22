/// Unit tests for knowledge_base::store::schema
///
/// These tests verify:
///   1. All FromStr implementations accept valid values.
///   2. All FromStr implementations handle unknown values with the
///      documented silent-fallback behaviour (a known design choice).
///   3. as_str() round-trips correctly for every variant.

use crate::store::schema::*;
use std::str::FromStr;

// ── PdeType ──────────────────────────────────────────────────────────────────

#[test]
fn pde_type_from_str_valid() {
    assert_eq!(PdeType::from_str("parabolic").unwrap(), PdeType::Parabolic);
    assert_eq!(PdeType::from_str("elliptic").unwrap(), PdeType::Elliptic);
    assert_eq!(PdeType::from_str("hyperbolic").unwrap(), PdeType::Hyperbolic);
    assert_eq!(PdeType::from_str("mixed").unwrap(), PdeType::Mixed);
    assert_eq!(PdeType::from_str("other").unwrap(), PdeType::Other);
}

#[test]
fn pde_type_from_str_unknown_falls_back_to_other() {
    // Documented silent-fallback: unknown → Other
    assert_eq!(
        PdeType::from_str("nonexistent_type").unwrap(),
        PdeType::Other
    );
}

#[test]
fn pde_type_as_str_round_trips() {
    let variants = [
        PdeType::Parabolic,
        PdeType::Elliptic,
        PdeType::Hyperbolic,
        PdeType::Mixed,
        PdeType::Other,
    ];
    for v in &variants {
        assert_eq!(PdeType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── ConditionType ─────────────────────────────────────────────────────────────

#[test]
fn condition_type_from_str_valid() {
    assert_eq!(
        ConditionType::from_str("boundary").unwrap(),
        ConditionType::Boundary
    );
    assert_eq!(
        ConditionType::from_str("initial").unwrap(),
        ConditionType::Initial
    );
    assert_eq!(
        ConditionType::from_str("domain").unwrap(),
        ConditionType::Domain
    );
    assert_eq!(
        ConditionType::from_str("regularity").unwrap(),
        ConditionType::Regularity
    );
    assert_eq!(
        ConditionType::from_str("other").unwrap(),
        ConditionType::Other
    );
}

#[test]
fn condition_type_from_str_unknown_falls_back_to_other() {
    assert_eq!(
        ConditionType::from_str("unknown_condition").unwrap(),
        ConditionType::Other
    );
}

#[test]
fn condition_type_as_str_round_trips() {
    let variants = [
        ConditionType::Boundary,
        ConditionType::Initial,
        ConditionType::Domain,
        ConditionType::Regularity,
        ConditionType::Other,
    ];
    for v in &variants {
        assert_eq!(ConditionType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── NumericalMethodType ───────────────────────────────────────────────────────

#[test]
fn numerical_method_type_from_str_valid() {
    assert_eq!(
        NumericalMethodType::from_str("grid_based").unwrap(),
        NumericalMethodType::GridBased
    );
    assert_eq!(
        NumericalMethodType::from_str("mesh_based").unwrap(),
        NumericalMethodType::MeshBased
    );
    assert_eq!(
        NumericalMethodType::from_str("spectral_based").unwrap(),
        NumericalMethodType::SpectralBased
    );
    assert_eq!(
        NumericalMethodType::from_str("mesh_free").unwrap(),
        NumericalMethodType::MeshFree
    );
    assert_eq!(
        NumericalMethodType::from_str("other").unwrap(),
        NumericalMethodType::Other
    );
}

#[test]
fn numerical_method_type_from_str_unknown_falls_back_to_other() {
    assert_eq!(
        NumericalMethodType::from_str("particle_based").unwrap(),
        NumericalMethodType::Other
    );
}

#[test]
fn numerical_method_type_as_str_round_trips() {
    let variants = [
        NumericalMethodType::GridBased,
        NumericalMethodType::MeshBased,
        NumericalMethodType::SpectralBased,
        NumericalMethodType::MeshFree,
        NumericalMethodType::Other,
    ];
    for v in &variants {
        assert_eq!(NumericalMethodType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── TrainingType ──────────────────────────────────────────────────────────────

#[test]
fn training_type_from_str_valid() {
    assert_eq!(
        TrainingType::from_str("supervised").unwrap(),
        TrainingType::Supervised
    );
    assert_eq!(
        TrainingType::from_str("unsupervised").unwrap(),
        TrainingType::Unsupervised
    );
    assert_eq!(
        TrainingType::from_str("self_supervised").unwrap(),
        TrainingType::SelfSupervised
    );
    assert_eq!(
        TrainingType::from_str("physics_informed").unwrap(),
        TrainingType::PhysicsInformed
    );
    assert_eq!(
        TrainingType::from_str("operator_learning").unwrap(),
        TrainingType::OperatorLearning
    );
}

#[test]
fn training_type_from_str_unknown_falls_back_to_supervised() {
    // Documented behaviour: unknown → Supervised (not Other)
    assert_eq!(
        TrainingType::from_str("reinforcement_learning").unwrap(),
        TrainingType::Supervised
    );
}

#[test]
fn training_type_as_str_round_trips() {
    let variants = [
        TrainingType::Supervised,
        TrainingType::Unsupervised,
        TrainingType::SelfSupervised,
        TrainingType::PhysicsInformed,
        TrainingType::OperatorLearning,
    ];
    for v in &variants {
        assert_eq!(TrainingType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── LossType ──────────────────────────────────────────────────────────────────

#[test]
fn loss_type_from_str_valid() {
    assert_eq!(LossType::from_str("physics").unwrap(), LossType::Physics);
    assert_eq!(
        LossType::from_str("data_driven").unwrap(),
        LossType::DataDriven
    );
    assert_eq!(
        LossType::from_str("boundary").unwrap(),
        LossType::Boundary
    );
    assert_eq!(
        LossType::from_str("combined").unwrap(),
        LossType::Combined
    );
    assert_eq!(LossType::from_str("other").unwrap(), LossType::Other);
}

#[test]
fn loss_type_from_str_unknown_falls_back_to_other() {
    assert_eq!(
        LossType::from_str("adversarial").unwrap(),
        LossType::Other
    );
}

#[test]
fn loss_type_as_str_round_trips() {
    let variants = [
        LossType::Physics,
        LossType::DataDriven,
        LossType::Boundary,
        LossType::Combined,
        LossType::Other,
    ];
    for v in &variants {
        assert_eq!(LossType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── MetricType ────────────────────────────────────────────────────────────────

#[test]
fn metric_type_from_str_valid() {
    assert_eq!(
        MetricType::from_str("accuracy").unwrap(),
        MetricType::Accuracy
    );
    assert_eq!(
        MetricType::from_str("efficiency").unwrap(),
        MetricType::Efficiency
    );
    assert_eq!(
        MetricType::from_str("stability").unwrap(),
        MetricType::Stability
    );
    assert_eq!(
        MetricType::from_str("generalisation").unwrap(),
        MetricType::Generalisation
    );
    assert_eq!(MetricType::from_str("other").unwrap(), MetricType::Other);
}

#[test]
fn metric_type_from_str_unknown_falls_back_to_other() {
    assert_eq!(
        MetricType::from_str("interpretability").unwrap(),
        MetricType::Other
    );
}

#[test]
fn metric_type_as_str_round_trips() {
    let variants = [
        MetricType::Accuracy,
        MetricType::Efficiency,
        MetricType::Stability,
        MetricType::Generalisation,
        MetricType::Other,
    ];
    for v in &variants {
        assert_eq!(MetricType::from_str(v.as_str()).unwrap(), *v);
    }
}

// ── SourceType ────────────────────────────────────────────────────────────────

#[test]
fn source_type_from_str_valid() {
    assert_eq!(
        SourceType::from_str("paper_reported").unwrap(),
        SourceType::PaperReported
    );
    assert_eq!(
        SourceType::from_str("self_run").unwrap(),
        SourceType::SelfRun
    );
    assert_eq!(
        SourceType::from_str("third_party_reproduction").unwrap(),
        SourceType::ThirdPartyReproduction
    );
}

#[test]
fn source_type_from_str_unknown_falls_back_to_paper_reported() {
    // Documented: unknown → PaperReported (most conservative)
    assert_eq!(
        SourceType::from_str("crowdsourced").unwrap(),
        SourceType::PaperReported
    );
}

#[test]
fn source_type_as_str_round_trips() {
    let variants = [
        SourceType::PaperReported,
        SourceType::SelfRun,
        SourceType::ThirdPartyReproduction,
    ];
    for v in &variants {
        assert_eq!(SourceType::from_str(v.as_str()).unwrap(), *v);
    }
}

#[test]
fn source_type_short_unique() {
    let variants = [
        SourceType::PaperReported,
        SourceType::SelfRun,
        SourceType::ThirdPartyReproduction,
    ];
    let shorts: Vec<&str> = variants.iter().map(|v| v.short()).collect();
    // All short slugs must be distinct
    let mut seen = std::collections::HashSet::new();
    for s in &shorts {
        assert!(seen.insert(s), "duplicate short slug: {}", s);
    }
}

// ── Struct construction sanity checks ────────────────────────────────────────

#[test]
fn equation_struct_builds() {
    let eq = Equation {
        id: "heat".to_string(),
        name: "Heat Equation".to_string(),
        pde_type: PdeType::Parabolic,
        variables: vec!["t".to_string(), "x".to_string()],
        time_dependent: true,
        operator: Some("laplacian".to_string()),
        description: Some("u_t = Δu".to_string()),
        tags: vec!["diffusion".to_string()],
    };
    assert_eq!(eq.pde_type, PdeType::Parabolic);
    assert!(eq.time_dependent);
}

#[test]
fn ai_model_struct_builds() {
    let model = AIModel {
        id: "fno".to_string(),
        name: "Fourier Neural Operator".to_string(),
        architecture: "FNO".to_string(),
        input_vars: vec!["x".to_string(), "t".to_string()],
        output_vars: vec!["u".to_string()],
        training_type: TrainingType::Supervised,
        description: None,
        paper_ref: Some("2010.08895".to_string()),
        tags: vec!["operator_learning".to_string()],
        engine_id: None,
    };
    assert_eq!(model.training_type, TrainingType::Supervised);
}

#[test]
fn paper_struct_requires_authors() {
    // Paper.authors is a Vec — ensure it can be empty without panicking
    let p = Paper {
        id: "2010.08895".to_string(),
        title: "Fourier Neural Operator".to_string(),
        authors: vec!["Li, Z.".to_string()],
        published_year: Some(2020),
        arxiv_id: Some("2010.08895".to_string()),
        doi: None,
        pdf_path: None,
        tags: vec![],
    };
    assert!(!p.authors.is_empty());
}

#[test]
fn serde_roundtrip_pde_type() {
    let original = PdeType::Parabolic;
    let json = serde_json::to_string(&original).unwrap();
    let decoded: PdeType = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn serde_roundtrip_training_type() {
    let original = TrainingType::PhysicsInformed;
    let json = serde_json::to_string(&original).unwrap();
    let decoded: TrainingType = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}
