use crate::simulation_core::contracts::{
    DeferredTransitionQueue, ReconciliationMarker, SimulationLevel, SimulationLevelDebugVisibility,
    SimulationPolicyProfile, SimulationSubject, TransitionAuthority, TransitionDisposition,
    TransitionExecutionContext, TransitionHandoff, TransitionMetricsSnapshot,
    TransitionOrderingPolicy, TransitionReason, TransitionRequest, TransitionResult,
    TransitionShardMergePolicy, TransitionShardOutput, TransitionTraceRecord,
};

#[inline(always)]
fn request_sort_key(request: TransitionRequest) -> (u64, u8, u8, u8) {
    let (subject, from, to, reason) = match request {
        TransitionRequest::Promote(r) => (r.subject.entity_id, r.from, r.to, r.reason),
        TransitionRequest::Demote(r) => (r.subject.entity_id, r.from, r.to, r.reason),
    };
    (
        subject,
        level_rank(from),
        level_rank(to),
        reason_rank(reason),
    )
}

#[inline(always)]
fn compare_requests(left: TransitionRequest, right: TransitionRequest) -> std::cmp::Ordering {
    request_sort_key(left).cmp(&request_sort_key(right))
}

#[derive(Clone, Copy)]
struct MergeCursor {
    shard_idx: usize,
    req_idx: usize,
    request: TransitionRequest,
    shard_id: u16,
}

#[inline(always)]
fn compare_cursor(left: MergeCursor, right: MergeCursor) -> std::cmp::Ordering {
    compare_requests(left.request, right.request).then(left.shard_id.cmp(&right.shard_id))
}

#[inline(always)]
fn heap_sift_up(heap: &mut [MergeCursor; 8], mut pos: usize) {
    while pos > 0 {
        let parent = (pos - 1) / 2;
        if compare_cursor(heap[pos], heap[parent]).is_lt() {
            heap.swap(pos, parent);
            pos = parent;
        } else {
            break;
        }
    }
}

#[inline(always)]
fn heap_sift_down(heap: &mut [MergeCursor; 8], mut pos: usize, len: usize) {
    loop {
        let left = pos * 2 + 1;
        if left >= len {
            break;
        }
        let right = left + 1;
        let mut best = left;
        if right < len && compare_cursor(heap[right], heap[left]).is_lt() {
            best = right;
        }
        if compare_cursor(heap[best], heap[pos]).is_lt() {
            heap.swap(best, pos);
            pos = best;
        } else {
            break;
        }
    }
}

#[inline(always)]
fn level_rank(level: SimulationLevel) -> u8 {
    match level {
        SimulationLevel::L0 => 0,
        SimulationLevel::L1 => 1,
        SimulationLevel::L2 => 2,
        SimulationLevel::L3 => 3,
    }
}

#[inline(always)]
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

pub trait TransitionTracer {
    fn on_transition(&mut self, record: TransitionTraceRecord);
}

/// Runtime integration point for streaming/persistence handoff consumption.
pub trait TransitionHandoffSink {
    fn on_transition_handoff(&mut self, handoff: &TransitionHandoff);
}

/// Runtime-owned orchestration skeleton for simulation-level transitions.
#[derive(Clone, Copy, Debug)]
pub struct TransitionOrchestrator {
    policy: SimulationPolicyProfile,
    ordering: TransitionOrderingPolicy,
}

impl Default for TransitionOrchestrator {
    fn default() -> Self {
        Self::new(
            SimulationPolicyProfile::default(),
            TransitionOrderingPolicy::SubjectThenFromToReason,
        )
    }
}

impl TransitionOrchestrator {
    pub fn new(policy: SimulationPolicyProfile, ordering: TransitionOrderingPolicy) -> Self {
        Self { policy, ordering }
    }

    pub fn policy(self) -> SimulationPolicyProfile {
        self.policy
    }

    pub fn ordering(self) -> TransitionOrderingPolicy {
        self.ordering
    }

    pub fn default_context(self, frame_index: u64, tick_index: u64) -> TransitionExecutionContext {
        TransitionExecutionContext::from_policy(self.policy, frame_index, tick_index)
    }

    pub fn classify_distance(self, distance: f32) -> SimulationLevel {
        self.policy.classify_distance(distance)
    }

    pub fn make_promotion_request(
        self,
        subject: SimulationSubject,
        from: SimulationLevel,
        to: SimulationLevel,
        reason: TransitionReason,
    ) -> crate::simulation_core::contracts::PromotionRequest {
        crate::simulation_core::contracts::PromotionRequest {
            subject,
            from,
            to,
            reason,
        }
    }

