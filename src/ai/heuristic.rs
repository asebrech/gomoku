use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct Heuristic;

const WINNING_SCORE: i32 = 1_000_000;
const FIVE_IN_ROW_SCORE: i32 = 100_000;
const LIVE_FOUR_SINGLE_SCORE: i32 = 15_000;
const LIVE_FOUR_MULTIPLE_SCORE: i32 = 20_000;
const WINNING_THREAT_SCORE: i32 = 10_000;
const DEAD_FOUR_SCORE: i32 = 1_000;
const LIVE_THREE_SCORE: i32 = 500;
const DEAD_THREE_SCORE: i32 = 100;
const LIVE_TWO_SCORE: i32 = 50;
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

        // FAST HEURISTIC: Only evaluate around placed stones instead of full board scan
        Self::fast_evaluate(state, depth)
    }

    // Much faster heuristic that only looks at meaningful positions
    fn fast_evaluate(state: &GameState, _depth: i32) -> i32 {
        let mut score = 0;
        
        // Simple material count + positional bonus
        let center = state.board.size() / 2;
        
        // Quick scan - much faster than full pattern analysis
        for row in 0..state.board.size() {
            for col in 0..state.board.size() {
                if let Some(player) = state.board.get_player(row, col) {
                    let distance_from_center = ((row as i32 - center as i32).abs() + (col as i32 - center as i32).abs()) as i32;
                    let positional_bonus = 10 - distance_from_center.min(10);
                    
                    match player {
                        Player::Max => {
                            score += 10 + positional_bonus;
                        },
                        Player::Min => {
                            score -= 10 + positional_bonus;
                        }
                    }
                }
            }
        }
        
        // Add capture bonus
        score += (state.max_captures as i32 - state.min_captures as i32) * 50;
        
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
