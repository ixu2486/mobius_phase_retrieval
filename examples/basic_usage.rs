//! SDK public example (application-layer only).
//! Runnable counterpart:
//! `cargo run -p retryix_memory --release --example mobius_phase_sdk_basic_usage`

use retryix_memory::mobius_phase_retrieval::{
    AttentionPoint, HybridPhaseRetrieval, MobiusPhaseRetrieval, PhaseImportanceClass,
    PhasePeriodMode, RetrievalQuery, VisibilityClass,
};

fn main() {
    let mut index = MobiusPhaseRetrieval::new();

    index.add_record_with_importance_and_corruption(
        100,
        "session_alpha_topic_road",
        VisibilityClass::FrontStage,
        PhaseImportanceClass::FrontCritical,
        "payload://hot/road/100",
        0.02,
    );
    index.add_record_with_importance_and_corruption(
        101,
        "session_alpha_topic_road",
        VisibilityClass::BackLatent,
        PhaseImportanceClass::BackArchive,
        "payload://archive/road/101",
        0.15,
    );
    index.add_record_with_importance_and_corruption(
        140,
        "safety_control_plane_alert",
        VisibilityClass::GlobalAnchor,
        PhaseImportanceClass::FrontCritical,
        "payload://control/alert/140",
        0.01,
    );

    let mut hybrid = HybridPhaseRetrieval::new();
    for r in index.records() {
        hybrid.add_record_with_importance_and_corruption(
            r.global_t,
            r.semantic_anchor.clone(),
            r.visibility,
            r.importance,
            r.payload_ref.clone(),
            r.corruption_score,
        );
    }

    let query = RetrievalQuery::new(101)
        .with_semantic_anchor("session_alpha_topic_road")
        .with_top_k(4)
        .with_phase_period_mode(PhasePeriodMode::ReptendCycle)
        .with_corruption_tolerance(0.50)
        .with_attention_points(vec![
            AttentionPoint::new(101, 1.0),
            AttentionPoint::new(100, 0.6),
        ]);

    let (decision, hits) = hybrid.adaptive_retrieve_with_decision(&query);

    println!("selected_profile: {:?}", decision.profile);
    println!("adaptive_reason: {:?}", decision.reason);
    for h in hits {
        println!(
            "payload_ref={} visibility={:?} importance={:?} latent_recoverable={} score={:.4}",
            h.payload_ref,
            h.visibility,
            h.importance,
            h.breakdown.latent_recoverable,
            h.breakdown.total_score
        );
    }
}

