use crate::core::board::{Board, Player};

const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
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

        let player_bits = if Board::is_bit_set(&board.max_bits, idx) {
            &board.max_bits
        } else {
            &board.min_bits
        };

        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += Self::count_direction(board, player_bits, row, col, dx, dy);
            count += Self::count_direction(board, player_bits, row, col, -dx, -dy);

            if count >= win_condition {
                return true;
            }
        }

        false
    }

    fn count_direction(
        board: &Board,
        player_bits: &[u64],
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
    ) -> usize {
        let mut count = 0;
        let mut step = 1;

        loop {
            let x = row as isize + dx * step;
            let y = col as isize + dy * step;

            if x < 0 || y < 0 || x >= board.size as isize || y >= board.size as isize {
                break;
            }

            let check_idx = board.index(x as usize, y as usize);
            if Board::is_bit_set(player_bits, check_idx) {
                count += 1;
                step += 1;
            } else {
                break;
            }
        }

        count
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
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        let opponent_bits = match player.opponent() {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        let mut stones = 0;
        let mut empty_found = false;
        let mut is_open = false;

        for i in 1..=MAX_SEARCH_DISTANCE {
            let new_row = row as isize + dr * i;
            let new_col = col as isize + dc * i;

            if !Self::is_valid_pos(board, new_row, new_col) {
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

    #[inline]
    fn is_valid_pos(board: &Board, row: isize, col: isize) -> bool {
        row >= 0 && col >= 0 && row < board.size as isize && col < board.size as isize
    }
}
