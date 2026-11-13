use portable_state_engine::adapters::memory_store::InMemoryEventStore;
use portable_state_engine::core::contracts::{AggregateDefinition, ExplicitContext};
use portable_state_engine::core::errors::ErrorCategory;
use portable_state_engine::core::runner::EngineRunner;
use portable_state_engine::domains::todo::{TodoCommand, TodoDomain, TodoItem};

#[test]
fn scenario_replay_a_todo_list() {
    // Given an empty list at revision 0
    let runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());
    let initial = runner.get_current_state("list-1").unwrap();
    assert_eq!(initial.revision, 0);
    assert!(initial.state.items().is_empty());

    // When AddTodo("a", "Write tests") is accepted
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Write tests".to_string(),
            },
        )
        .expect("AddTodo a should succeed");

    // And AddTodo("b", "Build engine") is accepted
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "b".to_string(),
                title: "Build engine".to_string(),
            },
        )
        .expect("AddTodo b should succeed");

    // And CompleteTodo("a") is accepted
    let after_complete = runner
        .execute(
            "list-1",
            &TodoCommand::CompleteTodo {
                id: "a".to_string(),
            },
        )
        .expect("CompleteTodo a should succeed");

    // Then the current revision is 3
    assert_eq!(after_complete.revision, 3);

    // And Todo "a" is completed
    let todo_a = after_complete
        .state
        .get("a")
        .expect("Todo a must be present");
    assert!(todo_a.completed);
    assert_eq!(todo_a.title, "Write tests");

    // And Todo "b" is open
    let todo_b = after_complete
        .state
        .get("b")
        .expect("Todo b must be present");
    assert!(!todo_b.completed);
    assert_eq!(todo_b.title, "Build engine");

    // Deterministic append order is preserved
    assert_eq!(
        after_complete.state.items(),
        &[
            TodoItem {
                id: "a".to_string(),
                title: "Write tests".to_string(),
                completed: true,
            },
            TodoItem {
                id: "b".to_string(),
                title: "Build engine".to_string(),
                completed: false,
            },
        ]
    );

    // And the state at revision 1 contains only open Todo "a"
    let rev1 = runner
        .get_state_at_revision("list-1", 1)
        .expect("rev 1 query should succeed");
    assert_eq!(rev1.revision, 1);
    assert_eq!(
        rev1.state.items(),
        &[TodoItem {
            id: "a".to_string(),
            title: "Write tests".to_string(),
            completed: false,
        }]
    );
}

#[test]
fn scenario_reject_duplicate_ids_and_permanent_id_after_deletion() {
    let runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());

    // Given AddTodo("a", "Write tests") has been committed
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Write tests".to_string(),
            },
        )
        .unwrap();

    // When AddTodo("a", "Something else") is requested
    let dup_err = runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Something else".to_string(),
            },
        )
        .expect_err("duplicate Todo ID must be rejected");

    // Then the command is rejected and no new event is appended
    assert_eq!(dup_err.category(), ErrorCategory::ValidationRejection);
    assert_eq!(runner.get_current_state("list-1").unwrap().revision, 1);

    // When Todo "a" is deleted
    let after_delete = runner
        .execute(
            "list-1",
            &TodoCommand::DeleteTodo {
                id: "a".to_string(),
            },
        )
        .expect("DeleteTodo a should succeed");
    assert_eq!(after_delete.revision, 2);
    assert!(after_delete.state.items().is_empty());

    // Once deleted, an ID MAY NOT be reused within the same list
    let reuse_err = runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Reused ID".to_string(),
            },
        )
        .expect_err("deleted Todo ID must not be reused");
    assert_eq!(reuse_err.category(), ErrorCategory::ValidationRejection);
    assert_eq!(runner.get_current_state("list-1").unwrap().revision, 2);

    // Deleted Todo remains reconstructible via earlier revision 1
    let rev1 = runner.get_state_at_revision("list-1", 1).unwrap();
    assert_eq!(rev1.state.items().len(), 1);
    assert_eq!(rev1.state.items()[0].id, "a");
}

#[test]
fn scenario_historical_read_is_side_effect_free() {
    let runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "First".to_string(),
            },
        )
        .unwrap();
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "b".to_string(),
                title: "Second".to_string(),
            },
        )
        .unwrap();
    runner
        .execute(
            "list-1",
            &TodoCommand::CompleteTodo {
                id: "a".to_string(),
            },
        )
        .unwrap();

    let history_before = runner.get_history("list-1").unwrap();
    assert_eq!(history_before.len(), 3);

    // When the state at revision 1 is requested
    let rev1 = runner.get_state_at_revision("list-1", 1).unwrap();
    let expected_rev1 = TodoDomain.evolve(&TodoDomain.initial_state(), &history_before[0].payload);
    assert_eq!(rev1.state, expected_rev1);

    // Then the current revision and event history are unchanged
    assert_eq!(runner.get_current_state("list-1").unwrap().revision, 3);
    assert_eq!(runner.get_history("list-1").unwrap(), history_before);
}

#[test]
fn todo_validation_edge_cases_and_idempotent_complete() {
    let runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());

    // Blank / whitespace-only title or ID is rejected
    for (id, title) in [("a", ""), ("a", "   \t\n "), ("", "Valid"), ("  ", "Valid")] {
        let err = runner
            .execute(
                "list-1",
                &TodoCommand::AddTodo {
                    id: id.to_string(),
                    title: title.to_string(),
                },
            )
            .expect_err("blank id or title must be rejected");
        assert_eq!(err.category(), ErrorCategory::ValidationRejection);
    }
    assert_eq!(runner.get_current_state("list-1").unwrap().revision, 0);

    // Complete or Delete on unknown ID is rejected
    assert_eq!(
        runner
            .execute(
                "list-1",
                &TodoCommand::CompleteTodo {
                    id: "unknown".to_string(),
                }
            )
            .unwrap_err()
            .category(),
        ErrorCategory::ValidationRejection
    );
    assert_eq!(
        runner
            .execute(
                "list-1",
                &TodoCommand::DeleteTodo {
                    id: "unknown".to_string(),
                }
            )
            .unwrap_err()
            .category(),
        ErrorCategory::ValidationRejection
    );

    // Completing an already completed Todo is a successful no-op (no event, revision unchanged)
    runner
        .execute(
            "list-1",
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Task A".to_string(),
            },
        )
        .unwrap();
    let first_complete = runner
        .execute(
            "list-1",
            &TodoCommand::CompleteTodo {
                id: "a".to_string(),
            },
        )
        .unwrap();
    assert_eq!(first_complete.revision, 2);
    assert_eq!(first_complete.committed_events.len(), 1);

    let second_complete = runner
        .execute(
            "list-1",
            &TodoCommand::CompleteTodo {
                id: "a".to_string(),
            },
        )
        .expect("completing already-completed todo must succeed as a no-op");
    assert_eq!(second_complete.revision, 2);
    assert!(second_complete.committed_events.is_empty());

    // Pure decide/evolve never mutate input state
    let domain = TodoDomain;
    let state_snapshot = second_complete.state.clone();
    let _ = domain.decide(
        &second_complete.state,
        &TodoCommand::AddTodo {
            id: "b".to_string(),
            title: "Task B".to_string(),
        },
        &ExplicitContext::default(),
    );
    assert_eq!(second_complete.state, state_snapshot);
}
