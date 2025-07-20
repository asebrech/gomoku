use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct Heuristic;

// Urgency-based scoring: Immediate threats get exponentially higher scores
const WINNING_SCORE: i32 = 10_000_000;
const IMMEDIATE_WIN_SCORE: i32 = 5_000_000;        // Can win in 1 move
const IMMEDIATE_THREAT_SCORE: i32 = 2_500_000;     // Must block or lose in 1 move  
const DOUBLE_THREAT_SCORE: i32 = 1_000_000;        // Creates multiple threats (fork)
const URGENT_THREAT_SCORE: i32 = 500_000;          // Strong threat in 2-3 moves
const TACTICAL_THREAT_SCORE: i32 = 100_000;        // Good tactical position
const LIVE_FOUR_SCORE: i32 = 50_000;
const DEAD_FOUR_SCORE: i32 = 10_000;
const LIVE_THREE_SCORE: i32 = 5_000;
const DEAD_THREE_SCORE: i32 = 1_000;
const LIVE_TWO_SCORE: i32 = 500;
const CAPTURE_BONUS_MULTIPLIER: i32 = 2_000;

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

impl Heuristic {
    pub fn evaluate(state: &GameState, depth: i32) -> i32 {
        if let Some(terminal_score) = Self::evaluate_terminal_states(state, depth) {
            return terminal_score;
        }

        if state.board.is_board_full() {
            return 0; // Draw
        }

        // PRIORITY 1: Immediate threat detection (0-2 moves ahead)
        let immediate_threat_bonus = Self::evaluate_immediate_threats(state, depth);
        if immediate_threat_bonus.abs() >= IMMEDIATE_THREAT_SCORE {
            return immediate_threat_bonus; // Return immediately for critical threats
        }

        // PRIORITY 2: Standard tactical evaluation with depth penalty
        let tactical_score = Self::evaluate_tactical_threats(state) + immediate_threat_bonus;
        
        // Apply depth penalty to encourage faster solutions
        let depth_penalty = depth * 100; // Prefer shallower wins
        let adjusted_score = if tactical_score > 0 { 
            tactical_score - depth_penalty 
        } else { 
            tactical_score + depth_penalty 
        };

        // PRIORITY 3: Positional evaluation (only if no major threats)
        if adjusted_score.abs() < URGENT_THREAT_SCORE {
            adjusted_score + Self::evaluate_all_patterns(state) / 10 // Reduce positional weight
        } else {
            adjusted_score
        }
    }

    // Detect threats that must be addressed in the next 1-2 moves
    fn evaluate_immediate_threats(state: &GameState, depth: i32) -> i32 {
        let mut max_score = 0;
        let mut min_score = 0;
        
        // Check all empty positions for immediate threats
        for row in 0..state.board.size() {
            for col in 0..state.board.size() {
                if state.board.is_empty_position(row, col) {
                    // Test move for Max player
                    let max_threat = Self::evaluate_move_urgency(&state.board, row, col, Player::Max, depth);
                    max_score = max_score.max(max_threat);
                    
                    // Test move for Min player  
                    let min_threat = Self::evaluate_move_urgency(&state.board, row, col, Player::Min, depth);
                    min_score = min_score.min(-min_threat);
                }
            }
        }
        
        // Return the most urgent threat
        if max_score >= IMMEDIATE_WIN_SCORE || min_score <= -IMMEDIATE_WIN_SCORE {
            return if max_score >= IMMEDIATE_WIN_SCORE { max_score } else { min_score };
        }
        
        // Check for opponent threats that must be blocked
        if (-min_score) >= IMMEDIATE_THREAT_SCORE {
            return min_score + IMMEDIATE_THREAT_SCORE; // Huge penalty for not blocking
        }
        
        max_score + min_score
    }

    // Evaluate how urgent a move is (immediate win, immediate threat, etc.)
    fn evaluate_move_urgency(board: &Board, row: usize, col: usize, player: Player, depth: i32) -> i32 {
        let mut urgency = 0;
        
        // Check all directions for immediate patterns
        for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
            let line_urgency = Self::evaluate_line_urgency(board, row, col, dx, dy, player, depth);
            urgency = urgency.max(line_urgency);
        }
        
