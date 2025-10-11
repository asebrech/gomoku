use crate::ai::zobrist::ZobristHash;
use crate::ai::pattern_history::PatternHistoryAnalyzer;
use crate::core::board::{Board, Player};
use crate::core::captures::CaptureHandler;
use crate::core::move_generation::MoveGenerator;
use crate::core::rules::GameRules;
use bevy::prelude::*;
use std::hash::Hash;

#[derive(Resource, Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub win_condition: usize,
    pub winner: Option<Player>,
    pub max_captures: usize,
    pub min_captures: usize,
    pub capture_history: Vec<Vec<(usize, usize)>>,
    pub move_history: Vec<(usize, usize)>,
    pub pattern_analyzer: PatternHistoryAnalyzer,
    pub zobrist_hash: ZobristHash,
    pub current_hash: u64,
}

impl GameState {
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        let zobrist_hash = ZobristHash::new(board_size);
        let board = Board::new(board_size);
        let current_player = Player::Max;
        let mut state = GameState {
            board,
            current_player,
            win_condition,
            winner: None,
            max_captures: 0,
            min_captures: 0,
            capture_history: Vec::new(),
            move_history: Vec::new(),
            pattern_analyzer: PatternHistoryAnalyzer::new(),
            zobrist_hash: zobrist_hash.clone(),
            current_hash: 0,
        };
        state.current_hash = zobrist_hash.compute_hash(&state);
        state
    }

    pub fn get_candidate_moves(&self) -> Vec<(usize, usize)> {
        MoveGenerator::get_candidate_moves(&self.board, self.current_player)
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
            return true;
        }

        if self.check_win_around(mv) {
            self.winner = Some(self.current_player);
            return true;
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

    fn check_win_around(&self, mv: (usize, usize)) -> bool {
        GameRules::check_win_around(&self.board, mv.0, mv.1, self.win_condition)
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        GameRules::check_capture_win(self.max_captures, self.min_captures)
    }
}
