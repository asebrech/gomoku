use std::collections::HashMap;

/// Evaluation cache entry
#[derive(Debug, Clone, Copy)]
pub struct EvalCacheEntry {
    pub score: i32,
    pub depth: i32,
}

/// Fast evaluation cache separate from transposition table
/// Uses LRU-like eviction when full
pub struct EvalCache {
    cache: HashMap<u64, EvalCacheEntry>,
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl EvalCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size.min(100_000)),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Store an evaluation in the cache
    pub fn store(&mut self, hash: u64, score: i32, depth: i32) {
        if self.cache.len() >= self.max_size {
            // Simple eviction: clear 25% of cache when full
            if self.cache.len() > self.max_size {
                let keys_to_remove: Vec<u64> = self.cache.keys()
                    .take(self.max_size / 4)
                    .copied()
                    .collect();
                for key in keys_to_remove {
                    self.cache.remove(&key);
                }
            }
        }

        self.cache.insert(hash, EvalCacheEntry { score, depth });
    }

    /// Probe the cache for an evaluation
    pub fn probe(&mut self, hash: u64, depth: i32) -> Option<i32> {
        if let Some(entry) = self.cache.get(&hash) {
            // Use cached value if depth is equal or greater
            if entry.depth >= depth {
                self.hits += 1;
                return Some(entry.score);
            }
        }
        self.misses += 1;
        None
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new(100_000)
    }
}
