use crate::core::board::{Board, Player};
use crate::core::state::GameState;
use crate::ai::patterns::get_pattern_table;

pub struct Heuristic;

impl Heuristic {
    pub fn evaluate(state: &GameState) -> i32 {
        // Check terminal states first
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => 1_000_000,
                Player::Min => -1_000_000,
            };
        }

        // Check capture wins
        if state.max_captures >= 5 { return 1_000_000; }
        if state.min_captures >= 5 { return -1_000_000; }

        // Use pattern-based evaluation
        let pattern_table = get_pattern_table();
        let mut score = 0;

        // Evaluate all lines using pattern lookup
        score += Self::evaluate_all_lines(&state.board, pattern_table);

        // Add capture bonus (increased from 2000 to 5000)
        score += (state.max_captures as i32 - state.min_captures as i32) * 5000;

        // Add capture opportunity bonus
        score += Self::evaluate_capture_opportunities(state);

        score
    }

    fn evaluate_all_lines(board: &Board, pattern_table: &crate::ai::patterns::PatternTable) -> i32 {
        let mut total_score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in &directions {
            for start_row in 0..board.size {
                for start_col in 0..board.size {
                    if Self::is_valid_line_start(start_row, start_col, dx, dy, board.size) {
                        let pattern = Self::extract_line_pattern(board, start_row, start_col, dx, dy);
                        total_score += pattern_table.lookup_pattern(pattern);
                    }
                }
            }
        }

        total_score
    }

    fn is_valid_line_start(row: usize, col: usize, dx: isize, dy: isize, size: usize) -> bool {
        let end_row = row as isize + 4 * dx;
        let end_col = col as isize + 4 * dy;
        
        end_row >= 0 && end_row < size as isize && 
        end_col >= 0 && end_col < size as isize
    }

    fn extract_line_pattern(board: &Board, start_row: usize, start_col: usize, dx: isize, dy: isize) -> u32 {
        let mut pattern = 0;
        let mut multiplier = 1;

        for i in 0..5 {
            let row = (start_row as isize + i * dx) as usize;
            let col = (start_col as isize + i * dy) as usize;
            
            let value = match board.get_player(row, col) {
                Some(Player::Max) => 1,
                Some(Player::Min) => 2,
                None => 0,
            };
            
            pattern += value * multiplier;
            multiplier *= 3;
        }

        pattern
    }

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        moves.sort_by_cached_key(|&(row, col)| {
            -(Self::move_priority(state, row, col) as i32)
        });
    }

    fn move_priority(state: &GameState, row: usize, col: usize) -> usize {
        use crate::core::captures::CaptureHandler;
        
        let mut priority = 0;
        let center = state.board.size / 2;

        // Check for blocking opponent's winning move (HIGHEST priority)
        // We need to check if the opponent would win if they played at this position
        let mut temp_state = state.clone();
        temp_state.current_player = state.current_player.opponent();
        temp_state.make_move((row, col));
        if temp_state.check_winner() == Some(state.current_player.opponent()) {
            priority += 10_000_000; // Extremely high priority for blocking moves
        }

        // Check for immediate winning move (second highest priority)
        let mut temp_state = state.clone();
        temp_state.make_move((row, col));
        if temp_state.check_winner() == Some(state.current_player) {
            priority += 1_000_000; // High priority for winning moves
        }

        // Check for capture opportunities (third highest priority)
        let captures = CaptureHandler::detect_captures(&state.board, row, col, state.current_player);
        if !captures.is_empty() {
            let capture_count = captures.len() / 2;
            priority += capture_count * 10000; // Very high priority for captures
            
            // Extra priority if this leads to capture win
            let current_captures = match state.current_player {
                Player::Max => state.max_captures,
                Player::Min => state.min_captures,
            };
            
            if current_captures + capture_count >= 5 {
                priority += 100000; // Extremely high priority for capture wins
            }
        }

        // Distance from center (inverse priority)
        let center_dist = ((row as isize - center as isize).abs() + 
                          (col as isize - center as isize).abs()) as usize;
        priority += (20 - center_dist.min(20)) * 10;

        // Count adjacent stones
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        for &(dx, dy) in &directions {
            let new_row = row as isize + dx;
            let new_col = col as isize + dy;
            
            if new_row >= 0 && new_row < state.board.size as isize &&
               new_col >= 0 && new_col < state.board.size as isize {
                if state.board.get_player(new_row as usize, new_col as usize).is_some() {
                    priority += 50;
                }
            }
        }

        priority
    }

    fn evaluate_capture_opportunities(state: &GameState) -> i32 {
        use crate::core::captures::CaptureHandler;
        
        let mut score = 0;
        
        // Get possible moves and evaluate capture opportunities for both players
        let moves = state.get_possible_moves();
        
        // Evaluate capture opportunities for Max player
        for &(row, col) in &moves {
            let max_captures = CaptureHandler::detect_captures(&state.board, row, col, Player::Max);
            if !max_captures.is_empty() {
                let capture_count = max_captures.len() / 2;
                let mut capture_bonus = capture_count as i32 * 3000; // Increased bonus
                
                // Extra bonus if this would lead to a capture win
                if state.max_captures + capture_count >= 5 {
                    capture_bonus += 900_000;
                }
                
                score += capture_bonus;
            }
            
            // Evaluate capture opportunities for Min player (subtract from score)
            let min_captures = CaptureHandler::detect_captures(&state.board, row, col, Player::Min);
            if !min_captures.is_empty() {
                let capture_count = min_captures.len() / 2;
                let mut capture_penalty = capture_count as i32 * 3000;
                
                // Extra penalty if this would lead to a capture win for Min
                if state.min_captures + capture_count >= 5 {
                    capture_penalty += 900_000;
                }
                
                score -= capture_penalty;
            }
        }
        
        score
    }
}
