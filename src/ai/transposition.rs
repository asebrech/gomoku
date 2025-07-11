use std::collections::HashMap;

/// Transposition table for memoizing minimax search results
/// 
/// This table stores previously computed evaluation scores for game positions
/// to avoid recomputing the same positions multiple times during the search.
pub struct TranspositionTable {
    table: HashMap<u64, i32>,
}

impl TranspositionTable {
    /// Create a new empty transposition table
    pub fn new() -> Self {
        TranspositionTable {
            table: HashMap::new(),
        }
    }
    
    /// Store an evaluation score for a given position hash
    /// 
    /// # Arguments
    /// 
    /// * `key` - Hash of the game position
    /// * `value` - Evaluation score for the position
    pub fn store(&mut self, key: u64, value: i32) {
        self.table.insert(key, value);
    }
    
    /// Look up a previously stored evaluation score
    /// 
    /// # Arguments
    /// 
    /// * `key` - Hash of the game position
    /// 
    /// # Returns
    /// 
    /// The stored evaluation score, or None if not found
    pub fn lookup(&self, key: u64) -> Option<i32> {
        self.table.get(&key).cloned()
    }
}
