//! Double-three forbidden pattern detection.
//!
//! In Gomoku, a "double-three" is a forbidden move that creates two or more
//! free-three patterns simultaneously. A free-three is an alignment of three
//! stones that, if not immediately blocked, allows for an indefendable
//! alignment of four stones (open four).
//!
//! This module implements sophisticated pattern recognition to detect:
//! - Free-three patterns with and without gaps
//! - Multiple simultaneous free-threes (double-three)
//! - Extension possibilities for open four formation

use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};
use crate::core::captures::CaptureHandler;

/// Double-three detection functionality.
pub struct DoubleThreeDetection;

impl DoubleThreeDetection {
    /// Check if a move creates a forbidden double-three pattern.
    ///
    /// A double-three occurs when placing a stone creates two or more free-three
    /// patterns simultaneously. This is typically forbidden in tournament Gomoku
    /// to prevent certain winning strategies from becoming too powerful.
    ///
    /// **Important Exception**: If the move results in capturing opponent stones,
    /// then it is NOT considered forbidden, even if it creates a double-three.
    /// The capture takes precedence over the double-three rule.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row where the stone would be placed
    /// * `col` - Column where the stone would be placed
    /// * `player` - The player making the move
    ///
    /// # Returns
    /// `true` if the move creates a forbidden double-three, `false` otherwise
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        // First check if this move would result in a capture
        // If it captures opponent stones, then double-three rule doesn't apply
        let captures = CaptureHandler::detect_captures(board, row, col, player);
        if !captures.is_empty() {
            return false; // Not forbidden if it captures
        }

