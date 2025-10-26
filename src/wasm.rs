//! WebAssembly bindings for the Gomoku game engine.
//!
//! This module exposes the game state and AI functionality to JavaScript
//! through wasm-bindgen. It provides a simple API for:
//! - Creating and managing game state
//! - Making moves
//! - Getting AI-suggested moves
//! - Querying game status

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::state::{GameState, WinReason};
use crate::core::board::Player;
use crate::ai::minimax::mtdf;
use crate::ai::transposition::TranspositionTable;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize)]
pub struct MoveResult {
    pub success: bool,
    pub error: Option<String>,
    pub captured: Vec<(usize, usize)>,
    pub game_over: bool,
    pub winner: Option<String>,
    pub win_reason: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AIMoveResult {
    pub row: usize,
    pub col: usize,
    pub score: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BoardState {
    pub size: usize,
    pub max_positions: Vec<(usize, usize)>,
    pub min_positions: Vec<(usize, usize)>,
    pub max_captures: usize,
    pub min_captures: usize,
    pub current_player: String,
}

/// The main game interface exposed to JavaScript
#[wasm_bindgen]
pub struct GomokuGame {
    state: GameState,
    tt: TranspositionTable,
}

#[wasm_bindgen]
impl GomokuGame {
    /// Create a new game with the specified board size and win condition
    #[wasm_bindgen(constructor)]
    pub fn new(board_size: usize, win_condition: usize) -> GomokuGame {
        console_log!("Creating new Gomoku game: size={}, win={}", board_size, win_condition);
        GomokuGame {
            state: GameState::new(board_size, win_condition),
            tt: TranspositionTable::new(2_000_000),
        }
    }

    /// Make a move at the specified position
    /// Returns a JSON string with the result
    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, row: usize, col: usize) -> String {
        console_log!("Making move at ({}, {})", row, col);
        
        let result = if !self.state.board.is_valid_position(row, col) {
            MoveResult {
                success: false,
                error: Some("Invalid position".to_string()),
                captured: vec![],
                game_over: false,
                winner: None,
                win_reason: None,
            }
        } else if !self.state.board.is_empty_position(row, col) {
            MoveResult {
                success: false,
                error: Some("Position already occupied".to_string()),
                captured: vec![],
                game_over: false,
                winner: None,
                win_reason: None,
            }
        } else if self.state.winner.is_some() {
            MoveResult {
                success: false,
                error: Some("Game is already over".to_string()),
                captured: vec![],
                game_over: true,
                winner: self.state.winner.as_ref().map(|p| format!("{:?}", p)),
                win_reason: self.state.win_reason.as_ref().map(|r| format!("{:?}", r)),
            }
        } else {
            // Make the move
            self.state.make_move((row, col));
            
            // Get captured stones from the capture history (last entry)
            let captures = if let Some(last_captures) = self.state.capture_history.last() {
                last_captures.clone()
            } else {
                vec![]
            };
            
            let game_over = self.state.winner.is_some();
            let winner = self.state.winner.as_ref().map(|p| match p {
                Player::Max => "Max".to_string(),
                Player::Min => "Min".to_string(),
            });
            let win_reason = self.state.win_reason.as_ref().map(|r| match r {
                WinReason::Alignment => "Alignment".to_string(),
                WinReason::Captures => "Captures".to_string(),
            });

            MoveResult {
                success: true,
                error: None,
                captured: captures,
                game_over,
                winner,
                win_reason,
            }
        };

        serde_json::to_string(&result).unwrap()
    }

    /// Get the best move from the AI
    /// depth: search depth (higher = stronger but slower)
    /// Returns a JSON string with the move and score
    #[wasm_bindgen(js_name = getAIMove)]
    pub fn get_ai_move(&mut self, depth: i32) -> String {
        console_log!("Getting AI move at depth {}", depth);
        
        use std::time::{Instant, Duration};
        
        let first_guess = 0;
        let start_time = Instant::now();
        let time_limit = Some(Duration::from_secs(5)); // 5 second limit
        
        let (score, _nodes, best_move) = mtdf(
            &mut self.state,
            first_guess,
            depth,
            &mut self.tt,
            &start_time,
            time_limit,
        );
        
        if let Some((row, col)) = best_move {
            let result = AIMoveResult { row, col, score };
            serde_json::to_string(&result).unwrap()
        } else {
            // No move available (shouldn't happen in a normal game)
            console_log!("Warning: No AI move found!");
            "null".to_string()
        }
    }

    /// Get the current board state as a JSON string
    #[wasm_bindgen(js_name = getBoardState)]
    pub fn get_board_state(&self) -> String {
        let mut max_positions = vec![];
        let mut min_positions = vec![];

        self.state.board.iterate_bits(&self.state.board.max_bits, |r, c| {
            max_positions.push((r, c));
        });

        self.state.board.iterate_bits(&self.state.board.min_bits, |r, c| {
            min_positions.push((r, c));
        });

        let current_player = match self.state.current_player {
            Player::Max => "Max".to_string(),
            Player::Min => "Min".to_string(),
        };

        let board_state = BoardState {
            size: self.state.board.size,
            max_positions,
            min_positions,
            max_captures: self.state.max_captures,
            min_captures: self.state.min_captures,
            current_player,
        };

        serde_json::to_string(&board_state).unwrap()
    }

    /// Undo the last move
    #[wasm_bindgen(js_name = undoMove)]
    pub fn undo_move(&mut self) -> bool {
        if let Some(&last_move) = self.state.move_history.last() {
            self.state.undo_move(last_move);
            true
        } else {
            false
        }
    }

    /// Reset the game to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        let size = self.state.board.size;
        let win_condition = self.state.win_condition;
        self.state = GameState::new(size, win_condition);
        self.tt.clear();
    }

    /// Check if the game is over
    #[wasm_bindgen(js_name = isGameOver)]
    pub fn is_game_over(&self) -> bool {
        self.state.winner.is_some()
    }

    /// Get the winner if the game is over
    #[wasm_bindgen(js_name = getWinner)]
    pub fn get_winner(&self) -> Option<String> {
        self.state.winner.as_ref().map(|p| match p {
            Player::Max => "Max".to_string(),
            Player::Min => "Min".to_string(),
        })
    }
}
