use std::hash::{Hash, Hasher};
use bevy::prelude::*;
use rand::Rng;

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
    pub max_pieces: [u128; 3],  // 3 u128s to handle 19x19 (361 bits)
    pub min_pieces: [u128; 3],  // 3 u128s to handle 19x19 (361 bits)
    pub size: usize,
    pub zobrist_hash: u64,
}

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            max_pieces: [0; 3],
            min_pieces: [0; 3],
            size,
            zobrist_hash: 0,
        }
    }

    #[inline]
    pub fn position_to_bit_index(row: usize, col: usize, size: usize) -> (usize, usize) {
        let linear_pos = row * size + col;
        let chunk_index = linear_pos / 128;
        let bit_index = linear_pos % 128;
        (chunk_index, bit_index)
    }

    #[inline]
    pub fn position_to_bit(row: usize, col: usize, size: usize) -> (usize, u128) {
        let linear_pos = row * size + col;
        let chunk_index = linear_pos / 128;
        let bit_index = linear_pos % 128;
        (chunk_index, 1u128 << bit_index)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.max_pieces.iter().all(|&x| x == 0) && 
        self.min_pieces.iter().all(|&x| x == 0)
    }

    #[inline]
    pub fn center(&self) -> (usize, usize) {
        (self.size / 2, self.size / 2)
    }

    #[inline]
    pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
        let (chunk_idx, bit_idx) = Self::position_to_bit_index(row, col, self.size);
        let bit = 1u128 << bit_idx;
        
        if chunk_idx < 3 {
            (self.max_pieces[chunk_idx] | self.min_pieces[chunk_idx]) & bit == 0
        } else {
            false // Out of bounds
        }
    }

    #[inline]
    pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
        let (chunk_idx, bit_idx) = Self::position_to_bit_index(row, col, self.size);
        if chunk_idx >= 3 {
            return None; // Out of bounds
        }
        
        let bit = 1u128 << bit_idx;
        if self.max_pieces[chunk_idx] & bit != 0 {
            Some(Player::Max)
        } else if self.min_pieces[chunk_idx] & bit != 0 {
            Some(Player::Min)
        } else {
            None
        }
    }

    #[inline]
    pub fn place_stone(&mut self, row: usize, col: usize, player: Player) {
        let (chunk_idx, bit_idx) = Self::position_to_bit_index(row, col, self.size);
        if chunk_idx >= 3 {
            return; // Out of bounds
        }
        
        let bit = 1u128 << bit_idx;
        match player {
            Player::Max => self.max_pieces[chunk_idx] |= bit,
            Player::Min => self.min_pieces[chunk_idx] |= bit,
        }
        
        unsafe {
            if ZOBRIST_INITIALIZED {
                self.zobrist_hash ^= ZOBRIST_TABLE[row * self.size + col][player as usize];
            }
        }
    }

    #[inline]
    pub fn remove_stone(&mut self, row: usize, col: usize) {
        let (chunk_idx, bit_idx) = Self::position_to_bit_index(row, col, self.size);
        if chunk_idx >= 3 {
            return; // Out of bounds
        }
        
        let player = self.get_player(row, col);
        if let Some(p) = player {
            let bit = 1u128 << bit_idx;
            match p {
                Player::Max => self.max_pieces[chunk_idx] &= !bit,
                Player::Min => self.min_pieces[chunk_idx] &= !bit,
            }
            
            unsafe {
                if ZOBRIST_INITIALIZED {
                    self.zobrist_hash ^= ZOBRIST_TABLE[row * self.size + col][p as usize];
                }
            }
        }
    }

    #[inline]
    pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        for &(dr, dc) in &directions {
            let new_row = row as isize + dr;
            let new_col = col as isize + dc;
            
            if new_row >= 0 && new_row < self.size as isize &&
               new_col >= 0 && new_col < self.size as isize {
                if self.get_player(new_row as usize, new_col as usize).is_some() {
                    return true;
                }
            }
        }
        false
    }

    pub fn hash(&self) -> u64 {
        self.zobrist_hash
    }

    // Helper method to check if position is in bounds
    #[inline]
    pub fn is_valid_position(&self, row: usize, col: usize) -> bool {
        row < self.size && col < self.size
    }
}

// Zobrist hashing tables
static mut ZOBRIST_TABLE: [[u64; 2]; 361] = [[0; 2]; 361]; // 19x19 max
static mut ZOBRIST_INITIALIZED: bool = false;

pub fn initialize_zobrist() {
    unsafe {
        if !ZOBRIST_INITIALIZED {
            let mut rng = rand::rng();
            
            for i in 0..361 {
                for j in 0..2 {
                    ZOBRIST_TABLE[i][j] = rng.random();
                }
            }
            ZOBRIST_INITIALIZED = true;
        }
    }
}
