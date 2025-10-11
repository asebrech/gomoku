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

    pub fn find_pattern_start(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> (usize, usize) {
        let player_bits = board.get_player_bits(player);
        
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

    #[inline]
    pub fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }

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
