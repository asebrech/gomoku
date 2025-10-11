use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};

const FREE_THREE_LENGTH: usize = 3;
const MAX_SEARCH_DISTANCE: isize = 4;

pub struct GameRules;

impl GameRules {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        if row >= board.size || col >= board.size {
            return false;
        }

        let idx = board.index(row, col);
        if !Board::is_bit_set(&board.occupied, idx) {
            return false;
        }

        // Determine which player's stone this is
        let player = if Board::is_bit_set(&board.max_bits, idx) {
            Player::Max
        } else {
            Player::Min
        };

        // Use PatternAnalyzer::count_consecutive instead of duplicate count_direction
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

    pub fn check_capture_win(max_captures: usize, min_captures: usize) -> Option<Player> {
        if max_captures >= 5 {
            Some(Player::Max)
        } else if min_captures >= 5 {
            Some(Player::Min)
        } else {
            None
        }
    }

    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        DIRECTIONS
            .iter()
            .filter(|&&dir| Self::is_free_three_in_direction(board, row, col, player, dir))
            .count()
            >= 2
    }

    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        (dr, dc): (isize, isize),
    ) -> bool {
        let (stones, left_open, right_open) = Self::analyze_line(board, row, col, player, dr, dc);
        stones == FREE_THREE_LENGTH && Self::can_form_open_four(left_open, right_open)
    }

    fn analyze_line(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool, bool) {
        let left_info = Self::scan_direction(board, row, col, player, -dr, -dc);
        let right_info = Self::scan_direction(board, row, col, player, dr, dc);

        let total_stones = 1 + left_info.0 + right_info.0;
        let left_open = left_info.1;
        let right_open = right_info.1;

        (total_stones, left_open, right_open)
    }

    fn scan_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool) {
        let player_bits = board.get_player_bits(player);
        let opponent_bits = board.get_player_bits(player.opponent());

        let mut stones = 0;
        let mut empty_found = false;
        let mut is_open = false;

        for i in 1..=MAX_SEARCH_DISTANCE {
            let new_row = row as isize + dr * i;
            let new_col = col as isize + dc * i;

            if !PatternAnalyzer::is_in_bounds(board, new_row, new_col) {
                break;
            }
            let idx = board.index(new_row as usize, new_col as usize);

            if Board::is_bit_set(player_bits, idx) {
                if empty_found {
                    break;
                }
                stones += 1;
            } else if !Board::is_bit_set(&board.occupied, idx) {
                if !empty_found && stones > 0 {
                    is_open = true;
                }
                empty_found = true;
                if stones > 0 {
                    break;
                }
            } else if Board::is_bit_set(opponent_bits, idx) {
                break;
            }
        }

        (stones, is_open)
    }

    #[inline]
    fn can_form_open_four(left_open: bool, right_open: bool) -> bool {
        left_open || right_open
    }
}
