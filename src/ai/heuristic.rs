//! Heuristic evaluation functions and utilities.
//!
//! This module contains the board pattern analysis and a heuristic evaluator
//! used by search routines to estimate a GameState's value when a terminal
//! state or full search depth hasn't been reached.
//!
//! The evaluator looks for common Gomoku patterns (five-in-a-row, live four,
//! half-free four, live three, etc.) and combines pattern counts with
//! capture information and a lightweight historical bonus to produce a single
//! score for the current position.
//!
//! Relevant concepts and further reading:
//! - Pattern-based heuristics: <https://en.wikipedia.org/wiki/Gomoku>
//! - Common pattern types and their evaluation in board games: <https://www.chessprogramming.org/Threats>
//!
//! Notes on scoring:
//! - Scores are chosen to (a) prefer immediate wins over long-term potential,
//!   (b) make captures meaningful, and (c) keep values within i32 range.
//! - Values are ordinal (relative) rather than absolute probabilities.
//!
//! The rest of the file contains helpers to scan and classify line patterns on
//! the board. These helpers are intentionally small and focused to make the
//! heuristic fast and easy to test.

use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;
use crate::ai::pattern_utils;

pub const WINNING_SCORE: i32 = 1_000_000;
pub const FIVE_IN_ROW_SCORE: i32 = 100_000;
pub const CHECK_PENALTY: i32 = 10_000;
pub const LIVE_FOUR_SINGLE_SCORE: i32 = 15_000;
pub const LIVE_FOUR_MULTIPLE_SCORE: i32 = 20_000;
pub const HALF_FREE_FOUR_SCORE: i32 = 3_500;
pub const WINNING_THREAT_SCORE: i32 = 10_000;
pub const DEAD_FOUR_SCORE: i32 = 400;
pub const LIVE_THREE_SCORE: i32 = 500;
pub const HALF_FREE_THREE_SCORE: i32 = 200;
pub const DEAD_THREE_SCORE: i32 = 50;
pub const LIVE_TWO_SCORE: i32 = 50;
pub const HALF_FREE_TWO_SCORE: i32 = 20;
pub const CAPTURE_BONUS_MULTIPLIER: i32 = 15_000;

pub struct Heuristic;

#[derive(Debug, Clone, Copy)]
struct PatternCounts {
    five_in_row: u8,
    live_four: u8,
    half_free_four: u8,
    dead_four: u8,
    live_three: u8,
    half_free_three: u8,
    dead_three: u8,
    live_two: u8,
    half_free_two: u8,
}

impl PatternCounts {
    const fn new() -> Self {
        Self {
            five_in_row: 0,
            live_four: 0,
            half_free_four: 0,
            dead_four: 0,
            live_three: 0,
            half_free_three: 0,
            dead_three: 0,
            live_two: 0,
            half_free_two: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PatternInfo {
    length: usize,
    freedom: PatternFreedom,
}

impl Heuristic {
    /// Evaluate a GameState and return an integer score.
    ///
    /// Positive values favour Player::Max, negative values favour
    /// Player::Min. The `depth` parameter is used to prefer faster
    /// wins/losses (a common technique: WIN_SCORE +/- depth).
    ///
    /// The evaluator combines:
    /// - Terminal checks (wins/losses/draws)
    /// - Pattern counts (five, live four, live three, ...)
    /// - Capture-based bonuses
    /// - A small historical bias from `PatternHistoryAnalyzer`.
    ///
    /// See also: pattern-based heuristics and Gomoku evaluation notes.
    pub fn evaluate(state: &GameState, depth: i32) -> i32 {
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => WINNING_SCORE + depth,
                Player::Min => -WINNING_SCORE - depth,
            };
        }

        if state.max_captures >= 5 {
            return WINNING_SCORE + depth;
        }
        if state.min_captures >= 5 {
            return -WINNING_SCORE - depth;
        }

        if state.board.is_full() {
            return 0;
        }

        if let Some(player_in_check) = state.player_in_check {
            let base_eval = Self::evaluate_patterns_and_position(state);
            let check_penalty = Self::calculate_check_penalty(state, player_in_check);

            return match player_in_check {
                Player::Max => base_eval - check_penalty,
                Player::Min => base_eval + check_penalty,
            };
        }

        Self::evaluate_patterns_and_position(state)
    }

    fn evaluate_patterns_and_position(state: &GameState) -> i32 {
        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition);

        if max_counts.five_in_row > 0 || max_counts.live_four > 1 {
            return WINNING_SCORE;
        }
        if min_counts.five_in_row > 0 || min_counts.live_four > 1 {
            return -WINNING_SCORE;
        }

        let max_score = Self::calculate_pattern_score(max_counts);
        let min_score = Self::calculate_pattern_score(min_counts);
        let capture_bonus = Self::calculate_capture_bonus(state);
        let historical_bonus = Self::calculate_historical_bonus(state);

        max_score - min_score + capture_bonus + historical_bonus
    }

