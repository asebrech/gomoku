use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer};
use crate::core::rules::DoubleThreeDetection;
use crate::core::captures::CaptureHandler;
use crate::ai::heuristic::{score_consecutive_pattern, score_gapped_pattern, CAPTURE_BONUS_MULTIPLIER};

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
                // Use the unified pattern analyzer
                let pattern_info = PatternAnalyzer::analyze_consecutive_from_position(
                    board,
                    row,
                    col,
                    dx,
                    dy,
                    player,
                    5, // win_condition
                );
                
                let Some((length, _pattern_start_row, _pattern_start_col, _total_space, freedom)) = pattern_info else {
                    continue;
                };
                
                // Only consider patterns of length 2-4 for move generation
                if length < 2 || length > 4 {
                    continue;
                }
                
                // Skip flanked patterns - they score 0 and are useless for move generation
                if freedom == crate::core::patterns::PatternFreedom::Flanked {
                    continue;
                }
                
                let score = score_consecutive_pattern(length, freedom);
                
                // Calculate backward and forward distances for move placement
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                
                // Add score to each empty position around this pattern
                for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                    let r = row as isize + dx * offset;
                    let c = col as isize + dy * offset;
                    if PatternAnalyzer::is_valid_empty(board, r, c) {
                        let pos = (r as usize, c as usize);
                        let mut move_score = score;
                        
                        // Check if this move would create a capture and add bonus
                        if Self::would_capture(board, r as usize, c as usize, player) {
                            move_score += CAPTURE_BONUS_MULTIPLIER;
                        }
                        
                        *move_scores.entry(pos).or_insert(0) += move_score;
                    }
                }
            }
        });
    }
    
    /// Check if placing a stone at (row, col) would create any captures
    /// Uses CaptureHandler to simulate the move
    fn would_capture(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
    ) -> bool {
        // Simulate placing the stone temporarily
        let mut temp_board = board.clone();
        let idx = temp_board.index(row, col);
        
        // Place the stone
        Board::set_bit(&mut temp_board.occupied, idx);
        match player {
            Player::Max => Board::set_bit(&mut temp_board.max_bits, idx),
            Player::Min => Board::set_bit(&mut temp_board.min_bits, idx),
        }
        
        // Check if this creates any captures
        let captures = CaptureHandler::detect_captures(&temp_board, row, col, player);
        !captures.is_empty()
    }

    fn collect_gapped_threats(
        board: &Board,
        player: Player,
        move_scores: &mut std::collections::HashMap<(usize, usize), i32>,
    ) {
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                // Use unified gapped stone collection
                let stones = PatternAnalyzer::collect_gapped_stones(board, row, col, dx, dy, player, 6);
                
                // Use unified gapped pattern analysis
                let Some((stone_count, gaps, _span)) = PatternAnalyzer::analyze_gapped_pattern(&stones) else {
                    continue;
                };
                
                // Extract gap positions for move scoring
                let threat_positions = PatternAnalyzer::extract_gap_positions(board, &stones, dx, dy);
                
                // Calculate score and add to each gap position
                let score = score_gapped_pattern(stone_count, gaps);
                for pos in threat_positions {
                    *move_scores.entry(pos).or_insert(0) += score;
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
