use portable_state_engine::core::contracts::{
    validate_aggregate_id, AggregateDefinition, DomainEvent, EventEnvelope, ExplicitContext,
    PendingEvent,
};
use portable_state_engine::core::errors::{DomainError, EngineError, ErrorCategory};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CounterState {
    value: i64,
    history_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CounterCommand {
    Increment(i64),
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CounterEvent {
    Incremented { amount: i64, tag: Option<String> },
}

impl DomainEvent for CounterEvent {
    fn event_type(&self) -> &'static str {
        match self {
            CounterEvent::Incremented { .. } => "CounterIncremented",
        }
    }
}

struct CounterDomain;

impl AggregateDefinition for CounterDomain {
    type State = CounterState;
    type Command = CounterCommand;
    type Event = CounterEvent;

    fn initial_state(&self) -> Self::State {
        CounterState {
            value: 0,
            history_tags: Vec::new(),
        }
    }

    fn decide(
        &self,
        _state: &Self::State,
        command: &Self::Command,
        context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError> {
        match command {
            CounterCommand::Increment(amount) => {
                if *amount <= 0 {
                    return Err(DomainError::new(
                        "NON_POSITIVE_INCREMENT",
                        "increment amount must be positive",
                    ));
                }
                let tag = context.metadata.get("tag").cloned();
                Ok(vec![CounterEvent::Incremented {
                    amount: *amount,
                    tag,
                }])
            }
            CounterCommand::NoOp => Ok(Vec::new()),
        }
    }

    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State {
        match event {
            CounterEvent::Incremented { amount, tag } => {
                let mut next_tags = state.history_tags.clone();
                if let Some(t) = tag {
                    next_tags.push(t.clone());
                }
                CounterState {
                    value: state.value + amount,
                    history_tags: next_tags,
                }
            }
        }
    }
}

#[test]
fn dummy_domain_distinguishes_accepted_events_noop_and_domain_rejection() {
    let domain = CounterDomain;
    let initial = domain.initial_state();
    let ctx = ExplicitContext::new().with_metadata("tag", "batch-1");

    // Accepted command emits events without mutating input state
    let accepted = domain
        .decide(&initial, &CounterCommand::Increment(5), &ctx)
        .expect("positive increment should be accepted");
    assert_eq!(
        accepted,
        vec![CounterEvent::Incremented {
            amount: 5,
            tag: Some("batch-1".to_string()),
        }]
    );
    assert_eq!(initial.value, 0);

    // Evolve creates new state without mutating original state
    let evolved = domain.evolve(&initial, &accepted[0]);
    assert_eq!(initial.value, 0);
    assert!(initial.history_tags.is_empty());
    assert_eq!(evolved.value, 5);
    assert_eq!(evolved.history_tags, vec!["batch-1".to_string()]);

    // No-op command returns empty event list
    let noop_events = domain
        .decide(&evolved, &CounterCommand::NoOp, &ExplicitContext::default())
        .expect("no-op should succeed");
    assert!(noop_events.is_empty());

    // Invalid command returns DomainError
    let rejection = domain
        .decide(
            &evolved,
            &CounterCommand::Increment(0),
            &ExplicitContext::default(),
        )
        .expect_err("zero increment must be rejected");
    assert_eq!(rejection.code, "NON_POSITIVE_INCREMENT");
    assert!(rejection.to_string().contains("NON_POSITIVE_INCREMENT"));
}

#[test]
fn error_categories_and_diagnostic_fields_are_distinct() {
    let domain_err = EngineError::domain_rejection(
        Some("todo-1"),
        DomainError::new("DUPLICATE_ID", "Todo 'a' already exists"),
    );
    assert_eq!(domain_err.category(), ErrorCategory::ValidationRejection);
    assert!(domain_err.to_string().contains("todo-1"));
    assert!(domain_err.to_string().contains("DUPLICATE_ID"));

    let invalid_id = EngineError::InvalidAggregateId {
        aggregate_id: "   ".to_string(),
        reason: "aggregate_id must not be blank".to_string(),
    };
    assert_eq!(invalid_id.category(), ErrorCategory::InvalidAggregateId);

    let not_found = EngineError::AggregateNotFound {
        aggregate_id: "missing-42".to_string(),
    };
    assert_eq!(not_found.category(), ErrorCategory::AggregateNotFound);

    let conflict = EngineError::RevisionConflict {
        aggregate_id: "game-1".to_string(),
        expected_revision: 2,
        actual_revision: 3,
    };
    assert_eq!(conflict.category(), ErrorCategory::RevisionConflict);
    assert!(conflict.to_string().contains("expected 2"));
    assert!(conflict.to_string().contains("actual 3"));

    let invalid_rev = EngineError::InvalidRevision {
        aggregate_id: Some("game-1".to_string()),
        requested_revision: -1,
        current_revision: 3,
        reason: "revision must be between 0 and current_revision".to_string(),
    };
    assert_eq!(invalid_rev.category(), ErrorCategory::InvalidRevision);

    let corrupted = EngineError::CorruptedStream {
        aggregate_id: Some("game-1".to_string()),
        expected_revision: 2,
        actual_revision: 4,
        reason: "non-contiguous revision in stream".to_string(),
    };
    assert_eq!(corrupted.category(), ErrorCategory::CorruptedStream);

    let storage = EngineError::StorageFailure {
        aggregate_id: Some("game-1".to_string()),
        operation: "append".to_string(),
        reason: "lock poisoned".to_string(),
    };
    assert_eq!(storage.category(), ErrorCategory::StorageFailure);
}

#[test]
fn event_envelope_and_aggregate_id_validation_enforce_invariants() {
    assert!(validate_aggregate_id("list-1").is_ok());
    assert_eq!(
        validate_aggregate_id("").unwrap_err().category(),
        ErrorCategory::InvalidAggregateId
    );
    assert_eq!(
        validate_aggregate_id("   \t\n ").unwrap_err().category(),
        ErrorCategory::InvalidAggregateId
    );

    let pending = PendingEvent::from_domain_event(
        "evt-1",
        CounterEvent::Incremented {
            amount: 2,
            tag: None,
        },
        "2026-11-13T00:00:00Z",
    );
    assert_eq!(pending.event_type, "CounterIncremented");
    assert_eq!(pending.schema_version, 1);

    let env = EventEnvelope::new(
        "counter-1",
        1,
        "evt-1",
        "CounterIncremented",
        1,
        CounterEvent::Incremented {
            amount: 2,
            tag: None,
        },
        "2026-11-13T00:00:00Z",
    )
    .expect("valid envelope should succeed");
    assert_eq!(env.aggregate_id, "counter-1");
    assert_eq!(env.revision, 1);
    assert_eq!(env.event_id, "evt-1");
    assert_eq!(env.event_type, "CounterIncremented");
    assert_eq!(env.schema_version, 1);
    assert_eq!(env.recorded_at, "2026-11-13T00:00:00Z");

    // Revision 0 is invalid for a committed event envelope (must be 1..=n)
    let rev_zero_err = EventEnvelope::new(
        "counter-1",
        0,
        "evt-1",
        "CounterIncremented",
        1,
        CounterEvent::Incremented {
            amount: 2,
            tag: None,
        },
        "2026-11-13T00:00:00Z",
    )
    .expect_err("revision 0 envelope must fail");
    assert_eq!(rev_zero_err.category(), ErrorCategory::InvalidRevision);

    // Blank aggregate_id in envelope is rejected
    let blank_agg_err = EventEnvelope::new(
        "  ",
        1,
        "evt-1",
        "CounterIncremented",
        1,
        CounterEvent::Incremented {
            amount: 2,
            tag: None,
        },
        "2026-11-13T00:00:00Z",
    )
    .expect_err("blank aggregate_id must fail");
    assert_eq!(blank_agg_err.category(), ErrorCategory::InvalidAggregateId);
}
