use std::collections::HashMap;
use crate::core::board::Player;
use bevy::prelude::*;

// Zobrist hash table for board position hashing
#[derive(Clone, Debug)]
pub struct ZobristTable {
    piece_keys: Vec<[u64; 2]>, // [position] -> [player] -> random key (0=Max/Black, 1=Min/White)
    capture_keys: [u64; 256],  // keys for capture counts (up to 255 each)
}

impl ZobristTable {
    pub fn new(width: usize, height: usize) -> Self {        
        let total_positions = width * height;
        let mut piece_keys = vec![[0u64; 2]; total_positions];
        let mut capture_keys = [0u64; 256];
        
        // Generate deterministic pseudo-random values using a simple PRNG
        let mut rng_state = 0x123456789abcdef0u64;
        
        // Simple LCG for pseudo-random number generation
        let mut next_random = || {
            rng_state = rng_state.wrapping_mul(6364136223846793005u64).wrapping_add(1);
            rng_state
        };
        
        // Initialize piece keys for each position and player
        for pos in 0..total_positions {
            piece_keys[pos] = [next_random(), next_random()];
        }
        
        // Initialize capture keys
        for i in 0..256 {
            capture_keys[i] = next_random();
        }
        
        Self {
            piece_keys,
            capture_keys,
        }
    }
    
    // Hash a board position using Zobrist hashing
    pub fn hash_board<T: BoardHashable>(&self, board: &T) -> u64 {
        let mut hash = 0u64;
        
        // Hash each piece on the board
        for row in 0..board.height() {
            for col in 0..board.width() {
                let pos_index = row * board.width() + col;
                
                match board.get_player(row, col) {
                    Some(Player::Max) => {
                        // Max player (black) -> use key index 0
                        hash ^= self.piece_keys[pos_index][0];
                    }
                    Some(Player::Min) => {
                        // Min player (white) -> use key index 1
                        hash ^= self.piece_keys[pos_index][1];
                    }
                    None => {
                        // Empty position contributes nothing to hash
                    }
                }
            }
        }
        
        hash
    }
    
    // Get the hash key for a specific position and player (for incremental updates)
    pub fn get_piece_key(&self, row: usize, col: usize, width: usize, player: Player) -> u64 {
        let pos_index = row * width + col;
        let player_index = if player == Player::Max { 0 } else { 1 };
        self.piece_keys[pos_index][player_index]
    }
}

// Trait for types that can be hashed by the Zobrist table
pub trait BoardHashable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get_player(&self, row: usize, col: usize) -> Option<Player>;
}

#[derive(Resource)]
pub struct TranspositionTable {
    table: HashMap<u64, i32>,
    zobrist: ZobristTable,
}

impl TranspositionTable {
    pub fn new(board_width: usize, board_height: usize) -> Self {
        TranspositionTable {
            table: HashMap::new(),
            zobrist: ZobristTable::new(board_width, board_height),
        }
    }
    
    pub fn store<T: BoardHashable>(&mut self, board: &T, value: i32) {
        let key = self.zobrist.hash_board(board);
        self.table.insert(key, value);
    }
    
    // Fast store using pre-computed hash
    pub fn store_with_hash(&mut self, hash: u64, value: i32) {
        self.table.insert(hash, value);
    }
    
    pub fn lookup<T: BoardHashable>(&self, board: &T) -> Option<i32> {
        let key = self.zobrist.hash_board(board);
        self.table.get(&key).cloned()
    }
    
    // Fast lookup using pre-computed hash
    pub fn lookup_with_hash(&self, hash: u64) -> Option<i32> {
        self.table.get(&hash).cloned()
    }
    
    pub fn clear(&mut self) {
        self.table.clear();
    }
    
    pub fn size(&self) -> usize {
        self.table.len()
    }
    
    // Legacy API for tests - direct key/value storage
    pub fn store_raw(&mut self, key: u64, value: i32) {
        self.table.insert(key, value);
    }
    
    pub fn lookup_raw(&self, key: u64) -> Option<i32> {
        self.table.get(&key).cloned()
    }
}
