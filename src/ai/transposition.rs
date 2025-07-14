use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TTEntry {
    pub value: i32,
    pub depth: i32,
    pub flag: TTFlag,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

pub struct TranspositionTable {
    table: HashMap<u64, TTEntry>,
    size_limit: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: HashMap::with_capacity(1_000_000),
            size_limit: 1_000_000,
        }
    }

    pub fn store(&mut self, key: u64, value: i32, depth: i32, flag: TTFlag) {
        // Check if we need to clear the table
        if self.table.len() >= self.size_limit {
            self.table.clear();
        }

        // Create new entry
        let entry = TTEntry { value, depth, flag };
        
        // Insert into table (this WILL overwrite existing entries)
        self.table.insert(key, entry);
    }

    pub fn lookup(&self, key: u64, depth: i32, alpha: i32, beta: i32) -> Option<i32> {
        // Get the entry from the table
        let entry = self.table.get(&key)?;
        
        // Check if the stored depth is sufficient
        if entry.depth < depth {
            return None;
        }
        
        // Apply the flag-specific logic
        match entry.flag {
            TTFlag::Exact => {
                // Exact values can always be used
                Some(entry.value)
            },
            TTFlag::LowerBound => {
                // LowerBound: actual value >= stored value
                // Use it if stored value >= beta (causes beta cutoff)
                if entry.value >= beta {
                    Some(entry.value)
                } else {
                    None
                }
            },
            TTFlag::UpperBound => {
                // UpperBound: actual value <= stored value
                // Use it if stored value <= alpha (causes alpha cutoff)
                if entry.value <= alpha {
                    Some(entry.value)
                } else {
                    None
                }
            },
        }
    }
}
