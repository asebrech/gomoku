use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::rules::DoubleThreeDetection;
use crate::ai::heuristic::{
    WINNING_SCORE, LIVE_FOUR_SCORE, HALF_FREE_FOUR_SCORE,
    LIVE_THREE_SCORE, HALF_FREE_THREE_SCORE,
    LIVE_TWO_SCORE, HALF_FREE_TWO_SCORE
};

pub struct MoveGenerator;
impl MoveGenerator {
    pub fn get_candidate_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }
        
        let threat_moves = Self::find_all_threat_moves(board, player);
        if !threat_moves.is_empty() {
            let legal_threats = Self::filter_double_three_moves(board, threat_moves, player);
            if !legal_threats.is_empty() {
                return legal_threats;
            }
        }
        
        Self::find_any_legal_move(board, player)
    }

    fn find_all_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        use std::collections::HashMap;
        let mut move_scores: HashMap<(usize, usize), i32> = HashMap::new();
        
        // Collect all threats with their scores from both players
        for &check_player in &[player, player.opponent()] {
            Self::collect_consecutive_threats(board, check_player, &mut move_scores);
            Self::collect_gapped_threats(board, check_player, &mut move_scores);
            Self::collect_capture_moves(board, check_player, &mut move_scores);
        }
        
        // If no threats found, use zone-based moves
        if move_scores.is_empty() {
            let zone_moves = Self::get_zone_based_moves(board, player);
            return zone_moves;
        }
        
        // Sort by score (highest first)
        let mut moves: Vec<((usize, usize), i32)> = move_scores.into_iter().collect();
        moves.sort_by_key(|(_, score)| -score);
        
        moves.into_iter().map(|(mv, _)| mv).collect()
    }
    
    fn collect_consecutive_threats(
        board: &Board,
        player: Player,
        move_scores: &mut std::collections::HashMap<(usize, usize), i32>,
    ) {
        let player_bits = board.get_player_bits(player);
        
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                let total = backward + forward + 1;
                
                if total >= 2 && total <= 4 {
                    // Calculate pattern score once
                    let pattern_start_row = row as isize - dx * backward as isize;
                    let pattern_start_col = col as isize - dy * backward as isize;
                    let total_space = PatternAnalyzer::count_total_space(
                        board,
                        pattern_start_row as usize,
                        pattern_start_col as usize,
                        dx,
                        dy,
                        total,
                    );
                    
                    if total_space >= 5 {
                        let freedom = PatternAnalyzer::analyze_pattern_freedom(
                            board,
                            pattern_start_row as usize,
                            pattern_start_col as usize,
                            dx,
                            dy,
                            total,
                        );
                        let score = get_pattern_score(total, freedom);
                        
                        // Add score to each empty position around this pattern
                        for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                            let r = row as isize + dx * offset;
                            let c = col as isize + dy * offset;
                            if PatternAnalyzer::is_valid_empty(board, r, c) {
                                let pos = (r as usize, c as usize);
                                *move_scores.entry(pos).or_insert(0) += score;
                            }
                        }
                    }
                }
            }
        });
    }

    fn collect_gapped_threats(
        board: &Board,
        player: Player,
        move_scores: &mut std::collections::HashMap<(usize, usize), i32>,
    ) {
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let mut stones_found = vec![(row, col)];
                for dist in 1..=6 {
                    let check_row = row as isize + dx * dist;
                    let check_col = col as isize + dy * dist;
                    if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                        let idx = board.index(check_row as usize, check_col as usize);
                        if Board::is_bit_set(&player_bits, idx) {
                            stones_found.push((check_row as usize, check_col as usize));
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
                backward_stones.append(&mut stones_found);
                stones_found = backward_stones;
                
                if stones_found.len() >= 2 {
                    let first = stones_found.first().unwrap();
                    let last = stones_found.last().unwrap();
                    let start_row = first.0 as isize;
                    let start_col = first.1 as isize;
                    let end_row = last.0 as isize;
                    let end_col = last.1 as isize;
                    let total_span = ((end_row - start_row).abs().max((end_col - start_col).abs())) + 1;
                    if total_span <= 7 {
                        let steps = ((end_row - start_row) / dx.max(1)).max((end_col - start_col) / dy.max(1));
                        let mut empty_gaps = 0;
                        let mut threat_positions = Vec::new();
                        for step in 1..steps {
                            let gap_row = start_row + dx * step;
                            let gap_col = start_col + dy * step;
                            if PatternAnalyzer::is_valid_empty(board, gap_row, gap_col) {
                                empty_gaps += 1;
                                threat_positions.push((gap_row as usize, gap_col as usize));
                            }
                        }
                        if empty_gaps > 0 && stones_found.len() + empty_gaps >= 4 && empty_gaps <= 3 {
                            // Calculate gapped pattern score once
                            let score = score_gapped_pattern(stones_found.len(), empty_gaps);
                            
                            for pos in threat_positions {
                                *move_scores.entry(pos).or_insert(0) += score;
                            }
                        }
                    }
                }
            }
        });
    }

    fn collect_capture_moves(
        board: &Board,
        player: Player,
        move_scores: &mut std::collections::HashMap<(usize, usize), i32>,
    ) {
        let opponent = player.opponent();
        let opponent_bits = board.get_player_bits(opponent);
        let player_bits = board.get_player_bits(player);
        const CAPTURE_MOVE_SCORE: i32 = 5000; // Score per capture opportunity
        
        // Look for opponent stones that could be captured
        board.iterate_bits(opponent_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                // Check if there's another opponent stone in this direction
                let next_row = row as isize + dx;
                let next_col = col as isize + dy;
                
                if !PatternAnalyzer::is_in_bounds(board, next_row, next_col) {
                    continue;
                }
                
                let next_idx = board.index(next_row as usize, next_col as usize);
                if !Board::is_bit_set(opponent_bits, next_idx) {
                    continue;
                }
                
                // Check both ends to see if placing a stone would create a capture
                // Check before the first stone
                let before_row = row as isize - dx;
                let before_col = col as isize - dy;
                if PatternAnalyzer::is_valid_empty(board, before_row, before_col) {
                    // Check if there's already a player stone after the second opponent stone
                    let after_row = next_row + dx;
                    let after_col = next_col + dy;
                    if PatternAnalyzer::is_in_bounds(board, after_row, after_col) {
                        let after_idx = board.index(after_row as usize, after_col as usize);
                        if Board::is_bit_set(player_bits, after_idx) {
                            let pos = (before_row as usize, before_col as usize);
                            *move_scores.entry(pos).or_insert(0) += CAPTURE_MOVE_SCORE;
                        }
                    }
                }
                
                // Check after the second stone
                let after_row = next_row + dx;
                let after_col = next_col + dy;
                if PatternAnalyzer::is_valid_empty(board, after_row, after_col) {
                    // Check if there's already a player stone before the first opponent stone
                    let before_row = row as isize - dx;
                    let before_col = col as isize - dy;
                    if PatternAnalyzer::is_in_bounds(board, before_row, before_col) {
                        let before_idx = board.index(before_row as usize, before_col as usize);
                        if Board::is_bit_set(player_bits, before_idx) {
                            let pos = (after_row as usize, after_col as usize);
                            *move_scores.entry(pos).or_insert(0) += CAPTURE_MOVE_SCORE;
                        }
                    }
                }
            }
        });
    }



    fn get_zone_based_moves(board: &Board, _player: Player) -> Vec<(usize, usize)> {
        let mut candidates = std::collections::HashSet::new();
        let zone_radius = 2;
        
        board.iterate_bits(&board.occupied, |row, col| {
            for dr in -(zone_radius as isize)..=(zone_radius as isize) {
                for dc in -(zone_radius as isize)..=(zone_radius as isize) {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if PatternAnalyzer::is_valid_empty(board, nr, nc) {
                        candidates.insert((nr as usize, nc as usize));
                    }
                }
            }
        });
        
        candidates.into_iter().collect()
    }

    fn filter_double_three_moves(board: &Board, moves: Vec<(usize, usize)>, player: Player) -> Vec<(usize, usize)> {
        moves
            .into_iter()
            .filter(|(row, col)| !DoubleThreeDetection::creates_double_three(board, *row, *col, player))
            .collect()
    }

    fn find_any_legal_move(board: &Board, player: Player) -> Vec<(usize, usize)> {
        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col).is_none() 
                    && !DoubleThreeDetection::creates_double_three(board, row, col, player) {
                    return vec![(row, col)];
                }
            }
        }
        vec![]
    }
}

fn get_pattern_score(length: usize, freedom: PatternFreedom) -> i32 {
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

fn score_gapped_pattern(stones: usize, gaps: usize) -> i32 {
    use crate::ai::heuristic::{
        GAPPED_FOUR_SCORE, GAPPED_THREE_ONE_SCORE, GAPPED_THREE_TWO_SCORE,
        GAPPED_TWO_ONE_SCORE, GAPPED_TWO_TWO_SCORE, GAPPED_OTHER_SCORE
    };
    
    match (stones, gaps) {
        (4, 1) => GAPPED_FOUR_SCORE,
        (3, 1) => GAPPED_THREE_ONE_SCORE,
        (3, 2) => GAPPED_THREE_TWO_SCORE,
        (2, 1) => GAPPED_TWO_ONE_SCORE,
        (2, 2) => GAPPED_TWO_TWO_SCORE,
        _ => GAPPED_OTHER_SCORE,
    }
}
