//! Reference domain B — 3×3 Tic-Tac-Toe (`SPEC.md` Section 6, `FR-026`).
//!
//! **Board addressing (`SPEC.md` Section 6):** Cells are addressed `0..8` in row-major order:
//! - Top row: `0, 1, 2`
//! - Middle row: `3, 4, 5`
//! - Bottom row: `6, 7, 8`
//!
//! **Initial state:** Empty 9-cell board at revision `0`, `Player::X` to move first,
//! status `GameStatus::InProgress`.

use std::fmt;

use crate::core::contracts::{AggregateDefinition, DomainEvent, ExplicitContext};
use crate::core::errors::DomainError;

const WINNING_LINES: [[usize; 3]; 8] = [
    [0, 1, 2], // top row
    [3, 4, 5], // middle row
    [6, 7, 8], // bottom row
    [0, 3, 6], // left column
    [1, 4, 7], // middle column
    [2, 5, 8], // right column
    [0, 4, 8], // main diagonal
    [2, 4, 6], // anti-diagonal
];

/// Tic-Tac-Toe participant mark (`X` or `O`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Game lifecycle status (`in-progress`, `X-won`, `O-won`, `draw`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    XWon,
    OWon,
    Draw,
}

impl GameStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            GameStatus::InProgress => "in-progress",
            GameStatus::XWon => "X-won",
            GameStatus::OWon => "O-won",
            GameStatus::Draw => "draw",
        }
    }

    pub fn is_terminal(self) -> bool {
        !matches!(self, GameStatus::InProgress)
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Immutable state of a 3×3 Tic-Tac-Toe game (`SPEC.md` Section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicTacToeState {
    pub board: [Option<Player>; 9],
    pub next_player: Option<Player>,
    pub status: GameStatus,
}

impl Default for TicTacToeState {
    fn default() -> Self {
        Self {
            board: [None; 9],
            next_player: Some(Player::X),
            status: GameStatus::InProgress,
        }
    }
}

/// Commands accepted by the Tic-Tac-Toe domain (`SPEC.md` Section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicTacToeCommand {
    MakeMove { player: Player, cell: i32 },
}

impl TicTacToeCommand {
    pub fn make_move(player: Player, cell: i32) -> Self {
        Self::MakeMove { player, cell }
    }
}

/// Events emitted by the Tic-Tac-Toe domain (`SPEC.md` Section 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicTacToeEvent {
    MoveMade { player: Player, cell: usize },
}

impl DomainEvent for TicTacToeEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TicTacToeEvent::MoveMade { .. } => "MoveMade",
        }
    }
}

fn compute_status_and_next_player(
    board: &[Option<Player>; 9],
    last_player: Player,
) -> (GameStatus, Option<Player>) {
    for [a, b, c] in WINNING_LINES {
        if let (Some(p1), Some(p2), Some(p3)) = (board[a], board[b], board[c]) {
            if p1 == p2 && p2 == p3 {
                let win_status = match p1 {
                    Player::X => GameStatus::XWon,
                    Player::O => GameStatus::OWon,
                };
                return (win_status, None);
            }
        }
    }

    if board.iter().all(Option::is_some) {
        return (GameStatus::Draw, None);
    }

    (GameStatus::InProgress, Some(last_player.opponent()))
}

/// Pure domain definition for 3×3 Tic-Tac-Toe.
#[derive(Debug, Clone, Copy, Default)]
pub struct TicTacToeDomain;

impl AggregateDefinition for TicTacToeDomain {
    type State = TicTacToeState;
    type Command = TicTacToeCommand;
    type Event = TicTacToeEvent;

    fn initial_state(&self) -> Self::State {
        TicTacToeState::default()
    }

    fn decide(
        &self,
        state: &Self::State,
        command: &Self::Command,
        _context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError> {
        match command {
            TicTacToeCommand::MakeMove { player, cell } => {
                if state.status.is_terminal() {
                    return Err(DomainError::new(
                        "GAME_ALREADY_FINISHED",
                        format!("cannot make a move when game status is '{}'", state.status),
                    ));
                }

                if state.next_player != Some(*player) {
                    return Err(DomainError::new(
                        "OUT_OF_TURN",
                        format!("it is not player '{}'s turn", player),
                    ));
                }

                if *cell < 0 || *cell > 8 {
                    return Err(DomainError::new(
                        "CELL_OUT_OF_RANGE",
                        format!("cell index {} must be in range 0..=8", cell),
                    ));
                }

                let cell_idx = *cell as usize;
                if state.board[cell_idx].is_some() {
                    return Err(DomainError::new(
                        "CELL_OCCUPIED",
                        format!("cell {} is already occupied", cell_idx),
                    ));
                }

                Ok(vec![TicTacToeEvent::MoveMade {
                    player: *player,
                    cell: cell_idx,
                }])
            }
        }
    }

    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State {
        match event {
            TicTacToeEvent::MoveMade { player, cell } => {
                let mut next_board = state.board;
                next_board[*cell] = Some(*player);
                let (status, next_player) = compute_status_and_next_player(&next_board, *player);
                TicTacToeState {
                    board: next_board,
                    next_player,
                    status,
                }
            }
        }
    }
}
