use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct Heuristic;

pub const WINNING_SCORE: i32 = 1_000_000;
const FIVE_IN_ROW_SCORE: i32 = 100_000;
const LIVE_FOUR_SINGLE_SCORE: i32 = 15_000;
const LIVE_FOUR_MULTIPLE_SCORE: i32 = 20_000;
const WINNING_THREAT_SCORE: i32 = 10_000;
const DEAD_FOUR_SCORE: i32 = 1_000;
const LIVE_THREE_SCORE: i32 = 500;
const DEAD_THREE_SCORE: i32 = 100;
const LIVE_TWO_SCORE: i32 = 50;
const CAPTURE_BONUS_MULTIPLIER: i32 = 1_000;

// Advanced pattern scoring constants
const SPLIT_FOUR_SCORE: i32 = 25_000;
const JUMP_FOUR_SCORE: i32 = 12_000;
const FORK_ATTACK_SCORE: i32 = 18_000;
const COMPLEX_JUMP_THREE_SCORE: i32 = 3_000;
const SIMPLE_JUMP_THREE_SCORE: i32 = 1_500;
const BROKEN_THREE_SCORE: i32 = 800;

const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
const ALL_DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[derive(Debug, Clone, Copy)]
struct PatternCounts {
    five_in_row: u8,
    live_four: u8,
    dead_four: u8,
    live_three: u8,
    dead_three: u8,
    live_two: u8,
}

impl PatternCounts {
    const fn new() -> Self {
        Self {
            five_in_row: 0,
            live_four: 0,
            dead_four: 0,
            live_three: 0,
            dead_three: 0,
            live_two: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PatternInfo {
    length: usize,
    is_live: bool,
}

// Advanced pattern types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdvancedPatternType {
    ConsecutiveLive(u8),
    ConsecutiveDead(u8),
    JumpPattern(u8, u8),  // (total_stones, gaps)
    SplitPattern(u8, u8), // (left_side, right_side)
    ForkPattern,          // Creates multiple threats
    ComplexThreat,        // Combination patterns
}

#[derive(Debug, Clone, Copy)]
pub struct AdvancedPattern {
    pub pattern_type: AdvancedPatternType,
    pub threat_level: u8, // 1-10 scale
    pub forcing: bool,    // Does this force opponent response?
    pub direction: (isize, isize),
    pub positions: [(usize, usize); 6], // Key positions in pattern
    pub position_count: u8,
}

#[derive(Debug, Clone, Copy)]
enum SequenceElement {
    Stone(usize, usize),
    Gap(usize, usize),
    Blocked,
}

impl Heuristic {
    pub fn evaluate(state: &GameState, depth: i32) -> i32 {
        if let Some(terminal_score) = Self::evaluate_terminal_states(state, depth) {
            return terminal_score;
        }

        if Self::is_board_full(&state.board) {
            return 0;
        }

        // Basic pattern analysis (existing)
        let (max_counts, min_counts) =
            Self::analyze_both_players(&state.board, state.win_condition);

        // Advanced pattern analysis (new)
        let max_advanced_patterns =
            Self::detect_advanced_patterns(&state.board, Player::Max, state.win_condition);
        let min_advanced_patterns =
            Self::detect_advanced_patterns(&state.board, Player::Min, state.win_condition);

        // Early termination for critical patterns
        if max_counts.five_in_row > 0 || max_counts.live_four > 1 {
            return WINNING_SCORE + depth;
        }
        if min_counts.five_in_row > 0 || min_counts.live_four > 1 {
            return -WINNING_SCORE - depth;
        }

        // Check for advanced winning patterns
        if Self::has_critical_advanced_pattern(&max_advanced_patterns) {
            return WINNING_SCORE + depth;
        }
        if Self::has_critical_advanced_pattern(&min_advanced_patterns) {
            return -WINNING_SCORE - depth;
        }

        let max_basic_score = Self::calculate_pattern_score(max_counts);
        let min_basic_score = Self::calculate_pattern_score(min_counts);
        let max_advanced_score = Self::calculate_advanced_pattern_score(&max_advanced_patterns);
        let min_advanced_score = Self::calculate_advanced_pattern_score(&min_advanced_patterns);
        let capture_bonus = Self::calculate_capture_bonus(state);

        (max_basic_score + max_advanced_score) - (min_basic_score + min_advanced_score)
            + capture_bonus
    }

    // Advanced Pattern Detection System
    fn detect_advanced_patterns(
        board: &Board,
        player: Player,
        win_condition: usize,
    ) -> Vec<AdvancedPattern> {
        let mut patterns = Vec::new();

        // Detect each type of advanced pattern
        patterns.extend(Self::detect_jump_patterns(board, player, win_condition));
        patterns.extend(Self::detect_split_patterns(board, player, win_condition));
        patterns.extend(Self::detect_fork_patterns(board, player));
        patterns.extend(Self::detect_complex_threats(board, player));

        // Remove duplicates and sort by threat level
        patterns.sort_by(|a, b| b.threat_level.cmp(&a.threat_level));
        patterns
    }

    pub fn detect_jump_patterns(
        board: &Board,
        player: Player,
        win_condition: usize,
    ) -> Vec<AdvancedPattern> {
        let mut patterns = Vec::new();

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col) == Some(player) {
                    for &(dx, dy) in &DIRECTIONS {
                        if let Some(pattern) = Self::analyze_jump_sequence(
                            board,
                            row,
                            col,
                            dx,
                            dy,
                            player,
                            win_condition,
                        ) {
                            patterns.push(pattern);
                        }
                    }
                }
            }
        }

