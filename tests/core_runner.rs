use std::sync::atomic::{AtomicUsize, Ordering};

use portable_state_engine::adapters::memory_store::InMemoryEventStore;
use portable_state_engine::core::contracts::{
    AggregateDefinition, DomainEvent, EventEnvelope, ExplicitContext, PendingEvent, Revision,
};
use portable_state_engine::core::errors::{DomainError, EngineError, ErrorCategory};
use portable_state_engine::core::runner::{EngineRunner, ExecutionOptions};
use portable_state_engine::core::store::EventStore;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WalletState {
    balance: i64,
    tx_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WalletCommand {
    Deposit(i64),
    Withdraw(i64),
    TransferWithFee { amount: i64, fee: i64 },
    EnsurePositiveBalance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WalletEvent {
    Deposited(i64),
    Withdrawn(i64),
    FeeCharged(i64),
}

impl DomainEvent for WalletEvent {
    fn event_type(&self) -> &'static str {
        match self {
            WalletEvent::Deposited(_) => "WalletDeposited",
            WalletEvent::Withdrawn(_) => "WalletWithdrawn",
            WalletEvent::FeeCharged(_) => "WalletFeeCharged",
        }
    }
}

struct WalletDomain;

impl AggregateDefinition for WalletDomain {
    type State = WalletState;
    type Command = WalletCommand;
    type Event = WalletEvent;

    fn initial_state(&self) -> Self::State {
        WalletState {
            balance: 0,
            tx_count: 0,
        }
    }

    fn decide(
        &self,
        state: &Self::State,
        command: &Self::Command,
        _context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError> {
        match command {
            WalletCommand::Deposit(amount) => {
                if *amount <= 0 {
                    return Err(DomainError::new(
                        "INVALID_DEPOSIT",
                        "deposit amount must be positive",
                    ));
                }
                Ok(vec![WalletEvent::Deposited(*amount)])
            }
            WalletCommand::Withdraw(amount) => {
                if *amount <= 0 {
                    return Err(DomainError::new(
                        "INVALID_WITHDRAWAL",
                        "withdrawal amount must be positive",
                    ));
                }
                if state.balance < *amount {
                    return Err(DomainError::new(
                        "INSUFFICIENT_FUNDS",
                        "insufficient balance for withdrawal",
                    ));
                }
                Ok(vec![WalletEvent::Withdrawn(*amount)])
            }
            WalletCommand::TransferWithFee { amount, fee } => {
                if *amount <= 0 || *fee < 0 {
                    return Err(DomainError::new("INVALID_TRANSFER", "invalid transfer/fee"));
                }
                if state.balance < (*amount + *fee) {
                    return Err(DomainError::new(
                        "INSUFFICIENT_FUNDS",
                        "insufficient balance for transfer + fee",
                    ));
                }
                Ok(vec![
                    WalletEvent::Withdrawn(*amount),
                    WalletEvent::FeeCharged(*fee),
                ])
            }
            WalletCommand::EnsurePositiveBalance => {
                if state.balance > 0 {
                    Ok(Vec::new())
                } else {
                    Err(DomainError::new("NOT_POSITIVE", "balance is not positive"))
                }
            }
        }
    }

    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State {
        match event {
            WalletEvent::Deposited(amount) => WalletState {
                balance: state.balance + amount,
                tx_count: state.tx_count + 1,
            },
            WalletEvent::Withdrawn(amount) => WalletState {
                balance: state.balance - amount,
                tx_count: state.tx_count + 1,
            },
            WalletEvent::FeeCharged(fee) => WalletState {
                balance: state.balance - fee,
                tx_count: state.tx_count + 1,
            },
        }
    }
}

/// Store wrapper that injects a concurrent conflicting write before the first `conflicts_to_inject`
/// `append_events` attempts so we can deterministically verify `FR-008` reload + re-decide retry behavior.
struct ConflictInjectingStore {
    inner: InMemoryEventStore<WalletEvent>,
    conflicts_to_inject: usize,
    injected_count: AtomicUsize,
}

impl ConflictInjectingStore {
    fn new(conflicts_to_inject: usize) -> Self {
        Self {
            inner: InMemoryEventStore::new(),
            conflicts_to_inject,
            injected_count: AtomicUsize::new(0),
        }
    }
}

impl EventStore<WalletEvent> for ConflictInjectingStore {
    fn load_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<WalletEvent>>, EngineError> {
        self.inner.load_events(aggregate_id)
    }

    fn append_events(
        &self,
        aggregate_id: &str,
        expected_revision: Revision,
        pending_events: &[PendingEvent<WalletEvent>],
    ) -> Result<Vec<EventEnvelope<WalletEvent>>, EngineError> {
        let prev = self.injected_count.load(Ordering::SeqCst);
        if prev < self.conflicts_to_inject {
            self.injected_count.fetch_add(1, Ordering::SeqCst);
            // Simulate a concurrent writer committing a Withdrawal of 40 at `expected_revision`
            let concurrent_event = PendingEvent::from_domain_event(
                format!("concurrent-{}", prev + 1),
                WalletEvent::Withdrawn(40),
                "2026-11-13T00:00:00Z",
            );
            let _ = self
                .inner
                .append_events(aggregate_id, expected_revision, &[concurrent_event]);
        }
        self.inner
            .append_events(aggregate_id, expected_revision, pending_events)
    }
}

#[test]
fn runner_executes_accepted_multi_event_noop_and_rejected_commands() {
    let runner = EngineRunner::new(WalletDomain, InMemoryEventStore::new());

    // 1. Accepted single-event command
    let res1 = runner
        .execute("w-1", &WalletCommand::Deposit(100))
        .expect("deposit should succeed");
    assert_eq!(res1.revision, 1);
    assert_eq!(res1.state.balance, 100);
    assert_eq!(res1.committed_events.len(), 1);
    assert_eq!(res1.attempts, 1);

    // 2. Accepted multi-event command (atomic batch)
    let res2 = runner
        .execute(
            "w-1",
            &WalletCommand::TransferWithFee { amount: 30, fee: 5 },
        )
        .expect("transfer with fee should succeed");
    assert_eq!(res2.revision, 3);
    assert_eq!(res2.state.balance, 65);
    assert_eq!(res2.state.tx_count, 3);
    assert_eq!(res2.committed_events.len(), 2);
    assert_eq!(res2.committed_events[0].revision, 2);
    assert_eq!(res2.committed_events[1].revision, 3);

    // 3. No-op command returns empty events and leaves revision unchanged
    let noop_res = runner
        .execute("w-1", &WalletCommand::EnsurePositiveBalance)
        .expect("no-op command should succeed");
    assert_eq!(noop_res.revision, 3);
    assert_eq!(noop_res.state.balance, 65);
    assert!(noop_res.committed_events.is_empty());

    // 4. Rejected command appends zero events (Invariant 5)
    let err = runner
        .execute("w-1", &WalletCommand::Withdraw(500))
        .expect_err("overdraft must be rejected");
    assert_eq!(err.category(), ErrorCategory::ValidationRejection);
    assert_eq!(runner.get_history("w-1").unwrap().len(), 3);
}

#[test]
fn runner_supports_side_effect_free_historical_queries_and_history_listing() {
    let runner = EngineRunner::new(WalletDomain, InMemoryEventStore::new());

    runner.execute("w-1", &WalletCommand::Deposit(50)).unwrap();
    runner.execute("w-1", &WalletCommand::Deposit(25)).unwrap();
    runner.execute("w-1", &WalletCommand::Withdraw(10)).unwrap();

    let history_before = runner.get_history("w-1").unwrap();
    assert_eq!(history_before.len(), 3);

    // Time travel to revision 0, 1, 2, 3
    assert_eq!(
        runner
            .get_state_at_revision("w-1", 0)
            .unwrap()
            .state
            .balance,
        0
    );
    assert_eq!(
        runner
            .get_state_at_revision("w-1", 1)
            .unwrap()
            .state
            .balance,
        50
    );
    assert_eq!(
        runner
            .get_state_at_revision("w-1", 2)
            .unwrap()
            .state
            .balance,
        75
    );
    assert_eq!(
        runner
            .get_state_at_revision("w-1", 3)
            .unwrap()
            .state
            .balance,
        65
    );
    assert_eq!(runner.get_current_state("w-1").unwrap().state.balance, 65);

    // Out-of-bounds revision fails without modifying state or history
    assert_eq!(
        runner
            .get_state_at_revision("w-1", 4)
            .unwrap_err()
            .category(),
        ErrorCategory::InvalidRevision
    );

    let history_after = runner.get_history("w-1").unwrap();
    assert_eq!(history_before, history_after);
}

#[test]
fn runner_handles_revision_conflicts_and_re_runs_decide_on_retry() {
    // Case A: Default max_retries = 0 returns RevisionConflict immediately
    {
        let runner = EngineRunner::new(WalletDomain, ConflictInjectingStore::new(1));
        // Seed initial deposit of 100 (this triggers the 1 injected conflict, so let's seed via store directly)
        runner
            .store()
            .inner
            .append_events(
                "w-conflict",
                0,
                &[PendingEvent::from_domain_event(
                    "seed-1",
                    WalletEvent::Deposited(100),
                    "2026-11-13T00:00:00Z",
                )],
            )
            .unwrap();

        let err = runner
            .execute("w-conflict", &WalletCommand::Withdraw(30))
            .expect_err("with max_retries = 0, concurrent write must cause RevisionConflict");
        assert_eq!(err.category(), ErrorCategory::RevisionConflict);
    }

    // Case B: With max_retries = 1, runner reloads state and re-runs `decide`.
    // If the concurrent withdrawal reduced balance from 100 to 60, a withdrawal of 80
    // that was valid at revision 1 MUST be rejected by `decide` on retry at revision 2 (FR-008: never blindly append stale events!).
    {
        let runner = EngineRunner::new(WalletDomain, ConflictInjectingStore::new(1));
        runner
            .store()
            .inner
            .append_events(
                "w-redecide",
                0,
                &[PendingEvent::from_domain_event(
                    "seed-1",
                    WalletEvent::Deposited(100),
                    "2026-11-13T00:00:00Z",
                )],
            )
            .unwrap();

        let opts = ExecutionOptions::new().with_max_retries(2);
        let err = runner
            .execute_with_options("w-redecide", &WalletCommand::Withdraw(80), &opts)
            .expect_err("re-deciding against updated balance (60) must reject Withdraw(80)");
        assert_eq!(err.category(), ErrorCategory::ValidationRejection);
        assert_eq!(runner.get_current_state("w-redecide").unwrap().revision, 2);
        assert_eq!(
            runner
                .get_current_state("w-redecide")
                .unwrap()
                .state
                .balance,
            60
        );
    }

    // Case C: With max_retries = 2, a command that remains valid after concurrent write succeeds on retry
    {
        let runner = EngineRunner::new(WalletDomain, ConflictInjectingStore::new(1));
        runner
            .store()
            .inner
            .append_events(
                "w-retry-ok",
                0,
                &[PendingEvent::from_domain_event(
                    "seed-1",
                    WalletEvent::Deposited(100),
                    "2026-11-13T00:00:00Z",
                )],
            )
            .unwrap();

        let opts = ExecutionOptions::new().with_max_retries(2);
        let ok = runner
            .execute_with_options("w-retry-ok", &WalletCommand::Withdraw(50), &opts)
            .expect("Withdraw(50) is still valid against balance 60 on retry");
        assert_eq!(ok.attempts, 2);
        assert_eq!(ok.revision, 3);
        assert_eq!(ok.state.balance, 10);
    }
}
