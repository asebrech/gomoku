//! Win detection rules and utilities.
//!
//! This module handles all aspects of win detection in Gomoku:
//! - Standard five-in-a-row wins
//! - Capture-based wins
//! - Breakable five detection
//!
//! Win conditions are checked by analyzing stone patterns in all four directions
//! (horizontal, vertical, and both diagonals) from a given position.

use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};
use crate::core::rules::capture_breaking::CaptureBreaking;

/// Win detection functionality.
pub struct WinDetection;

impl WinDetection {
    /// Check if there's a win around a specific position.
    ///
    /// This function checks if placing or having a stone at the given position
    /// creates a line of `win_condition` consecutive stones in any direction.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position to check
    /// * `col` - Column position to check
    /// * `win_condition` - Number of consecutive stones needed to win (typically 5)
    ///
    /// # Returns
    /// `true` if there's a win at this position, `false` otherwise
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        if row >= board.size || col >= board.size {
            return false;
        }

        let idx = board.index(row, col);
        if !Board::is_bit_set(&board.occupied, idx) {
            return false;
        }

        let player = if Board::is_bit_set(&board.max_bits, idx) {
            Player::Max
        } else {
            Player::Min
        };

        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= win_condition {
                return true;
            }
        }

        false
    }

    /// Check if there's a win around a position and if it can be broken by capture.
    ///
    /// This is similar to `check_win_around` but also determines if the winning
    /// line can be broken through opponent captures, which affects the game state.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position to check
    /// * `col` - Column position to check
    /// * `win_condition` - Number of consecutive stones needed to win
    ///
    /// # Returns
    /// A tuple `(is_win, is_breakable)`:
    /// - `is_win`: `true` if there's a win at this position
    /// - `is_breakable`: `true` if the win can be broken by capture
    pub fn check_win_and_breakable(board: &Board, row: usize, col: usize, win_condition: usize) -> (bool, bool) {
        if row >= board.size || col >= board.size {
            return (false, false);
        }

        let idx = board.index(row, col);
        if !Board::is_bit_set(&board.occupied, idx) {
            return (false, false);
        }

        let player = if Board::is_bit_set(&board.max_bits, idx) {
            Player::Max
        } else {
            Player::Min
        };

        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= win_condition {
                let is_breakable = CaptureBreaking::can_break_five_by_capture(board, row, col, player);
                return (true, is_breakable);
            }
        }

        (false, false)
    }

    /// Check if a player has won by capturing enough stones.
    ///
    /// In Gomoku, a player can win by capturing a certain number of opponent stones
    /// (typically 10 stones or 5 pairs). This function checks the capture counts
    /// against the win threshold.
    ///
    /// # Arguments
    /// * `max_captures` - Number of stones captured by Player::Max
    /// * `min_captures` - Number of stones captured by Player::Min
    /// * `capture_to_win` - Number of captures needed to win
    ///
    /// # Returns
    /// `Some(Player)` if a player has won by capture, `None` otherwise
    pub fn check_capture_win(max_captures: usize, min_captures: usize, capture_to_win: usize) -> Option<Player> {
        if max_captures >= capture_to_win {
            Some(Player::Max)
        } else if min_captures >= capture_to_win {
            Some(Player::Min)
        } else {
            None
        }
    }
}