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
    five_in_row: i32,
    live_four: i32,
    dead_four: i32,
    live_three: i32,
    dead_three: i32,
    live_two: i32,
}

impl PatternCounts {
    fn new() -> Self {
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

impl Heuristic {
    pub fn evaluate(state: &GameState, depth: i32) -> i32 {
        if let Some(terminal_score) = Self::evaluate_terminal_states(state, depth) {
            return terminal_score;
        }

        if Self::is_board_full(&state.board) {
            return 0;
        }

        let max_score =
            Self::evaluate_player_position(&state.board, Player::Max, state.win_condition);
        let min_score =
            Self::evaluate_player_position(&state.board, Player::Min, state.win_condition);

        let capture_bonus = Self::calculate_capture_bonus(state);

        max_score - min_score + capture_bonus
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

    fn evaluate_player_position(board: &Board, player: Player, win_condition: usize) -> i32 {
        let pattern_counts = Self::analyze_all_patterns(board, player, win_condition);
        Self::calculate_pattern_score(pattern_counts)
    }

    fn analyze_all_patterns(board: &Board, player: Player, win_condition: usize) -> PatternCounts {
        let mut counts = PatternCounts::new();

        // Analyze each direction separately to avoid double counting
        for &(dx, dy) in &DIRECTIONS {
            let direction_counts =
                Self::analyze_patterns_in_direction(board, player, win_condition, dx, dy);
            Self::merge_pattern_counts(&mut counts, direction_counts);
        }

        counts
    }

    fn analyze_patterns_in_direction(
        board: &Board,
        player: Player,
        win_condition: usize,
        dx: isize,
        dy: isize,
    ) -> PatternCounts {
        let mut counts = PatternCounts::new();
        let mut analyzed = vec![vec![false; board.size]; board.size];

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col) == Some(player) && !analyzed[row][col] {
                    if let Some(pattern_info) = Self::analyze_pattern_from_position(
                        board,
                        row,
                        col,
                        dx,
                        dy,
                        player,
                        win_condition,
                        &mut analyzed,
                    ) {
                        Self::update_counts_for_pattern(&mut counts, pattern_info);
                    }
                }
            }
        }

        counts
    }

    fn analyze_pattern_from_position(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
        analyzed: &mut Vec<Vec<bool>>,
    ) -> Option<PatternInfo> {
        // Always analyze from the actual pattern start to avoid duplicates
        let (pattern_start_row, pattern_start_col) =
            Self::find_pattern_start(board, start_row, start_col, dx, dy, player);

        if analyzed[pattern_start_row][pattern_start_col] {
            return None;
        }

        let length = Self::count_consecutive_pieces(
            board,
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            player,
        );

        if length < 2 {
            return None;
        }

        let length = length.min(win_condition);
        let is_live =
            Self::is_pattern_live(board, pattern_start_row, pattern_start_col, dx, dy, player);

        Self::mark_pattern_analyzed(
            pattern_start_row,
            pattern_start_col,
            dx,
            dy,
            length,
            analyzed,
        );

        Some(PatternInfo { length, is_live })
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

        // Move backwards to find the start of the pattern
        while Self::is_valid_position(board, current_row - dx, current_col - dy)
            && board.get_player((current_row - dx) as usize, (current_col - dy) as usize)
                == Some(player)
        {
            current_row -= dx;
            current_col -= dy;
        }

        (current_row as usize, current_col as usize)
    }

    fn count_consecutive_pieces(
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

        // Count consecutive pieces in positive direction only
        while Self::is_valid_position(board, current_row, current_col)
            && board.get_player(current_row as usize, current_col as usize) == Some(player)
        {
            count += 1;
            current_row += dx;
            current_col += dy;
        }

        count
    }

    fn is_pattern_live(
        board: &Board,
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> bool {
        // Check if pattern has open ends (both sides empty)
        let before_row = start_row as isize - dx;
        let before_col = start_col as isize - dy;
        let start_open = Self::is_position_empty(board, before_row, before_col);

        let mut end_row = start_row as isize;
        let mut end_col = start_col as isize;
        while Self::is_valid_position(board, end_row, end_col)
            && board.get_player(end_row as usize, end_col as usize) == Some(player)
        {
            end_row += dx;
            end_col += dy;
        }

        let end_open = Self::is_position_empty(board, end_row, end_col);

        start_open && end_open
    }

    fn is_valid_position(board: &Board, row: isize, col: isize) -> bool {
        row >= 0 && row < board.size as isize && col >= 0 && col < board.size as isize
    }

    fn is_position_empty(board: &Board, row: isize, col: isize) -> bool {
        Self::is_valid_position(board, row, col)
            && board.get_player(row as usize, col as usize).is_none()
    }

    fn mark_pattern_analyzed(
        start_row: usize,
        start_col: usize,
        dx: isize,
        dy: isize,
        length: usize,
        analyzed: &mut Vec<Vec<bool>>,
    ) {
        for i in 0..length {
            let row = (start_row as isize + i as isize * dx) as usize;
            let col = (start_col as isize + i as isize * dy) as usize;
            if row < analyzed.len() && col < analyzed[0].len() {
                analyzed[row][col] = true;
            }
        }
    }

    fn merge_pattern_counts(total: &mut PatternCounts, direction_counts: PatternCounts) {
        total.five_in_row += direction_counts.five_in_row;
        total.live_four += direction_counts.live_four;
        total.dead_four += direction_counts.dead_four;
        total.live_three += direction_counts.live_three;
        total.dead_three += direction_counts.dead_three;
        total.live_two += direction_counts.live_two;
    }

    fn update_counts_for_pattern(counts: &mut PatternCounts, pattern: PatternInfo) {
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

        if counts.live_four == 1 {
            score += LIVE_FOUR_SINGLE_SCORE;
        } else if counts.live_four > 1 {
            score += LIVE_FOUR_MULTIPLE_SCORE;
        }

        // Winning threats: multiple live threes, dead fours, or combination
        if counts.live_three >= 2
            || counts.dead_four >= 2
            || (counts.dead_four >= 1 && counts.live_three >= 1)
        {
            score += WINNING_THREAT_SCORE;
        }

        score += counts.dead_four * DEAD_FOUR_SCORE;
        score += counts.live_three * LIVE_THREE_SCORE;
        score += counts.dead_three * DEAD_THREE_SCORE;
        score += counts.live_two * LIVE_TWO_SCORE;

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

        let adjacency_bonus = Self::calculate_adjacency_bonus(&state.board, row, col);
        priority += adjacency_bonus;

        priority
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }

    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize) -> i32 {
        let mut bonus = 0;
        for &(dx, dy) in &ALL_DIRECTIONS {
            let new_row = row as isize + dx;
            let new_col = col as isize + dy;

            if Self::is_valid_position(board, new_row, new_col)
                && board
                    .get_player(new_row as usize, new_col as usize)
                    .is_some()
            {
                bonus += 50;
            }
        }
        bonus
    }
}

#[derive(Debug, Clone, Copy)]
struct PatternInfo {
    length: usize,
    is_live: bool,
}
