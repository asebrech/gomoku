//! GameState holds the full mutable game representation used by the engine.
//!
//! Responsibilities:
//! - Maintain the `Board` and player to move.
//! - Track captures, history and a running Zobrist hash for fast TT lookups.
//! - Provide helper methods used by search (make_move, undo_move,
//!   get_candidate_moves, is_terminal, ...).
//!
//! Important invariants:
//! - `current_hash` must reflect the board and current player and is updated
//!   on every make/undo move path.
//!
use crate::core::zobrist::ZobristHash;
use crate::ai::pattern_history::PatternHistoryAnalyzer;
use crate::ai::move_generation::MoveGenerator;
use crate::core::board::{Board, Player};
use crate::core::captures::CaptureHandler;
use crate::core::rules::GameRules;
use bevy::prelude::*;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum WinReason {
    Alignment,  // Won by placing N stones in a row
    Captures,   // Won by capturing required pairs
}

#[derive(Resource, Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub win_condition: usize,
    pub capture_to_win: usize,
    pub winner: Option<Player>,
    pub win_reason: Option<WinReason>,
    pub max_captures: usize,
    pub min_captures: usize,
    pub capture_history: Vec<Vec<(usize, usize)>>,
    pub move_history: Vec<(usize, usize)>,
    pub pattern_analyzer: PatternHistoryAnalyzer,
    pub zobrist_hash: ZobristHash,
    pub current_hash: u64,
    pub player_in_check: Option<Player>,
    pub check_position: Option<(usize, usize)>,
}

