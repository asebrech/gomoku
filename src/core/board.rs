use std::hash::Hash;

use bevy::prelude::*;
use crate::core::patterns::{PatternAnalyzer, ALL_DIRECTIONS};

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
    pub max_bits: Vec<u64>,
    pub min_bits: Vec<u64>,
    pub occupied: Vec<u64>,
    pub size: usize,
    pub u64_count: usize,
    pub total_cells: usize,
}

impl Board {
    pub fn new(size: usize) -> Self {
        let total_cells = size * size;
        let u64_count = (total_cells + 63) / 64;
        Board {
            max_bits: vec![0u64; u64_count],
            min_bits: vec![0u64; u64_count],
            occupied: vec![0u64; u64_count],
            size,
            u64_count,
            total_cells,
        }
    }

    pub fn index(&self, row: usize, col: usize) -> usize {
        row * self.size + col
    }

    pub fn set_bit(bits: &mut [u64], idx: usize) {
        let array_idx = idx / 64;
        if array_idx >= bits.len() {
            return;
        }
        let bit_idx = (idx % 64) as u32;
        bits[array_idx] |= 1u64 << bit_idx;
    }

    pub fn clear_bit(bits: &mut [u64], idx: usize) {
        let array_idx = idx / 64;
        if array_idx >= bits.len() {
            return;
        }
        let bit_idx = (idx % 64) as u32;
        bits[array_idx] &= !(1u64 << bit_idx);
    }

    pub fn is_bit_set(bits: &[u64], idx: usize) -> bool {
        let array_idx = idx / 64;
        if array_idx >= bits.len() {
            return false;
        }
        let bit_idx = (idx % 64) as u32;
        (bits[array_idx] & (1u64 << bit_idx)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.occupied.iter().all(|&b| b == 0)
    }

    pub fn count_stones(&self) -> usize {
        self.occupied.iter().map(|&bits| bits.count_ones() as usize).sum()
    }

    pub fn count_player_stones(&self, player: Player) -> usize {
        let bits = self.get_player_bits(player);
        bits.iter().map(|&b| b.count_ones() as usize).sum()
    }

    /// Get the bit array for a specific player
    #[inline]
    pub fn get_player_bits(&self, player: Player) -> &Vec<u64> {
        match player {
            Player::Max => &self.max_bits,
            Player::Min => &self.min_bits,
        }
    }

    pub fn center(&self) -> (usize, usize) {
        (self.size / 2, self.size / 2)
    }

    pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
        if row >= self.size || col >= self.size {
            return false;
        }
        let idx = self.index(row, col);
        !Self::is_bit_set(&self.occupied, idx)
    }

    pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
        if row >= self.size || col >= self.size {
            return None;
        }
        let idx = self.index(row, col);
        if Self::is_bit_set(&self.max_bits, idx) {
            Some(Player::Max)
        } else if Self::is_bit_set(&self.min_bits, idx) {
            Some(Player::Min)
        } else {
            None
        }
    }

    pub fn place_stone(&mut self, row: usize, col: usize, player: Player) {
        if row >= self.size || col >= self.size {
            return;
        }
        let idx = self.index(row, col);
        match player {
            Player::Max => Self::set_bit(&mut self.max_bits, idx),
            Player::Min => Self::set_bit(&mut self.min_bits, idx),
        }
        Self::set_bit(&mut self.occupied, idx);
    }

    pub fn remove_stone(&mut self, row: usize, col: usize) {
        if row >= self.size || col >= self.size {
            return;
        }
        let idx = self.index(row, col);
        Self::clear_bit(&mut self.max_bits, idx);
        Self::clear_bit(&mut self.min_bits, idx);
        Self::clear_bit(&mut self.occupied, idx);
    }

    pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
        if row >= self.size || col >= self.size {
            return false;
        }
        
        for &(dr, dc) in &ALL_DIRECTIONS {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if PatternAnalyzer::is_in_bounds(self, nr, nc) {
                let idx = self.index(nr as usize, nc as usize);
                if Self::is_bit_set(&self.occupied, idx) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_full(&self) -> bool {
        let mut total_set_bits = 0;

        for (i, &bits) in self.occupied.iter().enumerate() {
            if i == self.u64_count - 1 {
                let bits_in_last = self.total_cells % 64;
                let mask = if bits_in_last != 0 {
                    (1u64 << bits_in_last) - 1
                } else {
                    u64::MAX
                };
                total_set_bits += (bits & mask).count_ones() as usize;
            } else {
                total_set_bits += bits.count_ones() as usize;
            }
        }

        total_set_bits == self.total_cells
    }

    pub fn get_empty_positions(&self) -> Vec<(usize, usize)> {
        let mut empties = Vec::new();
        for array_idx in 0..self.u64_count {
            let mut bits = !self.occupied[array_idx];
            if array_idx == self.u64_count - 1 {
                let bits_in_last = self.total_cells % 64;
                if bits_in_last != 0 {
                    bits &= (1u64 << bits_in_last) - 1;
                }
            }
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < self.total_cells {
                    let row = global_idx / self.size;
                    let col = global_idx % self.size;
                    empties.push((row, col));
                }
                bits &= bits - 1;
            }
        }
        empties
    }

    pub fn get_occupied_positions(&self) -> Vec<((usize, usize), Player)> {
        let mut positions = Vec::new();
        for array_idx in 0..self.u64_count {
            let mut occupied_bits = self.occupied[array_idx];
            let max_bits = self.max_bits[array_idx];
            
            while occupied_bits != 0 {
                let bit_pos = occupied_bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < self.total_cells {
                    let row = global_idx / self.size;
                    let col = global_idx % self.size;
                    let player = if (max_bits & (1u64 << bit_pos)) != 0 {
                        Player::Max
                    } else {
                        Player::Min
                    };
                    positions.push(((row, col), player));
                }
                occupied_bits &= occupied_bits - 1;
            }
        }
        positions
    }
}
