use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::rules::DoubleThreeDetection;
use crate::ai::heuristic::{
    CAPTURE_BONUS_MULTIPLIER, FIVE_IN_ROW_SCORE, LIVE_FOUR_SINGLE_SCORE, 
    HALF_FREE_FOUR_SCORE, DEAD_FOUR_SCORE, LIVE_THREE_SCORE, 
    HALF_FREE_THREE_SCORE, DEAD_THREE_SCORE, LIVE_TWO_SCORE, HALF_FREE_TWO_SCORE
};
use std::collections::HashSet;

const CENTER_POSITION_BONUS: i32 = 10;

/// Prioritized move generator for Gomoku AI that reduces search space
/// by focusing on tactically relevant moves
pub struct MoveGenerator;
impl MoveGenerator {
    /// Generates prioritized candidate moves for the given player
    /// 
    /// Simplified priority order:
    /// 1. Winning moves (immediate five-in-a-row) - return single move
    /// 2. All threat moves (four-threats, three-threats, etc.) with smart filtering:
    ///    - If any four-threats exist, return ONLY four-threats
    ///    - Otherwise return all threats (prioritized)
    /// 3. Zone-based moves (around existing stones)
    /// 
    /// Returns empty board center if board is empty
    pub fn get_candidate_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }
        
        // 1. Check for immediate winning moves
        if let Some(winning_move) = Self::find_winning_move(board, player) {
            if !DoubleThreeDetection::creates_double_three(board, winning_move.0, winning_move.1, player) {
                return vec![winning_move];
            }
        }
        
        // 2. Find all threat moves (offensive + defensive, including gapped patterns)
        let threat_moves = Self::find_all_threat_moves(board, player);
        if !threat_moves.is_empty() {
            let legal_threats = Self::filter_double_three_moves(board, threat_moves, player);
            if !legal_threats.is_empty() {
                return legal_threats;
            }
        }
        
        // 3. Fall back to zone-based moves
        let zone_moves = Self::get_zone_based_moves(board, player);
        let legal_zone_moves = Self::filter_double_three_moves(board, zone_moves, player);
        
        if legal_zone_moves.is_empty() {
            return Self::find_any_legal_move(board, player);
        }
        
        legal_zone_moves
    }

    /// Finds a move that creates an immediate five-in-a-row win
    /// 
    /// Searches all player stones and checks if placing a stone in any direction
    /// would complete a winning line of five stones
    fn find_winning_move(board: &Board, player: Player) -> Option<(usize, usize)> {
        let player_bits = board.get_player_bits(player);
        let mut result = None;
        board.iterate_bits(player_bits, |row, col| {
            if result.is_some() {
                return;
            }
            for &(dx, dy) in &DIRECTIONS {
                if let Some(win_pos) = Self::find_win_in_direction(board, row, col, dx, dy, player)
                {
                    result = Some(win_pos);
                    return;
                }
            }
        });
        result
    }

    /// Searches for winning moves in a specific direction from a given stone
    /// 
    /// Checks positions at increasing distances from the stone to find
    /// empty squares that would complete a five-in-a-row when filled
    fn find_win_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> Option<(usize, usize)> {
        const PRIORITY_OFFSETS: &[isize] = &[-1, 1, -2, 2, -3, 3, -4, 4, -5, 5];
        for &offset in PRIORITY_OFFSETS {
            let check_row = row as isize + dx * offset;
            let check_col = col as isize + dy * offset;
            if PatternAnalyzer::is_valid_empty(board, check_row, check_col) {
                let pos = (check_row as usize, check_col as usize);
                if Self::creates_five_in_row(board, pos, player) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Checks if placing a stone at the given position creates five-in-a-row
    /// 
    /// Tests all eight directions to see if the new stone would complete
    /// a line of five consecutive stones
    fn creates_five_in_row(board: &Board, pos: (usize, usize), player: Player) -> bool {
        for &(dx, dy) in &DIRECTIONS {
            let total = PatternAnalyzer::count_consecutive_bidirectional(board, pos.0, pos.1, dx, dy, player);
            if total >= 5 {
                return true;
            }
        }
        false
    }

    /// Unified threat move finder that detects all tactical opportunities
    /// 
    /// Combines:
    /// - Offensive threats (our patterns)
    /// - Defensive threats (opponent patterns)
    /// - Standard consecutive patterns
    /// - Gapped patterns (X.X.X, XX.X, etc.)
    /// 
    /// Returns moves prioritized by threat level:
    /// - If any four-threats found, returns ONLY four-threats
    /// - Otherwise returns all threats sorted by priority
    fn find_all_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut all_moves: HashSet<(usize, usize)> = HashSet::new();
        
        // Collect threats for both players (offense + defense)
        for &check_player in &[player, player.opponent()] {
            // Standard consecutive threats
            let consecutive_threats = Self::find_consecutive_threats(board, check_player);
            all_moves.extend(consecutive_threats);
            
            // Gapped pattern threats
            let gapped_threats = Self::find_gapped_threats(board, check_player);
            all_moves.extend(gapped_threats);
        }
        
        // Convert to vec and prioritize
        let mut moves_with_priority: Vec<((usize, usize), i32, bool)> = all_moves
            .into_iter()
            .map(|mv| {
                let priority = Self::calculate_threat_priority(board, mv, player);
                let is_four_threat = Self::is_four_threat(board, mv, player);
                (mv, priority, is_four_threat)
            })
            .collect();
        
        // If we have any four-threats, ONLY return those (critical situations)
        let has_four_threats = moves_with_priority.iter().any(|(_, _, is_four)| *is_four);
        if has_four_threats {
            moves_with_priority.retain(|(_, _, is_four)| *is_four);
        }
        
        // Sort by priority (highest first)
        moves_with_priority.sort_by_key(|(_, priority, _)| -priority);
        
        // Limit to reasonable number
        let limit = if has_four_threats { 10 } else { 25 };
        moves_with_priority.truncate(limit);
        
        moves_with_priority.into_iter().map(|(mv, _, _)| mv).collect()
    }
    
    /// Finds threat moves from standard consecutive patterns
    fn find_consecutive_threats(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);
        
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                let total = backward + forward + 1;
                
                // Only consider meaningful patterns (2-4 stones)
                if total >= 2 && total <= 4 {
                    // Add all adjacent empty positions
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
    
    /// Checks if placing a move creates or blocks a four-threat
    fn is_four_threat(board: &Board, mv: (usize, usize), player: Player) -> bool {
        let (row, col) = mv;
        
        // Check both our four-threats and opponent's four-threats we'd be blocking
        for &check_player in &[player, player.opponent()] {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, check_player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, check_player);
                let total = backward + forward + 1;
                
                if total == 4 {
                    return true;
                }
            }
        }
        
        false
    }

    /// Finds positions that block opponent's gapped threats
    /// 
    /// Gapped threats are patterns like X.X.X or XX.X where stones are
    /// separated by gaps but could form five-in-a-row if gaps are filled
    pub fn find_gapped_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let mut stones_found = vec![(row, col)];
                // Look in positive direction (forward)
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
                // Look in negative direction (backward) and prepend to stones_found
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
                // Reverse backward stones and prepend them
                backward_stones.reverse();
                backward_stones.append(&mut stones_found);
                stones_found = backward_stones;
                
                // Require at least 3 stones for a meaningful gapped threat
                if stones_found.len() >= 3 {
                    let first = stones_found.first().unwrap();
                    let last = stones_found.last().unwrap();
                    let start_row = first.0 as isize;
                    let start_col = first.1 as isize;
                    let end_row = last.0 as isize;
                    let end_col = last.1 as isize;
                    // Fixed: use max instead of sum for span calculation
                    // This correctly handles diagonals where both dimensions change equally
                    let total_span = ((end_row - start_row).abs().max((end_col - start_col).abs())) + 1;
                    // Allow span up to 7 to catch patterns like X.X.X.X (4 stones with 3 gaps)
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
                        // Changed from >= 5 to >= 4: patterns like XX.X (4 total) are dangerous
                        // and should be detected as threats
                        // Allow up to 3 gaps for patterns like X.X.X.X
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



    /// Calculates priority score for a move based on multiple factors
    /// 
    /// Scoring components:
    /// - Pattern values (offensive moves weighted 2x, defensive 1x)
    /// - Capture bonus for moves creating capture opportunities
    /// - Small center position bonus decreasing with distance
    fn calculate_threat_priority(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;
        for &check_player in &[player, player.opponent()] {
            let player_priority = Self::calculate_player_threat_value(board, row, col, check_player);
            if check_player == player {
                priority += player_priority * 2; 
            } else {
                priority += player_priority; 
            }
        }
        let capture_bonus = Self::calculate_capture_bonus(board, row, col, player);
        priority += capture_bonus;
        let center = board.size / 2;
        let distance = Self::manhattan_distance(row, col, center, center) as i32;
        priority += CENTER_POSITION_BONUS - distance.min(CENTER_POSITION_BONUS);
        priority
    }

    /// Calculates the threat value of placing a stone for a specific player
    /// 
    /// Analyzes all directions to find the strongest pattern that would be
    /// created, considering pattern length and freedom of movement
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

    /// Calculates bonus points for moves that create capture opportunities
    /// 
    /// Searches all directions for patterns where placing a stone would
    /// create a capture situation (our stone - opponent - opponent - our stone)
    fn calculate_capture_bonus(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut bonus = 0;
        let opponent = player.opponent();
        for &(dx, dy) in &DIRECTIONS {
            let adj_row = row as isize + dx;
            let adj_col = col as isize + dy;
            if PatternAnalyzer::is_in_bounds(board, adj_row, adj_col) {
                let adj_row = adj_row as usize;
                let adj_col = adj_col as usize;
                if let Some(piece_player) = board.get_player(adj_row, adj_col) {
                    if piece_player == opponent {
                        let far_row = adj_row as isize + dx;
                        let far_col = adj_col as isize + dy;
                        if PatternAnalyzer::is_in_bounds(board, far_row, far_col) {
                            let far_row = far_row as usize;
                            let far_col = far_col as usize;
                            if let Some(piece_player) = board.get_player(far_row, far_col) {
                                if piece_player == opponent {
                                    let end_row = far_row as isize + dx;
                                    let end_col = far_col as isize + dy;
                                    if PatternAnalyzer::is_in_bounds(board, end_row, end_col) {
                                        let end_row = end_row as usize;
                                        let end_col = end_col as usize;
                                        if let Some(piece_player) = board.get_player(end_row, end_col) {
                                            if piece_player == player {
                                                bonus += CAPTURE_BONUS_MULTIPLIER / 50; 
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        bonus
    }



    /// Generates moves in zones around existing stones for general play
    /// 
    /// Creates a local area around placed stones, filters illegal double-three
    /// moves, and prioritizes by threat value for tactical relevance
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

    /// Calculates Manhattan distance between two board positions
    #[inline]
    pub fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }

    /// Filters out moves that would create illegal double-three patterns
    /// 
    /// This function applies the double-three rule validation to all candidate moves,
    /// ensuring that no move violates the forbidden double-three pattern rule.
    /// Returns only legal moves that don't create double-three violations.
    fn filter_double_three_moves(board: &Board, moves: Vec<(usize, usize)>, player: Player) -> Vec<(usize, usize)> {
        moves
            .into_iter()
            .filter(|(row, col)| !DoubleThreeDetection::creates_double_three(board, *row, *col, player))
            .collect()
    }

    /// Last resort: exhaustive search for ANY legal move on the board
    /// 
    /// This function is called only when normal move generation fails to find
    /// legal moves. It searches every empty position on the board to find at
    /// least one legal move that doesn't violate double-three rules. This
    /// prevents AI deadlock while ensuring no illegal moves are returned.
    /// 
    /// Returns empty Vec only if there are truly no legal moves (game should end).
    fn find_any_legal_move(board: &Board, player: Player) -> Vec<(usize, usize)> {
        for row in 0..board.size {
            for col in 0..board.size {
                // Check if position is empty and legal
                if board.get_player(row, col).is_none() 
                    && !DoubleThreeDetection::creates_double_three(board, row, col, player) {
                    return vec![(row, col)];
                }
            }
        }
        // Truly no legal moves - return empty (game should end)
        vec![]
    }
}

/// Gets the appropriate score for a pattern based on its length and freedom.
/// 
/// This function provides pattern scoring using the official heuristic constants
/// to ensure consistency between move generation and position evaluation.
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
            PatternFreedom::Flanked => 0, // Dead twos are ignored
        },
        _ => 0,
    }
}
