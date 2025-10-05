use crate::ai::precompute::DirectionTables;
use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct Heuristic;

const WINNING_SCORE: i32 = 1_000_000;
const FIVE_IN_ROW_SCORE: i32 = 100_000;
const LIVE_FOUR_SINGLE_SCORE: i32 = 15_000;
const LIVE_FOUR_MULTIPLE_SCORE: i32 = 20_000;
const HALF_FREE_FOUR_SCORE: i32 = 5_000;
const WINNING_THREAT_SCORE: i32 = 10_000;
const DEAD_FOUR_SCORE: i32 = 1_000;
const LIVE_THREE_SCORE: i32 = 500;
const HALF_FREE_THREE_SCORE: i32 = 200;
const DEAD_THREE_SCORE: i32 = 100;
const LIVE_TWO_SCORE: i32 = 50;
const HALF_FREE_TWO_SCORE: i32 = 20;
const CAPTURE_BONUS_MULTIPLIER: i32 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq)]
enum PatternFreedom {
    Free,
    HalfFree,
    Flanked,
}

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

        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition, &state.direction_tables);

        if max_counts.five_in_row > 0 || max_counts.live_four > 1 {
            return WINNING_SCORE + depth;
        }
        if min_counts.five_in_row > 0 || min_counts.live_four > 1 {
            return -WINNING_SCORE - depth;
        }

        let max_score = Self::calculate_pattern_score(max_counts);
        let min_score = Self::calculate_pattern_score(min_counts);
        let capture_bonus = Self::calculate_capture_bonus(state);
        let historical_bonus = Self::calculate_historical_bonus(state);

        max_score - min_score + capture_bonus + historical_bonus
    }

    fn calculate_historical_bonus(state: &GameState) -> i32 {
        state.pattern_analyzer.calculate_historical_bonus(state)
    }

    fn analyze_both_players(board: &Board, win_condition: usize, dir_tables: &DirectionTables) -> (PatternCounts, PatternCounts) {
        let mut max_counts = PatternCounts::new();
        let mut min_counts = PatternCounts::new();
        let total_cells = board.size * board.size;
        let mut analyzed = vec![0u8; total_cells];

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
                
                // Check all 4 directions using precomputed rays
                for direction in 0..4 {
                    let bit_mask = 1u8 << direction;

                    if analyzed[idx] & bit_mask == 0 {
                        if let Some(pattern_info) = Self::analyze_pattern(
                            board,
                            idx,
                            direction,
                            player,
                            win_condition,
                            &mut analyzed,
                            bit_mask,
                            dir_tables,
                        ) {
                            match player {
                                Player::Max => {
                                    Self::update_counts(&mut max_counts, pattern_info)
                                }
                                Player::Min => {
                                    Self::update_counts(&mut min_counts, pattern_info)
                                }
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
        start_idx: usize,
        direction: usize,
        player: Player,
        win_condition: usize,
        analyzed: &mut [u8],
        bit_mask: u8,
        dir_tables: &DirectionTables,
    ) -> Option<PatternInfo> {
        let pattern_start_idx =
            Self::find_pattern_start(board, start_idx, direction, player, dir_tables);

        if analyzed[pattern_start_idx] & bit_mask != 0 {
            return None;
        }

        let length =
            Self::count_consecutive(board, pattern_start_idx, direction, player, dir_tables);

        if length < 2 {
            return None;
        }

        let length = length.min(win_condition);
        
        // Check if this pattern has sufficient space to develop into a winning line
        if !Self::has_sufficient_space(
            board,
            pattern_start_idx,
            direction,
            length,
            player,
            win_condition,
            dir_tables,
        ) {
            // Mark as analyzed but don't score it
            Self::mark_pattern_analyzed(
                pattern_start_idx,
                direction,
                length,
                analyzed,
                bit_mask,
                dir_tables,
            );
            return None;
        }
        
        let freedom =
            Self::analyze_pattern_freedom(board, pattern_start_idx, direction, length, dir_tables);

        Self::mark_pattern_analyzed(
            pattern_start_idx,
            direction,
            length,
            analyzed,
            bit_mask,
            dir_tables,
        );

        Some(PatternInfo { length, freedom })
    }

    fn count_consecutive(
        board: &Board,
        start_idx: usize,
        direction: usize,
        player: Player,
        dir_tables: &DirectionTables,
    ) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        
        let ray = dir_tables.get_ray_forward(start_idx, direction);
        let mut count = 1; // Count the starting position
        
        for &idx in ray {
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
            } else {
                break;
            }
        }
        
        count
    }

    fn find_pattern_start(
        board: &Board,
        idx: usize,
        direction: usize,
        player: Player,
        dir_tables: &DirectionTables,
    ) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        
        let backward_ray = dir_tables.get_ray_backward(idx, direction);
        let mut pattern_start = idx;
        
        for &back_idx in backward_ray {
            if Board::is_bit_set(player_bits, back_idx) {
                pattern_start = back_idx;
            } else {
                break;
            }
        }
        
        pattern_start
    }

    fn analyze_pattern_freedom(
        board: &Board,
        start_idx: usize,
        direction: usize,
        length: usize,
        dir_tables: &DirectionTables,
    ) -> PatternFreedom {
        // Check before the pattern
        let backward_ray = dir_tables.get_ray_backward(start_idx, direction);
        let start_open = if let Some(&before_idx) = backward_ray.first() {
            !Board::is_bit_set(&board.occupied, before_idx)
        } else {
            false
        };

        // Check after the pattern
        let forward_ray = dir_tables.get_ray_forward(start_idx, direction);
        let end_open = if let Some(&after_idx) = forward_ray.get(length - 1) {
            !Board::is_bit_set(&board.occupied, after_idx)
        } else {
            false
        };

        match (start_open, end_open) {
            (true, true) => PatternFreedom::Free,
            (true, false) | (false, true) => PatternFreedom::HalfFree,
            (false, false) => PatternFreedom::Flanked,
        }
    }



    fn has_sufficient_space(
        board: &Board,
        start_idx: usize,
        direction: usize,
        length: usize,
        player: Player,
        win_condition: usize,
        dir_tables: &DirectionTables,
    ) -> bool {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        let opponent_bits = match player {
            Player::Max => &board.min_bits,
            Player::Min => &board.max_bits,
        };

        let mut total_space = length;
        
        // Count backwards
        let backward_ray = dir_tables.get_ray_backward(start_idx, direction);
        let mut backward_space = 0;
        for &idx in backward_ray.iter().take(win_condition) {
            if Board::is_bit_set(opponent_bits, idx) {
                break;
            }
            if !Board::is_bit_set(&board.occupied, idx) || Board::is_bit_set(player_bits, idx) {
                backward_space += 1;
            } else {
                break;
            }
        }
        
        // Count forwards
        let forward_ray = dir_tables.get_ray_forward(start_idx, direction);
        let mut forward_space = 0;
        for &idx in forward_ray.iter().skip(length - 1).take(win_condition) {
            if Board::is_bit_set(opponent_bits, idx) {
                break;
            }
            if !Board::is_bit_set(&board.occupied, idx) || Board::is_bit_set(player_bits, idx) {
                forward_space += 1;
            } else {
                break;
            }
        }
        
        total_space += backward_space + forward_space;
        total_space >= win_condition
    }

    fn mark_pattern_analyzed(
        start_idx: usize,
        direction: usize,
        length: usize,
        analyzed: &mut [u8],
        bit_mask: u8,
        dir_tables: &DirectionTables,
    ) {
        analyzed[start_idx] |= bit_mask;
        let ray = dir_tables.get_ray_forward(start_idx, direction);
        for &idx in ray.iter().take(length - 1) {
            analyzed[idx] |= bit_mask;
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
                PatternFreedom::Flanked => {}, // Don't count flanked twos
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
        (state.max_captures as i32 - state.min_captures as i32) * CAPTURE_BONUS_MULTIPLIER
    }
}
