//! Deterministic in-memory event-store adapter (`FR-014`, `NFR-003`).
//!
//! **Persistence note (`FR-014`):** `InMemoryEventStore` holds event streams exclusively
//! in process memory and loses all data when the process exits or restarts.

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::core::contracts::{
    validate_aggregate_id, DomainEvent, EventEnvelope, PendingEvent, Revision,
};
use crate::core::errors::EngineError;
use crate::core::store::EventStore;

/// Thread-safe in-memory implementation of [`EventStore`].
///
/// Serializes appends and checks `expected_revision` atomically per stream (`NFR-003`).
/// All data is volatile and lost on process restart (`FR-014`).
#[derive(Debug, Default)]
pub struct InMemoryEventStore<E> {
    streams: RwLock<BTreeMap<String, Vec<EventEnvelope<E>>>>,
}

impl<E: DomainEvent> InMemoryEventStore<E> {
    pub fn new() -> Self {
        Self {
            streams: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<E: DomainEvent> EventStore<E> for InMemoryEventStore<E> {
    fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope<E>>, EngineError> {
        validate_aggregate_id(aggregate_id)?;
        let guard = self
            .streams
            .read()
            .map_err(|_| EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id.to_string()),
                operation: "load_events".to_string(),
                reason: "in-memory store read lock poisoned".to_string(),
            })?;

        Ok(guard.get(aggregate_id).cloned().unwrap_or_default())
    }

    fn current_revision(&self, aggregate_id: &str) -> Result<Revision, EngineError> {
        validate_aggregate_id(aggregate_id)?;
        let guard = self
            .streams
            .read()
            .map_err(|_| EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id.to_string()),
                operation: "current_revision".to_string(),
                reason: "in-memory store read lock poisoned".to_string(),
            })?;

        Ok(guard
            .get(aggregate_id)
            .map(|events| events.len() as Revision)
            .unwrap_or(0))
    }

    fn append_events(
        &self,
        aggregate_id: &str,
        expected_revision: Revision,
        pending_events: &[PendingEvent<E>],
    ) -> Result<Vec<EventEnvelope<E>>, EngineError> {
        validate_aggregate_id(aggregate_id)?;

        let mut guard = self
            .streams
            .write()
            .map_err(|_| EngineError::StorageFailure {
                aggregate_id: Some(aggregate_id.to_string()),
                operation: "append_events".to_string(),
                reason: "in-memory store write lock poisoned".to_string(),
            })?;

        let actual_revision = guard
            .get(aggregate_id)
            .map(|events| events.len() as Revision)
            .unwrap_or(0);

        if actual_revision != expected_revision {
            return Err(EngineError::RevisionConflict {
                aggregate_id: aggregate_id.to_string(),
                expected_revision,
                actual_revision,
            });
        }

        if pending_events.is_empty() {
            return Ok(Vec::new());
        }

        // Validate and construct the entire batch before mutating the stream (FR-009 atomic batch)
        let mut new_envelopes = Vec::with_capacity(pending_events.len());
        for (offset, pending) in pending_events.iter().enumerate() {
            let next_revision = actual_revision + (offset as Revision) + 1;
            let envelope = EventEnvelope::new(
                aggregate_id.to_string(),
                next_revision,
                pending.event_id.clone(),
                pending.event_type.clone(),
                pending.schema_version,
                pending.payload.clone(),
                pending.recorded_at.clone(),
            )?;
            new_envelopes.push(envelope);
        }

        let stream = guard.entry(aggregate_id.to_string()).or_default();
        stream.extend(new_envelopes.clone());

        Ok(new_envelopes)
    }
}
