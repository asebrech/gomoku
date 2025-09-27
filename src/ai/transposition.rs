use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub value: i32,
    pub depth: i32,
    pub entry_type: EntryType,
    pub best_move: Option<(usize, usize)>,
    pub age: u32,
}

#[derive(Debug)]
struct TranspositionTableInner {
    table: HashMap<u64, TranspositionEntry>,
    current_age: u32,
    max_size: usize,
    hits: u64,
    misses: u64,
}

#[derive(Resource, Clone)]
pub struct TranspositionTable {
    inner: Arc<RwLock<TranspositionTableInner>>,
}

impl TranspositionTable {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(TranspositionTableInner {
                table: HashMap::with_capacity(max_size.min(1024 * 1024)),
                current_age: 0,
                max_size,
                hits: 0,
                misses: 0,
            })),
        }
    }
    
    pub fn store(&self, key: u64, value: i32, depth: i32, entry_type: EntryType, best_move: Option<(usize, usize)>) {
        let mut inner = self.inner.write().unwrap();
        
        if inner.table.len() >= inner.max_size {
            Self::cleanup_old_entries_inner(&mut inner);
        }
        
        let current_age = inner.current_age;
        let new_entry = TranspositionEntry {
            value,
            depth,
            entry_type,
            best_move,
            age: current_age,
        };
        
        match inner.table.get(&key) {
            Some(existing) => {
                if depth > existing.depth || (depth == existing.depth && current_age >= existing.age) {
                    inner.table.insert(key, new_entry);
                }
            }
            None => {
                inner.table.insert(key, new_entry);
            }
        }
    }
    
    pub fn probe(&self, key: u64, depth: i32, alpha: i32, beta: i32) -> TTResult {
        let inner = self.inner.read().unwrap();
        
        if let Some(entry) = inner.table.get(&key) {
            let result = if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => {
                        Some(TTResult::hit_with_cutoff(entry.value, entry.best_move))
                    }
                    EntryType::LowerBound => {
                        if entry.value >= beta {
                            Some(TTResult::hit_with_cutoff(entry.value, entry.best_move))
                        } else {
                            None
                        }
                    }
                    EntryType::UpperBound => {
                        if entry.value <= alpha {
                            Some(TTResult::hit_with_cutoff(entry.value, entry.best_move))
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            
            let best_move = entry.best_move;
            drop(inner);
            
            self.inner.write().unwrap().hits += 1;
            
            if let Some(result) = result {
                return result;
            } else {
                return TTResult::hit_move_only(best_move);
            }
        }
        
        drop(inner);
        self.inner.write().unwrap().misses += 1;
        TTResult::miss()
    }
    
    pub fn get_best_move(&self, key: u64) -> Option<(usize, usize)> {
        let inner = self.inner.read().unwrap();
        inner.table.get(&key).and_then(|entry| entry.best_move)
    }
    
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.table.clear();
        inner.current_age = 0;
        inner.hits = 0;
        inner.misses = 0;
    }
    
    pub fn advance_age(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.current_age += 1;
    }
    
    pub fn hit_rate(&self) -> f64 {
        let inner = self.inner.read().unwrap();
        let hits = inner.hits;
        let misses = inner.misses;
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    pub fn size(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.table.len()
    }
    
    pub fn get_stats(&self) -> (u64, u64) {
        let inner = self.inner.read().unwrap();
        (inner.hits, inner.misses)
    }
    
    pub fn add_stats(&self, hits: u64, misses: u64) {
        let mut inner = self.inner.write().unwrap();
        inner.hits += hits;
        inner.misses += misses;
    }
    
    fn cleanup_old_entries_inner(inner: &mut TranspositionTableInner) {
        let current_age = inner.current_age;
        if current_age < 10 {
            return;
        }
        
        let cutoff_age = current_age - 5;
        
        inner.table.retain(|_, entry| {
            entry.age >= cutoff_age || entry.depth > 10
        });
        
        if inner.table.len() > inner.max_size * 3 / 4 {
            let cutoff_age = current_age - 2;
            inner.table.retain(|_, entry| {
                entry.age >= cutoff_age && entry.depth > 5
            });
        }
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(1_000_000)
    }
}

#[derive(Debug)]
pub struct TTResult {
    pub value: Option<i32>,
    pub best_move: Option<(usize, usize)>,
    pub cutoff: bool,
}

impl TTResult {
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






