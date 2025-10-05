/// Shared pattern analysis utilities used across move generation, heuristic evaluation,
/// and move ordering. This eliminates code duplication and ensures consistent behavior.

use crate::core::board::{Board, Player};

/// Standard directions for line analysis (horizontal, vertical, diagonals)
pub const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

/// All 8 directions including reverse (for adjacency checks)
pub const ALL_DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1), (-1, 0), (-1, 1), (0, -1),
    (0, 1), (1, -1), (1, 0), (1, 1),
];

/// Pattern freedom classification (how many ends are open)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternFreedom {
    Free,      // Both ends open
    HalfFree,  // One end open
    Flanked,   // Both ends blocked
}

/// Shared pattern analysis functions
pub struct PatternAnalyzer;

impl PatternAnalyzer {
    /// Count consecutive stones of a player in a given direction from a position.
    /// Does NOT include the starting position itself.
    /// 
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Starting row position
    /// * `col` - Starting column position
    /// * `dx` - Row direction (-1, 0, or 1)
    /// * `dy` - Column direction (-1, 0, or 1)
    /// * `player` - Player whose stones to count
    /// 
    /// # Returns
    /// Number of consecutive stones in the direction (not including start position)
    #[inline]
    pub fn count_consecutive(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        
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

    /// Count consecutive stones in both directions from a position.
    /// Includes the starting position in the count.
    /// 
    /// # Returns
    /// Total consecutive stones including the position itself (backward + forward + 1)
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

    /// Check if a position is valid and empty
    #[inline]
    pub fn is_valid_empty(board: &Board, row: isize, col: isize) -> bool {
        if row < 0 || col < 0 || row >= board.size as isize || col >= board.size as isize {
            return false;
        }
        let idx = board.index(row as usize, col as usize);
        !Board::is_bit_set(&board.occupied, idx)
    }

    /// Check if a position is valid and occupied
    #[inline]
    pub fn is_valid_occupied(board: &Board, row: isize, col: isize) -> bool {
        if row < 0 || col < 0 || row >= board.size as isize || col >= board.size as isize {
            return false;
        }
        let idx = board.index(row as usize, col as usize);
        Board::is_bit_set(&board.occupied, idx)
    }

    /// Find the start of a pattern by moving backward along a direction
    pub fn find_pattern_start(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> (usize, usize) {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        
        let mut current_row = row as isize;
        let mut current_col = col as isize;

        loop {
            let prev_row = current_row - dx;
            let prev_col = current_col - dy;

            if prev_row >= 0
                && prev_row < board.size as isize
                && prev_col >= 0
                && prev_col < board.size as isize
            {
                let idx = board.index(prev_row as usize, prev_col as usize);
                if Board::is_bit_set(player_bits, idx) {
                    current_row = prev_row;
                    current_col = prev_col;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (current_row as usize, current_col as usize)
    }

    /// Analyze pattern freedom (how many ends are open)
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

        let end_row = start_row as isize + (length as isize * dx);
        let end_col = start_col as isize + (length as isize * dy);
        let end_open = Self::is_valid_empty(board, end_row, end_col);

        match (start_open, end_open) {
            (true, true) => PatternFreedom::Free,
            (true, false) | (false, true) => PatternFreedom::HalfFree,
            (false, false) => PatternFreedom::Flanked,
        }
    }

    /// Calculate Manhattan distance between two positions
    #[inline]
    pub fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }

    /// Count adjacent stones around a position (used for move ordering)
    pub fn count_adjacent_stones(board: &Board, row: usize, col: usize) -> i32 {
        let mut num_adjacent = 0;

        for &(dx, dy) in &ALL_DIRECTIONS {
            let nr = row as isize + dx;
            let nc = col as isize + dy;
            if Self::is_valid_occupied(board, nr, nc) {
                num_adjacent += 1;
            }
        }

        num_adjacent
    }
}
