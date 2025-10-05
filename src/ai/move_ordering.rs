use crate::core::board::{Board, Player};
use crate::core::state::GameState;
use crate::core::pattern_analysis::{PatternAnalyzer, DIRECTIONS};

pub struct MoveOrdering;

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
        PatternAnalyzer::count_consecutive_bidirectional(board, row, col, dx, dy, player)
    }



    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize) -> i32 {
        PatternAnalyzer::count_adjacent_stones(board, row, col) * 50
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        PatternAnalyzer::manhattan_distance(row1, col1, row2, col2)
    }
}
