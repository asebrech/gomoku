
//! Zobrist hashing utilities.
//!
//! Zobrist hashing assigns a random 64-bit key to each (position, player)
//! pair and combines them with XOR to form a compact hash of a position.
//! This hash is cheap to update incrementally (XOR-in / XOR-out) when moves
//! are made or undone, and is commonly used as a key for transposition
//! tables.
//!
//! References:
//! - <https://en.wikipedia.org/wiki/Zobrist_hashing>

use crate::core::board::Player;
use rand::Rng;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    position_keys: Vec<[u64; 2]>,
    player_key: u64,
    board_size: usize,
}

impl ZobristHash {
    pub fn new(board_size: usize) -> Self {
        let total_positions = board_size * board_size;
        let mut rng = ChaCha8Rng::seed_from_u64(0x123456789ABCDEF0);
        
        let mut position_keys = Vec::with_capacity(total_positions);
        
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
    
    pub fn board_size(&self) -> usize {
        self.board_size
    }
    
    #[inline]
    fn position_index(&self, row: usize, col: usize) -> usize {
        row * self.board_size + col
    }
    
    #[inline]
    fn player_index(player: Player) -> usize {
        match player {
            Player::Max => 0,
            Player::Min => 1,
        }
    }
    
    pub fn compute_hash(&self, state: &crate::core::state::GameState) -> u64 {
        // Compute full Zobrist hash from scratch for a given GameState.
        // This is used during initialization; incremental updates should
        // be performed with `update_hash_*` methods for speed.
        let mut hash = 0u64;
        
        for u64_idx in 0..state.board.u64_count {
            let max_bits = state.board.max_bits[u64_idx];
            let min_bits = state.board.min_bits[u64_idx];
            
            let mut remaining_max = max_bits;
            while remaining_max != 0 {
                let bit_pos = remaining_max.trailing_zeros() as usize;
                let global_pos = u64_idx * 64 + bit_pos;
                if global_pos < self.position_keys.len() {
                    hash ^= self.position_keys[global_pos][0];
                }
                remaining_max &= remaining_max - 1;
            }
            
            let mut remaining_min = min_bits;
            while remaining_min != 0 {
                let bit_pos = remaining_min.trailing_zeros() as usize;
                let global_pos = u64_idx * 64 + bit_pos;
                if global_pos < self.position_keys.len() {
                    hash ^= self.position_keys[global_pos][1];
                }
                remaining_min &= remaining_min - 1;
            }
        }
        
        if state.current_player == Player::Min {
            hash ^= self.player_key;
        }
        
        hash
    }
    
    pub fn update_hash_make_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let pos_idx = self.position_index(row, col);
        let player_idx = Self::player_index(player);
        
        // XOR the position/player key and flip the player bit to keep the
        // hash consistent with the implementation used in `compute_hash`.
        current_hash ^ self.position_keys[pos_idx][player_idx] ^ self.player_key
    }
    
    pub fn update_hash_undo_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        self.update_hash_make_move(current_hash, row, col, player)
    }
    
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
