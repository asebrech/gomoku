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
use crate::core::rules::{WinDetection, DoubleThreeDetection, CaptureBreaking};
use std::hash::Hash;
use wasm_bindgen::prelude::*;

// Conditional logging macro: uses web_sys::console in WASM, silent in native
macro_rules! log {
    ($($arg:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;
            web_sys::console::log_1(&JsValue::from_str(&format!($($arg)*)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Silent in native builds
        }
    };
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum WinReason {
    Alignment,  // Won by placing N stones in a row
    Captures,   // Won by capturing required pairs
}

#[wasm_bindgen]
pub struct Move {
    pub row: usize,
    pub col: usize,
}

#[wasm_bindgen]
pub struct AIMoveResult {
    pub row: usize,
    pub col: usize,
    pub depth_reached: i32,
    pub nodes_searched: f64, // u64 as f64 for JS compatibility
    pub score: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

// WASM wrapper for GameState
#[wasm_bindgen]
pub struct WasmGameState {
    inner: GameState,
}

#[wasm_bindgen]
impl WasmGameState {
    #[wasm_bindgen(constructor)]
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        WasmGameState {
            inner: GameState::new(board_size, win_condition),
        }
    }

    pub fn with_defaults(board_size: usize, win_condition: usize) -> Self {
        WasmGameState {
            inner: GameState::with_defaults(board_size, win_condition),
        }
    }

    pub fn get_board_size(&self) -> usize {
        self.inner.board.size
    }

    pub fn get_current_player(&self) -> Player {
        self.inner.current_player
    }

    pub fn get_winner(&self) -> Option<Player> {
        self.inner.winner
    }

    pub fn get_max_captures(&self) -> usize {
        self.inner.max_captures
    }

    pub fn get_min_captures(&self) -> usize {
        self.inner.min_captures
    }

    pub fn get_stone_at(&self, row: usize, col: usize) -> Option<Player> {
        self.inner.board.get_player(row, col)
    }

    pub fn is_move_legal_coords(&self, row: usize, col: usize) -> bool {
        self.inner.is_move_legal((row, col))
    }

    pub fn make_move_coords(&mut self, row: usize, col: usize) {
        self.inner.make_move((row, col));
    }

    pub fn undo_last_move(&mut self) {
        if let Some(last_move) = self.inner.move_history.last().cloned() {
            self.inner.undo_move(last_move);
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
    }

    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }

    pub fn reset(&mut self) {
        self.inner = GameState::new(self.inner.board.size, self.inner.win_condition);
    }

    pub fn get_ai_move(&mut self, depth: i32, time_limit_ms: f64) -> Option<AIMoveResult> {
        log!("[RUST] ===== get_ai_move ENTRY =====");
        
        use crate::ai::lazy_smp::lazy_smp_search;
        
        let time_limit = time_limit_ms as u64;
        log!("[RUST] get_ai_move: depth={}, time_limit={}ms", depth, time_limit);
        
        // Check if game is already over
        if self.inner.is_terminal() {
            log!("[RUST] Game is terminal, returning None");
            return None;
        }
        
        log!("[RUST] Calling lazy_smp_search with depth={}, time_limit={}ms", depth, time_limit);
        
        // Pass None for num_threads - it will use rayon::current_num_threads() in WASM
        let result = lazy_smp_search(&mut self.inner, time_limit, depth, None);
        
        log!("[RUST] lazy_smp_search completed");
        log!("[RUST] Depth reached: {}, Nodes: {}, Score: {}", 
            result.depth_reached, result.nodes_searched, result.score);
        
        result.best_move.map(|(row, col)| AIMoveResult {
            row,
            col,
            depth_reached: result.depth_reached,
            nodes_searched: result.nodes_searched as f64,
            score: result.score,
        })
    }

    /// Get all positions that would create a double-three for the current player
    /// Returns a JS array of Move objects
    pub fn get_double_three_positions(&self) -> js_sys::Array {
        let positions = js_sys::Array::new();
        let size = self.inner.board.size;
        
        // Check all empty positions on the board
        for row in 0..size {
            for col in 0..size {
                // Only check empty positions
                if self.inner.board.get_player(row, col).is_none() {
                    // Check if placing a stone here would create a double-three
                    if DoubleThreeDetection::creates_double_three(
                        &self.inner.board, 
                        row, 
                        col, 
                        self.inner.current_player
                    ) {
                        let move_obj = js_sys::Object::new();
                        js_sys::Reflect::set(&move_obj, &"row".into(), &JsValue::from(row)).unwrap();
                        js_sys::Reflect::set(&move_obj, &"col".into(), &JsValue::from(col)).unwrap();
                        positions.push(&move_obj);
                    }
                }
            }
        }
        
        positions
    }

    /// Get the complete move history as a JS array
    /// Returns an array of move objects with row and col properties
    pub fn get_move_history(&self) -> js_sys::Array {
        let history = js_sys::Array::new();
        
        for (row, col) in &self.inner.move_history {
            let move_obj = js_sys::Object::new();
            js_sys::Reflect::set(&move_obj, &"row".into(), &JsValue::from(*row)).unwrap();
            js_sys::Reflect::set(&move_obj, &"col".into(), &JsValue::from(*col)).unwrap();
            history.push(&move_obj);
        }
        
        history
    }

    /// Get all legal moves for the current player.
    /// When there's a breakable five, this returns only the forced capture positions.
    /// Otherwise, returns all empty positions that don't create double-three.
    /// Returns a JS array of Move objects.
    pub fn get_legal_moves(&self) -> js_sys::Array {
        let positions = js_sys::Array::new();
        let moves = self.inner.get_legal_moves();
        
        for (row, col) in moves {
            let move_obj = js_sys::Object::new();
            js_sys::Reflect::set(&move_obj, &"row".into(), &JsValue::from(row)).unwrap();
            js_sys::Reflect::set(&move_obj, &"col".into(), &JsValue::from(col)).unwrap();
            positions.push(&move_obj);
        }
        
        positions
    }
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
            // If a player is in check (has a breakable five), the OPPONENT must break it
            // player_in_check = player who HAS the breakable five
            // current_player = player whose turn it is (the opponent)
            if player_in_check != self.current_player {
                if let Some(check_pos) = self.check_position {
                    let breaking_moves = CaptureBreaking::get_breaking_capture_moves(&self.board, check_pos.0, check_pos.1, player_in_check);
                    if !breaking_moves.is_empty() {
                        return breaking_moves;
                    } else {
                        return vec![];
                    }
                }
            }
        }
        
        MoveGenerator::get_candidate_moves(&self.board, self.current_player)
    }

    pub fn get_legal_moves(&self) -> Vec<(usize, usize)> {
        // If there's a breakable five, only the forced capture moves are legal
        if let Some(player_in_check) = self.player_in_check {
            if player_in_check != self.current_player {
                if let Some(check_pos) = self.check_position {
                    let breaking_moves = CaptureBreaking::get_breaking_capture_moves(&self.board, check_pos.0, check_pos.1, player_in_check);
                    if !breaking_moves.is_empty() {
                        return breaking_moves;
                    } else {
                        return vec![];
                    }
                }
            }
        }
        
        // Otherwise, return all empty positions that don't create double-three
        let mut legal_moves = Vec::new();
        for row in 0..self.board.size {
            for col in 0..self.board.size {
                if self.is_move_legal((row, col)) {
                    legal_moves.push((row, col));
                }
            }
        }
        legal_moves
    }

    pub fn is_move_legal(&self, mv: (usize, usize)) -> bool {
        if mv.0 >= self.board.size || mv.1 >= self.board.size {
            return false;
        }
        
        if !self.board.is_empty_position(mv.0, mv.1) {
            return false;
        }
        
        if DoubleThreeDetection::creates_double_three(&self.board, mv.0, mv.1, self.current_player) {
            return false;
        }
        
        if let Some(player_in_check) = self.player_in_check {
            // If a player is in check (has a breakable five), the OPPONENT must break it
            // player_in_check = player who HAS the breakable five
            // current_player = player whose turn it is (the opponent)
            if player_in_check != self.current_player {
                if let Some(check_pos) = self.check_position {
                    let breaking_moves = CaptureBreaking::get_breaking_capture_moves(&self.board, check_pos.0, check_pos.1, player_in_check);
                    if !breaking_moves.is_empty() {
                        return breaking_moves.contains(&mv);
                    } else {
                        return false;
                    }
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
        
        // CRITICAL: After undo, we need to re-check if there's a breakable five on the board
        // that the current player must break
        self.recalculate_check_state();
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

        let (has_win, is_breakable) = WinDetection::check_win_and_breakable(&self.board, mv.0, mv.1, self.win_condition);
        
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

        if let Some(player_in_check) = self.player_in_check {
            if let Some(check_pos) = self.check_position {
                let (still_has_win, still_breakable) = WinDetection::check_win_and_breakable(&self.board, check_pos.0, check_pos.1, self.win_condition);
                
                if !still_has_win {
                    self.player_in_check = None;
                    self.check_position = None;
                } else if !still_breakable {
                    self.winner = Some(player_in_check);
                    self.win_reason = Some(WinReason::Alignment);
                    self.player_in_check = None;
                    self.check_position = None;
                    return true;
                }
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
    
    /// Recalculate the check state after an undo operation
    /// Scans the board for any breakable fives that the current player must break
    fn recalculate_check_state(&mut self) {
        // Clear the current check state
        self.player_in_check = None;
        self.check_position = None;
        
        // Scan the entire board for breakable fives
        // We need to check if the OPPONENT (not current player) has a breakable five
        let opponent = self.current_player.opponent();
        
        for x in 0..self.board.size {
            for y in 0..self.board.size {
                if self.board.get_player(x, y) == Some(opponent) {
                    // Check if this position is part of a breakable five
                    let (has_win, is_breakable) = WinDetection::check_win_and_breakable(
                        &self.board, 
                        x, 
                        y, 
                        self.win_condition
                    );
                    
                    if has_win && is_breakable {
                        // Found a breakable five! The opponent has it, current player must break it
                        self.player_in_check = Some(opponent);
                        self.check_position = Some((x, y));
                        return;
                    }
                }
            }
        }
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        WinDetection::check_capture_win(self.max_captures, self.min_captures, self.capture_to_win)
    }
}
