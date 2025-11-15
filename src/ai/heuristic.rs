use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;

pub struct Heuristic;

// Scoring constants
pub const WINNING_SCORE: i32 = 1_000_000;
pub const CAPTURE_BONUS_MULTIPLIER: i32 = 15_000;
pub const LIVE_FOUR_SCORE: i32 = 15_000;         // _XXXX_ (guaranteed win next move)
pub const HALF_FREE_FOUR_SCORE: i32 = 3_500;     // _XXXX| or |XXXX_ (one side open)
pub const LIVE_THREE_SCORE: i32 = 500;           // _XXX_ (can become _XXXX_)
pub const HALF_FREE_THREE_SCORE: i32 = 200;      // _XXX| or |XXX_ (one side open)
pub const LIVE_TWO_SCORE: i32 = 50;              // _XX_ (can grow in both directions)
pub const HALF_FREE_TWO_SCORE: i32 = 20;         // _XX| or |XX_ (one side open)

pub const GAPPED_FOUR_SCORE: i32 = 400;          // XXXX_X or X_XXXX (4 stones, 1 gap)
pub const GAPPED_THREE_ONE_SCORE: i32 = 150;     // XXX_X or X_XXX (3 stones, 1 gap)
pub const GAPPED_THREE_TWO_SCORE: i32 = 100;     // XX_X_X or X_X_X (3 stones, 2 gaps)
pub const GAPPED_TWO_ONE_SCORE: i32 = 25;        // XX_X or X_X (2 stones, 1 gap)
pub const GAPPED_TWO_TWO_SCORE: i32 = 15;        // X__XX or X__X (2 stones, 2 gaps)
pub const GAPPED_OTHER_SCORE: i32 = 5;           // Other gapped combinations

/// Score a consecutive pattern based on its length and freedom
#[inline]
pub fn score_consecutive_pattern(length: usize, freedom: PatternFreedom) -> i32 {
    match length {
        5 => WINNING_SCORE,
        4 => match freedom {
            PatternFreedom::Free => LIVE_FOUR_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_FOUR_SCORE,
            PatternFreedom::Flanked => 0,
        },
        3 => match freedom {
            PatternFreedom::Free => LIVE_THREE_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_THREE_SCORE,
            PatternFreedom::Flanked => 0,
        },
        2 => match freedom {
            PatternFreedom::Free => LIVE_TWO_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_TWO_SCORE,
            PatternFreedom::Flanked => 0,
        },
        _ => 0,
    }
}

/// Score a gapped pattern based on stone count and gap count
#[inline]
pub fn score_gapped_pattern(stones: usize, gaps: usize) -> i32 {
    match (stones, gaps) {
        (4, 1) => GAPPED_FOUR_SCORE,
        (3, 1) => GAPPED_THREE_ONE_SCORE,
        (3, 2) => GAPPED_THREE_TWO_SCORE,
        (2, 1) => GAPPED_TWO_ONE_SCORE,
        (2, 2) => GAPPED_TWO_TWO_SCORE,
        _ => GAPPED_OTHER_SCORE,
    }
}

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
        
        let (max_pattern_score, min_pattern_score) =
            Self::analyze_all_patterns(&state.board, state.win_condition);
        
        let capture_bonus = Self::calculate_capture_bonus(state);
        
        max_pattern_score - min_pattern_score + capture_bonus
    }

    fn analyze_all_patterns(board: &Board, win_condition: usize) -> (i32, i32) {
        let mut max_score = 0;
        let mut min_score = 0;
        let mut consecutive_analyzed = vec![vec![0u8; board.size]; board.size];
        let mut gap_analyzed = vec![vec![0u8; board.size]; board.size];
        
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
                    
                    if consecutive_analyzed[row][col] & bit_mask == 0 {
                        let bonus = Self::analyze_consecutive_pattern(
                            board,
                            row,
                            col,
                            dx,
                            dy,
                            player,
                            win_condition,
                            &mut consecutive_analyzed,
                            bit_mask,
                        );
                        match player {
                            Player::Max => max_score += bonus,
                            Player::Min => min_score += bonus,
                        }
                    }
                    
                    if gap_analyzed[row][col] & bit_mask == 0 {
                        let bonus = Self::analyze_gapped_pattern(
                            board,
                            row,
                            col,
                            dx,
                            dy,
                            player,
                            &mut gap_analyzed,
                            bit_mask,
                        );
                        match player {
                            Player::Max => max_score += bonus,
                            Player::Min => min_score += bonus,
                        }
                    }
                }
            }
        }
        (max_score, min_score)
    }
    fn analyze_consecutive_pattern(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
        analyzed: &mut [Vec<u8>],
        bit_mask: u8,
    ) -> i32 {
        // Use the unified pattern analyzer
        let pattern_info = PatternAnalyzer::analyze_consecutive_from_position(
            board,
            row,
            col,
            dx,
            dy,
            player,
            win_condition,
        );
        
        let Some((length, _pattern_start_row, _pattern_start_col, _total_space, freedom)) = pattern_info else {
            return 0;
        };
        
        // Mark all stones in the pattern as analyzed
        let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
        let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
        
        // Mark current stone
        analyzed[row][col] |= bit_mask;
        
        // Mark backward stones
        for dist in 1..=backward {
            let r = (row as isize - dx * dist as isize) as usize;
            let c = (col as isize - dy * dist as isize) as usize;
            analyzed[r][c] |= bit_mask;
        }
        
        // Mark forward stones
        for dist in 1..=forward {
            let r = (row as isize + dx * dist as isize) as usize;
            let c = (col as isize + dy * dist as isize) as usize;
            analyzed[r][c] |= bit_mask;
        }
        
        score_consecutive_pattern(length.min(win_condition), freedom)
    }

    fn analyze_gapped_pattern(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        analyzed: &mut [Vec<u8>],
        bit_mask: u8,
    ) -> i32 {
        // Use unified gapped stone collection
        let stones = PatternAnalyzer::collect_gapped_stones(board, row, col, dx, dy, player, 6);
        
        // Use unified gapped pattern analysis
        let Some((stone_count, gaps, _span)) = PatternAnalyzer::analyze_gapped_pattern(&stones) else {
            return 0;
        };
        
        // Mark all stones as analyzed
        for &(r, c) in &stones {
            analyzed[r][c] |= bit_mask;
        }
        
        score_gapped_pattern(stone_count, gaps)
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