        urgency
    }

    // Evaluate urgency of a line formation
    fn evaluate_line_urgency(board: &Board, row: usize, col: usize, dx: isize, dy: isize, player: Player, depth: i32) -> i32 {
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        let total_pieces = backwards + forwards;
        
        let back_blocked = Self::is_position_blocked(board, row, col, -dx, -dy, backwards + 1, player);
        let forward_blocked = Self::is_position_blocked(board, row, col, dx, dy, forwards + 1, player);
        
        // Urgency based on immediate threat level
        match total_pieces {
            4 => IMMEDIATE_WIN_SCORE - depth * 1000, // Immediate win - highest priority
            3 => {
                if !back_blocked || !forward_blocked {
                    IMMEDIATE_THREAT_SCORE - depth * 500 // Open 4-threat - must address NOW
                } else {
                    URGENT_THREAT_SCORE / 2 // Blocked 4 - still serious
                }
            },
            2 => {
                if !back_blocked && !forward_blocked {
                    URGENT_THREAT_SCORE / 4 // Open 3 - building threat
                } else {
                    TACTICAL_THREAT_SCORE / 2 // Blocked 3 - tactical value
                }
            },
            1 => {
                if !back_blocked && !forward_blocked {
                    TACTICAL_THREAT_SCORE / 4 // Open 2 - building
                } else {
                    100 // Basic value
                }
            },
            _ => 50,
        }
    }

    // Much better heuristic that actually understands Gomoku tactics
    fn fast_evaluate(state: &GameState, _depth: i32) -> i32 {
        let mut score = 0;
        
        // Check for immediate tactical threats (most important)
        score += Self::evaluate_tactical_threats(state);
        
        // Evaluate patterns for both players
        score += Self::evaluate_all_patterns(state);
        
        // Add capture advantage
        score += (state.max_captures as i32 - state.min_captures as i32) * 200;
        
        // Small positional bonus for center control
        score += Self::evaluate_center_control(state);
        
        score
    }
    
    fn evaluate_tactical_threats(state: &GameState) -> i32 {
        let mut score = 0;
        
        // Check all positions for immediate threats
        for row in 0..state.board.size() {
            for col in 0..state.board.size() {
                if state.board.is_empty_position(row, col) {
                    // Check what happens if each player plays here
                    let max_threat = Self::evaluate_move_strength(&state.board, row, col, Player::Max);
                    let min_threat = Self::evaluate_move_strength(&state.board, row, col, Player::Min);
                    
                    // Heavily prioritize blocking opponent immediate threats
                    let opponent_blocking_bonus = if min_threat >= 100000 { 150000 } else { 0 }; // Block wins
                    let opponent_threat_bonus = if min_threat >= 20000 { 25000 } else { 0 };     // Block strong threats
                    
                    // AI's own threats are important but blocking is more important
                    score += max_threat - (min_threat + opponent_blocking_bonus + opponent_threat_bonus);
                }
            }
        }
        
        score
    }
    
    fn evaluate_move_strength(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut strength = 0;
        
        // Check all directions for potential lines
        for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
            let line_strength = Self::evaluate_line_through_position(board, row, col, dx, dy, player);
            strength += line_strength;
        }
        
        strength
    }
    
    fn evaluate_line_through_position(board: &Board, row: usize, col: usize, dx: isize, dy: isize, player: Player) -> i32 {
        // Count pieces in both directions
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        let total_pieces = backwards + forwards;
        
        // Check if the line can be extended (not blocked by opponent)
        let back_blocked = Self::is_position_blocked(board, row, col, -dx, -dy, backwards + 1, player);
        let forward_blocked = Self::is_position_blocked(board, row, col, dx, dy, forwards + 1, player);
        
        // Much more aggressive scoring to prioritize line building/blocking
        let base_score = match total_pieces {
            4 => 100000, // Immediate win - highest priority
            3 => if back_blocked && forward_blocked { 500 } else { 20000 }, // Very strong threat - must block!
            2 => if back_blocked && forward_blocked { 100 } else { 2000 },   // Strong threat - important to block
            1 => if back_blocked && forward_blocked { 20 } else { 200 },     // Building position
            0 => if back_blocked && forward_blocked { 5 } else { 50 },       // Potential
            _ => 0,
        };
        
        // Massive bonus for open lines (can extend in both directions)
        if !back_blocked && !forward_blocked {
            match total_pieces {
                3 => base_score * 5, // Open 3-in-a-row is extremely dangerous
                2 => base_score * 3, // Open 2-in-a-row is quite dangerous
                1 => base_score * 2, // Open 1-in-a-row has potential
                _ => base_score,
            }
        } else {
            base_score
        }
    }
    
    fn is_position_blocked(board: &Board, row: usize, col: usize, dx: isize, dy: isize, distance: usize, player: Player) -> bool {
        let check_row = row as isize + dx * distance as isize;
        let check_col = col as isize + dy * distance as isize;
        
        if check_row < 0 || check_row >= board.size() as isize || 
           check_col < 0 || check_col >= board.size() as isize {
            return true; // Board edge
        }
        
        match board.get_player(check_row as usize, check_col as usize) {
            Some(p) if p != player => true, // Blocked by opponent
            _ => false, // Empty or our piece
        }
    }
    
    fn evaluate_all_patterns(state: &GameState) -> i32 {
        let mut score = 0;
        
        // Look for existing patterns on the board
        for row in 0..state.board.size() {
            for col in 0..state.board.size() {
                if let Some(player) = state.board.get_player(row, col) {
                    let pattern_value = Self::evaluate_patterns_from_position(&state.board, row, col, player);
                    match player {
                        Player::Max => score += pattern_value,
                        Player::Min => score -= pattern_value,
                    }
                }
            }
        }
        
        score
    }
    
    fn evaluate_patterns_from_position(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut value = 0;
        
        // Check all directions for patterns
        for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
            let pattern_value = Self::get_pattern_value_in_direction(board, row, col, dx, dy, player);
            value += pattern_value;
        }
        
        value
    }
    
    fn get_pattern_value_in_direction(board: &Board, row: usize, col: usize, dx: isize, dy: isize, player: Player) -> i32 {
        // Count consecutive pieces starting from this position
        let consecutive = Self::count_direction(board, row, col, dx, dy, player) + 1;
        
        if consecutive < 2 { return 0; }
        
        // Check if pattern is blocked
        let start_blocked = Self::is_position_blocked(board, row, col, -dx, -dy, 1, player);
        let end_blocked = Self::is_position_blocked(board, row, col, dx, dy, consecutive, player);
        
        match consecutive {
            5 => 100000, // Five in a row
            4 => if start_blocked && end_blocked { 200 } else { 10000 }, // Four in a row
            3 => if start_blocked && end_blocked { 50 } else { 1000 },   // Three in a row
            2 => if start_blocked && end_blocked { 10 } else { 100 },    // Two in a row
            _ => 0,
        }
    }
    
    fn evaluate_center_control(state: &GameState) -> i32 {
        let center = state.board.size() / 2;
        let mut score = 0;
        
        // Small bonus for pieces near center
        for row in 0..state.board.size() {
            for col in 0..state.board.size() {
                if let Some(player) = state.board.get_player(row, col) {
                    let distance = Self::manhattan_distance(row, col, center, center);
                    let bonus = (10 - distance.min(10)) as i32;
                    match player {
                        Player::Max => score += bonus,
                        Player::Min => score -= bonus,
                    }
                }
            }
        }
        
        score
    }

    fn analyze_both_players(board: &Board, win_condition: usize) -> (PatternCounts, PatternCounts) {
        let mut max_counts = PatternCounts::new();
        let mut min_counts = PatternCounts::new();
        let mut analyzed = vec![vec![0u8; board.size()]; board.size()];

        for row in 0..board.size() {
            for col in 0..board.size() {
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
        let max_row = board.size() as isize;
        let max_col = board.size() as isize;

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
                && prev_row < board.size() as isize
                && prev_col >= 0
                && prev_col < board.size() as isize
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
            && row < board.size() as isize
            && col >= 0
            && col < board.size() as isize
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
            score += IMMEDIATE_WIN_SCORE; // Use immediate win score for 5-in-row
        }

        score += match counts.live_four {
            1 => LIVE_FOUR_SCORE,
            n if n > 1 => DOUBLE_THREAT_SCORE, // Multiple live fours = fork
            _ => 0,
        };

        if counts.live_three >= 2
            || counts.dead_four >= 2
            || (counts.dead_four >= 1 && counts.live_three >= 1)
        {
            score += URGENT_THREAT_SCORE; // Urgent tactical threat
        }

        score += (counts.dead_four as i32) * DEAD_FOUR_SCORE
            + (counts.live_three as i32) * LIVE_THREE_SCORE
            + (counts.dead_three as i32) * DEAD_THREE_SCORE
            + (counts.live_two as i32) * LIVE_TWO_SCORE;

        score
    }

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        let center = state.board.size() / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
    }

    fn calculate_move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        let center_distance = Self::manhattan_distance(row, col, center, center);
        priority += 100 - center_distance as i32;

        priority += Self::calculate_threat_priority(&state.board, row, col);
        priority += Self::calculate_adjacency_bonus(&state.board, row, col);

        priority
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
            && current_row < board.size() as isize
            && current_col >= 0
            && current_col < board.size() as isize
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
                && new_row < board.size() as isize
                && new_col >= 0
                && new_col < board.size() as isize
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

    fn calculate_capture_bonus(state: &GameState) -> i32 {
        (state.max_captures as i32 - state.min_captures as i32) * CAPTURE_BONUS_MULTIPLIER
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