    pub fn make_demotion_request(
        self,
        subject: SimulationSubject,
        from: SimulationLevel,
        to: SimulationLevel,
        reason: TransitionReason,
    ) -> crate::simulation_core::contracts::DemotionRequest {
        crate::simulation_core::contracts::DemotionRequest {
            subject,
            from,
            to,
            reason,
        }
    }

    pub fn order_batch(self, requests: &mut [TransitionRequest]) {
        requests.sort_unstable_by(|a, b| self.ordering.compare(*a, *b));
    }

    /// Deterministic shard merge prep contract for future parallel execution.
    ///
    /// Reduction rule:
    /// 1. Accept up to `max_merge_inputs` shard outputs.
    /// 2. Sort by (`request` under `ordering`, `shard_id`) to break ties deterministically.
    /// 3. Emit exactly one merged ordered batch for subsequent bounded resolution.
    pub fn merge_shard_outputs(
        self,
        shard_outputs: &[TransitionShardOutput],
        merge_policy: TransitionShardMergePolicy,
        out: &mut Vec<TransitionRequest>,
        scratch: &mut Vec<(u16, TransitionRequest)>,
    ) {
        out.clear();
        scratch.clear();

        let merge_inputs = shard_outputs.len().min(merge_policy.max_merge_inputs);
        let inputs = &shard_outputs[..merge_inputs];

        if inputs.is_empty() {
            return;
        }

        let mut total = 0usize;
        for shard in inputs {
            total += shard.requests.len();
        }
        out.reserve(total.saturating_sub(out.capacity()));

        let mut can_kway_merge = true;
        for shard in inputs {
            for pair in shard.requests.windows(2) {
                if compare_requests(pair[0], pair[1]).is_gt() {
                    can_kway_merge = false;
                    break;
                }
            }
            if !can_kway_merge {
                break;
            }
        }

        if can_kway_merge {
            let mut concat_ok = true;
            let mut prev_last = 0u64;
            let mut has_prev = false;
            for shard in inputs {
                if shard.requests.is_empty() {
                    continue;
                }
                let first_id = shard.requests[0].subject().entity_id;
                let last_id = shard.requests[shard.requests.len() - 1].subject().entity_id;
                if has_prev && prev_last >= first_id {
                    concat_ok = false;
                    break;
                }
                has_prev = true;
                prev_last = last_id;
            }

            if concat_ok {
                for shard in inputs {
                    out.extend_from_slice(&shard.requests);
                }
                return;
            }

            let mut heap = [MergeCursor {
                shard_idx: 0,
                req_idx: 0,
                request: TransitionRequest::Promote(
                    crate::simulation_core::contracts::PromotionRequest {
                        subject: SimulationSubject { entity_id: 0 },
                        from: SimulationLevel::L1,
                        to: SimulationLevel::L1,
                        reason: TransitionReason::PlayerProximity,
                    },
                ),
                shard_id: 0,
            }; 8];
            let mut heap_len = 0usize;

            for (shard_idx, shard) in inputs.iter().enumerate() {
                if let Some(first) = shard.requests.first().copied() {
                    heap[heap_len] = MergeCursor {
                        shard_idx,
                        req_idx: 0,
                        request: first,
                        shard_id: shard.shard_id,
                    };
                    heap_sift_up(&mut heap, heap_len);
                    heap_len += 1;
                }
            }

            while heap_len > 0 {
                let top = heap[0];
                out.push(top.request);

                let next_idx = top.req_idx + 1;
                let shard = &inputs[top.shard_idx];
                if next_idx < shard.requests.len() {
                    heap[0] = MergeCursor {
                        shard_idx: top.shard_idx,
                        req_idx: next_idx,
                        request: shard.requests[next_idx],
                        shard_id: top.shard_id,
                    };
                    heap_sift_down(&mut heap, 0, heap_len);
                } else {
                    heap_len -= 1;
                    if heap_len > 0 {
                        heap[0] = heap[heap_len];
                        heap_sift_down(&mut heap, 0, heap_len);
                    }
                }
            }
            return;
        }

        scratch.reserve(total.saturating_sub(scratch.capacity()));
        for shard in inputs {
            for request in shard.requests.iter().copied() {
                scratch.push((shard.shard_id, request));
            }
        }

        scratch.sort_unstable_by(|left, right| {
            merge_policy
                .ordering
                .compare(left.1, right.1)
                .then(left.0.cmp(&right.0))
        });

        out.extend(scratch.iter().map(|(_, req)| *req));
    }

