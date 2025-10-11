//! Capture detection and execution utilities.
//!
//! Gomoku variants often include capture rules (take two opponent stones when
//! they are flanked). This module provides functions to detect captures that
//! result from a newly placed stone and to remove captured stones from the
//! board. The implementation follows a simple directional scan for the common
//! "2 in a row flanked" capture pattern.

use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer};

pub struct CaptureHandler;

impl CaptureHandler {
    pub fn detect_captures(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
    ) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let opponent = player.opponent();
        let player_bits = board.get_player_bits(player);
        let opponent_bits = board.get_player_bits(opponent);

        for &(dx, dy) in &DIRECTIONS {
            for &multiplier in &[1, -1] {
                let actual_dx = dx as isize * multiplier as isize;
                let actual_dy = dy as isize * multiplier as isize;

                let pos1_x = row as isize + actual_dx;
                let pos1_y = col as isize + actual_dy;
                if !PatternAnalyzer::is_in_bounds(board, pos1_x, pos1_y) {
                    continue;
                }
                let idx1 = board.index(pos1_x as usize, pos1_y as usize);
                if !Board::is_bit_set(opponent_bits, idx1) {
                    continue;
                }

                let pos2_x = pos1_x + actual_dx;
                let pos2_y = pos1_y + actual_dy;
                if !PatternAnalyzer::is_in_bounds(board, pos2_x, pos2_y) {
                    continue;
                }
                let idx2 = board.index(pos2_x as usize, pos2_y as usize);
                if !Board::is_bit_set(opponent_bits, idx2) {
                    continue;
                }

                let pos3_x = pos2_x + actual_dx;
                let pos3_y = pos2_y + actual_dy;
                if !PatternAnalyzer::is_in_bounds(board, pos3_x, pos3_y) {
                    continue;
                }
                let idx3 = board.index(pos3_x as usize, pos3_y as usize);
                if Board::is_bit_set(player_bits, idx3) {
                    captures.push((pos1_x as usize, pos1_y as usize));
                    captures.push((pos2_x as usize, pos2_y as usize));
                }
            }
        }

        captures
    }

    pub fn execute_captures(board: &mut Board, captures: &[(usize, usize)]) {
        for &(r, c) in captures {
            let idx = board.index(r, c);
            Board::clear_bit(&mut board.max_bits, idx);
            Board::clear_bit(&mut board.min_bits, idx);
            Board::clear_bit(&mut board.occupied, idx);
        }
    }
}
