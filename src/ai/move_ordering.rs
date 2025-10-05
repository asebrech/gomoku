use crate::ai::precompute::DirectionTables;
use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct MoveOrdering;

impl MoveOrdering {
    pub fn order_moves(state: &GameState, moves: &mut [(usize, usize)]) {
        let center = state.board.size / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
    }

    fn calculate_move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        let center_distance = Self::manhattan_distance(row, col, center, center);
        priority += 100 - center_distance as i32;

        priority += Self::calculate_threat_priority(&state.board, row, col, &state.direction_tables);
        priority += Self::calculate_adjacency_bonus(&state.board, row, col, &state.direction_tables);

        priority
    }

    fn calculate_threat_priority(board: &Board, row: usize, col: usize, dir_tables: &DirectionTables) -> i32 {
        let mut threat_score = 0;
        let idx = dir_tables.to_index(row, col);

        for &player in &[Player::Max, Player::Min] {
            for direction in 0..4 {
                let consecutive = Self::simulate_move_consecutive(board, idx, direction, player, dir_tables);
                let multiplier = if player == Player::Max { 1 } else { -1 };

                threat_score += multiplier
                    * match consecutive {
                        5 => 10000,
                        4 => 5000,
                        3 => 1000,
                        _ => 0,
                    };
            }
        }
        threat_score
    }

    fn simulate_move_consecutive(
        board: &Board,
        idx: usize,
        direction: usize,
        player: Player,
        dir_tables: &DirectionTables,
    ) -> usize {
        let backwards = Self::count_direction(board, idx, direction, true, player, dir_tables);
        let forwards = Self::count_direction(board, idx, direction, false, player, dir_tables);
        backwards + forwards + 1
    }

    fn count_direction(
        board: &Board,
        idx: usize,
        direction: usize,
        backward: bool,
        player: Player,
        dir_tables: &DirectionTables,
    ) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        
        let ray = if backward {
            dir_tables.get_ray_backward(idx, direction)
        } else {
            dir_tables.get_ray_forward(idx, direction)
        };
        
        let mut count = 0;
        for &ray_idx in ray {
            if Board::is_bit_set(player_bits, ray_idx) {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize, dir_tables: &DirectionTables) -> i32 {
        let idx = dir_tables.to_index(row, col);
        let adjacent = dir_tables.get_adjacent(idx);
        
        let num_adjacent = adjacent.iter()
            .filter(|&&n_idx| Board::is_bit_set(&board.occupied, n_idx))
            .count() as i32;

        num_adjacent * 50
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
