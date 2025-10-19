use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::state::GameState;
use crate::core::rules::DoubleThreeDetection;

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
    gapped_live_four: u8,
    gapped_half_free_four: u8,
    gapped_dead_four: u8,
    gapped_live_three: u8,
    gapped_half_free_three: u8,
    gapped_dead_three: u8,
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
            gapped_live_four: 0,
            gapped_half_free_four: 0,
            gapped_dead_four: 0,
            gapped_live_three: 0,
            gapped_half_free_three: 0,
            gapped_dead_three: 0,
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct PatternInfo {
    length: usize,
    freedom: PatternFreedom,
}

#[derive(Debug, Clone, Copy)]
struct GappedPatternInfo {
    stones: usize,
    freedom: PatternFreedom,
}

// Scoring constants for different game situations
pub const WINNING_SCORE: i32 = 1_000_000;
pub const FIVE_IN_ROW_SCORE: i32 = 100_000;      // XXXXX (immediate win)
pub const LIVE_FOUR_MULTIPLE_SCORE: i32 = 20_000; // _XXXX_ + _XXXX_ (multiple threats)
pub const LIVE_FOUR_SINGLE_SCORE: i32 = 15_000;  // _XXXX_ (open four, guaranteed win next move)
pub const CAPTURE_BONUS_MULTIPLIER: i32 = 25_000;
pub const CHECK_PENALTY: i32 = 10_000;
pub const WINNING_THREAT_SCORE: i32 = 10_000;    // _XXX_ + _XXX_ (double threat combinations)
pub const HALF_FREE_FOUR_SCORE: i32 = 3_500;     // _XXXX| or |XXXX_ (one-sided four)
pub const LIVE_THREE_SCORE: i32 = 500;           // _XXX_ (open three, can become four)
pub const DEAD_FOUR_SCORE: i32 = 400;            // |XXXX| (blocked four, limited threat)
pub const HALF_FREE_THREE_SCORE: i32 = 200;      // _XXX| or |XXX_ (one-sided three)
pub const DEAD_THREE_SCORE: i32 = 50;            // |XXX| (blocked three, minimal threat)
pub const LIVE_TWO_SCORE: i32 = 50;              // _XX_ (open two, growth potential)
pub const HALF_FREE_TWO_SCORE: i32 = 20;         // _XX| or |XX_ (one-sided two)

