use std::hash::{DefaultHasher, Hash, Hasher};

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

/// Represents the game board and basic board operations
pub struct Board {
    pub cells: Vec<Vec<Option<Player>>>,
    pub size: usize,
}

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            cells: vec![vec![None; size]; size],
            size,
        }
    }

    /// Check if the board is completely empty
    pub fn is_empty(&self) -> bool {
        self.cells
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_none()))
    }

    /// Get the center position of the board
    pub fn center(&self) -> (usize, usize) {
        (self.size / 2, self.size / 2)
    }


    /// Check if a position is empty
    pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
        self.cells[row][col].is_none()
    }

    /// Get the player at a position
    pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
        self.cells[row][col]
    }

    /// Place a stone at a position
    pub fn place_stone(&mut self, row: usize, col: usize, player: Player) {
        self.cells[row][col] = Some(player);
    }

    /// Remove a stone from a position
    pub fn remove_stone(&mut self, row: usize, col: usize) {
        self.cells[row][col] = None;
    }

    /// Check if a move is adjacent to any existing stone
    pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
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
                    if self.cells[new_row as usize][new_col as usize].is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }


    /// Generate a hash for the current board state
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for row in &self.cells {
            for cell in row {
                cell.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}
