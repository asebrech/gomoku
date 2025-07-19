use crate::core::board::{Board, Player};
use crate::core::rules::WinChecker;
use crate::ai::transposition::ZobristTable;
use bevy::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Resource, Component, Clone)]
pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub win_condition: usize,
    pub winner: Option<Player>,
    pub max_captures: usize,
    pub min_captures: usize,
    pub capture_history: Vec<Vec<(usize, usize)>>,
    // Incremental hash caching for transposition table
    pub zobrist_hash: u64,
    pub zobrist_table: ZobristTable,
}

impl GameState {
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        let board = Board::new(board_size);
        let zobrist_table = ZobristTable::new(board_size, board_size);
        let zobrist_hash = zobrist_table.hash_board(&board);
        
        GameState {
            board,
            current_player: Player::Max,
            win_condition,
            winner: None,
            max_captures: 0,
            min_captures: 0,
            capture_history: Vec::new(),
            zobrist_hash,
            zobrist_table,
        }
    }

    pub fn get_possible_moves(&mut self) -> Vec<(usize, usize)> {
		self.board.get_possible_moves_vec(self.current_player)
    }

    pub fn has_possible_moves(&mut self) -> bool {
        !self.board.get_possible_moves(self.current_player).is_empty()
    }

    // Fast iteration over possible moves without allocation - USE THIS IN AI!
    pub fn for_each_possible_move<F>(&mut self, f: F) 
    where 
        F: FnMut((usize, usize))
    {
        self.board.for_each_possible_move(self.current_player, f);
    }

    // Get possible moves count without allocation
    pub fn possible_moves_count(&mut self) -> usize {
        self.board.possible_moves_count(self.current_player)
    }

    pub fn make_move(&mut self, mv: (usize, usize)) {
        // Update hash for placing the stone
        self.zobrist_hash ^= self.zobrist_table.get_piece_key(mv.0, mv.1, self.board.size(), self.current_player);
        
        // Place the stone first
        self.board.place_stone(mv.0, mv.1, self.current_player);
        
        // Check for captures and execute them
        let captures = self.detect_and_execute_captures(mv.0, mv.1, self.current_player);
        
        // Update hash for captured stones
        for &(cap_row, cap_col) in &captures {
            // Remove the captured piece from hash (it was the opponent's piece)
            let opponent = self.current_player.opponent();
            self.zobrist_hash ^= self.zobrist_table.get_piece_key(cap_row, cap_col, self.board.size(), opponent);
        }
        
        // Track capture history for undo
        self.capture_history.push(captures);

        self.check_for_wins(mv);
        self.switch_player();
    }

    pub fn undo_move(&mut self, move_: (usize, usize)) {
        self.current_player = self.current_player.opponent();

        // Update hash to remove the placed stone
        self.zobrist_hash ^= self.zobrist_table.get_piece_key(move_.0, move_.1, self.board.size(), self.current_player);
        
        self.board.remove_stone(move_.0, move_.1);
        self.winner = None;

        // Restore captured stones and update hash
        let captures = self.capture_history.pop().unwrap_or_default();
        for &(cap_row, cap_col) in &captures {
            // Restore the captured stone (it was the opponent's piece)
            let opponent = self.current_player.opponent();
            self.board.place_stone(cap_row, cap_col, opponent);
            
            // Add the restored piece back to hash
            self.zobrist_hash ^= self.zobrist_table.get_piece_key(cap_row, cap_col, self.board.size(), opponent);
        }
        
        // Update capture counts: each pair of restored stones was 1 capture
        if !captures.is_empty() {
            let captures_made = captures.len() / 2; // Each capture removes 2 stones
            match self.current_player {
                Player::Max => {
                    if self.max_captures >= captures_made {
                        self.max_captures -= captures_made;
                    }
                },
                Player::Min => {
                    if self.min_captures >= captures_made {
                        self.min_captures -= captures_made;
                    }
                }
            }
        }
    }

    pub fn is_terminal(&mut self) -> bool {
        self.winner.is_some() || !self.has_possible_moves()
    }

    pub fn check_winner(&self) -> Option<Player> {
        self.winner
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.hash().hash(&mut hasher);
        self.current_player.hash(&mut hasher);
        hasher.finish()
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

    fn restore_captured_stones(&mut self) {
        if let Some(last_captures) = self.capture_history.pop() {
            if !last_captures.is_empty() {
                let opponent = self.current_player.opponent();

                // Restore the captured stones to the board
                for &(row, col) in &last_captures {
                    self.board.place_stone(row, col, opponent);
                }

                // Restore capture counts in GameState
                let pairs_captured = last_captures.len() / 2;
                match self.current_player {
                    Player::Max => {
                        // Current player is Max, so Min's pieces were captured
                        if self.max_captures >= pairs_captured {
                            self.max_captures -= pairs_captured;
                        }
                    }
                    Player::Min => {
                        // Current player is Min, so Max's pieces were captured
                        if self.min_captures >= pairs_captured {
                            self.min_captures -= pairs_captured;
                        }
                    }
                }
            }
        }
    }

    fn check_win_around(&self, mv: (usize, usize)) -> bool {
        WinChecker::check_win_around(&self.board, mv.0, mv.1, self.win_condition)
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        WinChecker::check_capture_win(self.max_captures, self.min_captures)
    }

    // Detect and execute captures for a move, returning captured positions for undo
    fn detect_and_execute_captures(&mut self, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let mut captured_positions = Vec::new();
        let directions = [
            (-1, 0), (1, 0),   // Vertical
            (0, -1), (0, 1),   // Horizontal
            (-1, -1), (1, 1),  // Diagonal \
            (-1, 1), (1, -1),  // Diagonal /
        ];

        for (dr, dc) in directions {
            let r1 = row as i32 + dr;
            let c1 = col as i32 + dc;
            let r2 = row as i32 + 2 * dr;
            let c2 = col as i32 + 2 * dc;
            let r3 = row as i32 + 3 * dr;
            let c3 = col as i32 + 3 * dc;

            // Check bounds
            if r1 >= 0 && r1 < self.board.size() as i32 && c1 >= 0 && c1 < self.board.size() as i32 &&
               r2 >= 0 && r2 < self.board.size() as i32 && c2 >= 0 && c2 < self.board.size() as i32 &&
               r3 >= 0 && r3 < self.board.size() as i32 && c3 >= 0 && c3 < self.board.size() as i32 {
                
                let r1 = r1 as usize;
                let c1 = c1 as usize;
                let r2 = r2 as usize;
                let c2 = c2 as usize;
                let r3 = r3 as usize;
                let c3 = c3 as usize;

                // Check for pattern: player-opponent-opponent-player
                if self.board.get_player(r1, c1) == Some(player.opponent()) &&
                   self.board.get_player(r2, c2) == Some(player.opponent()) &&
                   self.board.get_player(r3, c3) == Some(player) {
                    
                    // Execute capture by removing opponent pieces
                    self.board.remove_stone(r1, c1);
                    self.board.remove_stone(r2, c2);
                    
                    // Track captured positions for undo
                    captured_positions.push((r1, c1));
                    captured_positions.push((r2, c2));
                    
                    // Update capture count
                    match player {
                        Player::Max => self.max_captures += 1,
                        Player::Min => self.min_captures += 1,
                    }
                }
            }
        }

        captured_positions
    }
}