// Gapped pattern scores - Only patterns that pass double-three validation are counted
// This ensures all gapped patterns can be legally completed
pub const GAPPED_LIVE_FOUR_SCORE: i32 = 8_000;      // _XX.X_ or _X.XX_ (strong gapped threat)
pub const GAPPED_HALF_FREE_FOUR_SCORE: i32 = 2_000;  // XX.X| or |X.XX (one-sided gapped four)
pub const GAPPED_DEAD_FOUR_SCORE: i32 = 200;         // |XX.X| (blocked gapped four)
pub const GAPPED_LIVE_THREE_SCORE: i32 = 300;        // _X.X_ or _XX._ (gapped three potential)
pub const GAPPED_HALF_FREE_THREE_SCORE: i32 = 120;   // X.X| or |X.X (one-sided gapped three)
pub const GAPPED_DEAD_THREE_SCORE: i32 = 30;         // |X.X| (blocked gapped three)

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
    pub fn evaluate(state: &GameState, depth: i32) -> i32 {
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => WINNING_SCORE + depth,
                Player::Min => -WINNING_SCORE - depth,
            };
        }
        if state.board.is_full() {
            return 0;
        }
        
        // Pattern analysis and position evaluation
        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition);
        let max_score = Self::calculate_pattern_score(max_counts);
        let min_score = Self::calculate_pattern_score(min_counts);
        let capture_bonus = Self::calculate_capture_bonus(state);
        let historical_bonus = Self::calculate_historical_bonus(state);
        let check_penalty = Self::calculate_check_penalty(state);
        
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
    fn calculate_historical_bonus(state: &GameState) -> i32 {
        let max_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Max);
        let min_bonus = state
            .pattern_analyzer
            .calculate_historical_bonus(Player::Min);
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
    fn calculate_check_penalty(state: &GameState) -> i32 {
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
                let penalty = CHECK_PENALTY + (escape_factor * CHECK_PENALTY as f32) as i32;
                
                match player_in_check {
                    Player::Max => -penalty,
                    Player::Min => penalty,
                }
            } else {
                match player_in_check {
                    Player::Max => -CHECK_PENALTY,
                    Player::Min => CHECK_PENALTY,
                }
            }
        } else {
            0  // No check penalty
        }
    }

    /// Validates if a gapped pattern can be completed without violating the double-three rule.
    /// 
    /// This function checks whether filling the gaps in a gapped pattern would create
    /// forbidden double-three situations. It simulates placing stones in the gap positions
    /// and uses the DoubleThreeDetection to verify if such moves would be legal.
    /// 
    /// # Arguments
    /// * `board` - The current game board
    /// * `start_row` - Starting row of the gapped pattern
    /// * `start_col` - Starting column of the gapped pattern
    /// * `dx` - Direction increment for rows
    /// * `dy` - Direction increment for columns
    /// * `player` - The player whose pattern is being validated
    /// * `win_condition` - Length needed to win (typically 5)
    /// 
    /// # Returns
    /// `true` if at least one gap can be filled without creating a double-three, `false` otherwise
    pub fn can_complete_gapped_pattern_legally(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
    ) -> bool {
        let player_bits = board.get_player_bits(player);
        let mut gap_positions = Vec::new();
        
        // Find all gap positions in the pattern
        for i in 0..win_condition {
            let check_row = start_row as isize + i as isize * dx;
            let check_col = start_col as isize + i as isize * dy;
            
            if !PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                break;
            }
            
            let idx = board.index(check_row as usize, check_col as usize);
            if Board::is_bit_set(&board.occupied, idx) {
                if !Board::is_bit_set(player_bits, idx) {
                    // Opponent stone blocks the pattern
                    break;
                }
                // Player's own stone - continue
            } else {
                // Empty position - this is a potential gap to fill
                gap_positions.push((check_row as usize, check_col as usize));
            }
        }
        
        // Check if any gap can be filled without creating a double-three
        for &(gap_row, gap_col) in &gap_positions {
            if !DoubleThreeDetection::creates_double_three(board, gap_row, gap_col, player) {
                return true; // At least one gap can be filled legally
            }
        }
        
        // If there are no gaps, it's already complete (should not happen for gapped patterns)
        // If there are gaps but none can be filled legally, return false
        gap_positions.is_empty()
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
    /// Enhanced to also detect gapped patterns
    /// that could form winning sequences when gaps are filled.
    /// 
    /// Returns tuple of (max_player_patterns, min_player_patterns) containing detailed
    /// counts of each pattern type for strategic evaluation and move planning.
    fn analyze_both_players(board: &Board, win_condition: usize) -> (PatternCounts, PatternCounts) {
        let mut max_counts = PatternCounts::new();
        let mut min_counts = PatternCounts::new();
        let mut analyzed = vec![vec![0u8; board.size]; board.size];
        let mut gapped_analyzed = vec![vec![0u8; board.size]; board.size];
        
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
                    
                    if gapped_analyzed[row][col] & bit_mask == 0 {
                        if PatternAnalyzer::is_gapped_pattern_start(board, row, col, dx, dy, player) {
                            if let Some((stones, _gaps, freedom)) = PatternAnalyzer::analyze_gapped_sequence(
                                board, row, col, dx, dy, player, win_condition
                            ) {
                                // Mark this sequence as analyzed to avoid duplicates
                                Self::mark_gapped_sequence_analyzed(
                                    board, row, col, dx, dy, player, win_condition,
                                    &mut gapped_analyzed, bit_mask
                                );
                                
                                let gapped_info = GappedPatternInfo { stones, freedom };
                                
                                // Check if this gapped pattern can be completed legally
                                let is_legally_completable = Self::can_complete_gapped_pattern_legally(
                                    board, row, col, dx, dy, player, win_condition
                                );
                                
                                // Only count gapped patterns when they can be completed legally
                                if is_legally_completable {
                                    match player {
                                        Player::Max => Self::update_gapped_counts(&mut max_counts, gapped_info),
                                        Player::Min => Self::update_gapped_counts(&mut min_counts, gapped_info),
                                    }
                                }
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

    /// Updates gapped pattern counts based on analyzed gapped pattern information.
    /// 
    /// This function categorizes detected gapped patterns by their stone count and freedom level,
    /// incrementing the appropriate gapped counters in the PatternCounts structure. Gapped patterns
    /// are stones separated by small gaps that could form threats when gaps are filled.
    /// 
    /// Only patterns with 3 or 4 stones are considered as gapped patterns, since 2-stone
    /// gapped patterns have minimal threat value and 5-stone patterns would be wins.
    fn update_gapped_counts(counts: &mut PatternCounts, pattern: GappedPatternInfo) {
        match pattern.stones {
            4 => match pattern.freedom {
                PatternFreedom::Free => counts.gapped_live_four += 1,
                PatternFreedom::HalfFree => counts.gapped_half_free_four += 1,
                PatternFreedom::Flanked => counts.gapped_dead_four += 1,
            },
            3 => match pattern.freedom {
                PatternFreedom::Free => counts.gapped_live_three += 1,
                PatternFreedom::HalfFree => counts.gapped_half_free_three += 1,
                PatternFreedom::Flanked => counts.gapped_dead_three += 1,
            },
            _ => {} // Only consider 3 and 4 stone gapped patterns
        }
    }

    /// Marks positions in a gapped sequence as analyzed to prevent duplicate counting.
    /// 
    /// This function marks all stone positions that are part of a detected gapped pattern
    /// to ensure they aren't counted again during the same directional analysis.
    fn mark_gapped_sequence_analyzed(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
        analyzed: &mut [Vec<u8>],
        bit_mask: u8,
    ) {
        let player_bits = board.get_player_bits(player);
        
        for i in 0..win_condition {
            let check_row = start_row as isize + i as isize * dx;
            let check_col = start_col as isize + i as isize * dy;
            
            if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                let idx = board.index(check_row as usize, check_col as usize);
                if Board::is_bit_set(player_bits, idx) {
                    let row = check_row as usize;
                    let col = check_col as usize;
                    if row < analyzed.len() && col < analyzed[0].len() {
                        analyzed[row][col] |= bit_mask;
                    }
                }
            } else {
                break;
            }
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
    /// - Gapped patterns: Integrated scoring for stones separated by gaps
    /// 
    /// Winning threat detection identifies combinations like double live-three,
    /// mixed four-three threats, or multiple half-free fours that guarantee wins
    /// on the next move. The scoring system emphasizes both immediate threats and
    /// long-term positional advantages through balanced pattern valuation.

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
        
        // Gapped patterns are now pre-validated against double-three rule during analysis
        let total_half_fours = counts.half_free_four + counts.gapped_half_free_four;
        let total_dead_fours = counts.dead_four + counts.gapped_dead_four;
        let total_live_threes = counts.live_three + counts.gapped_live_three;
        
        if total_live_threes >= 2
            || total_dead_fours >= 2
            || (total_dead_fours >= 1 && total_live_threes >= 1)
            || (total_half_fours >= 1 && total_live_threes >= 1)
            || (total_half_fours >= 2)
            || (counts.gapped_live_four >= 1 && counts.live_three >= 1)
            || (counts.live_four >= 1 && counts.gapped_live_three >= 1)
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
        
        // Gapped patterns are now pre-validated (only legally completable patterns are counted)
        score += (counts.gapped_live_four as i32) * GAPPED_LIVE_FOUR_SCORE
            + (counts.gapped_half_free_four as i32) * GAPPED_HALF_FREE_FOUR_SCORE
            + (counts.gapped_dead_four as i32) * GAPPED_DEAD_FOUR_SCORE
            + (counts.gapped_live_three as i32) * GAPPED_LIVE_THREE_SCORE
            + (counts.gapped_half_free_three as i32) * GAPPED_HALF_FREE_THREE_SCORE
            + (counts.gapped_dead_three as i32) * GAPPED_DEAD_THREE_SCORE;
        
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
