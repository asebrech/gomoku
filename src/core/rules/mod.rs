//! Game rules and validation utilities.
//!
//! This module contains all game rule implementations separated into logical modules:
//! - `win_detection`: Functions for detecting wins and win conditions
//! - `double_three`: Double-three forbidden pattern detection
//! - `capture_breaking`: Rules for breaking five-in-a-row through captures
//!
//! The main `GameRules` struct provides a unified interface to all rule implementations.

use crate::core::board::{Board, Player};

pub mod win_detection;
pub mod double_three;
pub mod capture_breaking;

pub use win_detection::WinDetection;
pub use double_three::DoubleThreeDetection;
pub use capture_breaking::CaptureBreaking;

/// Main interface for all game rules.
/// 
/// This struct provides a unified API for accessing all game rule implementations.
/// Each category of rules is delegated to its respective specialized module.
pub struct GameRules;

impl GameRules {
    /// Check if there's a win around a specific position.
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        WinDetection::check_win_around(board, row, col, win_condition)
    }

    /// Check if there's a win around a position and if it can be broken by capture.
    pub fn check_win_and_breakable(board: &Board, row: usize, col: usize, win_condition: usize) -> (bool, bool) {
        WinDetection::check_win_and_breakable(board, row, col, win_condition)
    }

    /// Check if a player has won by capturing enough stones.
    pub fn check_capture_win(max_captures: usize, min_captures: usize, capture_to_win: usize) -> Option<Player> {
        WinDetection::check_capture_win(max_captures, min_captures, capture_to_win)
    }

    /// Check if a move creates a forbidden double-three pattern.
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        DoubleThreeDetection::creates_double_three(board, row, col, player)
    }

    /// Check if a five-in-a-row can be broken by capture.
    pub fn can_break_five_by_capture(board: &Board, row: usize, col: usize, player: Player) -> bool {
        CaptureBreaking::can_break_five_by_capture(board, row, col, player)
    }

    /// Get all moves that can break a five-in-a-row through capture.
    pub fn get_breaking_capture_moves(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        CaptureBreaking::get_breaking_capture_moves(board, row, col, player)
    }
}