use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Max,
    Min,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        }
    }
}

#[derive(Resource, Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    // Bitboards for each player - using Vec<u64> to support any board size
    max_pieces: Vec<u64>,
    min_pieces: Vec<u64>,
    pub size: usize,
    // Number of u64 chunks needed for this board size
    chunks: usize,
    // Precomputed full board mask for efficient is_full checks
    full_mask: Vec<u64>,
}

impl Board {
    pub fn new(size: usize) -> Self {
        let total_positions = size * size;
        let chunks = (total_positions + 63) / 64; // Round up to nearest multiple of 64
        
        // Create full mask for efficient is_full checks
        let mut full_mask = vec![u64::MAX; chunks];
        
        // Handle the last chunk - mask out unused bits
        if total_positions % 64 != 0 {
            let used_bits = total_positions % 64;
            full_mask[chunks - 1] = (1u64 << used_bits) - 1;
        }
        
        Board {
            max_pieces: vec![0u64; chunks],
            min_pieces: vec![0u64; chunks],
            size,
            chunks,
            full_mask,
        }
    }

    /// Convert (row, col) to bit position
    #[inline]
    fn pos_to_bit(&self, row: usize, col: usize) -> (usize, usize) {
        let pos = row * self.size + col;
        (pos / 64, pos % 64)
    }

    /// Check if position has any piece
    #[inline]
    fn is_occupied(&self, row: usize, col: usize) -> bool {
        let (chunk, bit) = self.pos_to_bit(row, col);
        let mask = 1u64 << bit;
        (self.max_pieces[chunk] | self.min_pieces[chunk]) & mask != 0
    }

    /// Set bit for specific player
    #[inline]
    fn set_bit(&mut self, row: usize, col: usize, player: Player) {
        let (chunk, bit) = self.pos_to_bit(row, col);
        let mask = 1u64 << bit;
        match player {
            Player::Max => self.max_pieces[chunk] |= mask,
            Player::Min => self.min_pieces[chunk] |= mask,
        }
    }

    /// Clear bit at position
    #[inline]
    fn clear_bit(&mut self, row: usize, col: usize) {
        let (chunk, bit) = self.pos_to_bit(row, col);
        let mask = !(1u64 << bit);
        self.max_pieces[chunk] &= mask;
        self.min_pieces[chunk] &= mask;
    }

    /// Get player at position
    #[inline]
    fn get_bit(&self, row: usize, col: usize) -> Option<Player> {
        let (chunk, bit) = self.pos_to_bit(row, col);
        let mask = 1u64 << bit;
        if self.max_pieces[chunk] & mask != 0 {
            Some(Player::Max)
        } else if self.min_pieces[chunk] & mask != 0 {
            Some(Player::Min)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        // Optimized: check all chunks in a single loop
        for i in 0..self.chunks {
            if (self.max_pieces[i] | self.min_pieces[i]) != 0 {
                return false;
            }
        }
        true
    }

    pub fn center(&self) -> (usize, usize) {
        (self.size / 2, self.size / 2)
    }

    pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
        !self.is_occupied(row, col)
    }

    pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
        self.get_bit(row, col)
    }

    pub fn place_stone(&mut self, row: usize, col: usize, player: Player) {
        self.set_bit(row, col, player);
    }

    pub fn remove_stone(&mut self, row: usize, col: usize) {
        self.clear_bit(row, col);
    }

    pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
        // Optimized version: avoid bounds checking when possible
        if row > 0 && row < self.size - 1 && col > 0 && col < self.size - 1 {
            // Interior positions - no bounds checking needed
            let directions = [
                (-1, -1), (-1, 0), (-1, 1),
                (0, -1),           (0, 1),
                (1, -1),  (1, 0),  (1, 1)
            ];
            
            for &(dr, dc) in &directions {
                let new_row = (row as isize + dr) as usize;
                let new_col = (col as isize + dc) as usize;
                
                if self.is_occupied(new_row, new_col) {
                    return true;
                }
            }
            false
        } else {
            // Edge/corner positions - need bounds checking
            let directions = [-1, 0, 1];
            
            for &dr in &directions {
                for &dc in &directions {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    
                    let new_row = row as isize + dr;
                    let new_col = col as isize + dc;
                    
                    if new_row >= 0
                        && new_col >= 0
                        && new_row < self.size as isize
                        && new_col < self.size as isize
                    {
                        if self.is_occupied(new_row as usize, new_col as usize) {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for &chunk in &self.max_pieces {
            chunk.hash(&mut hasher);
        }
        for &chunk in &self.min_pieces {
            chunk.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Fast count of stones for a player using bit operations
    pub fn count_player_stones(&self, player: Player) -> u32 {
        match player {
            Player::Max => self.max_pieces.iter().map(|chunk| chunk.count_ones()).sum(),
            Player::Min => self.min_pieces.iter().map(|chunk| chunk.count_ones()).sum(),
        }
    }

    /// Check if the board is full using precomputed bitmask
    pub fn is_full(&self) -> bool {
        // Optimized: combine occupancy and compare with mask in a single pass
        for i in 0..self.chunks {
            if (self.max_pieces[i] | self.min_pieces[i]) != self.full_mask[i] {
                return false;
            }
        }
        true
    }

    /// Get all occupied positions as an iterator (useful for debugging/display)
    pub fn occupied_positions(&self) -> impl Iterator<Item = (usize, usize, Player)> + '_ {
        (0..self.size).flat_map(move |row| {
            (0..self.size).filter_map(move |col| {
                self.get_player(row, col).map(|player| (row, col, player))
            })
        })
    }

    /// Get combined occupancy bitboard (both players)
    pub fn get_occupancy(&self) -> Vec<u64> {
        self.max_pieces.iter()
            .zip(self.min_pieces.iter())
            .map(|(&max, &min)| max | min)
            .collect()
    }

    /// Get player-specific bitboard
    pub fn get_player_bitboard(&self, player: Player) -> &Vec<u64> {
        match player {
            Player::Max => &self.max_pieces,
            Player::Min => &self.min_pieces,
        }
    }

    /// Check if any moves are available (fast check using bitwise NOT)
    pub fn has_empty_positions(&self) -> bool {
        // Optimized: early return on first empty position found
        for i in 0..self.chunks {
            if (self.max_pieces[i] | self.min_pieces[i]) != self.full_mask[i] {
                return true;
            }
        }
        false
    }

    /// Fast check if a specific player has any stones
    pub fn has_player_stones(&self, player: Player) -> bool {
        match player {
            Player::Max => {
                for &chunk in &self.max_pieces {
                    if chunk != 0 {
                        return true;
                    }
                }
                false
            },
            Player::Min => {
                for &chunk in &self.min_pieces {
                    if chunk != 0 {
                        return true;
                    }
                }
                false
            },
        }
    }

    /// Get the number of empty positions using bit operations
    pub fn count_empty_positions(&self) -> u32 {
        let total_positions = (self.size * self.size) as u32;
        let occupied: u32 = self.max_pieces.iter()
            .zip(self.min_pieces.iter())
            .map(|(&max, &min)| (max | min).count_ones())
            .sum();
        total_positions - occupied
    }
}
