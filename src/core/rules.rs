use crate::ai::precompute::DirectionTables;
use crate::core::board::{Board, Player};

pub struct WinChecker;

impl WinChecker {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize, dir_tables: &DirectionTables) -> bool {
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

        // Check all 4 directions using precomputed rays
        for direction in 0..4 {
            let mut count = 1; // Count the current position
            
            // Count forward
            let forward_ray = dir_tables.get_ray_forward(idx, direction);
            for &ray_idx in forward_ray {
                if Board::is_bit_set(player_bits, ray_idx) {
                    count += 1;
                    if count >= win_condition {
                        return true;
                    }
                } else {
                    break;
                }
            }
            
            // Count backward
            let backward_ray = dir_tables.get_ray_backward(idx, direction);
            for &ray_idx in backward_ray {
                if Board::is_bit_set(player_bits, ray_idx) {
                    count += 1;
                    if count >= win_condition {
                        return true;
                    }
                } else {
                    break;
                }
            }
            
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
}
