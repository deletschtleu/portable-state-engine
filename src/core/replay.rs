//! Pure deterministic event replay and historical revision reconstruction (`FR-002`, `FR-005`, `FR-006`, `FR-007`).

use crate::core::contracts::{AggregateDefinition, EventEnvelope, Revision};
use crate::core::errors::EngineError;

/// Reconstructed state together with the committed revision it represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayResult<S> {
    pub state: S,
    pub revision: Revision,
}

/// Validates that an event envelope slice has contiguous ascending revisions `1..=n`
/// and belongs to a single aggregate stream (`FR-007`, `FR-010`).
pub fn validate_contiguous_stream<E>(envelopes: &[EventEnvelope<E>]) -> Result<(), EngineError> {
    let Some(first) = envelopes.first() else {
        return Ok(());
    };
    let stream_aggregate_id = &first.aggregate_id;

    for (index, env) in envelopes.iter().enumerate() {
        let expected_revision = (index as u64) + 1;
        if env.aggregate_id != *stream_aggregate_id {
            return Err(EngineError::CorruptedStream {
                aggregate_id: Some(stream_aggregate_id.clone()),
                expected_revision,
                actual_revision: env.revision,
                reason: format!(
                    "mixed aggregate_id '{}' in stream for '{}'",
                    env.aggregate_id, stream_aggregate_id
                ),
            });
        }
        if env.revision != expected_revision {
            return Err(EngineError::CorruptedStream {
                aggregate_id: Some(stream_aggregate_id.clone()),
                expected_revision,
                actual_revision: env.revision,
                reason: "event stream revisions must be contiguous starting at 1".to_string(),
            });
        }
    }

    Ok(())
}

/// Folds a slice of domain events onto a starting state using `domain.evolve` without mutating inputs.
pub fn evolve_all<D: AggregateDefinition>(
    domain: &D,
    starting_state: &D::State,
    events: &[D::Event],
) -> D::State {
    events.iter().fold(starting_state.clone(), |acc, event| {
        domain.evolve(&acc, event)
    })
}

/// Reconstructs the current state and revision from an ordered stream of committed event envelopes (`FR-002`, `FR-005`).
pub fn replay<D: AggregateDefinition>(
    domain: &D,
    envelopes: &[EventEnvelope<D::Event>],
) -> Result<ReplayResult<D::State>, EngineError> {
    validate_contiguous_stream(envelopes)?;
    let state = envelopes.iter().fold(domain.initial_state(), |acc, env| {
        domain.evolve(&acc, &env.payload)
    });

    Ok(ReplayResult {
        state,
        revision: envelopes.len() as Revision,
    })
}

/// Reconstructs the historical state at `target_revision` (`0 <= target_revision <= current_revision`) (`FR-005`, `FR-006`).
pub fn replay_at_revision<D: AggregateDefinition>(
    domain: &D,
    envelopes: &[EventEnvelope<D::Event>],
    target_revision: i64,
) -> Result<ReplayResult<D::State>, EngineError> {
    validate_contiguous_stream(envelopes)?;
    let current_revision = envelopes.len() as Revision;
    let aggregate_id = envelopes.first().map(|e| e.aggregate_id.clone());

    if target_revision < 0 || (target_revision as u64) > current_revision {
        return Err(EngineError::InvalidRevision {
            aggregate_id,
            requested_revision: target_revision,
            current_revision,
            reason: format!(
                "requested revision {} is outside valid committed range 0..={}",
                target_revision, current_revision
            ),
        });
    }

    let prefix_len = target_revision as usize;
    let state = envelopes[..prefix_len]
        .iter()
        .fold(domain.initial_state(), |acc, env| {
            domain.evolve(&acc, &env.payload)
        });

    Ok(ReplayResult {
        state,
        revision: target_revision as Revision,
    })
}