    pub fn resolve_ordered_batch(
        self,
        requests: &mut Vec<TransitionRequest>,
        context: &mut TransitionExecutionContext,
        queue: &mut DeferredTransitionQueue,
        metrics: &mut TransitionMetricsSnapshot,
    ) {
        self.order_batch(requests.as_mut_slice());

        for request in requests.iter().copied() {
            metrics.requested += 1;

            let (from, to, is_promotion) = match request {
                TransitionRequest::Promote(r) => (r.from, r.to, true),
                TransitionRequest::Demote(r) => (r.from, r.to, false),
            };

            let disposition = if from == to {
                TransitionDisposition::RejectedNoLevelChange
            } else if is_promotion {
                if context.promotion_budget == 0 {
                    TransitionDisposition::DeferredByPromotionBudget
                } else {
                    context.promotion_budget -= 1;
                    TransitionDisposition::Applied
                }
            } else if context.demotion_budget == 0 {
                TransitionDisposition::DeferredByDemotionBudget
            } else {
                context.demotion_budget -= 1;
                TransitionDisposition::Applied
            };

            match disposition {
                TransitionDisposition::Applied => {
                    metrics.applied += 1;
                    if is_promotion {
                        metrics.promotion_applied += 1;
                    } else {
                        metrics.demotion_applied += 1;
                    }
                }
                TransitionDisposition::DeferredByPromotionBudget
                | TransitionDisposition::DeferredByDemotionBudget => {
                    queue.enqueue_new(
                        request,
                        context.frame_index,
                        context.tick_index,
                        disposition,
                    );
                    metrics.deferred += 1;
                    if is_promotion {
                        metrics.pending_promotions += 1;
                    } else {
                        metrics.pending_demotions += 1;
                    }
                }
                TransitionDisposition::DeferredByQueueBackpressure => {
                    metrics.deferred += 1;
                    if is_promotion {
                        metrics.pending_promotions += 1;
                    } else {
                        metrics.pending_demotions += 1;
                    }
                }
                TransitionDisposition::DroppedByQueuePolicy => {
                    metrics.dropped += 1;
                }
                TransitionDisposition::RejectedNoLevelChange => {
                    metrics.rejected += 1;
                }
            }
        }

        let queue_snapshot = queue.snapshot();
        metrics.queue_depth = queue_snapshot.queued;
        metrics.queue_capacity = queue_snapshot.capacity;
        metrics.queue_overflow_dropped = queue_snapshot.overflow_dropped;
        metrics.queue_policy_dropped = queue_snapshot.policy_dropped;
    }

    pub fn resolve_request(
        self,
        request: TransitionRequest,
        context: &mut TransitionExecutionContext,
        tracer: Option<&mut dyn TransitionTracer>,
        handoff_sink: Option<&mut dyn TransitionHandoffSink>,
    ) -> TransitionResult {
        let authority = authority_for_target_level(request.to());
        let reconciliation = reconciliation_marker_for_request(request);

        let disposition = if request.from() == request.to() {
            TransitionDisposition::RejectedNoLevelChange
        } else if request.is_promotion() {
            if context.promotion_budget == 0 {
                TransitionDisposition::DeferredByPromotionBudget
            } else {
                context.promotion_budget -= 1;
                TransitionDisposition::Applied
            }
        } else if context.demotion_budget == 0 {
            TransitionDisposition::DeferredByDemotionBudget
        } else {
            context.demotion_budget -= 1;
            TransitionDisposition::Applied
        };

        let applied = matches!(disposition, TransitionDisposition::Applied);
        let handoff = if applied {
            Some(TransitionHandoff {
                subject: request.subject(),
                from: request.from(),
                to: request.to(),
                authority,
                deterministic_seed: Some(context.frame_index ^ request.subject().entity_id),
                snapshot_key: None,
            })
        } else {
            None
        };

        let result = TransitionResult {
            request,
            authority,
            reconciliation,
            disposition,
            applied,
            handoff,
        };

        if let Some(tracer) = tracer {
            tracer.on_transition(TransitionTraceRecord {
                frame: context.frame_index,
                tick: context.tick_index,
                subject: request.subject(),
                from: request.from(),
                to: request.to(),
                reason: request.reason(),
                authority,
                reconciliation,
                disposition,
            });
        }

        if let (Some(sink), Some(handoff)) = (handoff_sink, result.handoff.as_ref()) {
            sink.on_transition_handoff(handoff);
        }

        result
    }

    pub fn accumulate_metrics(
        self,
        metrics: &mut TransitionMetricsSnapshot,
        result: &TransitionResult,
    ) {
        metrics.requested += 1;

        match result.disposition {
            TransitionDisposition::Applied => {
                metrics.applied += 1;
                if result.request.is_promotion() {
                    metrics.promotion_applied += 1;
                } else {
                    metrics.demotion_applied += 1;
                }
            }
            TransitionDisposition::DeferredByPromotionBudget
            | TransitionDisposition::DeferredByDemotionBudget
            | TransitionDisposition::DeferredByQueueBackpressure => {
                metrics.deferred += 1;
                if result.request.is_promotion() {
                    metrics.pending_promotions += 1;
                } else {
                    metrics.pending_demotions += 1;
                }
            }
            TransitionDisposition::DroppedByQueuePolicy => {
                metrics.dropped += 1;
            }
            TransitionDisposition::RejectedNoLevelChange => {
                metrics.rejected += 1;
            }
        }
    }

