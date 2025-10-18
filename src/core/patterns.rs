//! Pattern detection helpers used by the heuristic and move generator.
//!
//! This module exposes small utilities to count consecutive stones, check
//! in-bounds coordinates and determine whether an empty square is valid for
//! move consideration. These functions are intentionally generic and used in
//! several higher-level heuristics and rule checks.

use crate::core::board::{Board, Player};

pub const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

pub const ALL_DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1), (-1, 0), (-1, 1), (0, -1),
    (0, 1), (1, -1), (1, 0), (1, 1),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternFreedom {
    Free,
    HalfFree,
    Flanked,
}

pub struct PatternAnalyzer;

impl PatternAnalyzer {
    #[inline]
    pub fn count_consecutive(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let player_bits = board.get_player_bits(player);
        
        let mut count = 0;
        let mut current_row = row as isize + dx;
        let mut current_col = col as isize + dy;

        while current_row >= 0
            && current_row < board.size as isize
            && current_col >= 0
            && current_col < board.size as isize
        {
            let idx = board.index(current_row as usize, current_col as usize);
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
                current_row += dx;
                current_col += dy;
            } else {
                break;
            }
        }
        
        count
    }

    #[inline]
    pub fn is_in_bounds(board: &Board, row: isize, col: isize) -> bool {
        row >= 0 && col >= 0 && row < board.size as isize && col < board.size as isize
    }

    #[inline]
    pub fn is_valid_empty(board: &Board, row: isize, col: isize) -> bool {
        Self::is_in_bounds(board, row, col)
            && !Board::is_bit_set(&board.occupied, board.index(row as usize, col as usize))
    }

    #[inline]
    pub fn is_valid_occupied(board: &Board, row: isize, col: isize) -> bool {
        Self::is_in_bounds(board, row, col)
            && Board::is_bit_set(&board.occupied, board.index(row as usize, col as usize))
    }

    /// Count consecutive stones bidirectionally from a position
    #[inline]
    pub fn count_consecutive_bidirectional(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let backward = Self::count_consecutive(board, row, col, -dx, -dy, player);
        let forward = Self::count_consecutive(board, row, col, dx, dy, player);
        backward + forward + 1
    }

    /// Counts the total available space for a pattern in a given direction.
    /// 
    /// This includes the pattern length itself plus all empty squares that
    /// extend the pattern in both directions until blocked by stones or board edges.
    pub fn count_total_space(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        pattern_length: usize,
    ) -> usize {
        let mut space = pattern_length;

        space += Self::count_empty_in_direction(
            board,
            start_row as isize - dx,
            start_col as isize - dy,
            -dx,
            -dy,
        );

        let end_row = start_row as isize + (pattern_length - 1) as isize * dx;
        let end_col = start_col as isize + (pattern_length - 1) as isize * dy;
        space += Self::count_empty_in_direction(board, end_row + dx, end_col + dy, dx, dy);

        space
    }

    /// Counts empty squares in a specific direction from a starting position.
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

        while Self::is_in_bounds(board, current_row, current_col) {
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
    /// whether it's free, half-free, or flanked.
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
        let start_open = Self::is_valid_empty(board, before_row, before_col);

        let end_row = start_row as isize + (length - 1) as isize * dx;
        let end_col = start_col as isize + (length - 1) as isize * dy;
        let after_row = end_row + dx;
        let after_col = end_col + dy;
        let end_open = Self::is_valid_empty(board, after_row, after_col);

        match (start_open, end_open) {
            (true, true) => PatternFreedom::Free,
            (true, false) | (false, true) => PatternFreedom::HalfFree,
            (false, false) => PatternFreedom::Flanked,
        }
    }


}
