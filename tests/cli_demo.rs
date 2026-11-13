use portable_state_engine::cli::run_cli;

fn to_args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn default_demo_runs_todo_and_tic_tac_toe_with_history_and_time_travel() {
    let output = run_cli(&[]).expect("default demo should succeed");
    assert!(output.contains("=== Todo List Demo (listId: todo-demo) ==="));
    assert!(output.contains("Current revision: 3"));
    assert!(output.contains("[x] a: Write tests"));
    assert!(output.contains("[ ] b: Build engine"));
    assert!(output.contains("Rejected duplicate ID command"));
    assert!(output.contains("Time-travel state at revision 1:"));
    assert!(output.contains("Revision after time-travel query: 3 (unchanged)"));

    assert!(output.contains("=== Tic-Tac-Toe Demo (gameId: game-demo) ==="));
    assert!(output.contains("Status: X-won"));
    assert!(output.contains("Rejected illegal move"));
    assert!(output.contains("Time-travel board at revision 2:"));
    assert!(output.contains("Revision after time-travel query: 5 (unchanged)"));
}

#[test]
fn scripted_todo_cli_supports_commands_history_and_time_travel() {
    let args = to_args(&[
        "todo",
        "add:a:Write tests",
        "add:b:Build engine",
        "complete:a",
        "state",
        "history",
        "state-at:1",
        "state",
    ]);
    let output = run_cli(&args).expect("scripted todo CLI should succeed");
    assert!(output.contains("State@rev 3:"));
    assert!(output.contains("[x] a: Write tests"));
    assert!(output.contains("[ ] b: Build engine"));
    assert!(output.contains("History (3 events):"));
    assert!(output.contains("rev 1 | TodoAdded"));
    assert!(output.contains("rev 2 | TodoAdded"));
    assert!(output.contains("rev 3 | TodoCompleted"));
    assert!(output.contains("State@rev 1:"));
}

#[test]
fn scripted_tic_tac_toe_cli_supports_moves_history_and_time_travel() {
    let args = to_args(&[
        "tic-tac-toe",
        "move:X:0",
        "move:O:3",
        "move:X:1",
        "move:O:4",
        "move:X:2",
        "state",
        "history",
        "state-at:2",
    ]);
    let output = run_cli(&args).expect("scripted tic-tac-toe CLI should succeed");
    assert!(output.contains("State@rev 5: status=X-won, next=none"));
    assert!(output.contains("History (5 events):"));
    assert!(output.contains("rev 1 | MoveMade"));
    assert!(output.contains("State@rev 2: status=in-progress, next=X"));
}

#[test]
fn cli_reports_clear_errors_on_invalid_commands_or_out_of_range_revisions() {
    let bad_todo = to_args(&["todo", "add:a:   "]);
    let err1 = run_cli(&bad_todo).expect_err("blank title must fail in CLI");
    assert!(err1.contains("BLANK_TODO_TITLE"));

    let bad_rev = to_args(&["todo", "add:a:Valid", "state-at:99"]);
    let err2 = run_cli(&bad_rev).expect_err("out-of-range revision must fail in CLI");
    assert!(err2.contains("outside valid committed range"));

    let bad_move = to_args(&["tic-tac-toe", "move:X:0", "move:O:0"]);
    let err3 = run_cli(&bad_move).expect_err("occupied cell must fail in CLI");
    assert!(err3.contains("CELL_OCCUPIED"));

    let unknown_cmd = to_args(&["unknown-domain"]);
    let err4 = run_cli(&unknown_cmd).expect_err("unknown subcommand must fail");
    assert!(err4.contains("Usage:"));
}
