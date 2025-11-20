//! Capture-based rule breaking functionality.
//!
//! In Gomoku, a five-in-a-row can sometimes be "broken" by capturing one of the
//! stones in the line, preventing the win. This module handles:
//! - Detection of breakable five-in-a-row patterns
//! - Finding all possible moves that can break a five through capture
//! - Analysis of capture opportunities for strategic play
//!
//! This is particularly important for advanced Gomoku variants that allow
//! captures and for AI systems that need to evaluate defensive moves.

use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};

/// Capture-based rule breaking functionality.
pub struct CaptureBreaking;

impl CaptureBreaking {
    /// Check if a five-in-a-row can be broken by opponent capture.
    ///
    /// This function analyzes a five-in-a-row line to determine if the opponent
    /// can capture one of the stones in the line, thereby breaking the win condition.
    /// This is useful for determining if a win is truly undefendable.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position of a stone in the five-in-a-row
    /// * `col` - Column position of a stone in the five-in-a-row
    /// * `player` - The player who has the five-in-a-row
    ///
    /// # Returns
    /// `true` if the five can be broken by capture, `false` otherwise
    pub fn can_break_five_by_capture(board: &Board, row: usize, col: usize, player: Player) -> bool {
        let opponent = player.opponent();
        
        for &(dx, dy) in &DIRECTIONS {
            let count = PatternAnalyzer::count_consecutive_bidirectional(board, row, col, dx, dy, player);

            if count >= 5 {
                // Find all stones in this five-in-a-row line
                let mut line_positions = vec![(row, col)];
                
                // Search backward
                for i in 1..=4 {
                    let r = row as isize - dx * i;
                    let c = col as isize - dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Search forward
                for i in 1..=4 {
                    let r = row as isize + dx * i;
                    let c = col as isize + dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Check if any stone in the line can be captured
                for &(stone_row, stone_col) in &line_positions {
                    if Self::can_opponent_capture_stone(board, stone_row, stone_col, opponent) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    /// Get all moves that can break a five-in-a-row through capture.
    ///
    /// This function finds all possible moves the opponent can make to capture
    /// stones in a five-in-a-row line, thereby breaking the win condition.
    /// This is useful for defensive play and threat analysis.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position of a stone in the five-in-a-row
    /// * `col` - Column position of a stone in the five-in-a-row
    /// * `player` - The player who has the five-in-a-row
    ///
    /// # Returns
    /// Vector of (row, col) positions where the opponent can move to break the five
    pub fn get_breaking_capture_moves(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let opponent = player.opponent();
        let mut breaking_moves = Vec::new();
        
        for &(dx, dy) in &DIRECTIONS {
            let count = PatternAnalyzer::count_consecutive_bidirectional(board, row, col, dx, dy, player);

            if count >= 5 {
                // Find all stones in this five-in-a-row line
                let mut line_positions = vec![(row, col)];
                
                // Search backward
                for i in 1..=4 {
                    let r = row as isize - dx * i;
                    let c = col as isize - dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Search forward
                for i in 1..=4 {
                    let r = row as isize + dx * i;
                    let c = col as isize + dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // Get all capture moves for each stone in the line
                for &(stone_row, stone_col) in &line_positions {
                    let capture_moves = Self::get_moves_that_capture_stone(board, stone_row, stone_col, opponent);
                    breaking_moves.extend(capture_moves);
                }
            }
        }
        
        // Remove duplicates and sort
        breaking_moves.sort();
        breaking_moves.dedup();
        breaking_moves
    }

    /// Check if the opponent can capture a specific stone.
    ///
    /// This function analyzes if the opponent has any moves available that would
    /// result in capturing the stone at the given position. A capture occurs when
    /// the opponent can create a pattern like O-X-X-O where O is opponent and X is player.
    /// The stone can be captured if it's part of a pair that can be flanked by opponent stones.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position of the stone to check
    /// * `col` - Column position of the stone to check
    /// * `opponent` - The opponent player
    ///
    /// # Returns
    /// `true` if the opponent can capture this stone, `false` otherwise
    fn can_opponent_capture_stone(board: &Board, row: usize, col: usize, opponent: Player) -> bool {
        let player = opponent.opponent();
        let opponent_bits = board.get_player_bits(opponent);
        let player_bits = board.get_player_bits(player);
        
        for &(dx, dy) in &DIRECTIONS {
            // Check both directions for capture patterns
            for &dir_multiplier in &[1, -1] {
                let actual_dx = (dx * dir_multiplier) as isize;
                let actual_dy = (dy * dir_multiplier) as isize;
                
                // Look for adjacent player stone (forming X-X pattern)
                let adj_row = row as isize + actual_dx;
                let adj_col = col as isize + actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, adj_row, adj_col) {
                    continue;
                }
                
                let adj_idx = board.index(adj_row as usize, adj_col as usize);
                if !Board::is_bit_set(player_bits, adj_idx) {
                    continue;
                }
                
                // Check if opponent can place on one side (O-X-X-?)
                let place1_row = row as isize - actual_dx;
                let place1_col = col as isize - actual_dy;
                
                // Check if opponent can place on the other side (?-X-X-O)
                let place2_row = adj_row + actual_dx;
                let place2_col = adj_col + actual_dy;
                
                // Case 1: Opponent already has a stone on one side, can place on the other
                if PatternAnalyzer::is_in_bounds(board, place1_row, place1_col) {
                    let place1_idx = board.index(place1_row as usize, place1_col as usize);
                    if Board::is_bit_set(opponent_bits, place1_idx) {
                        // Opponent stone exists at place1, check if place2 is empty
                        if PatternAnalyzer::is_valid_empty(board, place2_row, place2_col) {
                            return true;
                        }
                    }
                }
                
                // Case 2: Opponent already has a stone on the other side, can place on first side
                if PatternAnalyzer::is_in_bounds(board, place2_row, place2_col) {
                    let place2_idx = board.index(place2_row as usize, place2_col as usize);
                    if Board::is_bit_set(opponent_bits, place2_idx) {
                        // Opponent stone exists at place2, check if place1 is empty
                        if PatternAnalyzer::is_valid_empty(board, place1_row, place1_col) {
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }

    /// Get all moves that would capture a specific stone.
    ///
    /// This function finds all positions where the opponent can place a stone
    /// to capture the stone at the given position. A capture requires the pattern
    /// O-X-X-O where the opponent can complete the flanking.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position of the stone that could be captured
    /// * `col` - Column position of the stone that could be captured
    /// * `opponent` - The opponent player who would make the capture
    ///
    /// # Returns
    /// Vector of (row, col) positions where the opponent can move to capture the stone
    fn get_moves_that_capture_stone(board: &Board, row: usize, col: usize, opponent: Player) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let player = opponent.opponent();
        let opponent_bits = board.get_player_bits(opponent);
        let player_bits = board.get_player_bits(player);
        
        for &(dx, dy) in &DIRECTIONS {
            for &dir_multiplier in &[1, -1] {
                let actual_dx = (dx * dir_multiplier) as isize;
                let actual_dy = (dy * dir_multiplier) as isize;
                
                // Look for adjacent player stone (forming X-X pattern)
                let adj_row = row as isize + actual_dx;
                let adj_col = col as isize + actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, adj_row, adj_col) {
                    continue;
                }
                
                let adj_idx = board.index(adj_row as usize, adj_col as usize);
                if !Board::is_bit_set(player_bits, adj_idx) {
                    continue;
                }
                
                // Check possible opponent moves to complete O-X-X-O pattern
                let place1_row = row as isize - actual_dx;
                let place1_col = col as isize - actual_dy;
                let place2_row = adj_row + actual_dx;
                let place2_col = adj_col + actual_dy;
                
                // Case 1: Opponent has stone at place1, can move to place2
                if PatternAnalyzer::is_in_bounds(board, place1_row, place1_col) {
                    let place1_idx = board.index(place1_row as usize, place1_col as usize);
                    if Board::is_bit_set(opponent_bits, place1_idx) {
                        if PatternAnalyzer::is_valid_empty(board, place2_row, place2_col) {
                            moves.push((place2_row as usize, place2_col as usize));
                        }
                    }
                }
                
                // Case 2: Opponent has stone at place2, can move to place1
                if PatternAnalyzer::is_in_bounds(board, place2_row, place2_col) {
                    let place2_idx = board.index(place2_row as usize, place2_col as usize);
                    if Board::is_bit_set(opponent_bits, place2_idx) {
                        if PatternAnalyzer::is_valid_empty(board, place1_row, place1_col) {
                            moves.push((place1_row as usize, place1_col as usize));
                        }
                    }
                }
            }
        }
        
        moves
    }
}