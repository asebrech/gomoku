use crate::core::board::{Board, Player};

/// Incremental pattern counts stored in GameState
/// These are updated on make_move/undo_move instead of recalculated every time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncrementalPatternCounts {
    pub max_five_in_row: u8,
    pub max_live_four: u8,
    pub max_half_free_four: u8,
    pub max_dead_four: u8,
    pub max_live_three: u8,
    pub max_half_free_three: u8,
    pub max_dead_three: u8,
    pub max_live_two: u8,
    pub max_half_free_two: u8,
    
    pub min_five_in_row: u8,
    pub min_live_four: u8,
    pub min_half_free_four: u8,
    pub min_dead_four: u8,
    pub min_live_three: u8,
    pub min_half_free_three: u8,
    pub min_dead_three: u8,
    pub min_live_two: u8,
    pub min_half_free_two: u8,
}

impl IncrementalPatternCounts {
    pub const fn new() -> Self {
        Self {
            max_five_in_row: 0,
            max_live_four: 0,
            max_half_free_four: 0,
            max_dead_four: 0,
            max_live_three: 0,
            max_half_free_three: 0,
            max_dead_three: 0,
            max_live_two: 0,
            max_half_free_two: 0,
            
            min_five_in_row: 0,
            min_live_four: 0,
            min_half_free_four: 0,
            min_dead_four: 0,
            min_live_three: 0,
            min_half_free_three: 0,
            min_dead_three: 0,
            min_live_two: 0,
            min_half_free_two: 0,
        }
    }

    /// Update pattern counts incrementally after a move
    /// This only analyzes the affected area around the move
    pub fn update_after_move(&mut self, board: &Board, row: usize, col: usize, player: Player, win_condition: usize) {
        // Clear old counts - in production, we'd do incremental delta updates
        // For now, do a faster local update around the placed stone
        self.update_local_patterns(board, row, col, player, win_condition);
    }

    /// Update patterns only in the vicinity of the last move
    fn update_local_patterns(&mut self, board: &Board, row: usize, col: usize, player: Player, _win_condition: usize) {
        const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
        
        // For each direction, count consecutive stones
        for &(dx, dy) in &DIRECTIONS {
            let count = Self::count_line(board, row, col, dx, dy, player);
            let (left_open, right_open) = Self::check_openness(board, row, col, dx, dy, count, player);
            
            self.update_counts_for_pattern(count, left_open, right_open, player);
        }
    }

    fn count_line(board: &Board, row: usize, col: usize, dx: isize, dy: isize, player: Player) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        let mut count = 1; // Include the current stone

        // Count backwards
        let mut r = row as isize - dx;
        let mut c = col as isize - dy;
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
                r -= dx;
                c -= dy;
            } else {
                break;
            }
        }

        // Count forwards
        let mut r = row as isize + dx;
        let mut c = col as isize + dy;
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
                r += dx;
                c += dy;
            } else {
                break;
            }
        }

        count
    }

    fn check_openness(board: &Board, row: usize, col: usize, dx: isize, dy: isize, length: usize, player: Player) -> (bool, bool) {
        let opponent_bits = match player.opponent() {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        // Check left side
        let mut r = row as isize;
        let mut c = col as isize;
        
        // Find start of pattern
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            let player_bits = match player {
                Player::Max => &board.max_bits,
                Player::Min => &board.min_bits,
            };
            if !Board::is_bit_set(player_bits, idx) {
                break;
            }
            r -= dx;
            c -= dy;
        }

        let left_open = if r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            !Board::is_bit_set(&board.occupied, idx) && !Board::is_bit_set(opponent_bits, idx)
        } else {
            false
        };

        // Check right side
        let r = row as isize + (length as isize - 1) * dx + dx;
        let c = col as isize + (length as isize - 1) * dy + dy;

        let right_open = if r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            !Board::is_bit_set(&board.occupied, idx) && !Board::is_bit_set(opponent_bits, idx)
        } else {
            false
        };

        (left_open, right_open)
    }

    fn update_counts_for_pattern(&mut self, length: usize, left_open: bool, right_open: bool, player: Player) {
        let freedom = if left_open && right_open {
            PatternFreedom::Free
        } else if left_open || right_open {
            PatternFreedom::HalfFree
        } else {
            PatternFreedom::Flanked
        };

        match player {
            Player::Max => match length {
                5.. => self.max_five_in_row = self.max_five_in_row.saturating_add(1),
                4 => match freedom {
                    PatternFreedom::Free => self.max_live_four = self.max_live_four.saturating_add(1),
                    PatternFreedom::HalfFree => self.max_half_free_four = self.max_half_free_four.saturating_add(1),
                    PatternFreedom::Flanked => self.max_dead_four = self.max_dead_four.saturating_add(1),
                },
                3 => match freedom {
                    PatternFreedom::Free => self.max_live_three = self.max_live_three.saturating_add(1),
                    PatternFreedom::HalfFree => self.max_half_free_three = self.max_half_free_three.saturating_add(1),
                    PatternFreedom::Flanked => self.max_dead_three = self.max_dead_three.saturating_add(1),
                },
                2 => match freedom {
                    PatternFreedom::Free => self.max_live_two = self.max_live_two.saturating_add(1),
                    PatternFreedom::HalfFree => self.max_half_free_two = self.max_half_free_two.saturating_add(1),
                    PatternFreedom::Flanked => {},
                },
                _ => {}
            },
            Player::Min => match length {
                5.. => self.min_five_in_row = self.min_five_in_row.saturating_add(1),
                4 => match freedom {
                    PatternFreedom::Free => self.min_live_four = self.min_live_four.saturating_add(1),
                    PatternFreedom::HalfFree => self.min_half_free_four = self.min_half_free_four.saturating_add(1),
                    PatternFreedom::Flanked => self.min_dead_four = self.min_dead_four.saturating_add(1),
                },
                3 => match freedom {
                    PatternFreedom::Free => self.min_live_three = self.min_live_three.saturating_add(1),
                    PatternFreedom::HalfFree => self.min_half_free_three = self.min_half_free_three.saturating_add(1),
                    PatternFreedom::Flanked => self.min_dead_three = self.min_dead_three.saturating_add(1),
                },
                2 => match freedom {
                    PatternFreedom::Free => self.min_live_two = self.min_live_two.saturating_add(1),
                    PatternFreedom::HalfFree => self.min_half_free_two = self.min_half_free_two.saturating_add(1),
                    PatternFreedom::Flanked => {},
                },
                _ => {}
            },
        }
    }

    /// Recompute all patterns from scratch (used after undo or when counts seem off)
    pub fn recompute_full(&mut self, board: &Board, win_condition: usize) {
        // Reset all counts
        *self = Self::new();

        // Scan all occupied positions
        for row in 0..board.size {
            for col in 0..board.size {
                let idx = board.index(row, col);
                if !Board::is_bit_set(&board.occupied, idx) {
                    continue;
                }

                let player = if Board::is_bit_set(&board.max_bits, idx) {
                    Player::Max
                } else {
                    Player::Min
                };

                self.update_local_patterns(board, row, col, player, win_condition);
            }
        }
    }
}

impl Default for IncrementalPatternCounts {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PatternFreedom {
    Free,
    HalfFree,
    Flanked,
}
