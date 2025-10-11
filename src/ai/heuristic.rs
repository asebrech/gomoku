use crate::core::board::{Board, Player};
use crate::core::state::GameState;
use crate::core::patterns::{PatternAnalyzer, PatternFreedom, DIRECTIONS};

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
            Self::analyze_both_players(&state.board, state.win_condition);

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
        let max_bonus = state.pattern_analyzer.calculate_historical_bonus(Player::Max);
        let min_bonus = state.pattern_analyzer.calculate_historical_bonus(Player::Min);
        max_bonus - min_bonus
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
            PatternAnalyzer::find_pattern_start(board, start_row, start_col, dx, dy, player);

        if analyzed[pattern_start_row][pattern_start_col] & bit_mask != 0 {
            return None;
        }

        let consecutive_after_start =
            PatternAnalyzer::count_consecutive(board, pattern_start_row, pattern_start_col, dx, dy, player);

        let length = consecutive_after_start + 1;

        if length < 2 {
            return None;
        }

        let length = length.min(win_condition);
        
        let total_available_space = Self::count_total_space(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );
        
        if total_available_space < win_condition {
            return None;
        }
        
        let freedom =
            PatternAnalyzer::analyze_pattern_freedom(board, pattern_start_row, pattern_start_col, dx, dy, length);

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

    fn count_total_space(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        pattern_length: usize,
    ) -> usize {
        let mut space = pattern_length;
        
        space += Self::count_empty_in_direction(board, start_row as isize - dx, start_col as isize - dy, -dx, -dy);
        
        let end_row = start_row as isize + (pattern_length - 1) as isize * dx;
        let end_col = start_col as isize + (pattern_length - 1) as isize * dy;
        space += Self::count_empty_in_direction(board, end_row + dx, end_col + dy, dx, dy);
        
        space
    }

    fn count_empty_in_direction(
        board: &Board,
        start_row: isize,
        start_col: isize,
        dx: isize,
        dy: isize,
    ) -> usize {
        let mut count = 0;
        let mut current_row = start_row;
        let mut current_col = start_col;
        
        while PatternAnalyzer::is_in_bounds(board, current_row, current_col) {
            let idx = board.index(current_row as usize, current_col as usize);
            if !Board::is_bit_set(&board.occupied, idx) {
                count += 1;
                current_row += dx;
                current_col += dy;
            } else {
                break;
            }
        }
        
        count
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
                PatternFreedom::Flanked => {},
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
