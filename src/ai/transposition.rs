use std::collections::HashMap;
use crate::core::board::Player;
use bevy::prelude::*;

// Types of bounds stored in transposition table
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundType {
    Exact,      // Exact minimax value
    LowerBound, // Alpha cutoff (value >= this)
    UpperBound, // Beta cutoff (value <= this)
}

// Entry in the transposition table with enhanced information
#[derive(Debug, Clone)]
pub struct TTEntry {
    pub hash: u64,                           // Full hash for verification
    pub score: i32,                          // Evaluation score
    pub depth: i32,                          // Depth at which this was evaluated
    pub bound_type: BoundType,               // Type of bound
    pub best_move: Option<(usize, usize)>,   // Best move found at this position
    pub age: u32,                            // Search generation when stored
}

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
    table: HashMap<u64, TTEntry>,
    zobrist: ZobristTable,
    current_age: u32,                        // Current search generation
    max_size: usize,                         // Maximum table size (for memory management)
    hits: usize,                             // Hit counter for statistics
    stores: usize,                           // Store counter for statistics
}

impl TranspositionTable {
    pub fn new(board_width: usize, board_height: usize) -> Self {
        TranspositionTable {
            table: HashMap::with_capacity(1_000_000), // Pre-allocate for better performance
            zobrist: ZobristTable::new(board_width, board_height),
            current_age: 0,
            max_size: 1_000_000, // 1M entries limit
            hits: 0,
            stores: 0,
        }
    }
    
    // Start a new search (increment age)
    pub fn new_search(&mut self) {
        self.current_age = self.current_age.wrapping_add(1);
    }
    
    // Store an entry with all the enhanced information
    pub fn store_enhanced(
        &mut self, 
        hash: u64, 
        score: i32, 
        depth: i32, 
        bound_type: BoundType,
        best_move: Option<(usize, usize)>
    ) {
        self.stores += 1;
        
        // Check if we should replace the existing entry
        if let Some(existing) = self.table.get(&hash) {
            // Only replace if:
            // 1. New entry has greater depth, OR
            // 2. Same depth but newer age, OR  
            // 3. New entry is exact value and old isn't
            let should_replace = depth > existing.depth ||
                (depth == existing.depth && self.current_age > existing.age) ||
                (bound_type == BoundType::Exact && existing.bound_type != BoundType::Exact);
                
            if !should_replace {
                return;
            }
        }
        
        let entry = TTEntry {
            hash,
            score,
            depth,
            bound_type,
            best_move,
            age: self.current_age,
        };
        
        self.table.insert(hash, entry);
        
    }
    
    // Enhanced lookup with depth and bound checking
    pub fn lookup_enhanced(&mut self, hash: u64, depth: i32, alpha: i32, beta: i32) -> Option<(i32, Option<(usize, usize)>)> {
        if let Some(entry) = self.table.get(&hash) {
            // Verify hash collision protection
            if entry.hash != hash {
                return None;
            }
            
            self.hits += 1;
            
            // Only use if stored depth >= current search depth
            if entry.depth >= depth {
                match entry.bound_type {
                    BoundType::Exact => {
                        // Exact value is always usable
                        return Some((entry.score, entry.best_move));
                    }
                    BoundType::LowerBound => {
                        // Lower bound: if score >= beta, we can cut off
                        if entry.score >= beta {
                            return Some((entry.score, entry.best_move));
                        }
                    }
                    BoundType::UpperBound => {
                        // Upper bound: if score <= alpha, we can cut off
                        if entry.score <= alpha {
                            return Some((entry.score, entry.best_move));
                        }
                    }
                }
            }
            
            // Even if we can't use the score, return the best move for move ordering
            if entry.best_move.is_some() {
                return Some((0, entry.best_move)); // Score unused, just for move
            }
        }
        
        None
    }
    
    // Get statistics
    pub fn get_stats(&self) -> (usize, usize, f64) {
        let hit_rate = if self.stores > 0 { 
            self.hits as f64 / (self.hits + self.stores) as f64 * 100.0 
        } else { 
            0.0 
        };
        (self.hits, self.stores, hit_rate)
    }
    
    pub fn clear(&mut self) {
        self.table.clear();
        self.hits = 0;
        self.stores = 0;
        self.current_age = 0;
    }
    
    pub fn size(&self) -> usize {
        self.table.len()
    }
    
    // LEGACY API for backward compatibility (used by tests)
    pub fn store<T: BoardHashable>(&mut self, board: &T, value: i32) {
        let key = self.zobrist.hash_board(board);
        self.store_enhanced(key, value, 0, BoundType::Exact, None);
    }
    
    // Fast store using pre-computed hash (legacy)
    pub fn store_with_hash(&mut self, hash: u64, value: i32) {
        self.store_enhanced(hash, value, 0, BoundType::Exact, None);
    }
    
    pub fn lookup<T: BoardHashable>(&mut self, board: &T) -> Option<i32> {
        let key = self.zobrist.hash_board(board);
        self.lookup_enhanced(key, 0, i32::MIN, i32::MAX).map(|(score, _)| score)
    }
    
    // Fast lookup using pre-computed hash (legacy)
    pub fn lookup_with_hash(&mut self, hash: u64) -> Option<i32> {
        self.lookup_enhanced(hash, 0, i32::MIN, i32::MAX).map(|(score, _)| score)
    }
    
    // Legacy API for tests - direct key/value storage (always overwrites)
    pub fn store_raw(&mut self, key: u64, value: i32) {
        // For legacy compatibility, always overwrite with raw store
        let entry = TTEntry {
            hash: key,
            score: value,
            depth: 0,
            bound_type: BoundType::Exact,
            best_move: None,
            age: self.current_age,
        };
        self.table.insert(key, entry);
    }
    
    pub fn lookup_raw(&self, key: u64) -> Option<i32> {
        self.table.get(&key).map(|entry| entry.score)
    }
}
