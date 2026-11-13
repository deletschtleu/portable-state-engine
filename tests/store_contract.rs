use std::sync::{Arc, Barrier};
use std::thread;

use portable_state_engine::adapters::memory_store::InMemoryEventStore;
use portable_state_engine::core::contracts::{DomainEvent, PendingEvent};
use portable_state_engine::core::errors::{EngineError, ErrorCategory};
use portable_state_engine::core::store::EventStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleStoreEvent {
    ItemAdded(String),
    ItemRemoved(String),
}

impl DomainEvent for SampleStoreEvent {
    fn event_type(&self) -> &'static str {
        match self {
            SampleStoreEvent::ItemAdded(_) => "ItemAdded",
            SampleStoreEvent::ItemRemoved(_) => "ItemRemoved",
        }
    }
}

fn pending(id: &str, item: &str) -> PendingEvent<SampleStoreEvent> {
    PendingEvent::from_domain_event(
        id,
        SampleStoreEvent::ItemAdded(item.to_string()),
        "2026-11-13T00:00:00Z",
    )
}

/// Reusable contract test suite that any `EventStore<SampleStoreEvent>` implementation must pass (`TASK-004`).
pub fn run_event_store_contract_suite<S, F>(create_store: F)
where
    S: EventStore<SampleStoreEvent> + 'static,
    F: Fn() -> S,
{
    // 1. Empty stream starts at revision 0
    {
        let store = create_store();
        assert_eq!(store.current_revision("agg-1").unwrap(), 0);
        assert!(store.load_events("agg-1").unwrap().is_empty());
    }

    // 2. Atomic batch append assigns contiguous revisions 1..=k and empty batch is a no-op
    {
        let store = create_store();
        let batch1 = vec![pending("e1", "first"), pending("e2", "second")];
        let committed1 = store
            .append_events("agg-1", 0, &batch1)
            .expect("initial batch append at rev 0 should succeed");
        assert_eq!(committed1.len(), 2);
        assert_eq!(committed1[0].revision, 1);
        assert_eq!(committed1[1].revision, 2);
        assert_eq!(store.current_revision("agg-1").unwrap(), 2);

        // Empty batch at matching expected_revision = 2 is a no-op
        let noop_committed = store
            .append_events("agg-1", 2, &[])
            .expect("empty batch with matching revision should succeed");
        assert!(noop_committed.is_empty());
        assert_eq!(store.current_revision("agg-1").unwrap(), 2);

        // Next non-empty batch continues at revision 3
        let batch2 = vec![pending("e3", "third")];
        let committed2 = store
            .append_events("agg-1", 2, &batch2)
            .expect("second batch at rev 2 should succeed");
        assert_eq!(committed2.len(), 1);
        assert_eq!(committed2[0].revision, 3);

        let all_events = store.load_events("agg-1").unwrap();
        assert_eq!(all_events.len(), 3);
        let revisions: Vec<u64> = all_events.iter().map(|e| e.revision).collect();
        assert_eq!(revisions, vec![1, 2, 3]);
    }

    // 3. Optimistic concurrency conflict (Invariant 6) and no partial batch commits (FR-009)
    {
        let store = create_store();
        store
            .append_events("agg-1", 0, &[pending("e1", "first")])
            .unwrap();

        // Stale expected_revision = 0 fails with RevisionConflict
        let stale_err = store
            .append_events("agg-1", 0, &[pending("e2", "stale")])
            .expect_err("stale expected_revision must fail");
        assert_eq!(stale_err.category(), ErrorCategory::RevisionConflict);
        assert_eq!(
            stale_err,
            EngineError::RevisionConflict {
                aggregate_id: "agg-1".to_string(),
                expected_revision: 0,
                actual_revision: 1,
            }
        );

        // Future expected_revision = 5 also fails with RevisionConflict
        let future_err = store
            .append_events("agg-1", 5, &[pending("e2", "future")])
            .expect_err("future expected_revision must fail");
        assert_eq!(future_err.category(), ErrorCategory::RevisionConflict);

        // Batch containing an invalid second event (blank event_id) fails atomically without committing the first event
        let invalid_batch = vec![
            pending("e2", "valid-first"),
            pending("   ", "invalid-second"),
        ];
        let batch_err = store
            .append_events("agg-1", 1, &invalid_batch)
            .expect_err("batch with invalid event must fail atomically");
        assert_eq!(batch_err.category(), ErrorCategory::StorageFailure);
        assert_eq!(store.current_revision("agg-1").unwrap(), 1);
        assert_eq!(store.load_events("agg-1").unwrap().len(), 1);
    }

    // 4. Separate aggregate IDs are completely isolated (FR-010) and blank IDs rejected
    {
        let store = create_store();
        store
            .append_events("agg-a", 0, &[pending("a1", "alpha"), pending("a2", "beta")])
            .unwrap();
        store
            .append_events("agg-b", 0, &[pending("b1", "gamma")])
            .unwrap();

        assert_eq!(store.current_revision("agg-a").unwrap(), 2);
        assert_eq!(store.current_revision("agg-b").unwrap(), 1);
        assert_eq!(store.current_revision("agg-c").unwrap(), 0);

        let events_a = store.load_events("agg-a").unwrap();
        let events_b = store.load_events("agg-b").unwrap();
        assert_eq!(events_a.len(), 2);
        assert_eq!(events_b.len(), 1);
        assert_eq!(events_b[0].revision, 1);
        assert_eq!(events_b[0].event_id, "b1");

        // Blank aggregate IDs are rejected on both load and append
        assert_eq!(
            store.load_events("  ").unwrap_err().category(),
            ErrorCategory::InvalidAggregateId
        );
        assert_eq!(
            store
                .append_events("", 0, &[pending("x1", "x")])
                .unwrap_err()
                .category(),
            ErrorCategory::InvalidAggregateId
        );
    }

    // 5. Returned event lists are defensive copies (mutating caller's Vec does not mutate store)
    {
        let store = create_store();
        store
            .append_events("agg-copy", 0, &[pending("e1", "immutable")])
            .unwrap();
        let mut loaded = store.load_events("agg-copy").unwrap();
        loaded.clear();
        assert_eq!(store.load_events("agg-copy").unwrap().len(), 1);
    }

    // 6. Concurrent appends with identical expected_revision serialize: exactly one succeeds (NFR-003, Invariant 6)
    {
        let store = Arc::new(create_store());
        let workers = 8;
        let barrier = Arc::new(Barrier::new(workers));
        let mut handles = Vec::new();

        for idx in 0..workers {
            let store_ref = Arc::clone(&store);
            let barrier_ref = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                let evt = pending(&format!("e-{}", idx), &format!("worker-{}", idx));
                barrier_ref.wait();
                store_ref.append_events("concurrent-agg", 0, &[evt])
            }));
        }

        let mut successes = 0;
        let mut conflicts = 0;
        for handle in handles {
            match handle.join().expect("worker thread panicked") {
                Ok(committed) => {
                    assert_eq!(committed.len(), 1);
                    assert_eq!(committed[0].revision, 1);
                    successes += 1;
                }
                Err(err) => {
                    assert_eq!(err.category(), ErrorCategory::RevisionConflict);
                    conflicts += 1;
                }
            }
        }

        assert_eq!(successes, 1);
        assert_eq!(conflicts, workers - 1);
        assert_eq!(store.current_revision("concurrent-agg").unwrap(), 1);
    }
}

#[test]
fn in_memory_event_store_satisfies_store_contract_suite() {
    run_event_store_contract_suite(InMemoryEventStore::new);
}
