use portable_state_engine::core::contracts::{
    AggregateDefinition, DomainEvent, EventEnvelope, ExplicitContext,
};
use portable_state_engine::core::errors::{DomainError, ErrorCategory};
use portable_state_engine::core::replay::{evolve_all, replay, replay_at_revision};

#[derive(Debug, Clone, PartialEq, Eq)]
struct LedgerState {
    balance: i64,
    entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LedgerCommand {
    Record { label: String, delta: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LedgerEvent {
    Recorded { label: String, delta: i64 },
}

impl DomainEvent for LedgerEvent {
    fn event_type(&self) -> &'static str {
        "LedgerRecorded"
    }
}

struct LedgerDomain;

impl AggregateDefinition for LedgerDomain {
    type State = LedgerState;
    type Command = LedgerCommand;
    type Event = LedgerEvent;

    fn initial_state(&self) -> Self::State {
        LedgerState {
            balance: 0,
            entries: Vec::new(),
        }
    }

    fn decide(
        &self,
        _state: &Self::State,
        command: &Self::Command,
        _context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError> {
        match command {
            LedgerCommand::Record { label, delta } => Ok(vec![LedgerEvent::Recorded {
                label: label.clone(),
                delta: *delta,
            }]),
        }
    }

    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State {
        match event {
            LedgerEvent::Recorded { label, delta } => {
                let mut entries = state.entries.clone();
                entries.push(format!("{}:{}", label, delta));
                LedgerState {
                    balance: state.balance + delta,
                    entries,
                }
            }
        }
    }
}

fn make_envelope(
    agg_id: &str,
    revision: u64,
    label: &str,
    delta: i64,
) -> EventEnvelope<LedgerEvent> {
    EventEnvelope::new(
        agg_id,
        revision,
        format!("evt-{}", revision),
        "LedgerRecorded",
        1,
        LedgerEvent::Recorded {
            label: label.to_string(),
            delta,
        },
        "2026-11-13T00:00:00Z",
    )
    .expect("valid test envelope")
}

#[test]
fn invariant_1_empty_stream_replays_to_initial_state_at_revision_zero() {
    let domain = LedgerDomain;
    let empty: Vec<EventEnvelope<LedgerEvent>> = Vec::new();
    let result = replay(&domain, &empty).expect("empty stream should replay cleanly");
    assert_eq!(result.revision, 0);
    assert_eq!(result.state, domain.initial_state());

    // Also verify decide works with the test command
    let decided = domain
        .decide(
            &result.state,
            &LedgerCommand::Record {
                label: "init".to_string(),
                delta: 1,
            },
            &ExplicitContext::default(),
        )
        .expect("record command should succeed");
    assert_eq!(decided.len(), 1);
}

#[test]
fn invariant_2_incremental_fold_equivalence_and_determinism() {
    let domain = LedgerDomain;
    let history = vec![
        make_envelope("acc-1", 1, "deposit", 100),
        make_envelope("acc-1", 2, "withdrawal", -30),
    ];
    let new_events = vec![
        LedgerEvent::Recorded {
            label: "bonus".to_string(),
            delta: 15,
        },
        LedgerEvent::Recorded {
            label: "fee".to_string(),
            delta: -5,
        },
    ];

    let mut combined = history.clone();
    combined.push(make_envelope("acc-1", 3, "bonus", 15));
    combined.push(make_envelope("acc-1", 4, "fee", -5));

    // Full replay of history + new_events
    let full_replay_1 = replay(&domain, &combined).expect("combined replay should succeed");
    let full_replay_2 = replay(&domain, &combined).expect("repeated replay must be deterministic");
    assert_eq!(full_replay_1, full_replay_2);
    assert_eq!(full_replay_1.revision, 4);
    assert_eq!(full_replay_1.state.balance, 80);

    // Incremental fold: replay(history) then evolve_all(new_events)
    let prefix_replay = replay(&domain, &history).expect("history replay should succeed");
    let folded_state = evolve_all(&domain, &prefix_replay.state, &new_events);
    assert_eq!(full_replay_1.state, folded_state);
}

#[test]
fn replay_at_revision_reconstructs_exact_prefix_without_mutating_inputs() {
    let domain = LedgerDomain;
    let stream = vec![
        make_envelope("acc-1", 1, "e1", 10),
        make_envelope("acc-1", 2, "e2", 20),
        make_envelope("acc-1", 3, "e3", -5),
    ];
    let snapshot_of_stream = stream.clone();

    let rev0 = replay_at_revision(&domain, &stream, 0).expect("rev 0 should succeed");
    assert_eq!(rev0.revision, 0);
    assert_eq!(rev0.state.balance, 0);
    assert!(rev0.state.entries.is_empty());

    let rev1 = replay_at_revision(&domain, &stream, 1).expect("rev 1 should succeed");
    assert_eq!(rev1.revision, 1);
    assert_eq!(rev1.state.balance, 10);
    assert_eq!(rev1.state.entries, vec!["e1:10"]);

    let rev2 = replay_at_revision(&domain, &stream, 2).expect("rev 2 should succeed");
    assert_eq!(rev2.revision, 2);
    assert_eq!(rev2.state.balance, 30);
    assert_eq!(rev2.state.entries, vec!["e1:10", "e2:20"]);

    let rev3 = replay_at_revision(&domain, &stream, 3).expect("rev 3 should succeed");
    assert_eq!(rev3.revision, 3);
    assert_eq!(rev3.state.balance, 25);
    assert_eq!(rev3.state.entries, vec!["e1:10", "e2:20", "e3:-5"]);

    // Verify input stream was never mutated
    assert_eq!(stream, snapshot_of_stream);
}

#[test]
fn replay_rejects_out_of_range_revisions_and_corrupted_non_contiguous_streams() {
    let domain = LedgerDomain;
    let stream = vec![
        make_envelope("acc-1", 1, "e1", 10),
        make_envelope("acc-1", 2, "e2", 20),
    ];

    // Negative revision is rejected
    let neg_err = replay_at_revision(&domain, &stream, -1)
        .expect_err("negative revision must return InvalidRevision");
    assert_eq!(neg_err.category(), ErrorCategory::InvalidRevision);

    // Revision beyond committed stream length is rejected
    let future_err = replay_at_revision(&domain, &stream, 3)
        .expect_err("future revision must return InvalidRevision");
    assert_eq!(future_err.category(), ErrorCategory::InvalidRevision);

    // Non-contiguous revisions (1, 3) are rejected as CorruptedStream
    let gap_stream = vec![
        make_envelope("acc-1", 1, "e1", 10),
        make_envelope("acc-1", 3, "e3", 30),
    ];
    let gap_err = replay(&domain, &gap_stream).expect_err("gap in revisions must fail");
    assert_eq!(gap_err.category(), ErrorCategory::CorruptedStream);

    // Duplicate revisions (1, 1) are rejected as CorruptedStream
    let dup_stream = vec![
        make_envelope("acc-1", 1, "e1", 10),
        make_envelope("acc-1", 1, "e1-dup", 20),
    ];
    let dup_err = replay(&domain, &dup_stream).expect_err("duplicate revision must fail");
    assert_eq!(dup_err.category(), ErrorCategory::CorruptedStream);

    // Mixed aggregate IDs in one stream are rejected as CorruptedStream
    let mixed_agg_stream = vec![
        make_envelope("acc-1", 1, "e1", 10),
        make_envelope("acc-2", 2, "e2", 20),
    ];
    let mixed_err =
        replay(&domain, &mixed_agg_stream).expect_err("mixed aggregate_id stream must fail");
    assert_eq!(mixed_err.category(), ErrorCategory::CorruptedStream);
}