impl GameState {
    /// Creates a new GameState with all parameters
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        let zobrist_hash = ZobristHash::new(board_size);
        let board = Board::new(board_size);
        let current_player = Player::Max;
        let mut state = GameState {
            board,
            current_player,
            win_condition,
            capture_to_win: 5,
            winner: None,
            win_reason: None,
            max_captures: 0,
            min_captures: 0,
            capture_history: Vec::new(),
            move_history: Vec::new(),
            pattern_analyzer: PatternHistoryAnalyzer::new(),
            zobrist_hash: zobrist_hash.clone(),
            current_hash: 0,
            player_in_check: None,
            check_position: None,
        };
        state.current_hash = zobrist_hash.compute_hash(&state);
        state
    }

    /// Creates a new GameState with default capture_to_win (5 pairs)
    pub fn with_defaults(board_size: usize, win_condition: usize) -> Self {
        Self::new(board_size, win_condition)
    }

    pub fn get_candidate_moves(&self) -> Vec<(usize, usize)> {
        if let Some(player_in_check) = self.player_in_check {
            if player_in_check == self.current_player.opponent() {
                if let Some(check_pos) = self.check_position {
                    let breaking_moves = GameRules::get_breaking_capture_moves(&self.board, check_pos.0, check_pos.1, player_in_check);
                    if !breaking_moves.is_empty() {
                        return breaking_moves;
                    }
                }
            }
        }
        
        MoveGenerator::get_candidate_moves(&self.board, self.current_player)
    }

    pub fn is_move_legal(&self, mv: (usize, usize)) -> bool {
        if mv.0 >= self.board.size || mv.1 >= self.board.size {
            return false;
        }
        
        if !self.board.is_empty_position(mv.0, mv.1) {
            return false;
        }
        
        if GameRules::creates_double_three(&self.board, mv.0, mv.1, self.current_player) {
            return false;
        }
        
        if let Some(player_in_check) = self.player_in_check {
            if player_in_check == self.current_player.opponent() {
                if let Some(check_pos) = self.check_position {
                    let breaking_moves = GameRules::get_breaking_capture_moves(&self.board, check_pos.0, check_pos.1, player_in_check);
                    return breaking_moves.contains(&mv);
                }
            }
        }
        
        true
    }

    pub fn make_move(&mut self, mv: (usize, usize)) {
        self.current_hash = self.zobrist_hash.update_hash_make_move(
            self.current_hash,
            mv.0,
            mv.1,
            self.current_player,
        );

        self.board.place_stone(mv.0, mv.1, self.current_player);

        let captures =
            CaptureHandler::detect_captures(&self.board, mv.0, mv.1, self.current_player);
        
        if !captures.is_empty() {
            let captured_player = self.current_player.opponent();
            self.current_hash = self.zobrist_hash.update_hash_capture(
                self.current_hash,
                &captures,
                captured_player,
            );
        }
        
        self.execute_captures(captures);
        self.move_history.push(mv);
        self.check_for_wins(mv);
        self.switch_player();
        self.update_pattern_analysis();
    }

    fn update_pattern_analysis(&mut self) {
        let current_player = self.current_player;
        let capture_history_len = self.capture_history.len();
        let last_captures = if capture_history_len > 0 {
            self.capture_history[capture_history_len - 1].clone()
        } else {
            Vec::new()
        };
        
        let move_player = current_player.opponent();
        let captures_made = last_captures.len() / 2;
        
        self.pattern_analyzer.analyze_move(move_player, captures_made);
    }

    pub fn undo_move(&mut self, move_: (usize, usize)) {
        let move_player = self.current_player.opponent();
        
        if let Some(last_captures) = self.capture_history.last() {
            if !last_captures.is_empty() {
                let captured_player = move_player.opponent();
                self.current_hash = self.zobrist_hash.update_hash_capture(
                    self.current_hash,
                    last_captures,
                    captured_player,
                );
            }
        }

        self.board.remove_stone(move_.0, move_.1);
        self.current_hash = self.zobrist_hash.update_hash_undo_move(
            self.current_hash,
            move_.0,
            move_.1,
            move_player,
        );

        self.current_player = move_player;
        self.winner = None;

        if let Some(last_move) = self.move_history.last() {
            if *last_move == move_ {
                self.move_history.pop();
            }
        }

        self.restore_captured_stones();
    }

    pub fn is_terminal(&self) -> bool {
        self.winner.is_some() || self.get_candidate_moves().is_empty()
    }

    pub fn check_winner(&self) -> Option<Player> {
        self.winner
    }

    pub fn hash(&self) -> u64 {
        self.current_hash
    }

    fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    fn check_for_wins(&mut self, mv: (usize, usize)) -> bool {
        if let Some(winner) = self.check_capture_win() {
            self.winner = Some(winner);
            self.win_reason = Some(WinReason::Captures);
            self.player_in_check = None;
            self.check_position = None;
            return true;
        }

        let (has_win, is_breakable) = GameRules::check_win_and_breakable(&self.board, mv.0, mv.1, self.win_condition);
        
        if has_win {
            if is_breakable {
                self.player_in_check = Some(self.current_player);
                self.check_position = Some(mv);
                self.winner = None;
                self.win_reason = None;
                return false;
            } else {
                self.winner = Some(self.current_player);
                self.win_reason = Some(WinReason::Alignment);
                self.player_in_check = None;
                self.check_position = None;
                return true;
            }
        }

        false
    }

    fn execute_captures(&mut self, captures: Vec<(usize, usize)>) {
        if captures.is_empty() {
            self.capture_history.push(Vec::new());
            return;
        }

        for &(row, col) in &captures {
            if row < self.board.size && col < self.board.size {
                let idx = self.board.index(row, col);
                Board::clear_bit(&mut self.board.max_bits, idx);
                Board::clear_bit(&mut self.board.min_bits, idx);
                Board::clear_bit(&mut self.board.occupied, idx);
            }
        }

        let pairs_captured = captures.len() / 2;
        match self.current_player {
            Player::Max => self.max_captures += pairs_captured,
            Player::Min => self.min_captures += pairs_captured,
        }

        self.capture_history.push(captures);
    }

    fn restore_captured_stones(&mut self) {
        if let Some(last_captures) = self.capture_history.pop() {
            if !last_captures.is_empty() {
                let opponent = self.current_player.opponent();

                for &(row, col) in &last_captures {
                    if row < self.board.size && col < self.board.size {

                        self.board.place_stone(row, col, opponent);
                    }
                }

                let pairs_captured = last_captures.len() / 2;
                match self.current_player {
                    Player::Max => {
                        if self.max_captures >= pairs_captured {
                            self.max_captures -= pairs_captured;
                        }
                    }
                    Player::Min => {
                        if self.min_captures >= pairs_captured {
                            self.min_captures -= pairs_captured;
                        }
                    }
                }
            }
        }
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        GameRules::check_capture_win(self.max_captures, self.min_captures, self.capture_to_win)
    }
}