        // Only check for double-three if no captures occur
        DIRECTIONS
            .iter()
            .filter(|&&dir| Self::is_free_three_in_direction(board, row, col, player, dir))
            .count()
            >= 2
    }

    /// Check if placing a stone creates a free-three in a specific direction.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position
    /// * `col` - Column position
    /// * `player` - The player
    /// * `direction` - Direction tuple (dr, dc)
    ///
    /// # Returns
    /// `true` if a free-three is formed in this direction
    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        (dr, dc): (isize, isize),
    ) -> bool {
        Self::would_create_free_three_line(board, row, col, player, dr, dc)
    }

    /// Analyze if placing a stone would create a free-three line in a direction.
    ///
    /// This function searches for existing stones in the specified direction and
    /// determines if adding a stone at the given position would create a valid
    /// free-three pattern. It handles patterns with gaps and validates that the
    /// resulting three-stone line can be extended to form an open four.
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `row` - Row position for the potential stone
    /// * `col` - Column position for the potential stone
    /// * `player` - The player making the move
    /// * `dr` - Row direction (-1, 0, or 1)
    /// * `dc` - Column direction (-1, 0, or 1)
    ///
    /// # Returns
    /// `true` if a free-three line would be created
    fn would_create_free_three_line(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> bool {
        // Collect all player stones in this direction within a reasonable distance
        let mut stones_in_line = vec![(row as isize, col as isize)]; // Include the move position
        
        // Search in both directions along the line for stones
        for &direction_multiplier in &[1, -1] {
            let actual_dr = dr * direction_multiplier;
            let actual_dc = dc * direction_multiplier;
            
            for distance in 1..=4 { // Search up to 4 positions away
                let check_row = row as isize + actual_dr * distance;
                let check_col = col as isize + actual_dc * distance;
                
                if !PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                    break;
                }
                
                let idx = board.index(check_row as usize, check_col as usize);
                let player_bits = board.get_player_bits(player);
                
                if Board::is_bit_set(player_bits, idx) {
                    stones_in_line.push((check_row, check_col));
                }
            }
        }
        
        // Need at least 3 stones to form a free-three
        if stones_in_line.len() < 3 {
            return false;
        }
        
        // Sort stones by position in the line direction
        stones_in_line.sort_by_key(|&(r, c)| {
            if dr != 0 { r } else { c }
        });
        
        // Check all possible 3-stone subsequences to see if any forms a free-three
        for i in 0..=stones_in_line.len().saturating_sub(3) {
            let three_stones = &stones_in_line[i..i+3];
            
            // For a free-three, we need the stones to be consecutive OR have exactly one gap
            // and have space to extend to form an open four
            if Self::is_valid_free_three_pattern(board, three_stones, dr, dc) {
                return true;
            }
        }
        
        false
    }

    /// Validate if a 3-stone pattern forms a valid free-three.
    ///
    /// A valid free-three pattern must:
    /// 1. Have stones in a valid arrangement (consecutive or with specific gaps)
    /// 2. Have space to extend on at least one end to form an open four
    ///
    /// Valid patterns include:
    /// - XXX: Three consecutive stones
    /// - XX-X: Two consecutive stones, gap, one stone
    /// - X-XX: One stone, gap, two consecutive stones
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `three_stones` - Array of three stone positions
    /// * `dr` - Row direction
    /// * `dc` - Column direction
    ///
    /// # Returns
    /// `true` if the pattern forms a valid free-three
    fn is_valid_free_three_pattern(
        board: &Board,
        three_stones: &[(isize, isize)],
        dr: isize,
        dc: isize,
    ) -> bool {
        let first_stone = three_stones[0];
        let middle_stone = three_stones[1];
        let last_stone = three_stones[2];
        
        // Check if the stones form a valid pattern (consecutive or with one gap)
        let gap1 = Self::calculate_gap(first_stone, middle_stone, dr, dc);
        let gap2 = Self::calculate_gap(middle_stone, last_stone, dr, dc);
        
        // Valid patterns: 
        // 1. All consecutive (gaps of 1): X-X-X
        // 2. One gap of 1 between any adjacent stones: X-X--X or X--X-X  
        let is_valid_pattern = (gap1 == 1 && gap2 == 1) || // XXX
                              (gap1 == 1 && gap2 == 2) || // XX-X  
                              (gap1 == 2 && gap2 == 1);   // X-XX
        
        if !is_valid_pattern {
            return false;
        }
        
        // Check if we can extend to form a true "open four" (unblockable by opponent)
        // For a free-three to be valid, it must be able to create an open four that
        // the opponent cannot block with a single move
        Self::can_form_open_four(board, three_stones, dr, dc)
    }

    /// Check if a three-stone pattern can form a threatening four.
    ///
    /// A free-three must be able to extend to form a four that creates a winning threat.
    /// This includes both open fours (with both ends free) and closed fours that still
    /// threaten to win. The key is that the opponent cannot prevent the threat formation.
    ///
    /// # Arguments  
    /// * `board` - The game board
    /// * `three_stones` - Array of three stone positions
    /// * `dr` - Row direction
    /// * `dc` - Column direction
    ///
    /// # Returns
    /// `true` if the pattern can form a threatening four
    fn can_form_open_four(
        board: &Board,
        three_stones: &[(isize, isize)],
        dr: isize,
        dc: isize,
    ) -> bool {
        let first_stone = three_stones[0];
        let last_stone = three_stones[2];
        
        // Check extension before the first stone
        let before_row = first_stone.0 - dr;
        let before_col = first_stone.1 - dc;
        if PatternAnalyzer::is_valid_empty(board, before_row, before_col) {
            // Can we extend here to form a threatening four?
            if Self::would_create_threatening_four(board, three_stones, (before_row, before_col), dr, dc) {
                return true;
            }
        }
        
        // Check extension after the last stone
        let after_row = last_stone.0 + dr;
        let after_col = last_stone.1 + dc;
        if PatternAnalyzer::is_valid_empty(board, after_row, after_col) {
            // Can we extend here to form a threatening four?
            if Self::would_create_threatening_four(board, three_stones, (after_row, after_col), dr, dc) {
                return true;
            }
        }
        
        false
    }

    /// Check if adding a fourth stone would create a threatening four.
    ///
    /// A threatening four is one that either:
    /// 1. Is completely open (both ends free) - unblockable
    /// 2. Has one open end that can be extended to win
    /// 3. Forms a pattern that the opponent cannot prevent from becoming winning
    ///
    /// # Arguments
    /// * `board` - The game board
    /// * `three_stones` - The existing three stones
    /// * `fourth_stone` - The potential fourth stone position
    /// * `dr` - Row direction
    /// * `dc` - Column direction
    ///
    /// # Returns
    /// `true` if the four would be threatening
    fn would_create_threatening_four(
        board: &Board,
        three_stones: &[(isize, isize)],
        fourth_stone: (isize, isize),
        dr: isize,
        dc: isize,
    ) -> bool {
        // Create the four-stone line by combining existing stones with the new one
        let mut four_stones = three_stones.to_vec();
        four_stones.push(fourth_stone);
        
        // Sort the stones by position
        four_stones.sort_by_key(|&(r, c)| {
            if dr != 0 { r } else { c }
        });
        
        let first = four_stones[0];
        let last = four_stones[3];
        
        // Check if we can extend to form a five (winning)
        let before_first = (first.0 - dr, first.1 - dc);
        let after_last = (last.0 + dr, last.1 + dc);
        
        let can_extend_before = PatternAnalyzer::is_valid_empty(board, before_first.0, before_first.1);
        let can_extend_after = PatternAnalyzer::is_valid_empty(board, after_last.0, after_last.1);
        
        // A threatening four needs at least one extension possibility
        can_extend_before || can_extend_after
    }

    /// Calculate the gap between two stones in a specific direction.
    ///
    /// # Arguments
    /// * `stone1` - First stone position
    /// * `stone2` - Second stone position
    /// * `dr` - Row direction (used to determine if we measure row or column distance)
    /// * `_dc` - Column direction (not used but kept for consistency)
    ///
    /// # Returns
    /// The absolute distance between the stones in the relevant dimension
    fn calculate_gap(stone1: (isize, isize), stone2: (isize, isize), dr: isize, _dc: isize) -> isize {
        if dr != 0 {
            (stone2.0 - stone1.0).abs()
        } else {
            (stone2.1 - stone1.1).abs()
        }
    }
}