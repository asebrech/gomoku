use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;

pub struct Heuristic;

#[derive(Debug, Clone, Copy)]
struct PatternCounts {
    live_four: u8,
    half_free_four: u8,
    live_three: u8,
    half_free_three: u8,
    live_two: u8,
    half_free_two: u8,
}
impl PatternCounts {
    const fn new() -> Self {
        Self {
            live_four: 0,
            half_free_four: 0,
            live_three: 0,
            half_free_three: 0,
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

pub const WINNING_SCORE: i32 = 1_000_000;
pub const CAPTURE_BONUS_MULTIPLIER: i32 = 15_000;
pub const LIVE_FOUR_SCORE: i32 = 15_000;         // _XXXX_ (guaranteed win next move)
pub const HALF_FREE_FOUR_SCORE: i32 = 3_500;     // _XXXX| or |XXXX_ (one side open)
pub const LIVE_THREE_SCORE: i32 = 500;           // _XXX_ (can become _XXXX_)
pub const HALF_FREE_THREE_SCORE: i32 = 200;      // _XXX| or |XXX_ (one side open)
pub const LIVE_TWO_SCORE: i32 = 50;              // _XX_ (can grow in both directions)
pub const HALF_FREE_TWO_SCORE: i32 = 20;         // _XX| or |XX_ (one side open)

impl Heuristic {
    pub fn evaluate(state: &GameState, _depth: i32) -> i32 {
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => WINNING_SCORE,
                Player::Min => -WINNING_SCORE,
            };
        }
        if state.board.is_full() {
            return 0;
        }
        
        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition);
        let max_score = Self::calculate_pattern_score(max_counts);
        let min_score = Self::calculate_pattern_score(min_counts);
        let capture_bonus = Self::calculate_capture_bonus(state);
        
        max_score - min_score + capture_bonus
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
            PatternAnalyzer::count_total_space(board, pattern_start_row, pattern_start_col, dx, dy, length);
        if total_available_space < win_condition {
            return None;
        }
        let freedom = PatternAnalyzer::analyze_pattern_freedom(
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
            4 => match pattern.freedom {
                PatternFreedom::Free => counts.live_four += 1,
                PatternFreedom::HalfFree => counts.half_free_four += 1,
                PatternFreedom::Flanked => {}
            },
            3 => match pattern.freedom {
                PatternFreedom::Free => counts.live_three += 1,
                PatternFreedom::HalfFree => counts.half_free_three += 1,
                PatternFreedom::Flanked => {}
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
        (counts.live_four as i32) * LIVE_FOUR_SCORE
            + (counts.half_free_four as i32) * HALF_FREE_FOUR_SCORE
            + (counts.live_three as i32) * LIVE_THREE_SCORE
            + (counts.half_free_three as i32) * HALF_FREE_THREE_SCORE
            + (counts.live_two as i32) * LIVE_TWO_SCORE
            + (counts.half_free_two as i32) * HALF_FREE_TWO_SCORE
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
}
