//! Event-store boundary contract (`FR-007`..`FR-011`, `FR-016`, `FR-024`).

use crate::core::contracts::{DomainEvent, EventEnvelope, PendingEvent, Revision};
use crate::core::errors::EngineError;

/// Replaceable storage boundary for aggregate event streams.
pub trait EventStore<E: DomainEvent>: Send + Sync {
    /// Loads all committed event envelopes for `aggregate_id` in ascending revision order (`1..=n`).
    /// Returns an empty vector (revision `0`) if no events have been committed for `aggregate_id`.
    fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope<E>>, EngineError>;

    /// Returns the current committed revision for `aggregate_id` (`0` if empty).
    fn current_revision(&self, aggregate_id: &str) -> Result<Revision, EngineError> {
        let events = self.load_events(aggregate_id)?;
        Ok(events.len() as Revision)
    }

    /// Atomically appends `pending_events` to `aggregate_id` if and only if its current revision
    /// equals `expected_revision` (`FR-008`, `FR-009`).
    ///
    /// Returns the newly committed `EventEnvelope<E>` records in ascending revision order.
    fn append_events(
        &self,
        aggregate_id: &str,
        expected_revision: Revision,
        pending_events: &[PendingEvent<E>],
    ) -> Result<Vec<EventEnvelope<E>>, EngineError>;
}
