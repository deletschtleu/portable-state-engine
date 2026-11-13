//! CLI demonstrator for Todo and Tic-Tac-Toe with event history and read-only time travel (`FR-024`, `FR-025`, `FR-026`).

use std::fmt::Write;

use crate::adapters::memory_store::InMemoryEventStore;
use crate::core::runner::EngineRunner;
use crate::domains::tic_tac_toe::{Player, TicTacToeCommand, TicTacToeDomain, TicTacToeState};
use crate::domains::todo::{TodoCommand, TodoDomain, TodoListState};

fn usage() -> String {
    [
        "Usage:",
        "  cargo run -- [demo]",
        "  cargo run -- todo <step> [<step> ...]",
        "    steps: add:<id>:<title> | complete:<id> | delete:<id> | state | history | state-at:<rev>",
        "  cargo run -- tic-tac-toe <step> [<step> ...]",
        "    steps: move:<X|O>:<0..8> | state | history | state-at:<rev>",
    ]
    .join("\n")
}

fn format_todo_state(out: &mut String, state: &TodoListState) {
    if state.items().is_empty() {
        let _ = writeln!(out, "  (empty todo list)");
        return;
    }
    for item in state.items() {
        let check = if item.completed { "x" } else { " " };
        let _ = writeln!(out, "  [{}] {}: {}", check, item.id, item.title);
    }
}

fn format_board(out: &mut String, state: &TicTacToeState) {
    for row in 0..3 {
        let mut cells = Vec::with_capacity(3);
        for col in 0..3 {
            let idx = row * 3 + col;
            let symbol = match state.board[idx] {
                Some(Player::X) => "X",
                Some(Player::O) => "O",
                None => ".",
            };
            cells.push(symbol);
        }
        let _ = writeln!(out, "  {}", cells.join(" "));
    }
}

