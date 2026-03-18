use engine_runtime::simulation_core::{
    DeferredTransitionQueue, SimulationSubject, TransitionDisposition, TransitionExecutionContext,
    TransitionHandoff, TransitionHandoffSink, TransitionMetricsSnapshot, TransitionOrchestrator,
    TransitionRequest,
};

use crate::core::ecs::Ecs;
use crate::simulation::simulation_level::{
    runtime_to_world_level, to_demotion_reason, to_promotion_reason, world_to_runtime_level,
};
use crate::world::components::SimLevel;

pub fn update_simulation_levels(
    ecs: &mut Ecs,
    player_x: f32,
    player_y: f32,
    frame_index: u64,
    tick_index: u64,
    deferred_queue: &mut DeferredTransitionQueue,
    transition_batch: &mut Vec<TransitionRequest>,
) -> TransitionMetricsSnapshot {
    let entities: Vec<_> = ecs.alive.clone();
    let orchestrator = TransitionOrchestrator::default();
    let mut context =
        TransitionExecutionContext::from_policy(orchestrator.policy(), frame_index, tick_index);
    let mut metrics = TransitionMetricsSnapshot::default();
    let mut handoff_sink = MonolithTransitionHandoffSink;

    transition_batch.clear();
    debug_assert_eq!(transition_batch.len(), 0);
    debug_assert!(transition_batch.capacity() >= deferred_queue.policy().replay_batch_limit);
    let replay_limit = deferred_queue
        .policy()
        .replay_batch_limit
        .min(transition_batch.capacity());
    deferred_queue.drain_ready_into(transition_batch, tick_index, replay_limit);

    for entity in entities {
        let distance = {
            let t = match ecs.get_transform(entity) {
                Some(t) => t,
                None => continue,
            };
            ((t.x - player_x).powi(2) + (t.y - player_y).powi(2)).sqrt()
        };

        let current_world_level = ecs
            .get_sim_level(entity)
            .map(|sim| sim.level)
            .unwrap_or_else(|| runtime_to_world_level(orchestrator.classify_distance(distance)));
        let current_runtime_level = world_to_runtime_level(current_world_level);
        let target_runtime_level = orchestrator.classify_distance(distance);

        if target_runtime_level == current_runtime_level {
            continue;
        }

        let request = if level_rank(target_runtime_level) < level_rank(current_runtime_level) {
            TransitionRequest::Promote(orchestrator.make_promotion_request(
                SimulationSubject {
                    entity_id: entity as u64,
                },
                current_runtime_level,
                target_runtime_level,
                to_promotion_reason(distance),
            ))
        } else {
            TransitionRequest::Demote(orchestrator.make_demotion_request(
                SimulationSubject {
                    entity_id: entity as u64,
                },
                current_runtime_level,
                target_runtime_level,
                to_demotion_reason(distance),
            ))
        };

        if transition_batch.len() < transition_batch.capacity() {
            transition_batch.push(request);
        } else {
            let queued = deferred_queue.enqueue_new(
                request,
                frame_index,
                tick_index,
                TransitionDisposition::DeferredByQueueBackpressure,
            );
            if queued {
                metrics.deferred += 1;
            } else {
                metrics.dropped += 1;
            }
        }
    }

    orchestrator.order_batch(transition_batch);

    for request in transition_batch.iter().copied() {
        let result =
            orchestrator.resolve_request(request, &mut context, None, Some(&mut handoff_sink));

        if matches!(
            result.disposition,
            TransitionDisposition::DeferredByPromotionBudget
                | TransitionDisposition::DeferredByDemotionBudget
        ) {
            let disposition = deferred_queue.requeue_deferred(
                engine_runtime::simulation_core::DeferredTransitionEntry {
                    request,
                    deferred_at_frame: frame_index,
                    deferred_at_tick: tick_index,
                    disposition: result.disposition,
                    retry_attempts: 0,
                    next_retry_tick: tick_index,
                },
                tick_index,
            );
            if matches!(disposition, TransitionDisposition::DroppedByQueuePolicy) {
                metrics.dropped += 1;
            }
        }

        orchestrator.accumulate_metrics(&mut metrics, &result);

        if result.applied {
            ecs.set_sim_level(
                request.subject().entity_id,
                SimLevel {
                    level: runtime_to_world_level(request.to()),
                },
            );
        }
    }

    let queue_snapshot = deferred_queue.snapshot();
    metrics.queue_depth = queue_snapshot.queued;
    metrics.queue_capacity = queue_snapshot.capacity;
    metrics.queue_overflow_dropped = queue_snapshot.overflow_dropped;
    metrics.queue_policy_dropped = queue_snapshot.policy_dropped;

    metrics
}

struct MonolithTransitionHandoffSink;

impl TransitionHandoffSink for MonolithTransitionHandoffSink {
    fn on_transition_handoff(&mut self, _handoff: &TransitionHandoff) {
        // Compatibility bridge: runtime handoff contracts are acknowledged here,
        // while full world/persistence streaming integration remains in future batches.
    }
}

fn level_rank(level: engine_runtime::simulation_core::SimulationLevel) -> u8 {
    match level {
        engine_runtime::simulation_core::SimulationLevel::L0 => 0,
        engine_runtime::simulation_core::SimulationLevel::L1 => 1,
        engine_runtime::simulation_core::SimulationLevel::L2 => 2,
        engine_runtime::simulation_core::SimulationLevel::L3 => 3,
    }
}
