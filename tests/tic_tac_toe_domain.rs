use portable_state_engine::adapters::memory_store::InMemoryEventStore;
use portable_state_engine::core::errors::ErrorCategory;
use portable_state_engine::core::replay::replay;
use portable_state_engine::core::runner::EngineRunner;
use portable_state_engine::domains::tic_tac_toe::{
    GameStatus, Player, TicTacToeCommand, TicTacToeDomain,
};

#[test]
fn scenario_deterministic_victory() {
    let runner = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());

    // Given a new game with X to move
    let initial = runner.get_current_state("game-1").unwrap();
    assert_eq!(initial.revision, 0);
    assert_eq!(initial.state.next_player, Some(Player::X));
    assert_eq!(initial.state.status, GameStatus::InProgress);

    // When X moves at 0 and O at 3, X moves at 1 and O at 4, and X moves at 2
    for (player, cell) in [
        (Player::X, 0),
        (Player::O, 3),
        (Player::X, 1),
        (Player::O, 4),
        (Player::X, 2),
    ] {
        runner
            .execute("game-1", &TicTacToeCommand::make_move(player, cell))
            .expect("legal move should succeed");
    }

    // Then X is the winner and the game is terminal
    let final_state = runner.get_current_state("game-1").unwrap();
    assert_eq!(final_state.revision, 5);
    assert_eq!(final_state.state.status, GameStatus::XWon);
    assert_eq!(final_state.state.status.as_str(), "X-won");
    assert!(final_state.state.status.is_terminal());
    assert_eq!(final_state.state.next_player, None);

    // And a further move is rejected without an event
    let post_win_err = runner
        .execute("game-1", &TicTacToeCommand::make_move(Player::O, 5))
        .expect_err("move after terminal win must be rejected");
    assert_eq!(post_win_err.category(), ErrorCategory::ValidationRejection);
    assert_eq!(runner.get_current_state("game-1").unwrap().revision, 5);
}

#[test]
fn scenario_reject_illegal_moves() {
    let runner = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());

    // Given X has moved at cell 0
    runner
        .execute("game-1", &TicTacToeCommand::make_move(Player::X, 0))
        .unwrap();
    let after_first = runner.get_current_state("game-1").unwrap();
    assert_eq!(after_first.revision, 1);

    // When O attempts occupied cell 0
    let occupied_err = runner
        .execute("game-1", &TicTacToeCommand::make_move(Player::O, 0))
        .expect_err("occupied cell must be rejected");
    assert_eq!(occupied_err.category(), ErrorCategory::ValidationRejection);

    // When X attempts to move out of turn
    let turn_err = runner
        .execute("game-1", &TicTacToeCommand::make_move(Player::X, 1))
        .expect_err("out-of-turn move must be rejected");
    assert_eq!(turn_err.category(), ErrorCategory::ValidationRejection);

    // When O attempts out-of-range cells (-1 and 9)
    for bad_cell in [-1, 9, 100] {
        let range_err = runner
            .execute("game-1", &TicTacToeCommand::make_move(Player::O, bad_cell))
            .expect_err("out-of-range cell must be rejected");
        assert_eq!(range_err.category(), ErrorCategory::ValidationRejection);
    }

    // Revision and board are unchanged
    let unchanged = runner.get_current_state("game-1").unwrap();
    assert_eq!(unchanged.revision, 1);
    assert_eq!(unchanged.state, after_first.state);
}

