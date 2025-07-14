use std::collections::HashMap;
use std::sync::OnceLock;

pub struct PatternTable {
    patterns: HashMap<u32, i32>,
}

impl PatternTable {
    pub fn new() -> Self {
        let mut table = PatternTable {
            patterns: HashMap::with_capacity(1024),
        };
        table.initialize_patterns();
        table
    }

    fn initialize_patterns(&mut self) {
        // Pre-compute all 5-piece patterns
        for pattern in 0..3_u32.pow(5) {
            let score = Self::evaluate_pattern_static(pattern);
            self.patterns.insert(pattern, score);
        }
    }

    fn evaluate_pattern_static(pattern: u32) -> i32 {
        let mut pieces = Vec::with_capacity(5);
        let mut temp = pattern;
        
        for _ in 0..5 {
            pieces.push(temp % 3);
            temp /= 3;
        }

        let max_count = pieces.iter().filter(|&&x| x == 1).count();
        let min_count = pieces.iter().filter(|&&x| x == 2).count();
        let empty_count = pieces.iter().filter(|&&x| x == 0).count();

        // Check for winning patterns
        if max_count == 5 { return 100_000; }
        if min_count == 5 { return -100_000; }
        
        // Check for four-in-a-row (one empty)
        if max_count == 4 && empty_count == 1 { 
            return if Self::is_open_four(&pieces, 1) { 50_000 } else { 10_000 };
        }
        if min_count == 4 && empty_count == 1 { 
            return if Self::is_open_four(&pieces, 2) { -50_000 } else { -10_000 };
        }
        
        // Check for three-in-a-row (two empty)
        if max_count == 3 && empty_count == 2 { 
            return if Self::is_open_three(&pieces, 1) { 5_000 } else { 1_000 };
        }
        if min_count == 3 && empty_count == 2 { 
            return if Self::is_open_three(&pieces, 2) { -5_000 } else { -1_000 };
        }

        // Check for two-in-a-row (three empty)
        if max_count == 2 && empty_count == 3 { 
            return if Self::is_open_two(&pieces, 1) { 100 } else { 10 };
        }
        if min_count == 2 && empty_count == 3 { 
            return if Self::is_open_two(&pieces, 2) { -100 } else { -10 };
        }

        0
    }

    fn is_open_four(pieces: &[u32], _player: u32) -> bool {
        // Check if the four pieces form an open four (not blocked)
        let first_empty = pieces.iter().position(|&x| x == 0);
        let _last_empty = pieces.iter().rposition(|&x| x == 0);
        
        // If there's only one empty space and it's not at the edges, it's open
        if let Some(pos) = first_empty {
            pos > 0 && pos < 4
        } else {
            false
        }
    }

    fn is_open_three(pieces: &[u32], player: u32) -> bool {
        // Check if the three pieces can form a winning sequence
        let player_positions: Vec<usize> = pieces.iter().enumerate()
            .filter(|&(_, &x)| x == player)
            .map(|(i, _)| i)
            .collect();
        
        if player_positions.len() != 3 {
            return false;
        }

        // Check if they're consecutive or have small gaps
        let min_pos = *player_positions.iter().min().unwrap();
        let max_pos = *player_positions.iter().max().unwrap();
        
        // If the span is 4 or less, it could be a threat
        max_pos - min_pos <= 3
    }

    fn is_open_two(pieces: &[u32], player: u32) -> bool {
        // Simple check for two pieces that could develop
        let player_positions: Vec<usize> = pieces.iter().enumerate()
            .filter(|&(_, &x)| x == player)
            .map(|(i, _)| i)
            .collect();
        
        if player_positions.len() != 2 {
            return false;
        }

        // Check if they're not too far apart
        let min_pos = *player_positions.iter().min().unwrap();
        let max_pos = *player_positions.iter().max().unwrap();
        
        max_pos - min_pos <= 3
    }

    pub fn lookup_pattern(&self, pattern: u32) -> i32 {
        *self.patterns.get(&pattern).unwrap_or(&0)
    }
}

static PATTERN_TABLE: OnceLock<PatternTable> = OnceLock::new();

pub fn get_pattern_table() -> &'static PatternTable {
    PATTERN_TABLE.get_or_init(|| PatternTable::new())
}