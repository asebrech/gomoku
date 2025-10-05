use crate::ai::precompute::DirectionTables;
use crate::core::board::{Board, Player};

pub struct CaptureHandler;

impl CaptureHandler {
    pub fn detect_captures(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dir_tables: &DirectionTables,
    ) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let opponent = player.opponent();
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        let opponent_bits = match opponent {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        let idx = dir_tables.to_index(row, col);
        
        // Use precomputed capture patterns (4 directions)
        for direction in 0..4 {
            let patterns = dir_tables.get_capture_patterns(idx, direction);
            
            for &(idx1, idx2, idx3) in patterns {
                // Check pattern: player at current, opponent at pos1, opponent at pos2, player at pos3
                // This forms: X-O-O-X where X is player and O is opponent
                if Board::is_bit_set(opponent_bits, idx1)
                    && Board::is_bit_set(opponent_bits, idx2)
                    && Board::is_bit_set(player_bits, idx3)
                {
                    let (r1, c1) = dir_tables.to_coords(idx1);
                    let (r2, c2) = dir_tables.to_coords(idx2);
                    captures.push((r1, c1));
                    captures.push((r2, c2));
                }
            }
        }

        captures
    }

    pub fn execute_captures(board: &mut Board, captures: &[(usize, usize)]) {
        for &(r, c) in captures {
            let idx = board.index(r, c);
            Board::clear_bit(&mut board.max_bits, idx);
            Board::clear_bit(&mut board.min_bits, idx);
            Board::clear_bit(&mut board.occupied, idx);
        }
    }
}