#[test]
fn detects_all_eight_win_lines_for_x_and_o_and_draw_game() {
    let win_lines: [[i32; 3]; 8] = [
        [0, 1, 2], // top row
        [3, 4, 5], // middle row
        [6, 7, 8], // bottom row
        [0, 3, 6], // left col
        [1, 4, 7], // middle col
        [2, 5, 8], // right col
        [0, 4, 8], // main diagonal
        [2, 4, 6], // anti-diagonal
    ];

    for (idx, win_line) in win_lines.iter().enumerate() {
        let other_cells: Vec<i32> = (0..9).filter(|c| !win_line.contains(c)).collect();

        // X wins on win_line (5 moves: X0, O0, X1, O1, X2)
        let runner_x = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
        let game_x = format!("x-win-{}", idx);
        runner_x
            .execute(
                &game_x,
                &TicTacToeCommand::make_move(Player::X, win_line[0]),
            )
            .unwrap();
        runner_x
            .execute(
                &game_x,
                &TicTacToeCommand::make_move(Player::O, other_cells[0]),
            )
            .unwrap();
        runner_x
            .execute(
                &game_x,
                &TicTacToeCommand::make_move(Player::X, win_line[1]),
            )
            .unwrap();
        runner_x
            .execute(
                &game_x,
                &TicTacToeCommand::make_move(Player::O, other_cells[1]),
            )
            .unwrap();
        let res_x = runner_x
            .execute(
                &game_x,
                &TicTacToeCommand::make_move(Player::X, win_line[2]),
            )
            .unwrap();
        assert_eq!(res_x.state.status, GameStatus::XWon);
        assert_eq!(res_x.state.next_player, None);

        // O wins on win_line (6 moves: find 3 non-winning X cells from `other_cells`)
        // Pick 3 cells from `other_cells` that do not themselves form a winning line
        let x_moves = other_cells
            .windows(3)
            .find(|triple| !win_lines.iter().any(|wl| wl == *triple))
            .expect("must find 3 non-winning cells for X");

        let runner_o = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
        let game_o = format!("o-win-{}", idx);
        runner_o
            .execute(&game_o, &TicTacToeCommand::make_move(Player::X, x_moves[0]))
            .unwrap();
        runner_o
            .execute(
                &game_o,
                &TicTacToeCommand::make_move(Player::O, win_line[0]),
            )
            .unwrap();
        runner_o
            .execute(&game_o, &TicTacToeCommand::make_move(Player::X, x_moves[1]))
            .unwrap();
        runner_o
            .execute(
                &game_o,
                &TicTacToeCommand::make_move(Player::O, win_line[1]),
            )
            .unwrap();
        runner_o
            .execute(&game_o, &TicTacToeCommand::make_move(Player::X, x_moves[2]))
            .unwrap();
        let res_o = runner_o
            .execute(
                &game_o,
                &TicTacToeCommand::make_move(Player::O, win_line[2]),
            )
            .unwrap();
        assert_eq!(res_o.state.status, GameStatus::OWon);
        assert_eq!(res_o.state.status.as_str(), "O-won");
        assert_eq!(res_o.state.next_player, None);
    }

    // Full 9-move draw board:
    // X O X
    // X X O
    // O X O
    let draw_runner = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
    let draw_moves = [
        (Player::X, 0),
        (Player::O, 1),
        (Player::X, 2),
        (Player::O, 5),
        (Player::X, 3),
        (Player::O, 6),
        (Player::X, 4),
        (Player::O, 8),
        (Player::X, 7),
    ];
    for (player, cell) in draw_moves {
        draw_runner
            .execute("draw-game", &TicTacToeCommand::make_move(player, cell))
            .unwrap();
    }
    let draw_state = draw_runner.get_current_state("draw-game").unwrap();
    assert_eq!(draw_state.revision, 9);
    assert_eq!(draw_state.state.status, GameStatus::Draw);
    assert_eq!(draw_state.state.status.as_str(), "draw");
    assert_eq!(draw_state.state.next_player, None);
}

#[test]
fn scenario_cross_instance_replay_and_time_travel() {
    let instance_a = EngineRunner::new(TicTacToeDomain, InMemoryEventStore::new());
    let moves = [
        (Player::X, 4),
        (Player::O, 0),
        (Player::X, 8),
        (Player::O, 2),
        (Player::X, 1),
    ];
    for (player, cell) in moves {
        instance_a
            .execute("shared-game", &TicTacToeCommand::make_move(player, cell))
            .unwrap();
    }

    let committed_history = instance_a.get_history("shared-game").unwrap();
    assert_eq!(committed_history.len(), 5);

    // Independent engine instance replays the same ordered game events
    let independent_domain = TicTacToeDomain;
    let replayed_b = replay(&independent_domain, &committed_history)
        .expect("independent instance replay must succeed");
    let current_a = instance_a.get_current_state("shared-game").unwrap();
    assert_eq!(replayed_b, current_a);

    // Historical time travel reconstructs earlier turns accurately without mutating current state
    let rev2 = instance_a.get_state_at_revision("shared-game", 2).unwrap();
    assert_eq!(rev2.revision, 2);
    assert_eq!(rev2.state.board[4], Some(Player::X));
    assert_eq!(rev2.state.board[0], Some(Player::O));
    assert_eq!(rev2.state.board[8], None);
    assert_eq!(rev2.state.next_player, Some(Player::X));
    assert_eq!(rev2.state.status, GameStatus::InProgress);

    assert_eq!(
        instance_a.get_current_state("shared-game").unwrap(),
        current_a
    );
}
