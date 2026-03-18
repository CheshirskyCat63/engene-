use std::cmp::Ordering;
use std::collections::VecDeque;

/// World-scale simulation level contract defined by runtime orchestration law.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimulationLevel {
    /// Immediate bubble; highest fidelity.
    L0,
    /// Near field; reduced fidelity.
    L1,
    /// Regional aggregate simulation.
    L2,
    /// Global aggregate simulation.
    L3,
}

/// Runtime-owned policy profile for level classification and cadence decisions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimulationPolicyProfile {
    pub l0_radius: f32,
    pub l1_radius: f32,
    pub l2_radius: f32,
    pub l1_tick_interval: u64,
    pub l2_tick_interval: u64,
    pub max_promotions_per_frame: usize,
    pub max_demotions_per_frame: usize,
}

impl Default for SimulationPolicyProfile {
    fn default() -> Self {
        Self {
            l0_radius: 300.0,
            l1_radius: 5_000.0,
            l2_radius: 50_000.0,
            l1_tick_interval: 12,
            l2_tick_interval: 60,
            max_promotions_per_frame: 64,
            max_demotions_per_frame: 256,
        }
    }
}

impl SimulationPolicyProfile {
    pub fn classify_distance(self, distance: f32) -> SimulationLevel {
        if distance <= self.l0_radius {
            SimulationLevel::L0
        } else if distance <= self.l1_radius {
            SimulationLevel::L1
        } else if distance <= self.l2_radius {
            SimulationLevel::L2
        } else {
            SimulationLevel::L3
        }
    }

    pub fn should_tick(self, level: SimulationLevel, frame: u64) -> bool {
        match level {
            SimulationLevel::L0 => true,
            SimulationLevel::L1 => frame % self.l1_tick_interval == 0,
            SimulationLevel::L2 => frame % self.l2_tick_interval == 0,
            SimulationLevel::L3 => false,
        }
    }
}

/// Runtime-owned transition-core speed law constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionSpeedLawTargets {
    pub update_typical_ms: f32,
    pub update_ceiling_ms: f32,
    pub hard_fail_spike_ms: f32,
    pub single_thread_floor_tps: u64,
    pub single_thread_target_tps: u64,
    pub multi_thread_target_tps: u64,
    pub scaling_min_x8: f32,
    pub scaling_target_x8: f32,
    pub queue_typical_occupancy_max: f32,
    pub queue_stress_occupancy_max: f32,
    pub queue_failure_occupancy_min: f32,
    pub dev_metrics_overhead_max_pct: f32,
    pub ship_metrics_overhead_max_pct: f32,
}

impl Default for TransitionSpeedLawTargets {
    fn default() -> Self {
        Self {
            update_typical_ms: 0.20,
            update_ceiling_ms: 0.50,
            hard_fail_spike_ms: 0.90,
            single_thread_floor_tps: 500_000,
            single_thread_target_tps: 800_000,
            multi_thread_target_tps: 2_000_000,
            scaling_min_x8: 3.0,
            scaling_target_x8: 4.5,
            queue_typical_occupancy_max: 0.25,
            queue_stress_occupancy_max: 0.70,
            queue_failure_occupancy_min: 0.90,
            dev_metrics_overhead_max_pct: 5.0,
            ship_metrics_overhead_max_pct: 1.0,
        }
    }
}

/// Explicit runtime-owned transition execution context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionExecutionContext {
    pub frame_index: u64,
    pub tick_index: u64,
    pub promotion_budget: usize,
    pub demotion_budget: usize,
}

impl TransitionExecutionContext {
    pub fn from_policy(policy: SimulationPolicyProfile, frame_index: u64, tick_index: u64) -> Self {
        Self {
            frame_index,
            tick_index,
            promotion_budget: policy.max_promotions_per_frame,
            demotion_budget: policy.max_demotions_per_frame,
        }
    }
}

/// Transition rationale that can be surfaced in overlays/logging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionReason {
    PlayerProximity,
    TacticalRelevance,
    StreamingEnter,
    StreamingExit,
    InteractionEntropyLow,
    BudgetPressure,
    PersistenceLoad,
    PersistenceSave,
}

/// Authority precedence marker used for reconciliation decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionAuthority {
    LocalActiveTruth,
    RegionalAggregate,
    GlobalAggregate,
}

/// Reconciliation disposition marker for transition observability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationMarker {
    NoConflict,
    UpgradedFromAggregate,
    CollapsedToAggregate,
    ConflictResolvedByAuthority,
}

/// Result disposition for applied/deferred/rejected transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionDisposition {
    Applied,
    DeferredByPromotionBudget,
    DeferredByDemotionBudget,
    DeferredByQueueBackpressure,
    DroppedByQueuePolicy,
    RejectedNoLevelChange,
}

