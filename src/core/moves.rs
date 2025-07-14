use crate::core::board::{Board, Player};

pub struct MoveHandler;

impl MoveHandler {
    pub fn get_possible_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = Vec::with_capacity(50);
        
        if board.is_empty() {
            let center = board.center();
            moves.push(center);
            return moves;
        }

        let occupied = Self::combine_bitboards(&board.max_pieces, &board.min_pieces);
        let adjacent_mask = Self::generate_adjacent_mask(occupied, board.size);
        
        for row in 0..board.size {
            for col in 0..board.size {
                let (chunk_idx, bit) = Board::position_to_bit(row, col, board.size);
                if chunk_idx < 3 && occupied[chunk_idx] & bit == 0 && adjacent_mask[chunk_idx] & bit != 0 {
                    if !RuleValidator::creates_double_three(board, row, col, player) {
                        moves.push((row, col));
                    }
                }
            }
        }
        
        moves
    }

    fn combine_bitboards(max_pieces: &[u128; 3], min_pieces: &[u128; 3]) -> [u128; 3] {
        [
            max_pieces[0] | min_pieces[0],
            max_pieces[1] | min_pieces[1],
            max_pieces[2] | min_pieces[2],
        ]
    }

    fn generate_adjacent_mask(occupied: [u128; 3], size: usize) -> [u128; 3] {
        let mut mask = [0u128; 3];
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        for row in 0..size {
            for col in 0..size {
                let (chunk_idx, bit) = Board::position_to_bit(row, col, size);
                if chunk_idx < 3 && occupied[chunk_idx] & bit != 0 {
                    for &(dr, dc) in &directions {
                        let new_row = row as isize + dr;
                        let new_col = col as isize + dc;
                        if new_row >= 0 && new_row < size as isize && 
                           new_col >= 0 && new_col < size as isize {
                            let (adj_chunk_idx, adj_bit) = Board::position_to_bit(new_row as usize, new_col as usize, size);
                            if adj_chunk_idx < 3 {
                                mask[adj_chunk_idx] |= adj_bit;
                            }
                        }
                    }
                }
            }
        }
        
        mask
    }
}

pub struct RuleValidator;

impl RuleValidator {
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut free_three_count = 0;

        for &direction in &directions {
            if Self::is_free_three_fast(board, row, col, player, direction) {
                free_three_count += 1;
                if free_three_count >= 2 {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_free_three_fast(board: &Board, row: usize, col: usize, player: Player, direction: (isize, isize)) -> bool {
        let (dx, dy) = direction;
        let mut count = 1; // Count the move itself
        let mut blocked_left = false;
        let mut blocked_right = false;

        // Check left direction
        for i in 1..=3 {
            let new_row = row as isize - i * dx;
            let new_col = col as isize - i * dy;
            
            if new_row < 0 || new_row >= board.size as isize || 
               new_col < 0 || new_col >= board.size as isize {
                blocked_left = true;
                break;
            }
            
            match board.get_player(new_row as usize, new_col as usize) {
                Some(p) if p == player => count += 1,
                Some(_) => { blocked_left = true; break; },
                None => break,
            }
        }

        // Check right direction
        for i in 1..=3 {
            let new_row = row as isize + i * dx;
            let new_col = col as isize + i * dy;
            
            if new_row < 0 || new_row >= board.size as isize || 
               new_col < 0 || new_col >= board.size as isize {
                blocked_right = true;
                break;
            }
            
            match board.get_player(new_row as usize, new_col as usize) {
                Some(p) if p == player => count += 1,
                Some(_) => { blocked_right = true; break; },
                None => break,
            }
        }

        // A free three requires exactly 3 stones and neither end blocked
        count == 3 && !blocked_left && !blocked_right
    }

    pub fn is_free_three(board: &Board, row: usize, col: usize, player: Player, direction: (isize, isize)) -> bool {
        // For the test that expects this signature, use the fast version
        Self::is_free_three_fast(board, row, col, player, direction)
    }
}

