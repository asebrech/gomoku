use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;
use crate::core::captures::CaptureHandler;

pub struct Heuristic;

pub const WINNING_SCORE: i32 = 1_000_000;
pub const CAPTURE_VULNERABILITY_BASE: i32 = 3_000;  // Reduced from 8,000 - tactics should take priority
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
        
        // Check if either player has a strong tactical position (live four or multiple threats)
        // If so, reduce capture vulnerability penalty as tactics take priority
        let has_strong_tactics = max_pattern_score >= LIVE_FOUR_SCORE || min_pattern_score >= LIVE_FOUR_SCORE;
        
        let capture_vulnerability_penalty = if has_strong_tactics {
            // Reduce penalty significantly when there are strong tactical threats
            Self::evaluate_capture_vulnerability(state) / 4
        } else {
            Self::evaluate_capture_vulnerability(state)
        };
        
        max_pattern_score - min_pattern_score + capture_bonus - capture_vulnerability_penalty
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
        
        let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
        let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
        
        analyzed[row][col] |= bit_mask;
        
        for dist in 1..=backward {
            let r = (row as isize - dx * dist as isize) as usize;
            let c = (col as isize - dy * dist as isize) as usize;
            analyzed[r][c] |= bit_mask;
        }
        
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
        let stones = PatternAnalyzer::collect_gapped_stones(board, row, col, dx, dy, player, 6);
        
        let Some((stone_count, gaps, _span)) = PatternAnalyzer::analyze_gapped_pattern(&stones) else {
            return 0;
        };
        
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
    
    /// Evaluates vulnerability to captures based on:
    /// 1. How many capturable pairs the current player has on the board
    /// 2. How close the opponent is to winning by capture
    /// 3. Whether opponent can capture and win immediately
    /// Returns a penalty score (higher = more vulnerable)
    fn evaluate_capture_vulnerability(state: &GameState) -> i32 {
        let current_player = state.current_player;
        let opponent = current_player.opponent();
        
        let opponent_captures = match opponent {
            Player::Max => state.max_captures,
            Player::Min => state.min_captures,
        };
        
        // Count how many capturable pairs the current player has
        let capturable_pairs = Self::count_capturable_pairs(&state.board, current_player, opponent);
        
        if capturable_pairs == 0 {
            return 0;
        }
        
        // Check if opponent can win by capture immediately
        let can_win_immediately = opponent_captures >= state.capture_to_win - 1 &&
            Self::opponent_can_capture_and_win(state, opponent);
        
        if can_win_immediately {
            // Critical: opponent can win by capture next move - huge penalty
            return 900_000;
        }
        
        // Calculate graduated penalty based on:
        // - Number of capturable pairs (more pairs = more vulnerable)
        // - Opponent's capture progress (closer to winning = more dangerous)
        let progress_multiplier = if opponent_captures >= state.capture_to_win - 2 {
            5  // Very close to winning
        } else if opponent_captures >= state.capture_to_win - 3 {
            3  // Getting close
        } else if opponent_captures >= state.capture_to_win / 2 {
            2  // Halfway there
        } else {
            1  // Early game
        };
        
        CAPTURE_VULNERABILITY_BASE * capturable_pairs as i32 * progress_multiplier
    }
    
    /// Counts how many capturable pair patterns exist for the given player
    fn count_capturable_pairs(board: &Board, player: Player, opponent: Player) -> usize {
        let mut count = 0;
        let player_bits = board.get_player_bits(player);
        
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                // Check if this stone is part of a capturable pair
                // Pattern: O X X _ (opponent can place at _ to capture)
                let next_r = row as isize + dx;
                let next_c = col as isize + dy;
                
                if !PatternAnalyzer::is_in_bounds(board, next_r, next_c) {
                    continue;
                }
                
                let next_idx = board.index(next_r as usize, next_c as usize);
                if !Board::is_bit_set(player_bits, next_idx) {
                    continue;
                }
                
                // Found two consecutive stones - check both ends
                let before_r = row as isize - dx;
                let before_c = col as isize - dy;
                let after_r = next_r + dx;
                let after_c = next_c + dy;
                
                let before_vulnerable = PatternAnalyzer::is_in_bounds(board, before_r, before_c) &&
                    board.get_player(before_r as usize, before_c as usize) == Some(opponent);
                    
                let after_vulnerable = PatternAnalyzer::is_in_bounds(board, after_r, after_c) &&
                    board.get_player(after_r as usize, after_c as usize) == Some(opponent);
                
                if before_vulnerable || after_vulnerable {
                    count += 1;
                }
            }
        });
        
        // Divide by 2 since each pair is counted twice
        count / 2
    }
    
    /// Checks if opponent can capture and win on their next move
    fn opponent_can_capture_and_win(state: &GameState, opponent: Player) -> bool {
        let opponent_captures = match opponent {
            Player::Max => state.max_captures,
            Player::Min => state.min_captures,
        };
        
        for row in 0..state.board.size {
            for col in 0..state.board.size {
                if state.board.get_player(row, col).is_some() {
                    continue;
                }
                
                let mut temp_board = state.board.clone();
                let idx = temp_board.index(row, col);
                
                Board::set_bit(&mut temp_board.occupied, idx);
                match opponent {
                    Player::Max => Board::set_bit(&mut temp_board.max_bits, idx),
                    Player::Min => Board::set_bit(&mut temp_board.min_bits, idx),
                }
                
                let captures = CaptureHandler::detect_captures(&temp_board, row, col, opponent);
                
                if !captures.is_empty() {
                    let capture_pairs = captures.len() / 2;
                    if opponent_captures + capture_pairs >= state.capture_to_win {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}

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
