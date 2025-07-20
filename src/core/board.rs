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
    pub max_bits: Vec<u64>,
    pub min_bits: Vec<u64>,
    pub size: usize,
    pub u64_count: usize,
    pub total_cells: usize,
}

impl Board {
    pub fn new(size: usize) -> Self {
        let total_cells = size * size;
        let u64_count = ((total_cells + 63) / 64) as usize;
        Board {
            max_bits: vec![0u64; u64_count],
            min_bits: vec![0u64; u64_count],
            size,
            u64_count,
            total_cells,
        }
    }

    fn index(&self, row: usize, col: usize) -> usize {
        row * self.size + col
    }

    fn set_bit(bits: &mut Vec<u64>, idx: usize) {
        let array_idx = idx / 64;
        let bit_idx = (idx % 64) as u32;
        bits[array_idx] |= 1u64 << bit_idx;
    }

    fn clear_bit(bits: &mut Vec<u64>, idx: usize) {
        let array_idx = idx / 64;
        let bit_idx = (idx % 64) as u32;
        bits[array_idx] &= !(1u64 << bit_idx);
    }

    fn is_bit_set(bits: &Vec<u64>, idx: usize) -> bool {
        let array_idx = idx / 64;
        let bit_idx = (idx % 64) as u32;
        (bits[array_idx] & (1u64 << bit_idx)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.max_bits.iter().all(|&b| b == 0) && self.min_bits.iter().all(|&b| b == 0)
    }

    pub fn center(&self) -> (usize, usize) {
        (self.size / 2, self.size / 2)
    }

    pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
        let idx = self.index(row, col);
        !Self::is_bit_set(&self.max_bits, idx) && !Self::is_bit_set(&self.min_bits, idx)
    }

    pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
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
        let idx = self.index(row, col);
        match player {
            Player::Max => Self::set_bit(&mut self.max_bits, idx),
            Player::Min => Self::set_bit(&mut self.min_bits, idx),
        }
    }

    pub fn remove_stone(&mut self, row: usize, col: usize) {
        let idx = self.index(row, col);
        Self::clear_bit(&mut self.max_bits, idx);
        Self::clear_bit(&mut self.min_bits, idx);
    }

    pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
        let directions = [-1isize, 0, 1];

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
                    if self
                        .get_player(new_row as usize, new_col as usize)
                        .is_some()
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for &b in &self.max_bits {
            b.hash(&mut hasher);
        }
        for &b in &self.min_bits {
            b.hash(&mut hasher);
        }
        hasher.finish()
    }
}
