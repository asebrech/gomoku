use crate::core::board::{Board, Player};

pub struct WinChecker;

impl WinChecker {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        if row >= board.size || col >= board.size {
            return false;
        }
        let idx = board.index(row, col);
        let is_max = Board::is_bit_set(&board.max_bits, idx);
        let is_min = Board::is_bit_set(&board.min_bits, idx);
        if !is_max && !is_min {
            return false;
        }
        let player_bits = if is_max {
            &board.max_bits
        } else {
            &board.min_bits
        };

        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in directions.iter() {
            let mut count = 1;

            let mut step = 1;
            loop {
                let x = row as isize + dx as isize * step;
                let y = col as isize + dy as isize * step;
                if x < 0 || y < 0 || x >= board.size as isize || y >= board.size as isize {
                    break;
                }
                let check_idx = board.index(x as usize, y as usize);
                if Board::is_bit_set(player_bits, check_idx) {
                    count += 1;
                } else {
                    break;
                }
                step += 1;
                if count >= win_condition {
                    return true;
                }
            }

            let mut step = 1;
            loop {
                let x = row as isize - dx as isize * step;
                let y = col as isize - dy as isize * step;
                if x < 0 || y < 0 || x >= board.size as isize || y >= board.size as isize {
                    break;
                }
                let check_idx = board.index(x as usize, y as usize);
                if Board::is_bit_set(player_bits, check_idx) {
                    count += 1;
                } else {
                    break;
                }
                step += 1;
                if count >= win_condition {
                    return true;
                }
            }

            if count >= win_condition {
                return true;
            }
        }

        false
    }

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
