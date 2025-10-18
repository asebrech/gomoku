//! Shared pattern analysis utilities for AI components.
//!
//! This module contains common functions and constants used by both
//! the heuristic evaluation system and move generation system to ensure
//! consistency in pattern analysis and scoring.

use crate::core::board::Board;
use crate::core::patterns::{PatternFreedom, PatternAnalyzer};
use crate::ai::heuristic::{
    FIVE_IN_ROW_SCORE, LIVE_FOUR_SINGLE_SCORE, HALF_FREE_FOUR_SCORE, DEAD_FOUR_SCORE,
    LIVE_THREE_SCORE, HALF_FREE_THREE_SCORE, DEAD_THREE_SCORE,
    LIVE_TWO_SCORE, HALF_FREE_TWO_SCORE
};


/// Counts the total available space for a pattern in a given direction.
/// 
/// This includes the pattern length itself plus all empty squares that
/// extend the pattern in both directions until blocked by stones or board edges.
/// 
/// # Arguments
/// * `board` - The game board
/// * `start_row` - Starting row of the pattern
/// * `start_col` - Starting column of the pattern  
/// * `dx` - Row direction vector
/// * `dy` - Column direction vector
/// * `pattern_length` - Length of the existing pattern
/// 
/// # Returns
/// Total spaces available for this pattern (including the pattern itself)
pub fn count_total_space(
    board: &Board,
    start_row: usize,
    start_col: usize,
    dx: isize,
    dy: isize,
    pattern_length: usize,
) -> usize {
    let mut space = pattern_length;

    space += count_empty_in_direction(
        board,
        start_row as isize - dx,
        start_col as isize - dy,
        -dx,
        -dy,
    );

    let end_row = start_row as isize + (pattern_length - 1) as isize * dx;
    let end_col = start_col as isize + (pattern_length - 1) as isize * dy;
    space += count_empty_in_direction(board, end_row + dx, end_col + dy, dx, dy);

    space
}

/// Counts empty squares in a specific direction from a starting position.
/// 
/// # Arguments
/// * `board` - The game board
/// * `start_row` - Starting row position
/// * `start_col` - Starting column position
/// * `dx` - Row direction vector
/// * `dy` - Column direction vector
/// 
/// # Returns
/// Number of consecutive empty squares in the given direction
pub fn count_empty_in_direction(
    board: &Board,
    start_row: isize,
    start_col: isize,
    dx: isize,
    dy: isize,
) -> usize {
    let mut count = 0;
    let mut current_row = start_row;
    let mut current_col = start_col;

    while PatternAnalyzer::is_in_bounds(board, current_row, current_col) {
        let idx = board.index(current_row as usize, current_col as usize);
        if !Board::is_bit_set(&board.occupied, idx) {
            count += 1;
            current_row += dx;
            current_col += dy;
        } else {
            break;
        }
    }

    count
}

/// Analyzes the freedom of a pattern to determine its tactical value.
/// 
/// This function examines the spaces before and after a pattern to classify
/// whether it's free (can extend in both directions), half-free (blocked on one side),
/// or flanked (blocked on both sides).
/// 
/// # Arguments
/// * `board` - The game board
/// * `start_row` - Starting row of the pattern
/// * `start_col` - Starting column of the pattern
/// * `dx` - Row direction vector
/// * `dy` - Column direction vector
/// * `length` - Length of the pattern
/// 
/// # Returns
/// PatternFreedom classification of the pattern
pub fn analyze_pattern_freedom(
    board: &Board,
    start_row: usize,
    start_col: usize,
    dx: isize,
    dy: isize,
    length: usize,
) -> PatternFreedom {
    let before_row = start_row as isize - dx;
    let before_col = start_col as isize - dy;
    let start_open = PatternAnalyzer::is_valid_empty(board, before_row, before_col);

    let end_row = start_row as isize + (length - 1) as isize * dx;
    let end_col = start_col as isize + (length - 1) as isize * dy;
    let after_row = end_row + dx;
    let after_col = end_col + dy;
    let end_open = PatternAnalyzer::is_valid_empty(board, after_row, after_col);

    match (start_open, end_open) {
        (true, true) => PatternFreedom::Free,
        (true, false) | (false, true) => PatternFreedom::HalfFree,
        (false, false) => PatternFreedom::Flanked,
    }
}

/// Gets the appropriate score for a pattern based on its length and freedom.
/// 
/// This function encapsulates the scoring logic used throughout the AI system
/// to ensure consistent pattern valuation.
/// 
/// # Arguments
/// * `length` - Number of stones in the pattern
/// * `freedom` - Freedom classification of the pattern
/// 
/// # Returns
/// Score value for the pattern
pub fn get_pattern_score(length: usize, freedom: PatternFreedom) -> i32 {
    match length {
        5 => FIVE_IN_ROW_SCORE,
        4 => match freedom {
            PatternFreedom::Free => LIVE_FOUR_SINGLE_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_FOUR_SCORE,
            PatternFreedom::Flanked => DEAD_FOUR_SCORE,
        },
        3 => match freedom {
            PatternFreedom::Free => LIVE_THREE_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_THREE_SCORE,
            PatternFreedom::Flanked => DEAD_THREE_SCORE,
        },
        2 => match freedom {
            PatternFreedom::Free => LIVE_TWO_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_TWO_SCORE,
            PatternFreedom::Flanked => 0, // Dead twos are ignored
        },
        _ => 0,
    }
}