/// Runtime-neutral subject handle for transition requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimulationSubject {
    pub entity_id: u64,
}

/// Request to promote simulation fidelity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromotionRequest {
    pub subject: SimulationSubject,
    pub from: SimulationLevel,
    pub to: SimulationLevel,
    pub reason: TransitionReason,
}

/// Request to demote simulation fidelity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemotionRequest {
    pub subject: SimulationSubject,
    pub from: SimulationLevel,
    pub to: SimulationLevel,
    pub reason: TransitionReason,
}

/// Unified transition request contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionRequest {
    Promote(PromotionRequest),
    Demote(DemotionRequest),
}

impl TransitionRequest {
    pub fn subject(self) -> SimulationSubject {
        match self {
            TransitionRequest::Promote(r) => r.subject,
            TransitionRequest::Demote(r) => r.subject,
        }
    }

    pub fn from(self) -> SimulationLevel {
        match self {
            TransitionRequest::Promote(r) => r.from,
            TransitionRequest::Demote(r) => r.from,
        }
    }

    pub fn to(self) -> SimulationLevel {
        match self {
            TransitionRequest::Promote(r) => r.to,
            TransitionRequest::Demote(r) => r.to,
        }
    }

    pub fn reason(self) -> TransitionReason {
        match self {
            TransitionRequest::Promote(r) => r.reason,
            TransitionRequest::Demote(r) => r.reason,
        }
    }

    pub fn is_promotion(self) -> bool {
        matches!(self, TransitionRequest::Promote(_))
    }
}

/// Deterministic ordering contract for transition batches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionOrderingPolicy {
    SubjectThenFromToReason,
}

impl TransitionOrderingPolicy {
    pub fn compare(self, left: TransitionRequest, right: TransitionRequest) -> Ordering {
        match self {
            TransitionOrderingPolicy::SubjectThenFromToReason => left
                .subject()
                .entity_id
                .cmp(&right.subject().entity_id)
                .then(level_rank(left.from()).cmp(&level_rank(right.from())))
                .then(level_rank(left.to()).cmp(&level_rank(right.to())))
                .then(reason_rank(left.reason()).cmp(&reason_rank(right.reason()))),
        }
    }
}

/// Runtime-owned shard output contract for future parallel classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionShardOutput {
    pub shard_id: u16,
    pub requests: Vec<TransitionRequest>,
}

/// Deterministic merge policy for shard outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionShardMergePolicy {
    pub ordering: TransitionOrderingPolicy,
    pub max_merge_inputs: usize,
}

impl Default for TransitionShardMergePolicy {
    fn default() -> Self {
        Self {
            ordering: TransitionOrderingPolicy::SubjectThenFromToReason,
            max_merge_inputs: 8,
        }
    }
}

/// Transition handoff payload for persistence/streaming integration points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionHandoff {
    pub subject: SimulationSubject,
    pub from: SimulationLevel,
    pub to: SimulationLevel,
    pub authority: TransitionAuthority,
    pub deterministic_seed: Option<u64>,
    pub snapshot_key: Option<String>,
}

/// Transition resolution result exposed by runtime orchestration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionResult {
    pub request: TransitionRequest,
    pub authority: TransitionAuthority,
    pub reconciliation: ReconciliationMarker,
    pub disposition: TransitionDisposition,
    pub applied: bool,
    pub handoff: Option<TransitionHandoff>,
}

/// Deferred queue replay policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeferredTransitionPolicy {
    pub max_queue_capacity: usize,
    pub replay_batch_limit: usize,
    pub max_retry_attempts: u8,
    pub max_age_ticks: u64,
    pub retry_cooldown_ticks: u64,
}

impl Default for DeferredTransitionPolicy {
    fn default() -> Self {
        Self {
            max_queue_capacity: 4096,
            replay_batch_limit: 512,
            max_retry_attempts: 3,
            max_age_ticks: 240,
            retry_cooldown_ticks: 1,
        }
    }
}

/// Deferred queue entry for deterministic replay in later resolution windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeferredTransitionEntry {
    pub request: TransitionRequest,
    pub deferred_at_frame: u64,
    pub deferred_at_tick: u64,
    pub disposition: TransitionDisposition,
    pub retry_attempts: u8,
    pub next_retry_tick: u64,
}

/// Observable deferred queue stats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeferredTransitionQueueSnapshot {
    pub queued: usize,
    pub capacity: usize,
    pub overflow_dropped: usize,
    pub policy_dropped: usize,
}

/// Runtime-owned deferred transition queue contract.
#[derive(Clone, Debug)]
pub struct DeferredTransitionQueue {
    entries: VecDeque<DeferredTransitionEntry>,
    policy: DeferredTransitionPolicy,
    overflow_dropped: usize,
    policy_dropped: usize,
}

