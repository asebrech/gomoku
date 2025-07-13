use std::collections::HashMap;

pub struct TranspositionTable {
    table: HashMap<u64, i32>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: HashMap::new(),
        }
    }
    
    pub fn store(&mut self, key: u64, value: i32) {
        self.table.insert(key, value);
    }
    
    pub fn lookup(&self, key: u64) -> Option<i32> {
        self.table.get(&key).cloned()
    }
}
