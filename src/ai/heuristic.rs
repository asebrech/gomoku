use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;

pub struct Heuristic;

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
        let player_bits = board.get_player_bits(player);
        let mut stones = vec![(row, col)];
        
        for dist in 1..=win_condition {
            let check_row = row as isize + dx * dist as isize;
            let check_col = col as isize + dy * dist as isize;
            if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    stones.push((check_row as usize, check_col as usize));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        let mut backward_stones = Vec::new();
        for dist in 1..=win_condition {
            let check_row = row as isize - dx * dist as isize;
            let check_col = col as isize - dy * dist as isize;
            if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    backward_stones.push((check_row as usize, check_col as usize));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        backward_stones.reverse();
        backward_stones.append(&mut stones);
        stones = backward_stones;
        
        let length = stones.len();
        if length < 2 {
            return 0;
        }
        
        let first = stones.first().unwrap();
        let pattern_start_row = first.0;
        let pattern_start_col = first.1;
        
        let length = length.min(win_condition);
        let total_available_space = PatternAnalyzer::count_total_space(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );
        if total_available_space < win_condition {
            return 0;
        }
        
        let freedom = PatternAnalyzer::analyze_pattern_freedom(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
        );
        
        for &(r, c) in &stones {
            analyzed[r][c] |= bit_mask;
        }
        
        Self::score_consecutive_pattern(length, freedom)
    }
    fn score_consecutive_pattern(length: usize, freedom: PatternFreedom) -> i32 {
        match length {
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
        let player_bits = board.get_player_bits(player);
        let mut stones = vec![(row, col)];
        
        for dist in 1..=6 {
            let check_row = row as isize + dx * dist;
            let check_col = col as isize + dy * dist;
            if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    stones.push((check_row as usize, check_col as usize));
                } else if Board::is_bit_set(&board.occupied, idx) {
                    break;
                }
            } else {
                break;
            }
        }
        
        let mut backward_stones = Vec::new();
        for dist in 1..=6 {
            let check_row = row as isize - dx * dist;
            let check_col = col as isize - dy * dist;
            if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(&player_bits, idx) {
                    backward_stones.push((check_row as usize, check_col as usize));
                } else if Board::is_bit_set(&board.occupied, idx) {
                    break;
                }
            } else {
                break;
            }
        }
        backward_stones.reverse();
        backward_stones.append(&mut stones);
        stones = backward_stones;
        
        if stones.len() >= 2 {
            let first = stones.first().unwrap();
            let last = stones.last().unwrap();
            let span = ((last.0 as isize - first.0 as isize).abs()
                .max((last.1 as isize - first.1 as isize).abs())) + 1;
            
            let gaps = (span as usize) - stones.len();
            
            if gaps > 0 && span <= 7 && stones.len() + gaps >= 4 && gaps <= 3 {
                for &(r, c) in &stones {
                    analyzed[r][c] |= bit_mask;
                }
                return Self::score_gapped_pattern(stones.len(), gaps);
            }
        }
        
        0
    }

    fn score_gapped_pattern(stones: usize, gaps: usize) -> i32 {
        match (stones, gaps) {
            (4, 1) => GAPPED_FOUR_SCORE,
            (3, 1) => GAPPED_THREE_ONE_SCORE,
            (3, 2) => GAPPED_THREE_TWO_SCORE,
            (2, 1) => GAPPED_TWO_ONE_SCORE,
            (2, 2) => GAPPED_TWO_TWO_SCORE,
            _ => GAPPED_OTHER_SCORE,
        }
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
