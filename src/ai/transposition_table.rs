use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

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

/// Concurrent transposition table implementation using Zobrist hashing
pub struct TranspositionTable {
    /// The actual hash table (lock-free concurrent)
    table: Arc<DashMap<u64, TranspositionEntry>>,
    /// Current age/generation for replacement strategy
    current_age: Arc<AtomicU32>,
    /// Maximum number of entries to prevent unbounded memory growth
    max_size: usize,
    /// Statistics (atomic for thread safety)
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    collisions: Arc<AtomicU64>,
}

impl TranspositionTable {
    /// Create a new transposition table with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            table: Arc::new(DashMap::with_capacity(max_size.min(1024 * 1024))),
            current_age: Arc::new(AtomicU32::new(0)),
            max_size,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            collisions: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Create a new transposition table with default size (1M entries)
    pub fn new_default() -> Self {
        Self::new(1_000_000)
    }
    
    /// Clone for sharing between threads
    pub fn clone_shared(&self) -> Self {
        Self {
            table: Arc::clone(&self.table),
            current_age: Arc::clone(&self.current_age),
            max_size: self.max_size,
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
            collisions: Arc::clone(&self.collisions),
        }
    }
    
    /// Store an entry in the transposition table
    pub fn store(&self, key: u64, value: i32, depth: i32, entry_type: EntryType, best_move: Option<(usize, usize)>) {
        // Check if we need to clean up the table
        if self.table.len() >= self.max_size {
            self.cleanup_old_entries();
        }
        
        let current_age = self.current_age.load(Ordering::Relaxed);
        let new_entry = TranspositionEntry {
            key,
            value,
            depth,
            entry_type,
            best_move,
            age: current_age,
        };
        
        // DashMap handles concurrent access automatically
        match self.table.get(&key) {
            Some(existing_ref) => {
                let existing = existing_ref.value();
                
                // Check for hash collision
                if existing.key != key {
                    self.collisions.fetch_add(1, Ordering::Relaxed);
                }
                
                // Replace if new entry is more valuable (deeper search or same depth but newer)
                if depth > existing.depth || (depth == existing.depth && current_age > existing.age) {
                    drop(existing_ref); // Release the reference before inserting
                    self.table.insert(key, new_entry);
                }
            }
            None => {
                self.table.insert(key, new_entry);
            }
        }
    }
    
    /// Probe the transposition table for an entry
    pub fn probe(&self, key: u64, depth: i32, alpha: i32, beta: i32) -> TTResult {
        if let Some(entry_ref) = self.table.get(&key) {
            let entry = entry_ref.value();
            
            // Verify the key matches (detect hash collisions)
            if entry.key != key {
                self.collisions.fetch_add(1, Ordering::Relaxed);
                self.misses.fetch_add(1, Ordering::Relaxed);
                return TTResult::miss();
            }
            
            self.hits.fetch_add(1, Ordering::Relaxed);
            
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
        
        self.misses.fetch_add(1, Ordering::Relaxed);
        TTResult::miss()
    }
    
    /// Get the best move from a position (for move ordering)
    pub fn get_best_move(&self, key: u64) -> Option<(usize, usize)> {
        self.table.get(&key).and_then(|entry_ref| {
            let entry = entry_ref.value();
            if entry.key == key {
                entry.best_move
            } else {
                None
            }
        })
    }
    
    /// Clear the transposition table
    pub fn clear(&self) {
        self.table.clear();
        self.current_age.store(0, Ordering::Relaxed);
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.collisions.store(0, Ordering::Relaxed);
    }
    
    /// Advance the age (call this between searches)
    pub fn advance_age(&self) {
        self.current_age.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get hit rate for statistics
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
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
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.collisions.load(Ordering::Relaxed),
        )
    }
    
    /// Clean up old entries when the table is getting full
    fn cleanup_old_entries(&self) {
        let current_age = self.current_age.load(Ordering::Relaxed);
        if current_age < 10 {
            return; // Don't cleanup too early
        }
        
        let cutoff_age = current_age - 5; // Keep last 5 generations
        let original_size = self.table.len();
        
        // DashMap retain with closure
        self.table.retain(|_, entry| {
            // Keep entries that are recent or very deep
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transposition_table_basic() {
        let tt = TranspositionTable::new(100);
        
        // Store an exact entry
        tt.store(123456, 50, 5, EntryType::Exact, Some((7, 7)));
        
        // Probe for the entry
        let result = tt.probe(123456, 5, -100, 100);
        assert!(result.cutoff);
        assert_eq!(result.value, Some(50));
        assert_eq!(result.best_move, Some((7, 7)));
        
        let (hits, misses, _) = tt.get_stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 0);
    }
    
    #[test]
    fn test_transposition_table_bounds() {
        let tt = TranspositionTable::new(100);
        
        // Store a lower bound entry
        tt.store(123456, 50, 5, EntryType::LowerBound, Some((7, 7)));
        
        // Probe with beta cutoff
        let result = tt.probe(123456, 5, 10, 40);
        assert!(result.cutoff);
        assert_eq!(result.value, Some(50)); // Should cutoff since 50 >= 40 (beta)
        
        // Probe without beta cutoff
        let result = tt.probe(123456, 5, 10, 60);
        assert!(!result.cutoff); // No cutoff, but still useful for move ordering
        assert!(result.best_move.is_some());
    }
    
    #[test]
    fn test_transposition_table_depth_requirement() {
        let tt = TranspositionTable::new(100);
        
        // Store an entry at depth 5
        tt.store(123456, 50, 5, EntryType::Exact, Some((7, 7)));
        
        // Probe at deeper depth - shouldn't use the value
        let result = tt.probe(123456, 8, -100, 100);
        assert!(!result.cutoff); // Can't use value from shallower search
        assert!(result.best_move.is_some()); // But move is still useful
        
        // Probe at same or shallower depth - should use the value
        let result = tt.probe(123456, 5, -100, 100);
        assert!(result.cutoff);
        assert_eq!(result.value, Some(50));
    }
}
