use crate::core::board::Player;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Zobrist hashing implementation for board positions
/// Uses precomputed random numbers for each position and player combination
#[derive(Clone)]
pub struct ZobristHash {
    /// Random values for each position and player
    /// [position][player] where player: 0 = Max, 1 = Min
    position_keys: Vec<[u64; 2]>,
    /// Random value for player to move (0 or this value)
    player_key: u64,
    /// Board size for validation
    board_size: usize,
}

impl ZobristHash {
    /// Create a new Zobrist hash table for a given board size
    /// Uses a fixed seed for reproducible results
    pub fn new(board_size: usize) -> Self {
        let total_positions = board_size * board_size;
        let mut rng = ChaCha8Rng::seed_from_u64(0x123456789ABCDEF0);
        
        let mut position_keys = Vec::with_capacity(total_positions);
        
        // Generate random keys for each position and player combination
        for _ in 0..total_positions {
            position_keys.push([rng.gen::<u64>(), rng.gen::<u64>()]);
        }
        
        let player_key = rng.gen::<u64>();
        
        Self {
            position_keys,
            player_key,
            board_size,
        }
    }
    
    /// Get the board size
    pub fn board_size(&self) -> usize {
        self.board_size
    }
    
    /// Get the index for a board position
    #[inline]
    fn position_index(&self, row: usize, col: usize) -> usize {
        row * self.board_size + col
    }
    
    /// Get the player index (0 for Max, 1 for Min)
    #[inline]
    fn player_index(player: Player) -> usize {
        match player {
            Player::Max => 0,
            Player::Min => 1,
        }
    }
    
    /// Compute the complete hash for a game state
    pub fn compute_hash(&self, state: &crate::core::state::GameState) -> u64 {
        let mut hash = 0u64;
        
        // Hash all stones on the board
        for row in 0..self.board_size {
            for col in 0..self.board_size {
                if let Some(player) = state.board.get_player(row, col) {
                    let pos_idx = self.position_index(row, col);
                    let player_idx = Self::player_index(player);
                    hash ^= self.position_keys[pos_idx][player_idx];
                }
            }
        }
        
        // Hash the current player to move
        if state.current_player == Player::Min {
            hash ^= self.player_key;
        }
        
        hash
    }
    
    /// Update hash incrementally when making a move
    pub fn update_hash_make_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let pos_idx = self.position_index(row, col);
        let player_idx = Self::player_index(player);
        
        // XOR in the piece placement and XOR the player key to switch turns
        current_hash ^ self.position_keys[pos_idx][player_idx] ^ self.player_key
    }
    
    /// Update hash incrementally when undoing a move
    pub fn update_hash_undo_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let pos_idx = self.position_index(row, col);
        let player_idx = Self::player_index(player);
        
        // XOR out the piece placement and XOR the player key to switch turns back
        current_hash ^ self.position_keys[pos_idx][player_idx] ^ self.player_key
    }
    
    /// Update hash when capturing stones
    pub fn update_hash_capture(&self, current_hash: u64, captured_positions: &[(usize, usize)], captured_player: Player) -> u64 {
        let mut hash = current_hash;
        let player_idx = Self::player_index(captured_player);
        
        for &(row, col) in captured_positions {
            let pos_idx = self.position_index(row, col);
            hash ^= self.position_keys[pos_idx][player_idx];
        }
        
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::GameState;
    
    #[test]
    fn test_zobrist_consistency() {
        let zobrist = ZobristHash::new(15);
        let mut state = GameState::new(15, 5);
        
        // Compute initial hash
        let initial_hash = zobrist.compute_hash(&state);
        
        // Make a move and compute hash both ways
        let move_pos = (7, 7);
        let incremental_hash = zobrist.update_hash_make_move(initial_hash, move_pos.0, move_pos.1, state.current_player);
        
        state.make_move(move_pos);
        let computed_hash = zobrist.compute_hash(&state);
        
        assert_eq!(incremental_hash, computed_hash, "Incremental and computed hashes should match");
    }
    
    #[test]
    fn test_zobrist_undo_consistency() {
        let zobrist = ZobristHash::new(15);
        let mut state = GameState::new(15, 5);
        
        let initial_hash = zobrist.compute_hash(&state);
        
        // Make a move
        let move_pos = (7, 7);
        let after_move_hash = zobrist.update_hash_make_move(initial_hash, move_pos.0, move_pos.1, state.current_player);
        state.make_move(move_pos);
        
        // Undo the move
        let after_undo_hash = zobrist.update_hash_undo_move(after_move_hash, move_pos.0, move_pos.1, state.current_player.opponent());
        state.undo_move(move_pos);
        
        let final_computed_hash = zobrist.compute_hash(&state);
        
        assert_eq!(initial_hash, final_computed_hash, "Hash should return to initial value after undo");
        assert_eq!(after_undo_hash, final_computed_hash, "Incremental undo should match computed hash");
    }
}
