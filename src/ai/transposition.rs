use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

// Packed entry for thread-safe atomic operations
#[derive(Debug, Clone, Copy)]
struct PackedEntry {
    // Packed into u64: hash(16) | value(24) | depth(8) | entry_type(2) | move_row(7) | move_col(7)
    data: u64,
}

impl PackedEntry {
    const HASH_MASK: u64 = 0xFFFF000000000000;
    const VALUE_MASK: u64 = 0x0000FFFFFF000000;
    const DEPTH_MASK: u64 = 0x0000000000FF0000;
    const TYPE_MASK: u64 = 0x000000000000C000;
    const MOVE_ROW_MASK: u64 = 0x0000000000003F80;
    const MOVE_COL_MASK: u64 = 0x000000000000007F;

    const HASH_SHIFT: u64 = 48;
    const VALUE_SHIFT: u64 = 24;
    const DEPTH_SHIFT: u64 = 16;
    const TYPE_SHIFT: u64 = 14;
    const MOVE_ROW_SHIFT: u64 = 7;

    fn new(
        hash: u64,
        value: i32,
        depth: i32,
        entry_type: EntryType,
        best_move: Option<(usize, usize)>,
        _age: u32, // Age removed to save bits
    ) -> Self {
        let hash_part = (hash & 0xFFFF) << Self::HASH_SHIFT;
        
        // Clamp value to 24-bit signed range: -8,388,608 to 8,388,607
        let clamped_value = value.clamp(-8_388_608, 8_388_607);
        let value_part = ((clamped_value as u64) & 0xFFFFFF) << Self::VALUE_SHIFT;
        
        let depth_part = ((depth as u64) & 0xFF) << Self::DEPTH_SHIFT;
        let type_part = ((entry_type as u64) & 0x3) << Self::TYPE_SHIFT;
        
        let (move_row_part, move_col_part) = if let Some((row, col)) = best_move {
            (
                ((row.min(127) as u64) & 0x7F) << Self::MOVE_ROW_SHIFT,
                (col.min(127) as u64) & 0x7F,
            )
        } else {
            (0x7F << Self::MOVE_ROW_SHIFT, 0x7F)
        };

        Self {
            data: hash_part | value_part | depth_part | type_part | move_row_part | move_col_part,
        }
    }

    fn hash(&self) -> u16 {
        ((self.data & Self::HASH_MASK) >> Self::HASH_SHIFT) as u16
    }

    fn value(&self) -> i32 {
        let val = ((self.data & Self::VALUE_MASK) >> Self::VALUE_SHIFT) as u32;
        // Sign extend from 24 bits
        if val & 0x800000 != 0 {
            // Negative number - sign extend
            (val | 0xFF000000) as i32
        } else {
            // Positive number
            val as i32
        }
    }

    fn depth(&self) -> i32 {
        ((self.data & Self::DEPTH_MASK) >> Self::DEPTH_SHIFT) as i32
    }

    fn entry_type(&self) -> EntryType {
        match (self.data & Self::TYPE_MASK) >> Self::TYPE_SHIFT {
            0 => EntryType::Exact,
            1 => EntryType::LowerBound,
            2 => EntryType::UpperBound,
            _ => EntryType::Exact,
        }
    }