        patterns
    }

    fn analyze_jump_sequence(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
    ) -> Option<AdvancedPattern> {
        // First, find the actual start of the pattern
        let (pattern_start_row, pattern_start_col) =
            Self::find_pattern_start(board, start_row, start_col, dx, dy, player);

        let mut sequence = Vec::new();
        let mut current_row = pattern_start_row as isize;
        let mut current_col = pattern_start_col as isize;
        let mut gap_count = 0;
        let mut stone_count = 0;
        let mut consecutive_gaps = 0;

        // Analyze sequence up to win_condition + 2 positions
        for _i in 0..=(win_condition + 2) {
            if !Self::is_valid_position(board, current_row, current_col) {
                break;
            }

            match board.get_player(current_row as usize, current_col as usize) {
                Some(p) if p == player => {
                    sequence.push(SequenceElement::Stone(
                        current_row as usize,
                        current_col as usize,
                    ));
                    stone_count += 1;
                    consecutive_gaps = 0; // Reset consecutive gap counter
                }
                None => {
                    if gap_count < 2 && consecutive_gaps < 2 {
                        // Only consider patterns with max 2 gaps and limit consecutive gaps
                        sequence.push(SequenceElement::Gap(
                            current_row as usize,
                            current_col as usize,
                        ));
                        gap_count += 1;
                        consecutive_gaps += 1;
                    } else {
                        break;
                    }
                }
                _ => {
                    sequence.push(SequenceElement::Blocked);
                    break; // Opponent stone, end sequence
                }
            }

            current_row += dx;
            current_col += dy;
        }

        // Evaluate if this forms a meaningful jump pattern
        Self::evaluate_jump_sequence(sequence, stone_count, gap_count, dx, dy)
    }

    fn is_meaningful_pattern(
        sequence: &[SequenceElement],
        stone_count: usize,
        gap_count: usize,
    ) -> bool {
        // Must have minimum stones and reasonable gap ratio
        if stone_count < 2 || gap_count == 0 {
            return false;
        }

        // Don't allow too many gaps relative to stones
        if gap_count > stone_count {
            return false;
        }

        // Ensure sequence isn't too sparse
        if sequence.len() > stone_count + gap_count + 1 {
            return false;
        }

        true
    }

    fn evaluate_jump_sequence(
        sequence: Vec<SequenceElement>,
        stone_count: usize,
        gap_count: usize,
        dx: isize,
        dy: isize,
    ) -> Option<AdvancedPattern> {
        if !Self::is_meaningful_pattern(&sequence, stone_count, gap_count) {
            return None;
        }

        let positions = Self::extract_positions_from_sequence(&sequence);
        let pattern = match (stone_count, gap_count, sequence.len()) {
            // Jump four patterns (very dangerous)
            (4, 1, 5) => AdvancedPattern {
                pattern_type: AdvancedPatternType::JumpPattern(4, 1),
                threat_level: 9,
                forcing: true,
                direction: (dx, dy),
                positions,
                position_count: sequence.len() as u8,
            },
            // Jump three patterns
            (3, 1, 4) => AdvancedPattern {
                pattern_type: AdvancedPatternType::JumpPattern(3, 1),
                threat_level: 6,
                forcing: false,
                direction: (dx, dy),
                positions,
                position_count: sequence.len() as u8,
            },
            // Broken three (3 stones, 2 gaps in 5 positions)
            (3, 2, 5) => AdvancedPattern {
                pattern_type: AdvancedPatternType::JumpPattern(3, 2),
                threat_level: 4,
                forcing: false,
                direction: (dx, dy),
                positions,
                position_count: sequence.len() as u8,
            },
            _ => return None,
        };

        Some(pattern)
    }

    pub fn detect_split_patterns(
        board: &Board,
        player: Player,
        _win_condition: usize,
    ) -> Vec<AdvancedPattern> {
        let mut patterns = Vec::new();

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col) == Some(player) {
                    for &(dx, dy) in &DIRECTIONS {
                        patterns.extend(Self::find_split_fours(board, row, col, dx, dy, player));
                    }
                }
            }
        }

        patterns
    }

    fn find_split_fours(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> Vec<AdvancedPattern> {
        let mut patterns = Vec::new();

        // Check for XX_XX pattern (split four)
        if Self::matches_pattern(
            board,
            start_row,
            start_col,
            dx,
            dy,
            player,
            &[Some(player), Some(player), None, Some(player), Some(player)],
        ) {
            let positions = Self::get_pattern_positions(start_row, start_col, dx, dy, 5);
            patterns.push(AdvancedPattern {
                pattern_type: AdvancedPatternType::SplitPattern(2, 2),
                threat_level: 10, // Extremely dangerous!
                forcing: true,
                direction: (dx, dy),
                positions,
                position_count: 5,
            });
        }

        // Check for XXX_X pattern
        if Self::matches_pattern(
            board,
            start_row,
            start_col,
            dx,
            dy,
            player,
            &[Some(player), Some(player), Some(player), None, Some(player)],
        ) {
            let positions = Self::get_pattern_positions(start_row, start_col, dx, dy, 5);
            patterns.push(AdvancedPattern {
                pattern_type: AdvancedPatternType::SplitPattern(3, 1),
                threat_level: 9,
                forcing: true,
                direction: (dx, dy),
                positions,
                position_count: 5,
            });
        }

        // Check for X_XXX pattern
        if Self::matches_pattern(
            board,
            start_row,
            start_col,
            dx,
            dy,
            player,
            &[Some(player), None, Some(player), Some(player), Some(player)],
        ) {
            let positions = Self::get_pattern_positions(start_row, start_col, dx, dy, 5);
            patterns.push(AdvancedPattern {
                pattern_type: AdvancedPatternType::SplitPattern(1, 3),
                threat_level: 9,
                forcing: true,
                direction: (dx, dy),
                positions,
                position_count: 5,
            });
        }

        patterns
    }

    pub fn detect_fork_patterns(board: &Board, player: Player) -> Vec<AdvancedPattern> {
        let mut fork_patterns = Vec::new();

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col).is_none() {
                    // Check if placing a stone here creates multiple threats
                    let threat_count = Self::count_threats_from_position(board, row, col, player);

                    if threat_count >= 2 {
                        let mut positions = [(0, 0); 6];
                        positions[0] = (row, col);

                        fork_patterns.push(AdvancedPattern {
                            pattern_type: AdvancedPatternType::ForkPattern,
                            threat_level: 10,
                            forcing: true,
                            direction: (0, 0), // Forks don't have a single direction
                            positions,
                            position_count: 1,
                        });
                    }
                }
            }
        }

        fork_patterns
    }

    pub fn detect_complex_threats(board: &Board, player: Player) -> Vec<AdvancedPattern> {
        let mut complex_patterns = Vec::new();

        // Look for combinations of patterns that create complex threats
        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col).is_none() {
                    if Self::creates_complex_threat(board, row, col, player) {
                        let mut positions = [(0, 0); 6];
                        positions[0] = (row, col);

                        complex_patterns.push(AdvancedPattern {
                            pattern_type: AdvancedPatternType::ComplexThreat,
                            threat_level: 7,
                            forcing: false,
                            direction: (0, 0),
                            positions,
                            position_count: 1,
                        });
                    }
                }
            }
        }

        complex_patterns
    }

    // Helper functions for advanced pattern detection
    fn matches_pattern(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        _player: Player,
        pattern: &[Option<Player>],
    ) -> bool {
        for (i, &expected) in pattern.iter().enumerate() {
            let row = start_row as isize + i as isize * dx;
            let col = start_col as isize + i as isize * dy;

            if !Self::is_valid_position(board, row, col) {
                return false;
            }

            let actual = board.get_player(row as usize, col as usize);
            if actual != expected {
                return false;
            }
        }
        true
    }

    fn get_pattern_positions(
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        length: usize,
    ) -> [(usize, usize); 6] {
        let mut positions = [(0, 0); 6];
        for i in 0..length.min(6) {
            let row = (start_row as isize + i as isize * dx) as usize;
            let col = (start_col as isize + i as isize * dy) as usize;
            positions[i] = (row, col);
        }
        positions
    }

    fn extract_positions_from_sequence(sequence: &[SequenceElement]) -> [(usize, usize); 6] {
        let mut positions = [(0, 0); 6];
        let mut pos_idx = 0;

        for element in sequence {
            if pos_idx >= 6 {
                break;
            }
            match element {
                SequenceElement::Stone(row, col) | SequenceElement::Gap(row, col) => {
                    positions[pos_idx] = (*row, *col);
                    pos_idx += 1;
                }
                SequenceElement::Blocked => break,
            }
        }
        positions
    }

    pub fn count_threats_from_position(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
    ) -> u8 {
        let mut threat_count = 0;

        for &(dx, dy) in &DIRECTIONS {
            // Simulate placing stone and check for immediate threats
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);

            if consecutive >= 4 {
                // Only count as threat if it would create 4+ in a row
                threat_count += 1;
            } else if consecutive == 3 {
                // Only count live threes that are actually open on both ends
                if Self::would_create_live_three(board, row, col, dx, dy, player) {
                    // Additional check: ensure the three is actually significant
                    let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
                    let forwards = Self::count_direction(board, row, col, dx, dy, player);

                    // Only count if it forms a substantial threat
                    if backwards + forwards >= 2 {
                        threat_count += 1;
                    }
                }
            }
        }

        threat_count
    }

    fn would_create_live_three(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> bool {
        // Find the extent of the three that would be created
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);

        if backwards + forwards + 1 != 3 {
            return false;
        }

        // Check if both ends are open
        let start_row = row as isize - (backwards as isize) * dx - dx;
        let start_col = col as isize - (backwards as isize) * dy - dy;
        let end_row = row as isize + (forwards as isize) * dx + dx;
        let end_col = col as isize + (forwards as isize) * dy + dy;

        Self::is_position_empty(board, start_row, start_col)
            && Self::is_position_empty(board, end_row, end_col)
    }

    pub fn creates_complex_threat(board: &Board, row: usize, col: usize, player: Player) -> bool {
        let mut live_threes = 0;
        let mut dead_fours = 0;

        for &(dx, dy) in &DIRECTIONS {
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);

            match consecutive {
                4 => dead_fours += 1,
                3 => {
                    if Self::would_create_live_three(board, row, col, dx, dy, player) {
                        live_threes += 1;
                    }
                }
                _ => {}
            }
        }

        // Complex threat: multiple live threes or combination of dead four + live three
        live_threes >= 2 || (dead_fours >= 1 && live_threes >= 1)
    }

    fn has_critical_advanced_pattern(patterns: &[AdvancedPattern]) -> bool {
        patterns.iter().any(|p| {
            p.threat_level >= 9
                && matches!(
                    p.pattern_type,
                    AdvancedPatternType::SplitPattern(_, _) | AdvancedPatternType::ForkPattern
                )
        })
    }

    pub fn calculate_advanced_pattern_score(patterns: &[AdvancedPattern]) -> i32 {
        let mut score = 0;

        for pattern in patterns {
            let base_score = match pattern.pattern_type {
                AdvancedPatternType::JumpPattern(stones, gaps) => {
                    match (stones, gaps) {
                        (4, 1) => JUMP_FOUR_SCORE,          // Very dangerous jump four
                        (3, 1) => COMPLEX_JUMP_THREE_SCORE, // Dangerous jump three
                        (3, 2) => SIMPLE_JUMP_THREE_SCORE,  // Simple broken three
                        _ => BROKEN_THREE_SCORE,            // Other patterns
                    }
                }
                AdvancedPatternType::SplitPattern(left, right) => {
                    match (left, right) {
                        (2, 2) => SPLIT_FOUR_SCORE,         // Split four - game winning
                        (3, 1) | (1, 3) => JUMP_FOUR_SCORE, // Almost as dangerous
                        _ => COMPLEX_JUMP_THREE_SCORE,      // Other split patterns
                    }
                }
                AdvancedPatternType::ForkPattern => FORK_ATTACK_SCORE, // Fork attack - decisive
                AdvancedPatternType::ComplexThreat => COMPLEX_JUMP_THREE_SCORE, // Complex threats
                _ => 1000,
            };

            // Apply multipliers
            let threat_multiplier = if pattern.forcing { 150 } else { 100 };
            let level_bonus = (pattern.threat_level as i32) * 50;

            score += (base_score * threat_multiplier / 100) + level_bonus;
        }

        score
    }

    #[inline(always)]
    fn is_valid_position(board: &Board, row: isize, col: isize) -> bool {
        row >= 0 && row < board.size as isize && col >= 0 && col < board.size as isize
    }

    // Original functions (unchanged)
    fn analyze_both_players(board: &Board, win_condition: usize) -> (PatternCounts, PatternCounts) {
        let mut max_counts = PatternCounts::new();
        let mut min_counts = PatternCounts::new();
        let mut analyzed = vec![vec![0u8; board.size]; board.size];

        for row in 0..board.size {
            for col in 0..board.size {
                if let Some(player) = board.get_player(row, col) {
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
        analyzed: &mut Vec<Vec<u8>>,
        bit_mask: u8,
    ) -> Option<PatternInfo> {
        let (pattern_start_row, pattern_start_col) =
            Self::find_pattern_start(board, start_row, start_col, dx, dy, player);

        if analyzed[pattern_start_row][pattern_start_col] & bit_mask != 0 {
            return None;
        }

        let length =
            Self::count_consecutive(board, pattern_start_row, pattern_start_col, dx, dy, player);

        if length < 2 {
            return None;
        }

        let length = length.min(win_condition);
        let is_live =
            Self::is_pattern_live(board, pattern_start_row, pattern_start_col, dx, dy, length);

        Self::mark_pattern_analyzed(
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
            analyzed,
            bit_mask,
        );

        Some(PatternInfo { length, is_live })
    }

    fn count_consecutive(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let mut count = 0;
        let mut current_row = row as isize;
        let mut current_col = col as isize;
        let max_row = board.size as isize;
        let max_col = board.size as isize;

        while current_row >= 0 && current_row < max_row && current_col >= 0 && current_col < max_col
        {
            if board.get_player(current_row as usize, current_col as usize) == Some(player) {
                count += 1;
                current_row += dx;
                current_col += dy;
            } else {
                break;
            }
        }
        count
    }

    fn find_pattern_start(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> (usize, usize) {
        let mut current_row = row as isize;
        let mut current_col = col as isize;

        loop {
            let prev_row = current_row - dx;
            let prev_col = current_col - dy;

            if prev_row >= 0
                && prev_row < board.size as isize
                && prev_col >= 0
                && prev_col < board.size as isize
                && board.get_player(prev_row as usize, prev_col as usize) == Some(player)
            {
                current_row = prev_row;
                current_col = prev_col;
            } else {
                break;
            }
        }

        (current_row as usize, current_col as usize)
    }

    fn is_pattern_live(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        length: usize,
    ) -> bool {
        let before_row = start_row as isize - dx;
        let before_col = start_col as isize - dy;
        let start_open = Self::is_position_empty(board, before_row, before_col);

        let end_row = start_row as isize + (length as isize * dx);
        let end_col = start_col as isize + (length as isize * dy);
        let end_open = Self::is_position_empty(board, end_row, end_col);

        start_open && end_open
    }

    #[inline(always)]
    fn is_position_empty(board: &Board, row: isize, col: isize) -> bool {
        row >= 0
            && row < board.size as isize
            && col >= 0
            && col < board.size as isize
            && board.get_player(row as usize, col as usize).is_none()
    }

    fn mark_pattern_analyzed(
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        length: usize,
        analyzed: &mut Vec<Vec<u8>>,
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
            4 => {
                if pattern.is_live {
                    counts.live_four += 1;
                } else {
                    counts.dead_four += 1;
                }
            }
            3 => {
                if pattern.is_live {
                    counts.live_three += 1;
                } else {
                    counts.dead_three += 1;
                }
            }
            2 => {
                if pattern.is_live {
                    counts.live_two += 1;
                }
            }
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
        {
            score += WINNING_THREAT_SCORE;
        }

        score += (counts.dead_four as i32) * DEAD_FOUR_SCORE
            + (counts.live_three as i32) * LIVE_THREE_SCORE
            + (counts.dead_three as i32) * DEAD_THREE_SCORE
            + (counts.live_two as i32) * LIVE_TWO_SCORE;

        score
    }

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        let center = state.board.size / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
    }

    fn calculate_move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        let center_distance = Self::manhattan_distance(row, col, center, center);
        priority += 100 - center_distance as i32;

        priority += Self::calculate_threat_priority(&state.board, row, col);
        priority += Self::calculate_adjacency_bonus(&state.board, row, col);

        // Add advanced pattern priority
        priority += Self::calculate_advanced_move_priority(&state.board, row, col);

        priority
    }

    fn calculate_advanced_move_priority(board: &Board, row: usize, col: usize) -> i32 {
        let mut priority = 0;

        // Check if this move creates advanced patterns
        for &player in &[Player::Max, Player::Min] {
            let multiplier = if player == Player::Max { 1 } else { -1 };

            // Check for potential fork creation
            let threat_count = Self::count_threats_from_position(board, row, col, player);
            if threat_count >= 2 {
                priority += multiplier * 5000;
            }

            // Check for potential split four creation
            for &(dx, dy) in &DIRECTIONS {
                if Self::would_create_split_four(board, row, col, dx, dy, player) {
                    priority += multiplier * 4000;
                }
            }
        }

        priority
    }

    fn would_create_split_four(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> bool {
        // Check if placing stone creates a split four pattern
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);

        // Look for patterns that would create XX_XX or similar
        if backwards + forwards + 1 == 4 {
            // Check if there's a gap that would make this a split four
            let gap_row = row as isize + dx;
            let gap_col = col as isize + dy;
            if Self::is_position_empty(board, gap_row, gap_col) {
                let after_gap_row = gap_row + dx;
                let after_gap_col = gap_col + dy;
                if Self::is_valid_position(board, after_gap_row, after_gap_col) {
                    return board.get_player(after_gap_row as usize, after_gap_col as usize)
                        == Some(player);
                }
            }
        }

        false
    }

    fn calculate_threat_priority(board: &Board, row: usize, col: usize) -> i32 {
        let mut threat_score = 0;

        for &player in &[Player::Max, Player::Min] {
            for &(dx, dy) in &DIRECTIONS {
                let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
                let multiplier = if player == Player::Max { 1 } else { -1 };

                threat_score += multiplier
                    * match consecutive {
                        5 => 10000,
                        4 => 5000,
                        3 => 1000,
                        _ => 0,
                    };
            }
        }
        threat_score
    }

    fn simulate_move_consecutive(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        backwards + forwards + 1
    }

    fn count_direction(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let mut count = 0;
        let mut current_row = row as isize + dx;
        let mut current_col = col as isize + dy;

        while current_row >= 0
            && current_row < board.size as isize
            && current_col >= 0
            && current_col < board.size as isize
            && board.get_player(current_row as usize, current_col as usize) == Some(player)
        {
            count += 1;
            current_row += dx;
            current_col += dy;
        }
        count
    }

    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize) -> i32 {
        let mut bonus = 0;
        for &(dx, dy) in &ALL_DIRECTIONS {
            let new_row = row as isize + dx;
            let new_col = col as isize + dy;
            if new_row >= 0
                && new_row < board.size as isize
                && new_col >= 0
                && new_col < board.size as isize
                && board
                    .get_player(new_row as usize, new_col as usize)
                    .is_some()
            {
                bonus += 50;
            }
        }
        bonus
    }

    fn evaluate_terminal_states(state: &GameState, depth: i32) -> Option<i32> {
        if let Some(winner) = state.check_winner() {
            return Some(match winner {
                Player::Max => WINNING_SCORE + depth,
                Player::Min => -WINNING_SCORE - depth,
            });
        }
        if state.max_captures >= 5 {
            return Some(WINNING_SCORE + depth);
        }
        if state.min_captures >= 5 {
            return Some(-WINNING_SCORE - depth);
        }
        None
    }

    fn is_board_full(board: &Board) -> bool {
        board
            .cells
            .iter()
            .all(|row| row.iter().all(|&cell| cell.is_some()))
    }

    fn calculate_capture_bonus(state: &GameState) -> i32 {
        (state.max_captures as i32 - state.min_captures as i32) * CAPTURE_BONUS_MULTIPLIER
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
