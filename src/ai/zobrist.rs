
use crate::core::board::Player;
use rand::Rng;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

/// Zobrist hashing implementation for board positions
/// Uses precomputed random numbers for each position and player combination
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
            position_keys.push([rng.random::<u64>(), rng.random::<u64>()]);
        }
        
        let player_key = rng.random::<u64>();
        
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
