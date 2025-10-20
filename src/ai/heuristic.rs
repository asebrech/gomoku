use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;
use crate::ai::config::AIConfig;

/// Gomoku position evaluation using pattern analysis and tactical bonuses
pub struct Heuristic;

/// Counts of different pattern types for a player
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
    /// Evaluates a game position returning a score from the Max player's perspective
    /// 
    /// Positive scores favor Max player, negative scores favor Min player.
    /// Higher depth bonus for quicker wins/losses.
    /// 
    /// Evaluation components:
    /// - Terminal positions (wins detected by game state, draw by full board)
    /// - Pattern analysis (stone formations, threats, winning combinations)
    /// - Capture bonuses (scaled with square root for diminishing returns)
    /// - Historical pattern bonuses (momentum and initiative tracking)
    /// - Check penalties (vulnerability to capture, escape difficulty scaling)
    /// 
    /// The function performs comprehensive pattern analysis for both players,
    /// detecting immediate wins, threatening combinations, and positional advantages.
    /// All evaluation components are combined into a single unified score.
    pub fn evaluate(state: &GameState, depth: i32, ai_config: &AIConfig) -> i32 {
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => ai_config.heuristic.scores.winning_score + depth,
                Player::Min => -ai_config.heuristic.scores.winning_score - depth,
            };
        }
        if state.board.is_full() {
            return 0;
        }
        
        // Pattern analysis and position evaluation
        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition);
        let max_score = Self::calculate_pattern_score(max_counts, ai_config);
        let min_score = Self::calculate_pattern_score(min_counts, ai_config);
        let capture_bonus = Self::calculate_capture_bonus(state, ai_config);
        let historical_bonus = Self::calculate_historical_bonus(state, ai_config);
        let check_penalty = Self::calculate_check_penalty(state, ai_config);
        
        max_score - min_score + capture_bonus + historical_bonus + check_penalty
    }

    /// Calculates the historical pattern bonus differential between players.
    /// 
    /// This function evaluates patterns that have been historically successful
    /// for each player, providing a bonus based on pattern recognition and learning.
    /// The historical bonus helps the AI prefer moves that have led to successful
    /// positions in the past, improving play over time through pattern memory.
    /// 
    /// Returns the difference between Max player's historical bonus and Min player's
    /// historical bonus, allowing the AI to prefer moves that align with successful
    /// historical patterns while avoiding patterns that have led to losses.
    fn calculate_historical_bonus(state: &GameState, ai_config: &AIConfig) -> i32 {
        let max_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Max, ai_config);
        let min_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Min, ai_config);
        max_bonus - min_bonus
    }
    /// Calculates penalty for being in check (captured position vulnerability).
    /// 
    /// When a player is in check (their pieces can be captured), this function
    /// applies a penalty that scales with the difficulty of escaping the check.
    /// The penalty increases when there are fewer available escape moves, making
    /// positions with limited escape options heavily penalized.
    /// 
    /// The base CHECK_PENALTY is applied, with additional penalty based on escape
    /// difficulty. When fewer escape moves are available, the penalty increases
    /// exponentially, encouraging the AI to avoid vulnerable positions or quickly
    /// resolve check situations when they occur.
    /// 
    /// Returns 0 if no player is in check, or the appropriate penalty/bonus
    /// based on which player is in check (negative for Max, positive for Min).
    fn calculate_check_penalty(state: &GameState, ai_config: &AIConfig) -> i32 {
        if let Some(player_in_check) = state.player_in_check {
            if let Some(check_pos) = state.check_position {
                let breaking_moves = crate::core::rules::CaptureBreaking::get_breaking_capture_moves(
                    &state.board,
                    check_pos.0,
                    check_pos.1,
                    player_in_check,
                );
                let num_escapes = breaking_moves.len().max(1) as f32;
                let escape_factor = 5.0 / num_escapes;
                let penalty = ai_config.heuristic.scores.check_penalty + (escape_factor * ai_config.heuristic.scores.check_penalty as f32) as i32;
                
                match player_in_check {
                    Player::Max => -penalty,  // Negative penalty for Max player
                    Player::Min => penalty,   // Positive penalty for Min player
                }
            } else {
                match player_in_check {
                    Player::Max => -ai_config.heuristic.scores.check_penalty,
                    Player::Min => ai_config.heuristic.scores.check_penalty,
                }
            }
        } else {
            0  // No check penalty
        }
    }
    /// Analyzes the entire board to count patterns for both players.
    /// 
    /// This function performs a comprehensive scan of the board to identify and count
    /// all significant patterns (sequences of stones) for both Max and Min players.
    /// It uses directional analysis to examine horizontal, vertical, and diagonal
    /// sequences, avoiding double-counting by tracking already analyzed positions.
    /// 
    /// The analysis considers the win condition length and evaluates pattern freedom
    /// (whether patterns are blocked or have room to grow). Each pattern is classified
    /// by length (2-5 stones) and freedom level (free, half-free, or flanked), which
    /// determines its strategic value and potential for creating winning sequences.
    /// 
    /// Returns tuple of (max_player_patterns, min_player_patterns) containing detailed
    /// counts of each pattern type for strategic evaluation and move planning.
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
    /// Analyzes a specific pattern starting from a given position in a direction.
    /// 
    /// This function examines a sequence of stones belonging to a specific player
    /// in a given direction, determining the pattern's length, freedom, and strategic
    /// value. It finds the true start of the pattern, counts consecutive stones,
    /// and evaluates the available space and freedom around the pattern.
    /// 
    /// The analysis includes checking for pattern validity (minimum length of 2),
    /// ensuring sufficient space for potential win condition completion, and
    /// determining freedom level based on blocking stones or board edges. Patterns
    /// are marked as analyzed to prevent duplicate counting during board traversal.
    /// 
    /// Returns PatternInfo containing length and freedom classification, or None
    /// if the pattern is invalid, too short, or already analyzed.
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
    /// Marks all positions in a pattern as analyzed to prevent duplicate counting.
    /// 
    /// This function iterates through all positions that make up a detected pattern
    /// and marks them with the appropriate directional bit mask in the analyzed
    /// tracking matrix. This prevents the same pattern from being counted multiple
    /// times when the board analysis encounters different stones within the same
    /// sequence, ensuring accurate pattern counting and performance optimization.
    /// 
    /// The bit mask represents the direction of analysis, allowing the same position
    /// to be part of different patterns in different directions while preventing
    /// duplicate counting within the same directional analysis.
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
    /// Updates pattern counts based on analyzed pattern information.
    /// 
    /// This function categorizes detected patterns by their length and freedom level,
    /// incrementing the appropriate counters in the PatternCounts structure. The
    /// categorization determines the strategic value of each pattern, with longer
    /// patterns and higher freedom levels being more valuable for winning.
    /// 
    /// Pattern classification:
    /// - Length 5: Immediate win (five in a row)
    /// - Length 4: Strong threat patterns (live/half-free/dead four)
    /// - Length 3: Medium threat patterns (live/half-free/dead three)
    /// - Length 2: Basic building patterns (live/half-free two)
    /// 
    /// Dead (flanked) patterns of length 2 are ignored as they have no growth potential.
    /// Each combination of length and freedom has different strategic implications for
    /// position evaluation and move selection.
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
    /// Calculates the total strategic score from pattern counts.
    /// 
    /// This function converts pattern counts into a numerical score that reflects
    /// the strategic strength of a position. It applies different scoring weights
    /// based on pattern types and implements special logic for threat combinations
    /// that create winning opportunities.
    /// 
    /// Scoring hierarchy:
    /// - Five in row: Immediate win condition (highest priority)
    /// - Live four: Different values for single vs multiple occurrences
    /// - Threat combinations: Special bonus for winning threat scenarios
    /// - Individual patterns: Weighted by strategic value and frequency
    /// 
    /// Winning threat detection identifies combinations like double live-three,
    /// mixed four-three threats, or multiple half-free fours that guarantee wins
    /// on the next move. The scoring system emphasizes both immediate threats and
    /// long-term positional advantages through balanced pattern valuation.
    fn calculate_pattern_score(counts: PatternCounts, ai_config: &AIConfig) -> i32 {
        let scores = &ai_config.heuristic.scores;
        let mut score = 0;
        if counts.five_in_row > 0 {
            score += scores.five_in_row_score;
        }
        score += match counts.live_four {
            1 => scores.live_four_single_score,
            n if n > 1 => scores.live_four_multiple_score,
            _ => 0,
        };
        if counts.live_three >= 2
            || counts.dead_four >= 2
            || (counts.dead_four >= 1 && counts.live_three >= 1)
            || (counts.half_free_four >= 1 && counts.live_three >= 1)
            || (counts.half_free_four >= 2)
        {
            score += scores.winning_threat_score;
        }
        score += (counts.half_free_four as i32) * scores.half_free_four_score
            + (counts.dead_four as i32) * scores.dead_four_score
            + (counts.live_three as i32) * scores.live_three_score
            + (counts.half_free_three as i32) * scores.half_free_three_score
            + (counts.dead_three as i32) * scores.dead_three_score
            + (counts.live_two as i32) * scores.live_two_score
            + (counts.half_free_two as i32) * scores.half_free_two_score;
        score
    }
    /// Calculates capture bonus differential between players.
    /// 
    /// This function evaluates the strategic advantage gained from capturing
    /// opponent stones, applying a bonus that scales with the square root of
    /// capture count to provide diminishing returns. This prevents excessive
    /// focus on captures while still rewarding successful capture sequences.
    /// 
    /// The square root scaling ensures that the first few captures provide
    /// substantial bonuses while additional captures offer reduced incremental
    /// value, maintaining balanced gameplay where captures enhance but don't
    /// dominate the strategic evaluation.
    /// 
    /// Returns the difference between Max player's capture bonus and Min player's
    /// capture bonus, allowing the evaluation to favor the player with more
    /// successful captures while maintaining proportional scaling.
    fn calculate_capture_bonus(state: &GameState, ai_config: &AIConfig) -> i32 {
        let multiplier = ai_config.heuristic.scores.capture_bonus_multiplier;
        let max_bonus = if state.max_captures > 0 {
            (multiplier as f32 * (state.max_captures as f32).sqrt()) as i32
        } else {
            0
        };
        let min_bonus = if state.min_captures > 0 {
            (multiplier as f32 * (state.min_captures as f32).sqrt()) as i32
        } else {
            0
        };
        max_bonus - min_bonus
    }
    /// Finds the starting position of a pattern in the reverse direction.
    /// 
    /// This function traces backward from a given position to locate the true
    /// beginning of a pattern sequence. By moving in the opposite direction
    /// (negative dx, dy), it identifies where the continuous sequence of stones
    /// for the specified player actually starts, ensuring accurate pattern
    /// length calculation and preventing partial pattern analysis.
    /// 
    /// The function continues tracing backward as long as it finds stones
    /// belonging to the same player, stopping when it encounters an empty
    /// space, opponent stone, or board boundary. This ensures that pattern
    /// analysis begins from the actual start of the sequence rather than
    /// from an arbitrary position within the pattern.
    /// 
    /// Returns the coordinates of the true pattern start position for accurate
    /// pattern length counting and freedom analysis.
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
