use std::collections::HashMap;

/// Entry type for transposition table values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Exact value (PV node)
    Exact,
    /// Lower bound (beta cutoff, value >= beta)
    LowerBound,
    /// Upper bound (alpha cutoff, value <= alpha)
    UpperBound,
}

/// Entry in the transposition table
#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    /// The hash key for verification
    pub key: u64,
    /// The evaluation score
    pub value: i32,
    /// The depth at which this position was evaluated
    pub depth: i32,
    /// The type of this entry (exact, lower bound, upper bound)
    pub entry_type: EntryType,
    /// The best move found at this position (optional)
    pub best_move: Option<(usize, usize)>,
    /// Age/generation for replacement strategy
    pub age: u32,
}

/// Transposition table implementation using Zobrist hashing
pub struct TranspositionTable {
    /// The actual hash table
    table: HashMap<u64, TranspositionEntry>,
    /// Current age/generation for replacement strategy
    current_age: u32,
    /// Maximum number of entries to prevent unbounded memory growth
    max_size: usize,
    /// Statistics
    hits: u64,
    misses: u64,
    collisions: u64,
}

impl TranspositionTable {
    /// Create a new transposition table with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            table: HashMap::with_capacity(max_size.min(1024 * 1024)),
            current_age: 0,
            max_size,
            hits: 0,
            misses: 0,
            collisions: 0,
        }
    }
    
    /// Create a new transposition table with default size (1M entries)
    pub fn new_default() -> Self {
        Self::new(1_000_000)
    }
    
    /// Store an entry in the transposition table
    pub fn store(&mut self, key: u64, value: i32, depth: i32, entry_type: EntryType, best_move: Option<(usize, usize)>) {
        // Check if we need to clean up the table
        if self.table.len() >= self.max_size {
            self.cleanup_old_entries();
        }
        
        let current_age = self.current_age;
        let new_entry = TranspositionEntry {
            key,
            value,
            depth,
            entry_type,
            best_move,
            age: current_age,
        };
        
        // Check if entry already exists
        match self.table.get(&key) {
            Some(existing) => {
                // Check for hash collision
                if existing.key != key {
                    self.collisions += 1;
                }
                
                // Replace if new entry is more valuable (deeper search or same depth but newer)
                if depth > existing.depth || (depth == existing.depth && current_age >= existing.age) {
                    self.table.insert(key, new_entry);
                }
            }
            None => {
                self.table.insert(key, new_entry);
            }
        }
    }
    
    /// Probe the transposition table for an entry
    pub fn probe(&mut self, key: u64, depth: i32, alpha: i32, beta: i32) -> TTResult {
        if let Some(entry) = self.table.get(&key) {
            // Verify the key matches (detect hash collisions)
            if entry.key != key {
                self.collisions += 1;
                self.misses += 1;
                return TTResult::miss();
            }
            
            self.hits += 1;
            
            // Only use the entry if it was searched to at least the same depth
            if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => {
                        // Exact value, can use directly
                        return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                    }
                    EntryType::LowerBound => {
                        // Lower bound: if value >= beta, we can cutoff
                        if entry.value >= beta {
                            return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                        }
                    }
                    EntryType::UpperBound => {
                        // Upper bound: if value <= alpha, we can cutoff
                        if entry.value <= alpha {
                            return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                        }
                    }
                }
            }
            
            // Even if we can't use the value, we might use the best move for move ordering
            return TTResult::hit_move_only(entry.best_move);
        }
        
        self.misses += 1;
        TTResult::miss()
    }
    
    /// Get the best move from a position (for move ordering)
    pub fn get_best_move(&self, key: u64) -> Option<(usize, usize)> {
        self.table.get(&key).and_then(|entry| {
            if entry.key == key {
                entry.best_move
            } else {
                None
            }
        })
    }
    
    /// Clear the transposition table
    pub fn clear(&mut self) {
        self.table.clear();
        self.current_age = 0;
        self.hits = 0;
        self.misses = 0;
        self.collisions = 0;
    }
    
    /// Advance the age (call this between searches)
    pub fn advance_age(&mut self) {
        self.current_age += 1;
    }
    
    /// Get hit rate for statistics
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits;
        let misses = self.misses;
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    /// Get the current number of entries
    pub fn size(&self) -> usize {
        self.table.len()
    }
    
    /// Get statistics as a tuple (hits, misses, collisions)
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.hits,
            self.misses,
            self.collisions,
        )
    }
    
    /// Clean up old entries when the table is getting full
    fn cleanup_old_entries(&mut self) {
        let current_age = self.current_age;
        if current_age < 10 {
            return; // Don't cleanup too early
        }
        
        let cutoff_age = current_age - 5; // Keep last 5 generations
        let original_size = self.table.len();
        
        // Retain entries that are recent or very deep
        self.table.retain(|_, entry| {
            entry.age >= cutoff_age || entry.depth > 10
        });
        
        // If we didn't free enough space, remove entries more aggressively
        if self.table.len() > self.max_size * 3 / 4 {
            let cutoff_age = current_age - 2; // More aggressive cleanup
            self.table.retain(|_, entry| {
                entry.age >= cutoff_age && entry.depth > 5
            });
        }
        
        let cleaned = original_size - self.table.len();
        if cleaned > 0 {
            println!("Cleaned {} old entries from transposition table", cleaned);
        }
    }
}

/// Result of a transposition table lookup that can be used in minimax
#[derive(Debug)]
pub struct TTResult {
    pub value: Option<i32>,
    pub best_move: Option<(usize, usize)>,
    pub cutoff: bool,
}

impl TTResult {
    pub fn new(value: Option<i32>, best_move: Option<(usize, usize)>, cutoff: bool) -> Self {
        Self { value, best_move, cutoff }
    }
    
    pub fn miss() -> Self {
        Self { value: None, best_move: None, cutoff: false }
    }
    
    pub fn hit_with_cutoff(value: i32, best_move: Option<(usize, usize)>) -> Self {
        Self { value: Some(value), best_move, cutoff: true }
    }
    
    pub fn hit_move_only(best_move: Option<(usize, usize)>) -> Self {
        Self { value: None, best_move, cutoff: false }
    }
}






