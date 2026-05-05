//! SDK public example (application-layer only).
//! Runnable counterpart:
//! `cargo run --example basic_usage`

use mobius_phase_retrieval_sdk::{
    build_demo_index, AttentionPoint, PhasePeriodMode, RetrievalQuery,
};

fn main() {
    let hybrid = build_demo_index();

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
