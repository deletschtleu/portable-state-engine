//! Generic domain contracts and event envelope types (`FR-001`, `FR-003`, `FR-016`).

use std::collections::BTreeMap;
use std::fmt;

use crate::core::errors::{DomainError, EngineError};

/// Nonnegative aggregate revision equal to the count of committed events (`0` initially).
pub type Revision = u64;

/// Explicit, deterministic context supplied at the boundary when needed (`SPEC.md` Section 2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExplicitContext {
    pub command_id: Option<String>,
    pub actor_id: Option<String>,
    pub recorded_at: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

impl ExplicitContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command_id(mut self, command_id: impl Into<String>) -> Self {
        self.command_id = Some(command_id.into());
        self
    }

    pub fn with_actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    pub fn with_recorded_at(mut self, recorded_at: impl Into<String>) -> Self {
        self.recorded_at = Some(recorded_at.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Trait implemented by domain event types so envelopes capture stable type names and schema versions (`FR-016`).
pub trait DomainEvent: Clone + PartialEq + Eq + fmt::Debug + Send + Sync {
    fn event_type(&self) -> &'static str;

    fn schema_version(&self) -> u32 {
        1
    }
}

/// Pure domain aggregate contract parameterized by immutable `State`, `Command`, and `Event` (`FR-001`, `FR-003`).
pub trait AggregateDefinition {
    type State: Clone + PartialEq + Eq + fmt::Debug;
    type Command: Clone + PartialEq + Eq + fmt::Debug;
    type Event: DomainEvent;

    /// Returns the deterministic initial state at revision `0`.
    fn initial_state(&self) -> Self::State;

    /// Pure decision function: validates `command` against `state` and `context`, returning
    /// either zero or more accepted events or a `DomainError` rejection.
    fn decide(
        &self,
        state: &Self::State,
        command: &Self::Command,
        context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError>;

    /// Pure transition function: folds a single committed `event` into `state` to produce the next state.
    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State;
}

/// Uncommitted event prepared for atomic append to an aggregate stream (`FR-016`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEvent<E> {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: u32,
    pub payload: E,
    pub recorded_at: String,
}

impl<E: DomainEvent> PendingEvent<E> {
    pub fn from_domain_event(
        event_id: impl Into<String>,
        payload: E,
        recorded_at: impl Into<String>,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            event_type: payload.event_type().to_string(),
            schema_version: payload.schema_version(),
            payload,
            recorded_at: recorded_at.into(),
        }
    }
}

/// Committed event envelope persisted in an aggregate stream (`FR-016`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope<E> {
    pub aggregate_id: String,
    pub revision: Revision,
    pub event_id: String,
    pub event_type: String,
    pub schema_version: u32,
    pub payload: E,
    pub recorded_at: String,
}

impl<E> EventEnvelope<E> {
    pub fn new(
        aggregate_id: impl Into<String>,
        revision: Revision,
        event_id: impl Into<String>,
        event_type: impl Into<String>,
        schema_version: u32,
        payload: E,
        recorded_at: impl Into<String>,
    ) -> Result<Self, EngineError> {
        let aggregate_id = aggregate_id.into();
        validate_aggregate_id(&aggregate_id)?;

        if revision == 0 {
            return Err(EngineError::InvalidRevision {
                aggregate_id: Some(aggregate_id),
                requested_revision: 0,
                current_revision: 0,
                reason: "committed event envelope revision must be >= 1".to_string(),
            });
        }

        let event_id = event_id.into();
        if event_id.trim().is_empty() {
            return Err(EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id),
                operation: "envelope_validation".to_string(),
                reason: "event_id must not be blank".to_string(),
            });
        }

        let event_type = event_type.into();
        if event_type.trim().is_empty() {
            return Err(EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id),
                operation: "envelope_validation".to_string(),
                reason: "event_type must not be blank".to_string(),
            });
        }

        if schema_version == 0 {
            return Err(EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id),
                operation: "envelope_validation".to_string(),
                reason: "schema_version must be >= 1".to_string(),
            });
        }

        Ok(Self {
            aggregate_id,
            revision,
            event_id,
            event_type,
            schema_version,
            payload,
            recorded_at: recorded_at.into(),
        })
    }
}

/// Validates that an `aggregate_id` is non-empty and not whitespace-only (`FR-013`).
pub fn validate_aggregate_id(aggregate_id: &str) -> Result<(), EngineError> {
    if aggregate_id.trim().is_empty() {
        return Err(EngineError::InvalidAggregateId {
            aggregate_id: aggregate_id.to_string(),
            reason: "aggregate_id must not be empty or whitespace-only".to_string(),
        });
    }
    Ok(())
}
