use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct MoveOrdering;

const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
const ALL_DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

impl MoveOrdering {
    pub fn order_moves(state: &GameState, moves: &mut [(usize, usize)]) {
        let center = state.board.size / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
    }

    /// Order and limit moves based on search depth for aggressive pruning
    pub fn order_and_limit_moves(state: &GameState, moves: &mut Vec<(usize, usize)>, depth: i32) {
        let center = state.board.size / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
        
        // Aggressive move limiting at higher depths
        let limit = match depth {
            d if d >= 8 => 12,   // Deep search: only top 12 moves
            d if d >= 6 => 18,   // Medium: top 18 moves
            d if d >= 4 => 25,   // Shallow: top 25 moves
            _ => moves.len(),    // Very shallow: all moves
        };
        
        moves.truncate(limit.min(moves.len()));
    }

    fn calculate_move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        let center_distance = Self::manhattan_distance(row, col, center, center);
        priority += 100 - center_distance as i32;

        priority += Self::calculate_threat_priority(&state.board, row, col);
        priority += Self::calculate_adjacency_bonus(&state.board, row, col);

        priority
    }

    fn calculate_threat_priority(board: &Board, row: usize, col: usize) -> i32 {
        let mut threat_score = 0;

        for &player in &[Player::Max, Player::Min] {
            for &(dx, dy) in &DIRECTIONS {
                let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
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
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        backwards + forwards + 1
    }

    fn count_direction(
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

    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize) -> i32 {
        let mut neighbor_mask = vec![0u64; board.u64_count];
        let mut num_adjacent = 0;

        for &(dx, dy) in &ALL_DIRECTIONS {
            let nr = row as isize + dx;
            let nc = col as isize + dy;
            if nr >= 0 && nc >= 0 && nr < board.size as isize && nc < board.size as isize {
                let idx = board.index(nr as usize, nc as usize);
                Board::set_bit(&mut neighbor_mask, idx);
            }
        }

        for (&o, &m) in board.occupied.iter().zip(&neighbor_mask) {
            num_adjacent += (o & m).count_ones() as i32;
        }

        num_adjacent * 50
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