impl DeferredTransitionQueue {
    pub fn with_policy(policy: DeferredTransitionPolicy) -> Self {
        let entries = VecDeque::with_capacity(policy.max_queue_capacity);
        debug_assert!(entries.capacity() >= policy.max_queue_capacity);
        Self {
            entries,
            policy,
            overflow_dropped: 0,
            policy_dropped: 0,
        }
    }

    pub fn policy(&self) -> DeferredTransitionPolicy {
        self.policy
    }

    pub fn enqueue_new(
        &mut self,
        request: TransitionRequest,
        frame_index: u64,
        tick_index: u64,
        disposition: TransitionDisposition,
    ) -> bool {
        self.enqueue_entry(DeferredTransitionEntry {
            request,
            deferred_at_frame: frame_index,
            deferred_at_tick: tick_index,
            disposition,
            retry_attempts: 0,
            next_retry_tick: tick_index + self.policy.retry_cooldown_ticks,
        })
    }

    pub fn requeue_deferred(
        &mut self,
        mut entry: DeferredTransitionEntry,
        tick_index: u64,
    ) -> TransitionDisposition {
        entry.retry_attempts = entry.retry_attempts.saturating_add(1);

        if entry.retry_attempts > self.policy.max_retry_attempts
            || tick_index.saturating_sub(entry.deferred_at_tick) > self.policy.max_age_ticks
        {
            self.policy_dropped += 1;
            return TransitionDisposition::DroppedByQueuePolicy;
        }

        entry.next_retry_tick = tick_index + self.policy.retry_cooldown_ticks;
        if self.enqueue_entry(entry) {
            entry.disposition
        } else {
            TransitionDisposition::DeferredByQueueBackpressure
        }
    }

    pub fn drain_ready_into(
        &mut self,
        out: &mut Vec<TransitionRequest>,
        current_tick: u64,
        max_count: usize,
    ) -> usize {
        let mut drained = 0;
        while drained < max_count {
            let Some(entry) = self.entries.front().copied() else {
                break;
            };
            if entry.next_retry_tick > current_tick {
                break;
            }
            let entry = self.entries.pop_front().expect("front exists");
            out.push(entry.request);
            drained += 1;
        }
        drained
    }

    pub fn snapshot(&self) -> DeferredTransitionQueueSnapshot {
        DeferredTransitionQueueSnapshot {
            queued: self.entries.len(),
            capacity: self.policy.max_queue_capacity,
            overflow_dropped: self.overflow_dropped,
            policy_dropped: self.policy_dropped,
        }
    }

    fn enqueue_entry(&mut self, entry: DeferredTransitionEntry) -> bool {
        debug_assert!(self.entries.capacity() >= self.policy.max_queue_capacity);
        if self.entries.len() >= self.policy.max_queue_capacity {
            self.overflow_dropped += 1;
            return false;
        }
        self.entries.push_back(entry);
        true
    }
}

/// Minimal debug contract for simulation-level visibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimulationLevelDebugVisibility {
    pub l0_count: usize,
    pub l1_count: usize,
    pub l2_count: usize,
    pub l3_count: usize,
    pub pending_promotions: usize,
    pub pending_demotions: usize,
}

/// Runtime transition metrics snapshot for overlays/inspection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransitionMetricsSnapshot {
    pub requested: usize,
    pub applied: usize,
    pub deferred: usize,
    pub rejected: usize,
    pub dropped: usize,
    pub promotion_applied: usize,
    pub demotion_applied: usize,
    pub pending_promotions: usize,
    pub pending_demotions: usize,
    pub queue_depth: usize,
    pub queue_capacity: usize,
    pub queue_overflow_dropped: usize,
    pub queue_policy_dropped: usize,
}

/// Trace record for transition observability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionTraceRecord {
    pub frame: u64,
    pub tick: u64,
    pub subject: SimulationSubject,
    pub from: SimulationLevel,
    pub to: SimulationLevel,
    pub reason: TransitionReason,
    pub authority: TransitionAuthority,
    pub reconciliation: ReconciliationMarker,
    pub disposition: TransitionDisposition,
}

fn level_rank(level: SimulationLevel) -> u8 {
    match level {
        SimulationLevel::L0 => 0,
        SimulationLevel::L1 => 1,
        SimulationLevel::L2 => 2,
        SimulationLevel::L3 => 3,
    }
}

fn reason_rank(reason: TransitionReason) -> u8 {
    match reason {
        TransitionReason::PlayerProximity => 0,
        TransitionReason::TacticalRelevance => 1,
        TransitionReason::StreamingEnter => 2,
        TransitionReason::StreamingExit => 3,
        TransitionReason::InteractionEntropyLow => 4,
        TransitionReason::BudgetPressure => 5,
        TransitionReason::PersistenceLoad => 6,
        TransitionReason::PersistenceSave => 7,
    }
}
