//! Standalone public SDK for Möbius Phase Retrieval.
//!
//! Boundary: application-layer retrieval and memory-governance only.

use std::cmp::Ordering;

const DEFAULT_PRIME_PHASE_DENOMINATORS: [u64; 7] = [7, 17, 19, 23, 29, 47, 59];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PhasePeriodMode {
    Denominator,
    ReptendCycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VisibilityClass {
    FrontStage,
    BackLatent,
    DualPlane,
    GlobalAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PhaseImportanceClass {
    FrontCritical,
    FrontImportant,
    FrontContext,
    BackCritical,
    BackArchive,
    BackCold,
}

impl PhaseImportanceClass {
    pub const fn default_for_visibility(visibility: VisibilityClass) -> Self {
        match visibility {
            VisibilityClass::FrontStage => Self::FrontImportant,
            VisibilityClass::BackLatent => Self::BackArchive,
            VisibilityClass::DualPlane => Self::BackCritical,
            VisibilityClass::GlobalAnchor => Self::FrontCritical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttentionPoint {
    pub global_t: u64,
    pub weight: f64,
}

impl AttentionPoint {
    pub fn new(global_t: u64, weight: f64) -> Self {
        Self {
            global_t,
            weight: weight.max(0.0),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalQuery {
    pub global_t: u64,
    pub semantic_anchor: Option<String>,
    pub attention_points: Vec<AttentionPoint>,
    pub top_k: usize,
    pub corruption_tolerance: f32,
    pub phase_period_mode: PhasePeriodMode,
}

impl RetrievalQuery {
    pub fn new(global_t: u64) -> Self {
        Self {
            global_t,
            semantic_anchor: None,
            attention_points: Vec::new(),
            top_k: 8,
            corruption_tolerance: 0.60,
            phase_period_mode: PhasePeriodMode::ReptendCycle,
        }
    }

    pub fn with_semantic_anchor(mut self, anchor: impl Into<String>) -> Self {
        self.semantic_anchor = Some(anchor.into());
        self
    }

    pub fn with_attention_points(mut self, points: Vec<AttentionPoint>) -> Self {
        self.attention_points = points;
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k.max(1);
        self
    }

    pub fn with_corruption_tolerance(mut self, tolerance: f32) -> Self {
        self.corruption_tolerance = tolerance.clamp(0.0, 1.0);
        self
    }

    pub fn with_phase_period_mode(mut self, mode: PhasePeriodMode) -> Self {
        self.phase_period_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct RetrievalScoreBreakdown {
    pub phase_score: f64,
    pub semantic_score: f64,
    pub mobius_score: f64,
    pub global_score: f64,
    pub attention_score: f64,
    pub corruption_penalty: f64,
    pub total_score: f64,
    pub latent_recoverable: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalHit {
    pub payload_ref: String,
    pub global_t: u64,
    pub semantic_anchor: String,
    pub visibility: VisibilityClass,
    pub importance: PhaseImportanceClass,
    pub breakdown: RetrievalScoreBreakdown,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PhaseRetrievalMetrics {
    pub query_count: u64,
    pub phase_hit_count: u64,
    pub mobius_recovery_count: u64,
    pub latent_recoverable_count: u64,
    pub corrupt_filtered_count: u64,
}

impl PhaseRetrievalMetrics {
    pub fn phase_hit_rate(&self) -> f64 {
        ratio(self.phase_hit_count, self.query_count)
    }

    pub fn mobius_recovery_rate(&self) -> f64 {
        ratio(self.mobius_recovery_count, self.query_count)
    }

    pub fn latent_recoverable_rate(&self) -> f64 {
        ratio(self.latent_recoverable_count, self.query_count)
    }

    pub fn corruption_penalty_rate(&self) -> f64 {
        ratio(self.corrupt_filtered_count, self.query_count)
    }
}

fn ratio(n: u64, d: u64) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 }
}

#[derive(Debug, Clone)]
struct MobiusPhaseRecord {
    global_t: u64,
    semantic_anchor: String,
    visibility: VisibilityClass,
    importance: PhaseImportanceClass,
    payload_ref: String,
    corruption_score: f32,
    phase_denominator: [f64; 7],
    phase_reptend: [f64; 7],
}

impl MobiusPhaseRecord {
    fn new(
        global_t: u64,
        semantic_anchor: impl Into<String>,
        visibility: VisibilityClass,
        importance: PhaseImportanceClass,
        payload_ref: impl Into<String>,
        corruption_score: f32,
    ) -> Self {
        Self {
            global_t,
            semantic_anchor: semantic_anchor.into(),
            visibility,
            importance,
            payload_ref: payload_ref.into(),
            corruption_score: corruption_score.clamp(0.0, 1.0),
            phase_denominator: phase_coordinates(global_t, PhasePeriodMode::Denominator),
            phase_reptend: phase_coordinates(global_t, PhasePeriodMode::ReptendCycle),
        }
    }

    fn phase_for_mode(&self, mode: PhasePeriodMode) -> [f64; 7] {
        match mode {
            PhasePeriodMode::Denominator => self.phase_denominator,
            PhasePeriodMode::ReptendCycle => self.phase_reptend,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MobiusPhaseRetrieval {
    records: Vec<MobiusPhaseRecord>,
    corruption_penalty_gain: f64,
}

impl MobiusPhaseRetrieval {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            corruption_penalty_gain: 0.65,
        }
    }

    pub fn add_record_with_importance_and_corruption(
        &mut self,
        global_t: u64,
        semantic_anchor: impl Into<String>,
        visibility: VisibilityClass,
        importance: PhaseImportanceClass,
        payload_ref: impl Into<String>,
        corruption_score: f32,
    ) {
        self.records.push(MobiusPhaseRecord::new(
            global_t,
            semantic_anchor,
            visibility,
            importance,
            payload_ref,
            corruption_score,
        ));
    }

    pub fn retrieve(&self, query: &RetrievalQuery) -> Vec<RetrievalHit> {
        retrieve_impl(
            &self.records,
            query,
            self.corruption_penalty_gain,
            ScoreProfile::Fast,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HybridRetrievalProfile {
    LowLatency,
    Balanced,
    HighAccuracy,
}

impl HybridRetrievalProfile {
    pub const fn params(self) -> (usize, usize) {
        match self {
            Self::LowLatency => (32, 8),
            Self::Balanced => (128, 16),
            Self::HighAccuracy => (256, 16),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AdaptiveRetrievalReason {
    SmallTopKLowLatency,
    ImportantQueryHighAccuracy,
    MultiPointHighAccuracy,
    CorruptionSensitiveHighAccuracy,
    DefaultBalanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AdaptiveRetrievalDecision {
    pub profile: HybridRetrievalProfile,
    pub reason: AdaptiveRetrievalReason,
}

pub fn choose_adaptive_profile(query: &RetrievalQuery) -> AdaptiveRetrievalDecision {
    if query.top_k <= 5 && query.attention_points.len() <= 1 {
        return AdaptiveRetrievalDecision {
            profile: HybridRetrievalProfile::LowLatency,
            reason: AdaptiveRetrievalReason::SmallTopKLowLatency,
        };
    }

    if query
        .semantic_anchor
        .as_deref()
        .map(is_important_anchor)
        .unwrap_or(false)
    {
        return AdaptiveRetrievalDecision {
            profile: HybridRetrievalProfile::HighAccuracy,
            reason: AdaptiveRetrievalReason::ImportantQueryHighAccuracy,
        };
    }

    if query.attention_points.len() >= 3 {
        return AdaptiveRetrievalDecision {
            profile: HybridRetrievalProfile::HighAccuracy,
            reason: AdaptiveRetrievalReason::MultiPointHighAccuracy,
        };
    }

    if query.semantic_anchor.is_some() && query.corruption_tolerance < 0.45 {
        return AdaptiveRetrievalDecision {
            profile: HybridRetrievalProfile::HighAccuracy,
            reason: AdaptiveRetrievalReason::CorruptionSensitiveHighAccuracy,
        };
    }

    AdaptiveRetrievalDecision {
        profile: HybridRetrievalProfile::Balanced,
        reason: AdaptiveRetrievalReason::DefaultBalanced,
    }
}

#[derive(Debug, Clone, Default)]
pub struct HybridPhaseRetrieval {
    inner: MobiusPhaseRetrieval,
}

impl HybridPhaseRetrieval {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_record_with_importance_and_corruption(
        &mut self,
        global_t: u64,
        semantic_anchor: impl Into<String>,
        visibility: VisibilityClass,
        importance: PhaseImportanceClass,
        payload_ref: impl Into<String>,
        corruption_score: f32,
    ) {
        self.inner.add_record_with_importance_and_corruption(
            global_t,
            semantic_anchor,
            visibility,
            importance,
            payload_ref,
            corruption_score,
        );
    }

    pub fn hybrid_retrieve(&self, query: &RetrievalQuery, fast_k: usize, final_k: usize) -> Vec<RetrievalHit> {
        let mut q1 = query.clone();
        q1.top_k = fast_k.max(1);
        let fast_hits = self.inner.retrieve(&q1);
        if fast_hits.is_empty() {
            return Vec::new();
        }

        let candidates: Vec<&MobiusPhaseRecord> = fast_hits
            .iter()
            .filter_map(|h| {
                self.inner
                    .records
                    .iter()
                    .find(|r| r.global_t == h.global_t && r.payload_ref == h.payload_ref)
            })
            .collect();

        if candidates.is_empty() {
            let mut fallback = fast_hits;
            fallback.truncate(final_k.max(1));
            return fallback;
        }

        let mut reranked = retrieve_impl(
            &candidates.into_iter().cloned().collect::<Vec<_>>(),
            query,
            self.inner.corruption_penalty_gain,
            ScoreProfile::Precision,
        );
        reranked.truncate(final_k.max(1));
        reranked
    }

    pub fn hybrid_retrieve_with_profile(
        &self,
        query: &RetrievalQuery,
        profile: HybridRetrievalProfile,
    ) -> Vec<RetrievalHit> {
        let (fast_k, final_k) = profile.params();
        let mut hits = self.hybrid_retrieve(query, fast_k, final_k.max(query.top_k));
        hits.truncate(query.top_k.max(1));
        hits
    }

    pub fn adaptive_retrieve_with_decision(
        &self,
        query: &RetrievalQuery,
    ) -> (AdaptiveRetrievalDecision, Vec<RetrievalHit>) {
        let decision = choose_adaptive_profile(query);
        let hits = self.hybrid_retrieve_with_profile(query, decision.profile);
        (decision, hits)
    }

    pub fn adaptive_retrieve(&self, query: &RetrievalQuery) -> Vec<RetrievalHit> {
        let (_, hits) = self.adaptive_retrieve_with_decision(query);
        hits
    }
}

#[derive(Clone, Copy)]
enum ScoreProfile {
    Fast,
    Precision,
}

fn retrieve_impl(
    records: &[MobiusPhaseRecord],
    query: &RetrievalQuery,
    corruption_penalty_gain: f64,
    profile: ScoreProfile,
) -> Vec<RetrievalHit> {
    if records.is_empty() {
        return Vec::new();
    }

    let query_phase = phase_coordinates(query.global_t, query.phase_period_mode);
    let query_half = query_phase.map(mobius_half_turn_pair);

    let (w_phase, w_semantic, w_mobius, w_global, w_attention) = match profile {
        ScoreProfile::Fast => (0.40, 0.25, 0.20, 0.15, 0.20),
        ScoreProfile::Precision => (0.35, 0.30, 0.25, 0.10, 0.20),
    };

    let mut hits: Vec<RetrievalHit> = records
        .iter()
        .map(|r| {
            let record_phase = r.phase_for_mode(query.phase_period_mode);
            let phase_score = circular_phase_similarity(&query_phase, &record_phase);
            let semantic_score = semantic_similarity(query.semantic_anchor.as_deref(), &r.semantic_anchor);
            let mobius_score = circular_phase_similarity(&query_half, &record_phase);
            let global_score = global_visibility_score(r.visibility);
            let attention_score = attention_points_score(
                &query.attention_points,
                &record_phase,
                query.phase_period_mode,
            );
            let corruption_penalty = (r.corruption_score as f64 * corruption_penalty_gain).min(1.0);
            let total_score = (w_phase * phase_score)
                + (w_semantic * semantic_score)
                + (w_mobius * mobius_score)
                + (w_global * global_score)
                + (w_attention * attention_score)
                - corruption_penalty;
            let latent_recoverable = is_latent_recoverable(
                r.visibility,
                r.corruption_score,
                semantic_score,
                mobius_score,
                attention_score,
                query.corruption_tolerance,
            );

            RetrievalHit {
                payload_ref: r.payload_ref.clone(),
                global_t: r.global_t,
                semantic_anchor: r.semantic_anchor.clone(),
                visibility: r.visibility,
                importance: r.importance,
                breakdown: RetrievalScoreBreakdown {
                    phase_score,
                    semantic_score,
                    mobius_score,
                    global_score,
                    attention_score,
                    corruption_penalty,
                    total_score,
                    latent_recoverable,
                },
            }
        })
        .collect();

    hits.sort_by(|a, b| {
        b.breakdown
            .total_score
            .partial_cmp(&a.breakdown.total_score)
            .unwrap_or(Ordering::Equal)
    });
    hits.truncate(query.top_k.max(1));
    hits
}

fn is_important_anchor(anchor: &str) -> bool {
    let a = anchor.to_ascii_lowercase();
    a.contains("critical")
        || a.contains("urgent")
        || a.contains("p0")
        || a.contains("safety")
        || a.contains("control_plane")
}

fn phase_coordinates(global_t: u64, mode: PhasePeriodMode) -> [f64; 7] {
    DEFAULT_PRIME_PHASE_DENOMINATORS.map(|p| {
        let period = match mode {
            PhasePeriodMode::Denominator => p,
            PhasePeriodMode::ReptendCycle => (p - 1).max(1),
        };
        (global_t % period) as f64 / period as f64
    })
}

fn mobius_half_turn_pair(phase: f64) -> f64 {
    let p = phase.rem_euclid(1.0);
    (p + 0.5).rem_euclid(1.0)
}

fn circular_phase_similarity(a: &[f64; 7], b: &[f64; 7]) -> f64 {
    let mut acc = 0.0;
    for i in 0..a.len() {
        let da = (a[i] - b[i]).abs();
        let d = da.min(1.0 - da);
        let local = (1.0 - (2.0 * d)).clamp(0.0, 1.0);
        acc += local;
    }
    acc / a.len() as f64
}

fn semantic_similarity(query_anchor: Option<&str>, record_anchor: &str) -> f64 {
    let Some(query) = query_anchor else {
        return 0.5;
    };

    let q = query.trim().to_ascii_lowercase();
    let r = record_anchor.trim().to_ascii_lowercase();

    if q.is_empty() || r.is_empty() {
        return 0.0;
    }
    if q == r {
        return 1.0;
    }
    if q.contains(&r) || r.contains(&q) {
        return 0.80;
    }

    let q_tokens: Vec<&str> = q.split(['_', '-', ' ']).filter(|s| !s.is_empty()).collect();
    let r_tokens: Vec<&str> = r.split(['_', '-', ' ']).filter(|s| !s.is_empty()).collect();
    if q_tokens.is_empty() || r_tokens.is_empty() {
        return 0.0;
    }

    let common = q_tokens.iter().filter(|t| r_tokens.contains(t)).count() as f64;
    let union = (q_tokens.len() + r_tokens.len()) as f64 - common;
    if union <= 0.0 {
        0.0
    } else {
        (common / union).clamp(0.0, 1.0)
    }
}

fn attention_points_score(points: &[AttentionPoint], record_phase: &[f64; 7], mode: PhasePeriodMode) -> f64 {
    if points.is_empty() {
        return 0.0;
    }

    let mut weighted = 0.0;
    let mut total_weight = 0.0;
    for p in points {
        let w = p.weight.max(0.0);
        if w <= f64::EPSILON {
            continue;
        }
        let pp = phase_coordinates(p.global_t, mode);
        let sim = circular_phase_similarity(&pp, record_phase);
        weighted += sim * w;
        total_weight += w;
    }

    if total_weight <= f64::EPSILON {
        0.0
    } else {
        (weighted / total_weight).clamp(0.0, 1.0)
    }
}

fn global_visibility_score(v: VisibilityClass) -> f64 {
    match v {
        VisibilityClass::FrontStage => 0.70,
        VisibilityClass::BackLatent => 0.65,
        VisibilityClass::DualPlane => 0.82,
        VisibilityClass::GlobalAnchor => 1.0,
    }
}

fn is_latent_recoverable(
    visibility: VisibilityClass,
    corruption_score: f32,
    semantic_score: f64,
    mobius_score: f64,
    attention_score: f64,
    corruption_tolerance: f32,
) -> bool {
    if !matches!(visibility, VisibilityClass::BackLatent | VisibilityClass::DualPlane) {
        return false;
    }
    if corruption_score > corruption_tolerance {
        return false;
    }
    semantic_score >= 0.70 || mobius_score >= 0.55 || attention_score >= 0.50
}

/// Small helper to build a deterministic demo index.
pub fn build_demo_index() -> HybridPhaseRetrieval {
    let mut hybrid = HybridPhaseRetrieval::new();
    hybrid.add_record_with_importance_and_corruption(
        100,
        "session_alpha_topic_road",
        VisibilityClass::FrontStage,
        PhaseImportanceClass::FrontCritical,
        "payload://hot/road/100",
        0.02,
    );
    hybrid.add_record_with_importance_and_corruption(
        101,
        "session_alpha_topic_road",
        VisibilityClass::BackLatent,
        PhaseImportanceClass::BackArchive,
        "payload://archive/road/101",
        0.12,
    );
    hybrid.add_record_with_importance_and_corruption(
        140,
        "critical_safety_control_plane_alert",
        VisibilityClass::GlobalAnchor,
        PhaseImportanceClass::FrontCritical,
        "payload://control/alert/140",
        0.01,
    );
    hybrid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_retrieve_returns_hits() {
        let index = build_demo_index();
        let query = RetrievalQuery::new(101)
            .with_semantic_anchor("session_alpha_topic_road")
            .with_top_k(3)
            .with_phase_period_mode(PhasePeriodMode::ReptendCycle)
            .with_attention_points(vec![AttentionPoint::new(101, 1.0)]);
        let hits = index.adaptive_retrieve(&query);
        assert!(!hits.is_empty());
    }

    #[test]
    fn profile_selection_is_stable() {
        let q = RetrievalQuery::new(100)
            .with_top_k(3)
            .with_attention_points(vec![AttentionPoint::new(100, 1.0)]);
        let d = choose_adaptive_profile(&q);
        assert_eq!(d.profile, HybridRetrievalProfile::LowLatency);
    }
}

