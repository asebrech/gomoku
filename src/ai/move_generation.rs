use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::rules::DoubleThreeDetection;
use crate::ai::heuristic::{
    FIVE_IN_ROW_SCORE, LIVE_FOUR_SINGLE_SCORE, 
    HALF_FREE_FOUR_SCORE, DEAD_FOUR_SCORE, LIVE_THREE_SCORE, 
    HALF_FREE_THREE_SCORE, DEAD_THREE_SCORE, LIVE_TWO_SCORE, HALF_FREE_TWO_SCORE
};
use std::collections::HashSet;

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
        
        let zone_moves = Self::get_zone_based_moves(board, player);
        let legal_zone_moves = Self::filter_double_three_moves(board, zone_moves, player);
        
        if legal_zone_moves.is_empty() {
            return Self::find_any_legal_move(board, player);
        }
        
        legal_zone_moves
    }

    fn find_all_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut all_moves: HashSet<(usize, usize)> = HashSet::new();
        
        for &check_player in &[player, player.opponent()] {
            let consecutive_threats = Self::find_consecutive_threats(board, check_player);
            all_moves.extend(consecutive_threats);
            
            let gapped_threats = Self::find_gapped_threats(board, check_player);
            all_moves.extend(gapped_threats);
        }
        
        let mut moves_with_priority: Vec<((usize, usize), i32)> = all_moves
            .into_iter()
            .map(|mv| {
                let priority = Self::calculate_threat_priority(board, mv, player);
                (mv, priority)
            })
            .collect();
        
        moves_with_priority.sort_by_key(|(_, priority)| -priority);
        
        moves_with_priority.into_iter().map(|(mv, _)| mv).collect()
    }
    
    fn find_consecutive_threats(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);
        
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                let total = backward + forward + 1;
                
                if total >= 2 && total <= 4 {
                    for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                        let r = row as isize + dx * offset;
                        let c = col as isize + dy * offset;
                        if PatternAnalyzer::is_valid_empty(board, r, c) {
                            threats.insert((r as usize, c as usize));
                        }
                    }
                }
            }
        });
        
        threats
    }

    pub fn find_gapped_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
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
                
                if stones_found.len() >= 3 {
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
                            for pos in threat_positions {
                                threats.insert(pos);
                            }
                        }
                    }
                }
            }
        });
        threats.into_iter().collect()
    }

    fn calculate_threat_priority(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;
        for &check_player in &[player, player.opponent()] {
            priority += Self::calculate_player_threat_value(board, row, col, check_player);
        }
        priority
    }

    fn calculate_player_threat_value(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut max_value = 0;
        for &(dx, dy) in &DIRECTIONS {
            let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
            let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            let total_stones = backward + forward + 1;
            if total_stones < 2 {
                continue;
            }
            let pattern_start_row = row as isize - dx * backward as isize;
            let pattern_start_col = col as isize - dy * backward as isize;
            let total_space = PatternAnalyzer::count_total_space(
                board,
                pattern_start_row as usize,
                pattern_start_col as usize,
                dx,
                dy,
                total_stones,
            );
            if total_space < 5 { 
                continue;
            }
            let pattern_start_row = row as isize - dx * backward as isize;
            let pattern_start_col = col as isize - dy * backward as isize;
            let freedom = PatternAnalyzer::analyze_pattern_freedom(
                board,
                pattern_start_row as usize,
                pattern_start_col as usize,
                dx,
                dy,
                total_stones,
            );
            let pattern_value = get_pattern_score(total_stones, freedom);
            max_value = max_value.max(pattern_value);
        }
        max_value
    }

    fn get_zone_based_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut candidates = HashSet::new();
        let stone_count = board.count_stones();
        let zone_radius = if stone_count < 10 { 3 } else { 2 };
        let max_zone_moves = if stone_count < 10 { 25 } else { 20 };
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
        let mut filtered_moves: Vec<(usize, usize)> = candidates.into_iter().collect();
        filtered_moves.sort_by_key(|&mv| -Self::calculate_threat_priority(board, mv, player));
        filtered_moves.truncate(max_zone_moves);
        filtered_moves
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
        5 => FIVE_IN_ROW_SCORE,
        4 => match freedom {
            PatternFreedom::Free => LIVE_FOUR_SINGLE_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_FOUR_SCORE,
            PatternFreedom::Flanked => DEAD_FOUR_SCORE,
        },
        3 => match freedom {
            PatternFreedom::Free => LIVE_THREE_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_THREE_SCORE,
            PatternFreedom::Flanked => DEAD_THREE_SCORE,
        },
        2 => match freedom {
            PatternFreedom::Free => LIVE_TWO_SCORE,
            PatternFreedom::HalfFree => HALF_FREE_TWO_SCORE,
            PatternFreedom::Flanked => 0,
        },
        _ => 0,
    }
}