    pub fn compute_debug_visibility<'a, I>(
        self,
        levels: I,
        metrics: TransitionMetricsSnapshot,
    ) -> SimulationLevelDebugVisibility
    where
        I: IntoIterator<Item = &'a SimulationLevel>,
    {
        let mut visibility = SimulationLevelDebugVisibility {
            pending_promotions: metrics.pending_promotions,
            pending_demotions: metrics.pending_demotions,
            ..SimulationLevelDebugVisibility::default()
        };

        for level in levels {
            match level {
                SimulationLevel::L0 => visibility.l0_count += 1,
                SimulationLevel::L1 => visibility.l1_count += 1,
                SimulationLevel::L2 => visibility.l2_count += 1,
                SimulationLevel::L3 => visibility.l3_count += 1,
            }
        }
        visibility
    }
}

fn authority_for_target_level(level: SimulationLevel) -> TransitionAuthority {
    match level {
        SimulationLevel::L0 | SimulationLevel::L1 => TransitionAuthority::LocalActiveTruth,
        SimulationLevel::L2 => TransitionAuthority::RegionalAggregate,
        SimulationLevel::L3 => TransitionAuthority::GlobalAggregate,
    }
}

fn reconciliation_marker_for_request(request: TransitionRequest) -> ReconciliationMarker {
    match request {
        TransitionRequest::Promote(_) => ReconciliationMarker::UpgradedFromAggregate,
        TransitionRequest::Demote(_) => ReconciliationMarker::CollapsedToAggregate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_deterministic_by_subject_then_levels() {
        let orchestrator = TransitionOrchestrator::default();
        let mut requests = vec![
            TransitionRequest::Demote(orchestrator.make_demotion_request(
                SimulationSubject { entity_id: 7 },
                SimulationLevel::L0,
                SimulationLevel::L2,
                TransitionReason::InteractionEntropyLow,
            )),
            TransitionRequest::Promote(orchestrator.make_promotion_request(
                SimulationSubject { entity_id: 2 },
                SimulationLevel::L1,
                SimulationLevel::L0,
                TransitionReason::PlayerProximity,
            )),
        ];

        orchestrator.order_batch(&mut requests);
        assert_eq!(requests[0].subject().entity_id, 2);
        assert_eq!(requests[1].subject().entity_id, 7);
    }

    #[test]
    fn merge_shards_is_deterministic() {
        let orchestrator = TransitionOrchestrator::default();
        let shards = vec![
            TransitionShardOutput {
                shard_id: 2,
                requests: vec![TransitionRequest::Promote(
                    orchestrator.make_promotion_request(
                        SimulationSubject { entity_id: 7 },
                        SimulationLevel::L1,
                        SimulationLevel::L0,
                        TransitionReason::PlayerProximity,
                    ),
                )],
            },
            TransitionShardOutput {
                shard_id: 1,
                requests: vec![TransitionRequest::Promote(
                    orchestrator.make_promotion_request(
                        SimulationSubject { entity_id: 7 },
                        SimulationLevel::L1,
                        SimulationLevel::L0,
                        TransitionReason::PlayerProximity,
                    ),
                )],
            },
        ];

        let mut out = Vec::new();
        let mut scratch = Vec::new();
        orchestrator.merge_shard_outputs(
            &shards,
            TransitionShardMergePolicy::default(),
            &mut out,
            &mut scratch,
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn deferred_requests_are_queued() {
        let orchestrator = TransitionOrchestrator::default();
        let mut requests = vec![TransitionRequest::Promote(
            orchestrator.make_promotion_request(
                SimulationSubject { entity_id: 11 },
                SimulationLevel::L1,
                SimulationLevel::L0,
                TransitionReason::PlayerProximity,
            ),
        )];
        let mut context = TransitionExecutionContext {
            frame_index: 1,
            tick_index: 1,
            promotion_budget: 0,
            demotion_budget: 1,
        };
        let mut queue = DeferredTransitionQueue::with_policy(
            crate::simulation_core::contracts::DeferredTransitionPolicy::default(),
        );
        let mut metrics = TransitionMetricsSnapshot::default();
        orchestrator.resolve_ordered_batch(&mut requests, &mut context, &mut queue, &mut metrics);
        assert_eq!(metrics.deferred, 1);
        assert_eq!(queue.snapshot().queued, 1);
    }
}