    fn best_move(&self) -> Option<(usize, usize)> {
        let row = ((self.data & Self::MOVE_ROW_MASK) >> Self::MOVE_ROW_SHIFT) as usize;
        let col = (self.data & Self::MOVE_COL_MASK) as usize;
        
        if row == 0x7F || col == 0x7F {
            None
        } else {
            Some((row, col))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub value: i32,
    pub depth: i32,
    pub entry_type: EntryType,
    pub best_move: Option<(usize, usize)>,
    pub age: u32,
}

struct SharedTTData {
    entries: Vec<AtomicU64>,
    // For extreme values that don't fit in packed format, use a separate map
    overflow_entries: std::sync::RwLock<std::collections::HashMap<u64, TranspositionEntry>>,
    current_age: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
}

#[derive(Resource, Clone)]
pub struct TranspositionTable {
    data: Arc<SharedTTData>,
    max_size: usize,
}

impl TranspositionTable {
    pub fn new(max_size: usize) -> Self {
        let actual_size = max_size.min(1024 * 1024);
        let mut entries = Vec::with_capacity(actual_size);
        entries.resize_with(actual_size, || AtomicU64::new(0));
        
        Self {
            data: Arc::new(SharedTTData {
                entries,
                overflow_entries: std::sync::RwLock::new(std::collections::HashMap::new()),
                current_age: AtomicU64::new(0),
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
            }),
            max_size: actual_size,
        }
    }
    
    pub fn store(&self, key: u64, value: i32, depth: i32, entry_type: EntryType, best_move: Option<(usize, usize)>) {
        let index = (key as usize) % self.max_size;
        
        // Check if value fits in 24-bit signed range
        if value < -8_388_608 || value > 8_388_607 {
            // Use overflow storage for extreme values
            let mut overflow = self.data.overflow_entries.write().unwrap();
            let entry = TranspositionEntry {
                value,
                depth,
                entry_type,
                best_move,
                age: 0,
            };
            overflow.insert(key, entry);
            return;
        }
        
        let new_entry = PackedEntry::new(key, value, depth, entry_type, best_move, 0);
        
        // Load existing entry to check replacement policy
        let existing_data = self.data.entries[index].load(Ordering::Relaxed);
        if existing_data == 0 {
            // Empty slot, just store
            self.data.entries[index].store(new_entry.data, Ordering::Relaxed);
            return;
        }
        
        let existing = PackedEntry { data: existing_data };
        // Replace if new depth is greater than or equal to existing depth
        let should_replace = depth >= existing.depth();
        
        if should_replace {
            self.data.entries[index].store(new_entry.data, Ordering::Relaxed);
        }
    }
    
    pub fn probe(&self, key: u64, depth: i32, alpha: i32, beta: i32) -> TTResult {
        // First check overflow storage for extreme values
        if let Ok(overflow) = self.data.overflow_entries.read() {
            if let Some(entry) = overflow.get(&key) {
                self.data.hits.fetch_add(1, Ordering::Relaxed);
                
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
        }
        
        // Check regular packed storage
        let index = (key as usize) % self.max_size;
        let data = self.data.entries[index].load(Ordering::Relaxed);
        
        if data == 0 {
            self.data.misses.fetch_add(1, Ordering::Relaxed);
            return TTResult::miss();
        }
        
        let entry = PackedEntry { data };
        let stored_hash = entry.hash() as u64;
        let key_hash = key & 0xFFFF;
        
        if stored_hash != key_hash {
            self.data.misses.fetch_add(1, Ordering::Relaxed);
            return TTResult::miss();
        }
        
        self.data.hits.fetch_add(1, Ordering::Relaxed);
        
        if entry.depth() >= depth {
            match entry.entry_type() {
                EntryType::Exact => {
                    return TTResult::hit_with_cutoff(entry.value(), entry.best_move());
                }
                EntryType::LowerBound => {
                    if entry.value() >= beta {
                        return TTResult::hit_with_cutoff(entry.value(), entry.best_move());
                    }
                }
                EntryType::UpperBound => {
                    if entry.value() <= alpha {
                        return TTResult::hit_with_cutoff(entry.value(), entry.best_move());
                    }
                }
            }
        }
        
        TTResult::hit_move_only(entry.best_move())
    }
    
    pub fn get_best_move(&self, key: u64) -> Option<(usize, usize)> {
        // First check overflow storage
        if let Ok(overflow) = self.data.overflow_entries.read() {
            if let Some(entry) = overflow.get(&key) {
                return entry.best_move;
            }
        }
        
        // Check regular packed storage
        let index = (key as usize) % self.max_size;
        let data = self.data.entries[index].load(Ordering::Relaxed);
        
        if data == 0 {
            return None;
        }
        
        let entry = PackedEntry { data };
        let stored_hash = entry.hash() as u64;
        let key_hash = key & 0xFFFF;
        
        if stored_hash == key_hash {
            entry.best_move()
        } else {
            None
        }
    }
    
    pub fn clear(&self) {
        for entry in &self.data.entries {
            entry.store(0, Ordering::Relaxed);
        }
        if let Ok(mut overflow) = self.data.overflow_entries.write() {
            overflow.clear();
        }
        self.data.current_age.store(0, Ordering::Relaxed);
        self.data.hits.store(0, Ordering::Relaxed);
        self.data.misses.store(0, Ordering::Relaxed);
    }
    
    pub fn advance_age(&self) {
        self.data.current_age.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn hit_rate(&self) -> f64 {
        let hits = self.data.hits.load(Ordering::Relaxed);
        let misses = self.data.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    pub fn size(&self) -> usize {
        self.max_size
    }
    
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.data.hits.load(Ordering::Relaxed),
            self.data.misses.load(Ordering::Relaxed),
        )
    }
    
    pub fn add_stats(&self, hits: u64, misses: u64) {
        self.data.hits.fetch_add(hits, Ordering::Relaxed);
        self.data.misses.fetch_add(misses, Ordering::Relaxed);
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