    fn calculate_historical_bonus(state: &GameState) -> i32 {
        let max_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Max);
        let min_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Min);
        max_bonus - min_bonus
    }

    fn calculate_check_penalty(state: &GameState, player_in_check: Player) -> i32 {
        if let Some(check_pos) = state.check_position {
            let breaking_moves = crate::core::rules::CaptureBreaking::get_breaking_capture_moves(
                &state.board,
                check_pos.0,
                check_pos.1,
                player_in_check,
            );

            let num_escapes = breaking_moves.len().max(1) as f32;
            let escape_factor = 5.0 / num_escapes;
            CHECK_PENALTY + (escape_factor * 10_000.0) as i32
        } else {
            CHECK_PENALTY
        }
    }

    fn analyze_both_players(board: &Board, win_condition: usize) -> (PatternCounts, PatternCounts) {
        let mut max_counts = PatternCounts::new();
        let mut min_counts = PatternCounts::new();
        let mut analyzed = vec![vec![0u8; board.size]; board.size];

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

                for (dir_idx, &(dx, dy)) in DIRECTIONS.iter().enumerate() {
                    let bit_mask = 1u8 << dir_idx;

                    if analyzed[row][col] & bit_mask == 0 {
                        if let Some(pattern_info) = Self::analyze_pattern(
                            board,
                            row,
                            col,
                            dx,
                            dy,
                            player,
                            win_condition,
                            &mut analyzed,
                            bit_mask,
                        ) {
                            match player {
                                Player::Max => Self::update_counts(&mut max_counts, pattern_info),
                                Player::Min => Self::update_counts(&mut min_counts, pattern_info),
                            }
                        }
                    }
                }
            }
        }

        (max_counts, min_counts)
    }

    fn analyze_pattern(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
        analyzed: &mut [Vec<u8>],
        bit_mask: u8,
    ) -> Option<PatternInfo> {
        let (pattern_start_row, pattern_start_col) =
            Self::find_pattern_start(board, start_row, start_col, dx, dy, player);

        if analyzed[pattern_start_row][pattern_start_col] & bit_mask != 0 {
            return None;
        }

        let consecutive_after_start = PatternAnalyzer::count_consecutive(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            player,
        );

        let length = consecutive_after_start + 1;

        if length < 2 {
            return None;
        }

        let length = length.min(win_condition);

        let total_available_space =
            pattern_utils::count_total_space(board, pattern_start_row, pattern_start_col, dx, dy, length);

        if total_available_space < win_condition {
            return None;
        }

        let freedom = pattern_utils::analyze_pattern_freedom(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );

        Self::mark_pattern_analyzed(
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
            analyzed,
            bit_mask,
        );

        Some(PatternInfo { length, freedom })
    }

    fn mark_pattern_analyzed(
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        length: usize,
        analyzed: &mut [Vec<u8>],
        bit_mask: u8,
    ) {
        for i in 0..length {
            let row = (start_row as isize + i as isize * dx) as usize;
            let col = (start_col as isize + i as isize * dy) as usize;
            if row < analyzed.len() && col < analyzed[0].len() {
                analyzed[row][col] |= bit_mask;
            }
        }
    }



    fn update_counts(counts: &mut PatternCounts, pattern: PatternInfo) {
        match pattern.length {
            5 => counts.five_in_row += 1,
            4 => match pattern.freedom {
                PatternFreedom::Free => counts.live_four += 1,
                PatternFreedom::HalfFree => counts.half_free_four += 1,
                PatternFreedom::Flanked => counts.dead_four += 1,
            },
            3 => match pattern.freedom {
                PatternFreedom::Free => counts.live_three += 1,
                PatternFreedom::HalfFree => counts.half_free_three += 1,
                PatternFreedom::Flanked => counts.dead_three += 1,
            },
            2 => match pattern.freedom {
                PatternFreedom::Free => counts.live_two += 1,
                PatternFreedom::HalfFree => counts.half_free_two += 1,
                PatternFreedom::Flanked => {}
            },
            _ => {}
        }
    }

    fn calculate_pattern_score(counts: PatternCounts) -> i32 {
        let mut score = 0;

        if counts.five_in_row > 0 {
            score += FIVE_IN_ROW_SCORE;
        }

        score += match counts.live_four {
            1 => LIVE_FOUR_SINGLE_SCORE,
            n if n > 1 => LIVE_FOUR_MULTIPLE_SCORE,
            _ => 0,
        };

        if counts.live_three >= 2
            || counts.dead_four >= 2
            || (counts.dead_four >= 1 && counts.live_three >= 1)
            || (counts.half_free_four >= 1 && counts.live_three >= 1)
            || (counts.half_free_four >= 2)
        {
            score += WINNING_THREAT_SCORE;
        }

        score += (counts.half_free_four as i32) * HALF_FREE_FOUR_SCORE
            + (counts.dead_four as i32) * DEAD_FOUR_SCORE
            + (counts.live_three as i32) * LIVE_THREE_SCORE
            + (counts.half_free_three as i32) * HALF_FREE_THREE_SCORE
            + (counts.dead_three as i32) * DEAD_THREE_SCORE
            + (counts.live_two as i32) * LIVE_TWO_SCORE
            + (counts.half_free_two as i32) * HALF_FREE_TWO_SCORE;

        score
    }

    fn calculate_capture_bonus(state: &GameState) -> i32 {
        let max_bonus = if state.max_captures > 0 {
            (CAPTURE_BONUS_MULTIPLIER as f32 * (state.max_captures as f32).sqrt()) as i32
        } else {
            0
        };

        let min_bonus = if state.min_captures > 0 {
            (CAPTURE_BONUS_MULTIPLIER as f32 * (state.min_captures as f32).sqrt()) as i32
        } else {
            0
        };

        max_bonus - min_bonus
    }

    fn find_pattern_start(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> (usize, usize) {
        let player_bits = board.get_player_bits(player);

        let mut current_row = row as isize;
        let mut current_col = col as isize;

        loop {
            let prev_row = current_row - dx;
            let prev_col = current_col - dy;

            if prev_row >= 0
                && prev_row < board.size as isize
                && prev_col >= 0
                && prev_col < board.size as isize
            {
                let idx = board.index(prev_row as usize, prev_col as usize);
                if Board::is_bit_set(player_bits, idx) {
                    current_row = prev_row;
                    current_col = prev_col;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (current_row as usize, current_col as usize)
    }


}
