//! SDK public JSON demo (application-layer only).
//! Runnable counterpart:
//! `cargo run --example json_retrieval_demo -- examples/json_retrieval_demo_input.json`

use mobius_phase_retrieval_sdk::{
    AttentionPoint, HybridPhaseRetrieval, PhaseImportanceClass, PhasePeriodMode, RetrievalQuery,
    VisibilityClass,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct JsonRecord {
    global_t: u64,
    semantic_anchor: String,
    visibility: String,
    importance: String,
    payload_ref: String,
    #[serde(default)]
    corruption_score: f32,
}

#[derive(Debug, Deserialize)]
struct JsonAttentionPoint {
    global_t: u64,
    weight: f64,
}

#[derive(Debug, Deserialize)]
struct JsonQuery {
    global_t: u64,
    semantic_anchor: Option<String>,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default = "default_corruption_tolerance")]
    corruption_tolerance: f32,
    #[serde(default = "default_phase_period_mode")]
    phase_period_mode: String,
    #[serde(default)]
    attention_points: Vec<JsonAttentionPoint>,
}

fn default_top_k() -> usize {
    8
}
fn default_corruption_tolerance() -> f32 {
    0.60
}
fn default_phase_period_mode() -> String {
    "ReptendCycle".to_string()
}

#[derive(Debug, Deserialize)]
struct JsonDemoInput {
    records: Vec<JsonRecord>,
    query: JsonQuery,
}

#[derive(Debug, Serialize)]
struct JsonHitOutput {
    payload_ref: String,
    global_t: u64,
    semantic_anchor: String,
    visibility: String,
    importance: String,
    latent_recoverable: bool,
    total_score: f64,
}

#[derive(Debug, Serialize)]
struct JsonDemoOutput {
    checkpoint: String,
    selected_profile: String,
    adaptive_reason: String,
    hit_count: usize,
    hits: Vec<JsonHitOutput>,
}

fn parse_visibility(v: &str) -> VisibilityClass {
    match v {
        "FrontStage" => VisibilityClass::FrontStage,
        "BackLatent" => VisibilityClass::BackLatent,
        "DualPlane" => VisibilityClass::DualPlane,
        "GlobalAnchor" => VisibilityClass::GlobalAnchor,
        _ => VisibilityClass::FrontStage,
    }
}

fn parse_importance(v: &str) -> PhaseImportanceClass {
    match v {
        "FrontCritical" => PhaseImportanceClass::FrontCritical,
        "FrontImportant" => PhaseImportanceClass::FrontImportant,
        "FrontContext" => PhaseImportanceClass::FrontContext,
        "BackCritical" => PhaseImportanceClass::BackCritical,
        "BackArchive" => PhaseImportanceClass::BackArchive,
        "BackCold" => PhaseImportanceClass::BackCold,
        _ => PhaseImportanceClass::FrontImportant,
    }
}

fn parse_phase_mode(v: &str) -> PhasePeriodMode {
    match v {
        "Denominator" => PhasePeriodMode::Denominator,
        _ => PhasePeriodMode::ReptendCycle,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("examples/json_retrieval_demo_input.json")
    });
    let raw = fs::read_to_string(&input_path)?;
    let parsed: JsonDemoInput = serde_json::from_str(&raw)?;

    let mut hybrid = HybridPhaseRetrieval::new();
    for r in parsed.records {
        hybrid.add_record_with_importance_and_corruption(
            r.global_t,
            r.semantic_anchor,
            parse_visibility(&r.visibility),
            parse_importance(&r.importance),
            r.payload_ref,
            r.corruption_score,
        );
    }

    let mut query = RetrievalQuery::new(parsed.query.global_t)
        .with_top_k(parsed.query.top_k)
        .with_corruption_tolerance(parsed.query.corruption_tolerance)
        .with_phase_period_mode(parse_phase_mode(&parsed.query.phase_period_mode))
        .with_attention_points(
            parsed
                .query
                .attention_points
                .into_iter()
                .map(|p| AttentionPoint::new(p.global_t, p.weight))
                .collect(),
        );
    if let Some(anchor) = parsed.query.semantic_anchor {
        query = query.with_semantic_anchor(anchor);
    }

    let (decision, hits) = hybrid.adaptive_retrieve_with_decision(&query);
    let out = JsonDemoOutput {
        checkpoint: "mobius_phase_retrieval_json_demo_v1".to_string(),
        selected_profile: format!("{:?}", decision.profile),
        adaptive_reason: format!("{:?}", decision.reason),
        hit_count: hits.len(),
        hits: hits
            .into_iter()
            .map(|h| JsonHitOutput {
                payload_ref: h.payload_ref,
                global_t: h.global_t,
                semantic_anchor: h.semantic_anchor,
                visibility: format!("{:?}", h.visibility),
                importance: format!("{:?}", h.importance),
                latent_recoverable: h.breakdown.latent_recoverable,
                total_score: h.breakdown.total_score,
            })
            .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
