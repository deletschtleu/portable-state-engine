//! Generic command runner and read-only historical query service (`FR-004`..`FR-009`, `FR-012`, `FR-024`).

use crate::core::contracts::{
    validate_aggregate_id, AggregateDefinition, EventEnvelope, ExplicitContext, PendingEvent,
    Revision,
};
use crate::core::errors::EngineError;
use crate::core::replay::{evolve_all, replay, replay_at_revision, ReplayResult};
use crate::core::store::EventStore;

const DEFAULT_RECORDED_AT: &str = "1970-01-01T00:00:00Z";

/// Per-command execution options including explicit context, optimistic revision pin, and bounded conflict retries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionOptions {
    pub context: ExplicitContext,
    pub expected_revision: Option<Revision>,
    pub max_retries: usize,
}

impl ExecutionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_context(mut self, context: ExplicitContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_expected_revision(mut self, expected_revision: Revision) -> Self {
        self.expected_revision = Some(expected_revision);
        self
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Outcome of a command execution against an aggregate stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult<S, E> {
    pub aggregate_id: String,
    pub state: S,
    pub revision: Revision,
    pub committed_events: Vec<EventEnvelope<E>>,
    pub attempts: usize,
}

/// Generic application runner orchestrating `load -> decide -> append -> evolve` (`FR-012`).
#[derive(Debug)]
pub struct EngineRunner<D, S> {
    domain: D,
    store: S,
}

impl<D, S> EngineRunner<D, S>
where
    D: AggregateDefinition,
    S: EventStore<D::Event>,
{
    pub fn new(domain: D, store: S) -> Self {
        Self { domain, store }
    }

    pub fn domain(&self) -> &D {
        &self.domain
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn execute(
        &self,
        aggregate_id: &str,
        command: &D::Command,
    ) -> Result<ExecutionResult<D::State, D::Event>, EngineError> {
        self.execute_with_options(aggregate_id, command, &ExecutionOptions::default())
    }

    pub fn execute_with_options(
        &self,
        aggregate_id: &str,
        command: &D::Command,
        options: &ExecutionOptions,
    ) -> Result<ExecutionResult<D::State, D::Event>, EngineError> {
        validate_aggregate_id(aggregate_id)?;

        for attempt in 0..=options.max_retries {
            let history = self.store.load_events(aggregate_id)?;
            let current = replay(&self.domain, &history)?;

            if let Some(caller_expected) = options.expected_revision {
                if current.revision != caller_expected {
                    return Err(EngineError::RevisionConflict {
                        aggregate_id: aggregate_id.to_string(),
                        expected_revision: caller_expected,
                        actual_revision: current.revision,
                    });
                }
            }

            let decided_events = self
                .domain
                .decide(&current.state, command, &options.context)
                .map_err(|err| EngineError::domain_rejection(Some(aggregate_id), err))?;

            if decided_events.is_empty() {
                return Ok(ExecutionResult {
                    aggregate_id: aggregate_id.to_string(),
                    state: current.state,
                    revision: current.revision,
                    committed_events: Vec::new(),
                    attempts: attempt + 1,
                });
            }

            let recorded_at = options
                .context
                .recorded_at
                .as_deref()
                .unwrap_or(DEFAULT_RECORDED_AT);

            let pending_batch: Vec<PendingEvent<D::Event>> = decided_events
                .iter()
                .enumerate()
                .map(|(offset, event)| {
                    let target_rev = current.revision + (offset as Revision) + 1;
                    let event_id = match &options.context.command_id {
                        Some(cmd_id) => format!("{}:{}:rev-{}", aggregate_id, cmd_id, target_rev),
                        None => format!("{}:rev-{}", aggregate_id, target_rev),
                    };
                    PendingEvent::from_domain_event(event_id, event.clone(), recorded_at)
                })
                .collect();

            match self
                .store
                .append_events(aggregate_id, current.revision, &pending_batch)
            {
                Ok(committed_events) => {
                    let next_state = evolve_all(&self.domain, &current.state, &decided_events);
                    let next_revision = current.revision + (committed_events.len() as Revision);
                    return Ok(ExecutionResult {
                        aggregate_id: aggregate_id.to_string(),
                        state: next_state,
                        revision: next_revision,
                        committed_events,
                        attempts: attempt + 1,
                    });
                }
                Err(EngineError::RevisionConflict { .. })
                    if options.expected_revision.is_none() && attempt < options.max_retries =>
                {
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        unreachable!("loop always returns within 0..=max_retries")
    }

    /// Reconstructs the current state and revision of `aggregate_id` without modifying stored history.
    pub fn get_current_state(
        &self,
        aggregate_id: &str,
    ) -> Result<ReplayResult<D::State>, EngineError> {
        let history = self.store.load_events(aggregate_id)?;
        replay(&self.domain, &history)
    }

    /// Reconstructs the read-only historical state of `aggregate_id` at `target_revision` (`0 <= r <= current`) (`FR-005`, `FR-006`).
    pub fn get_state_at_revision(
        &self,
        aggregate_id: &str,
        target_revision: i64,
    ) -> Result<ReplayResult<D::State>, EngineError> {
        let history = self.store.load_events(aggregate_id)?;
        replay_at_revision(&self.domain, &history, target_revision)
    }

    /// Lists committed event envelopes for `aggregate_id` ordered by revision (`FR-024`).
    pub fn get_history(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<D::Event>>, EngineError> {
        self.store.load_events(aggregate_id)
    }
}
