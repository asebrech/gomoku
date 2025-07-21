use std::collections::HashMap;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub key: u64,
    pub value: i32,
    pub depth: i32,
    pub entry_type: EntryType,
    pub best_move: Option<(usize, usize)>,
    pub age: u32,
}

#[derive(Resource)]
pub struct TranspositionTable {
    table: HashMap<u64, TranspositionEntry>,
    current_age: u32,
    max_size: usize,
    hits: u64,
    misses: u64,
    collisions: u64,
}

impl TranspositionTable {
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
    
    pub fn new_default() -> Self {
        Self::new(1_000_000)
    }
    
    pub fn store(&mut self, key: u64, value: i32, depth: i32, entry_type: EntryType, best_move: Option<(usize, usize)>) {
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
        
        match self.table.get(&key) {
            Some(existing) => {
                if existing.key != key {
                    self.collisions += 1;
                }
                
                if depth > existing.depth || (depth == existing.depth && current_age >= existing.age) {
                    self.table.insert(key, new_entry);
                }
            }
            None => {
                self.table.insert(key, new_entry);
            }
        }
    }
    
    pub fn probe(&mut self, key: u64, depth: i32, alpha: i32, beta: i32) -> TTResult {
        if let Some(entry) = self.table.get(&key) {
            if entry.key != key {
                self.collisions += 1;
                self.misses += 1;
                return TTResult::miss();
            }
            
            self.hits += 1;
            
            if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => {
                        return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                    }
                    EntryType::LowerBound => {
                        if entry.value >= beta {
                            return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                        }
                    }
                    EntryType::UpperBound => {
                        if entry.value <= alpha {
                            return TTResult::hit_with_cutoff(entry.value, entry.best_move);
                        }
                    }
                }
            }
            
            return TTResult::hit_move_only(entry.best_move);
        }
        
        self.misses += 1;
        TTResult::miss()
    }
    
    pub fn get_best_move(&self, key: u64) -> Option<(usize, usize)> {
        self.table.get(&key).and_then(|entry| {
            if entry.key == key {
                entry.best_move
            } else {
                None
            }
        })
    }
    
    pub fn clear(&mut self) {
        self.table.clear();
        self.current_age = 0;
        self.hits = 0;
        self.misses = 0;
        self.collisions = 0;
    }
    
    pub fn advance_age(&mut self) {
        self.current_age += 1;
    }
    
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
    
    pub fn size(&self) -> usize {
        self.table.len()
    }
    
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.hits,
            self.misses,
            self.collisions,
        )
    }
    
    fn cleanup_old_entries(&mut self) {
        let current_age = self.current_age;
        if current_age < 10 {
            return;
        }
        
        let cutoff_age = current_age - 5;
        let original_size = self.table.len();
        
        self.table.retain(|_, entry| {
            entry.age >= cutoff_age || entry.depth > 10
        });
        
        if self.table.len() > self.max_size * 3 / 4 {
            let cutoff_age = current_age - 2;
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

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new_default()
    }
}

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






