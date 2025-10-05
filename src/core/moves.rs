use crate::ai::precompute::DirectionTables;
use crate::core::board::{Board, Player};

const FREE_THREE_LENGTH: usize = 3;
const MAX_SEARCH_DISTANCE: isize = 4;

pub struct MoveHandler;

impl MoveHandler {
    pub fn get_possible_moves(board: &Board, player: Player, dir_tables: &DirectionTables) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }

        board
            .get_empty_positions()
            .into_iter()
            .filter(|&(i, j)| {
                let idx = dir_tables.to_index(i, j);
                let adjacent = dir_tables.get_adjacent(idx);
                let has_neighbor = adjacent.iter().any(|&n_idx| {
                    Board::is_bit_set(&board.occupied, n_idx)
                });
                has_neighbor && !RuleValidator::creates_double_three(board, i, j, player, dir_tables)
            })
            .collect()
    }
}

pub struct RuleValidator;

impl RuleValidator {
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player, dir_tables: &DirectionTables) -> bool {
        // Check all 4 directions for free threes
        (0..4)
            .filter(|&dir| Self::is_free_three_in_direction(board, row, col, player, dir, dir_tables))
            .count()
            >= 2
    }

    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        direction: usize,
        dir_tables: &DirectionTables,
    ) -> bool {
        let (stones, left_open, right_open) = Self::analyze_line(board, row, col, player, direction, dir_tables);

        stones == FREE_THREE_LENGTH && Self::can_form_open_four(left_open, right_open)
    }

    fn analyze_line(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        direction: usize,
        dir_tables: &DirectionTables,
    ) -> (usize, bool, bool) {
        let idx = dir_tables.to_index(row, col);
        let left_info = Self::scan_direction(board, idx, player, direction, true, dir_tables);
        let right_info = Self::scan_direction(board, idx, player, direction, false, dir_tables);

        let total_stones = 1 + left_info.0 + right_info.0;
        let left_open = left_info.1;
        let right_open = right_info.1;

        (total_stones, left_open, right_open)
    }

    fn scan_direction(
        board: &Board,
        idx: usize,
        player: Player,
        direction: usize,
        backward: bool,
        dir_tables: &DirectionTables,
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

        let ray = if backward {
            dir_tables.get_ray_backward(idx, direction)
        } else {
            dir_tables.get_ray_forward(idx, direction)
        };

        for &ray_idx in ray.iter().take(MAX_SEARCH_DISTANCE as usize) {
            if Board::is_bit_set(player_bits, ray_idx) {
                if empty_found {
                    break;
                }
                stones += 1;
            } else if !Board::is_bit_set(&board.occupied, ray_idx) {
                if !empty_found && stones > 0 {
                    is_open = true;
                }
                empty_found = true;
                if stones > 0 {
                    break;
                }
            } else if Board::is_bit_set(opponent_bits, ray_idx) {
                break;
            }
        }

        (stones, is_open)
    }

    fn can_form_open_four(left_open: bool, right_open: bool) -> bool {
        left_open || right_open
    }
}