fn run_default_demo() -> Result<String, String> {
    let mut out = String::new();

    // Part 1: Todo domain demo
    let todo_runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());
    let list_id = "todo-demo";
    let _ = writeln!(out, "=== Todo List Demo (listId: {}) ===", list_id);

    todo_runner
        .execute(
            list_id,
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Write tests".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;
    todo_runner
        .execute(
            list_id,
            &TodoCommand::AddTodo {
                id: "b".to_string(),
                title: "Build engine".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;
    todo_runner
        .execute(
            list_id,
            &TodoCommand::CompleteTodo {
                id: "a".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;

    // Demonstrate rejected duplicate ID command
    let dup_err = todo_runner
        .execute(
            list_id,
            &TodoCommand::AddTodo {
                id: "a".to_string(),
                title: "Duplicate".to_string(),
            },
        )
        .expect_err("duplicate ID must be rejected in demo");
    let _ = writeln!(out, "Rejected duplicate ID command: {}", dup_err);

    let current_todo = todo_runner
        .get_current_state(list_id)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(out, "Current revision: {}", current_todo.revision);
    format_todo_state(&mut out, &current_todo.state);

    let todo_history = todo_runner
        .get_history(list_id)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(
        out,
        "Committed event history ({} events):",
        todo_history.len()
    );
    for env in &todo_history {
        let _ = writeln!(
            out,
            "  rev {} | {} | id={} | {:?}",
            env.revision, env.event_type, env.event_id, env.payload
        );
    }

    let rev1_todo = todo_runner
        .get_state_at_revision(list_id, 1)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(out, "Time-travel state at revision {}:", rev1_todo.revision);
    format_todo_state(&mut out, &rev1_todo.state);

    let after_time_travel_todo = todo_runner
        .get_current_state(list_id)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(
        out,
        "Revision after time-travel query: {} (unchanged)\n",
        after_time_travel_todo.revision
    );

    // Part 2: Tic-Tac-Toe domain demo
    let game_runner = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
    let game_id = "game-demo";
    let _ = writeln!(out, "=== Tic-Tac-Toe Demo (gameId: {}) ===", game_id);

    for (player, cell) in [
        (Player::X, 0),
        (Player::O, 3),
        (Player::X, 1),
        (Player::O, 4),
    ] {
        game_runner
            .execute(game_id, &TicTacToeCommand::make_move(player, cell))
            .map_err(|e| e.to_string())?;
    }

    // Demonstrate illegal move rejection (occupied cell 0)
    let illegal_err = game_runner
        .execute(game_id, &TicTacToeCommand::make_move(Player::X, 0))
        .expect_err("occupied cell move must be rejected in demo");
    let _ = writeln!(out, "Rejected illegal move: {}", illegal_err);

    // Winning move for X at cell 2
    let final_game = game_runner
        .execute(game_id, &TicTacToeCommand::make_move(Player::X, 2))
        .map_err(|e| e.to_string())?;
    let _ = writeln!(out, "Current revision: {}", final_game.revision);
    let _ = writeln!(out, "Status: {}", final_game.state.status);
    format_board(&mut out, &final_game.state);

    let game_history = game_runner
        .get_history(game_id)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(
        out,
        "Committed event history ({} events):",
        game_history.len()
    );
    for env in &game_history {
        let _ = writeln!(
            out,
            "  rev {} | {} | id={} | {:?}",
            env.revision, env.event_type, env.event_id, env.payload
        );
    }

    let rev2_game = game_runner
        .get_state_at_revision(game_id, 2)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(out, "Time-travel board at revision {}:", rev2_game.revision);
    let _ = writeln!(out, "  status={}", rev2_game.state.status);
    format_board(&mut out, &rev2_game.state);

    let after_time_travel_game = game_runner
        .get_current_state(game_id)
        .map_err(|e| e.to_string())?;
    let _ = writeln!(
        out,
        "Revision after time-travel query: {} (unchanged)",
        after_time_travel_game.revision
    );

    Ok(out)
}

fn run_scripted_todo(steps: &[String]) -> Result<String, String> {
    if steps.is_empty() {
        return Err(usage());
    }
    let runner = EngineRunner::new(TodoDomain, InMemoryEventStore::new());
    let list_id = "cli-todo";
    let mut out = String::new();

    for step in steps {
        if step == "state" {
            let current = runner
                .get_current_state(list_id)
                .map_err(|e| e.to_string())?;
            let _ = writeln!(out, "State@rev {}:", current.revision);
            format_todo_state(&mut out, &current.state);
        } else if step == "history" {
            let history = runner.get_history(list_id).map_err(|e| e.to_string())?;
            let _ = writeln!(out, "History ({} events):", history.len());
            for env in &history {
                let _ = writeln!(
                    out,
                    "  rev {} | {} | {:?}",
                    env.revision, env.event_type, env.payload
                );
            }
        } else if let Some(rev_str) = step.strip_prefix("state-at:") {
            let rev: i64 = rev_str
                .parse()
                .map_err(|_| format!("invalid revision '{}'", rev_str))?;
            let historical = runner
                .get_state_at_revision(list_id, rev)
                .map_err(|e| e.to_string())?;
            let _ = writeln!(out, "State@rev {}:", historical.revision);
            format_todo_state(&mut out, &historical.state);
        } else if let Some(rest) = step.strip_prefix("add:") {
            let Some((id, title)) = rest.split_once(':') else {
                return Err(format!(
                    "invalid add step '{}'; expected add:<id>:<title>",
                    step
                ));
            };
            runner
                .execute(
                    list_id,
                    &TodoCommand::AddTodo {
                        id: id.to_string(),
                        title: title.to_string(),
                    },
                )
                .map_err(|e| e.to_string())?;
        } else if let Some(id) = step.strip_prefix("complete:") {
            runner
                .execute(list_id, &TodoCommand::CompleteTodo { id: id.to_string() })
                .map_err(|e| e.to_string())?;
        } else if let Some(id) = step.strip_prefix("delete:") {
            runner
                .execute(list_id, &TodoCommand::DeleteTodo { id: id.to_string() })
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("unknown todo step '{}'\n{}", step, usage()));
        }
    }

    Ok(out)
}

fn parse_player(s: &str) -> Result<Player, String> {
    match s {
        "X" | "x" => Ok(Player::X),
        "O" | "o" => Ok(Player::O),
        _ => Err(format!("invalid player '{}'; expected X or O", s)),
    }
}

fn run_scripted_tic_tac_toe(steps: &[String]) -> Result<String, String> {
    if steps.is_empty() {
        return Err(usage());
    }
    let runner = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
    let game_id = "cli-game";
    let mut out = String::new();

    for step in steps {
        if step == "state" {
            let current = runner
                .get_current_state(game_id)
                .map_err(|e| e.to_string())?;
            let next_str = current
                .state
                .next_player
                .map(|p| p.as_str())
                .unwrap_or("none");
            let _ = writeln!(
                out,
                "State@rev {}: status={}, next={}",
                current.revision, current.state.status, next_str
            );
            format_board(&mut out, &current.state);
        } else if step == "history" {
            let history = runner.get_history(game_id).map_err(|e| e.to_string())?;
            let _ = writeln!(out, "History ({} events):", history.len());
            for env in &history {
                let _ = writeln!(
                    out,
                    "  rev {} | {} | {:?}",
                    env.revision, env.event_type, env.payload
                );
            }
        } else if let Some(rev_str) = step.strip_prefix("state-at:") {
            let rev: i64 = rev_str
                .parse()
                .map_err(|_| format!("invalid revision '{}'", rev_str))?;
            let historical = runner
                .get_state_at_revision(game_id, rev)
                .map_err(|e| e.to_string())?;
            let next_str = historical
                .state
                .next_player
                .map(|p| p.as_str())
                .unwrap_or("none");
            let _ = writeln!(
                out,
                "State@rev {}: status={}, next={}",
                historical.revision, historical.state.status, next_str
            );
            format_board(&mut out, &historical.state);
        } else if let Some(rest) = step.strip_prefix("move:") {
            let Some((player_str, cell_str)) = rest.split_once(':') else {
                return Err(format!(
                    "invalid move step '{}'; expected move:<X|O>:<cell>",
                    step
                ));
            };
            let player = parse_player(player_str)?;
            let cell: i32 = cell_str
                .parse()
                .map_err(|_| format!("invalid cell '{}'", cell_str))?;
            runner
                .execute(game_id, &TicTacToeCommand::make_move(player, cell))
                .map_err(|e| e.to_string())?;
        } else {
            return Err(format!("unknown tic-tac-toe step '{}'\n{}", step, usage()));
        }
    }

    Ok(out)
}

/// Executes the CLI with the given command-line arguments (excluding `argv[0]`).
pub fn run_cli(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        None | Some("demo") => run_default_demo(),
        Some("todo") => run_scripted_todo(&args[1..]),
        Some("tic-tac-toe") => run_scripted_tic_tac_toe(&args[1..]),
        Some("-h") | Some("--help") | Some("help") => Ok(format!("{}\n", usage())),
        Some(other) => Err(format!("unknown command '{}'\n{}", other, usage())),
    }
}
