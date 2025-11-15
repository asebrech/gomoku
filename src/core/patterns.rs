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

    /// Analyzes a consecutive pattern from a stone position in a given direction.
    /// Returns (pattern_length, pattern_start_row, pattern_start_col, total_space, freedom)
    /// or None if the pattern is too short or doesn't have enough space.
    pub fn analyze_consecutive_from_position(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
    ) -> Option<(usize, usize, usize, usize, PatternFreedom)> {
        let backward = Self::count_consecutive(board, row, col, -dx, -dy, player);
        let forward = Self::count_consecutive(board, row, col, dx, dy, player);
        let length = backward + forward + 1;
        
        if length < 2 {
            return None;
        }
        
        let pattern_start_row = (row as isize - dx * backward as isize) as usize;
        let pattern_start_col = (col as isize - dy * backward as isize) as usize;
        
        let total_space = Self::count_total_space(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );
        
        if total_space < win_condition {
            return None;
        }
        
        let freedom = Self::analyze_pattern_freedom(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );
        
        Some((length, pattern_start_row, pattern_start_col, total_space, freedom))
    }

    /// Collects all stones (with gaps) in a direction from a starting position.
    /// Returns a vector of stone positions found within the scan distance.
    /// Stops when hitting an opponent stone or board edge.
    pub fn collect_gapped_stones(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        max_distance: isize,
    ) -> Vec<(usize, usize)> {
        let player_bits = board.get_player_bits(player);
        let mut stones = vec![(row, col)];
        
        // Collect forward
        for dist in 1..=max_distance {
            let check_row = row as isize + dx * dist;
            let check_col = col as isize + dy * dist;
            if Self::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    stones.push((check_row as usize, check_col as usize));
                } else if Board::is_bit_set(&board.occupied, idx) {
                    break;
                }
            } else {
                break;
            }
        }
        
        // Collect backward
        let mut backward_stones = Vec::new();
        for dist in 1..=max_distance {
            let check_row = row as isize - dx * dist;
            let check_col = col as isize - dy * dist;
            if Self::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    backward_stones.push((check_row as usize, check_col as usize));
                } else if Board::is_bit_set(&board.occupied, idx) {
                    break;
                }
            } else {
                break;
            }
        }
        
        backward_stones.reverse();
        backward_stones.append(&mut stones);
        backward_stones
    }

    /// Analyzes a gapped pattern and returns (stones_count, gaps_count, span) or None.
    /// A valid gapped pattern has:
    /// - At least 2 stones
    /// - Span of 7 or less
    /// - At least 1 gap
    /// - Total potential (stones + gaps) >= 4
    /// - Gaps <= 3
    pub fn analyze_gapped_pattern(
        stones: &[(usize, usize)],
    ) -> Option<(usize, usize, isize)> {
        if stones.len() < 2 {
            return None;
        }
        
        let first = stones.first().unwrap();
        let last = stones.last().unwrap();
        let span = ((last.0 as isize - first.0 as isize).abs()
            .max((last.1 as isize - first.1 as isize).abs())) + 1;
        
        let gaps = (span as usize) - stones.len();
        
        if gaps > 0 && span <= 7 && stones.len() + gaps >= 4 && gaps <= 3 {
            Some((stones.len(), gaps, span))
        } else {
            None
        }
    }

    /// Extracts gap positions from a gapped pattern.
    /// Returns positions between first and last stone that are empty.
    pub fn extract_gap_positions(
        board: &Board,
        stones: &[(usize, usize)],
        dx: isize,
        dy: isize,
    ) -> Vec<(usize, usize)> {
        if stones.len() < 2 {
            return Vec::new();
        }
        
        let first = stones.first().unwrap();
        let last = stones.last().unwrap();
        let start_row = first.0 as isize;
        let start_col = first.1 as isize;
        let end_row = last.0 as isize;
        let end_col = last.1 as isize;
        
        let steps = ((end_row - start_row) / dx.max(1)).max((end_col - start_col) / dy.max(1));
        let mut gap_positions = Vec::new();
        
        for step in 1..steps {
            let gap_row = start_row + dx * step;
            let gap_col = start_col + dy * step;
            if Self::is_valid_empty(board, gap_row, gap_col) {
                gap_positions.push((gap_row as usize, gap_col as usize));
            }
        }
        
        gap_positions
    }
}



