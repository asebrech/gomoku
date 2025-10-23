use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer, PatternFreedom};
use crate::core::rules::DoubleThreeDetection;
use crate::ai::heuristic::{
    CAPTURE_BONUS_MULTIPLIER, FIVE_IN_ROW_SCORE, LIVE_FOUR_SINGLE_SCORE, 
    HALF_FREE_FOUR_SCORE, DEAD_FOUR_SCORE, LIVE_THREE_SCORE, 
    HALF_FREE_THREE_SCORE, DEAD_THREE_SCORE, LIVE_TWO_SCORE, HALF_FREE_TWO_SCORE
};
use std::collections::HashSet;
use bevy::prelude::info;

const CENTER_POSITION_BONUS: i32 = 10;

/// Prioritized move generator for Gomoku AI that reduces search space
/// by focusing on tactically relevant moves
pub struct MoveGenerator;
impl MoveGenerator {
    /// Generates prioritized candidate moves for the given player
    /// 
    /// Priority order:
    /// 1. Winning moves (immediate five-in-a-row)
    /// 2. Must-block moves (prevent opponent from winning)
    /// 3. Threat moves (create or block threats)
    /// 4. Zone-based moves (around existing stones)
    /// 
    /// Returns empty board center if board is empty
    pub fn get_candidate_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }
        if let Some(winning_move) = Self::find_winning_move(board, player) {
            if !DoubleThreeDetection::creates_double_three(board, winning_move.0, winning_move.1, player) {
                return vec![winning_move];
            }
        }
        
        // Get both offensive and defensive moves, then sort by priority
        let mut all_candidate_moves = Vec::new();
        
        // Check must-block moves (opponent threats)
        let block_moves = Self::find_must_block_moves(board, player.opponent());
        
        // CRITICAL: If opponent has open-four threats, ONLY return blocking moves
        // Don't even consider offensive moves - we must block or lose immediately
        if let Some(ref blocks) = block_moves {
            let opponent_open_fours = Self::find_open_four_threats(board, player.opponent());
            if !opponent_open_fours.is_empty() && !blocks.is_empty() {
                // Filter to only open-four blocks
                let critical_blocks: Vec<_> = blocks.iter()
                    .filter(|&&mv| opponent_open_fours.contains(&mv))
                    .copied()
                    .collect();
                if !critical_blocks.is_empty() {
                    let legal_blocks = Self::filter_double_three_moves(board, critical_blocks, player);
                    // Only return early if we found legal blocks
                    // If all blocks are illegal, fall through to find other moves
                    if !legal_blocks.is_empty() {
                        return legal_blocks;
                    }
                }
            }
        }
        
        // No critical threats, continue with normal move generation
        if let Some(blocks) = block_moves {
            all_candidate_moves.extend(blocks);
        }
        
        // Check our offensive threat moves (includes trap bonuses)
        let threat_moves = Self::find_threat_moves(board, player);
        all_candidate_moves.extend(threat_moves);
        
        // Filter out illegal double-three moves
        all_candidate_moves = Self::filter_double_three_moves(board, all_candidate_moves, player);
        
        if !all_candidate_moves.is_empty() {
            // CRITICAL: Sort ALL moves by priority (trap bonuses included!)
            // Must-block moves need to be evaluated with calculate_threat_priority
            // to ensure trap moves get their huge bonus and beat regular blocks
            
            // Get opponent's open-four threats for absolute priority
            let opponent_open_fours = Self::find_open_four_threats(board, player.opponent());
            
            let mut prioritized: Vec<((usize, usize), i32)> = all_candidate_moves
                .into_iter()
                .map(|mv| {
                    let mut priority = Self::calculate_threat_priority(board, mv, player);
                    
                    // ABSOLUTE PRIORITY: Blocking opponent's open-four must beat EVERYTHING
                    // Including our own trap moves! Defense first when facing immediate loss
                    if opponent_open_fours.contains(&mv) {
                        priority += 1_000_000; // Add 1 million to ensure it's always first
                    }
                    
                    (mv, priority)
                })
                .collect();
            
            prioritized.sort_by_key(|(_, priority)| -priority);
            return prioritized.into_iter().map(|(mv, _)| mv).collect();
        }
        
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

    /// Finds moves that must be played to prevent opponent from winning
    /// 
    /// Priority order:
    /// 1. Block immediate opponent wins
    /// 2. Block open four threats (unblocked fours)
    /// 3. Block gapped threats (patterns like X.X.X)
    fn find_must_block_moves(board: &Board, opponent: Player) -> Option<Vec<(usize, usize)>> {
        if let Some(opp_win) = Self::find_winning_move(board, opponent) {
            return Some(vec![opp_win]);
        }
        let open_fours = Self::find_open_four_threats(board, opponent);
        if !open_fours.is_empty() {
            return Some(open_fours);
        }
        let gapped_threats = Self::find_gapped_threats(board, opponent);
        if !gapped_threats.is_empty() {
            return Some(gapped_threats);
        }
        None
    }

    /// Finds positions that block opponent's open four threats
    /// 
    /// An open four is a line of four stones with empty spaces on both ends,
    /// creating an immediate winning threat that must be blocked.
    /// 
    /// SMART RULE AWARENESS: This function now filters out threat positions that
    /// would be illegal for the opponent (e.g., double-three violations), preventing
    /// the AI from worrying about threats the opponent cannot actually execute.
    fn find_open_four_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward =
                    PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                if backward + forward + 1 == 4 {
                    let back_row = row as isize - dx * (backward as isize + 1);
                    let back_col = col as isize - dy * (backward as isize + 1);
                    let fwd_row = row as isize + dx * (forward as isize + 1);
                    let fwd_col = col as isize + dy * (forward as isize + 1);
                    if PatternAnalyzer::is_valid_empty(board, back_row, back_col) {
                        let pos = (back_row as usize, back_col as usize);
                        // Only add if this position is legal for the player
                        if !DoubleThreeDetection::creates_double_three(board, pos.0, pos.1, player) {
                            threats.insert(pos);
                        }
                    }
                    if PatternAnalyzer::is_valid_empty(board, fwd_row, fwd_col) {
                        let pos = (fwd_row as usize, fwd_col as usize);
                        // Only add if this position is legal for the player
                        if !DoubleThreeDetection::creates_double_three(board, pos.0, pos.1, player) {
                            threats.insert(pos);
                        }
                    }
                }
            }
        });
        threats.into_iter().collect()
    }

    /// Finds positions that block opponent's gapped threats
    /// 
    /// Gapped threats are patterns like X.X.X or XX.X where stones are
    /// separated by gaps but could form five-in-a-row if gaps are filled.
    /// 
    /// SMART RULE AWARENESS: This function now filters out threat positions that
    /// would be illegal for the opponent (e.g., double-three violations), preventing
    /// the AI from worrying about threats the opponent cannot actually execute.
    fn find_gapped_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let mut stones_found = vec![(row, col)];
                for dist in 2..=6 {
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
                // Check for 2+ stones to find threat extensions (X X _ patterns)
                if stones_found.len() >= 2 {
                    let first = stones_found.first().unwrap();
                    let last = stones_found.last().unwrap();
                    let start_row = first.0 as isize;
                    let start_col = first.1 as isize;
                    let end_row = last.0 as isize;
                    let end_col = last.1 as isize;
                    let total_span = ((end_row - start_row).abs() + (end_col - start_col).abs()) + 1;
                    if total_span <= 5 {
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
                        // Only include gapped threats (X.X patterns) - don't add simple 2-stone extensions here
                        // The find_threat_creating_moves function handles simple extensions better
                        if empty_gaps > 0 && stones_found.len() + empty_gaps >= 5 && empty_gaps <= 2 {
                            for pos in threat_positions {
                                // Only add if this position is legal for the player
                                if !DoubleThreeDetection::creates_double_three(board, pos.0, pos.1, player) {
                                    threats.insert(pos);
                                }
                            }
                        }
                    }
                }
            }
        });
        threats.into_iter().collect()
    }

    /// Finds moves that create or block tactical threats
    /// 
    /// Combines offensive moves (create our threats) and defensive moves
    /// (block opponent threats), prioritized by threat strength.
    /// 
    /// CRITICAL FIX: Opponent threat positions are now filtered to exclude
    /// illegal double-three moves. There's no point blocking a position the
    /// opponent can't legally play!
    fn find_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = HashSet::new();
        let our_threats = Self::find_threat_creating_moves(board, player);
        moves.extend(our_threats);
        
        // Get opponent threats but FILTER OUT illegal double-three positions
        let opp_threats = Self::find_threat_creating_moves(board, player.opponent());
        let opponent = player.opponent();
        
        // Only include opponent threats that are actually legal for them
        let legal_opp_threats = opp_threats
            .into_iter()
            .filter(|&(row, col)| !DoubleThreeDetection::creates_double_three(board, row, col, opponent));
        
        moves.extend(legal_opp_threats);
        
        let filtered_moves: Vec<(usize, usize)> = moves.into_iter().collect();
        let mut prioritized_moves: Vec<((usize, usize), i32)> = filtered_moves
            .into_iter()
            .map(|mv| {
                let priority = Self::calculate_threat_priority(board, mv, player);
                (mv, priority)
            })
            .collect();
        prioritized_moves.sort_by_key(|(_, priority)| -priority);
        if prioritized_moves.len() > 25 {
            prioritized_moves.truncate(25);
        }
        prioritized_moves.into_iter().map(|(mv, _)| mv).collect()
    }

    /// Calculates priority score for a move based on multiple factors
    /// 
    /// Scoring components:
    /// - Pattern values (offensive moves weighted 2x, defensive 1x)
    /// - Capture bonus for moves creating capture opportunities
    /// - Small center position bonus decreasing with distance
    /// 
    /// SMART RULE AWARENESS: Adds bonus if this move creates threats in positions
    /// where opponent cannot legally respond (e.g., double-three restrictions).
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
        
        // ADVANCED TACTIC: "Double-Three Trap"
        // Award huge bonus if this move creates a threat that FORCES opponent to respond,
        // but all response positions would create illegal double-three for them!
        // This is a devastating tactical weapon - opponent must choose between:
        // 1. Ignore our threat and let us win
        // 2. Respond and create illegal double-three (forfeit/invalid move)
        let trap_bonus = Self::calculate_double_three_trap_bonus(board, row, col, player);
        priority += trap_bonus;
        
        let center = board.size / 2;
        let distance = Self::manhattan_distance(row, col, center, center) as i32;
        priority += CENTER_POSITION_BONUS - distance.min(CENTER_POSITION_BONUS);
        priority
    }

    /// Calculates bonus for creating a "Double-Three Trap"
    /// 
    /// This advanced tactic creates a forcing threat where the opponent MUST respond,
    /// but all natural blocking positions would create illegal double-three for them.
    /// 
    /// The trap works by:
    /// 1. Our move creates a strong threat (3 or 4 in a row)
    /// 2. Opponent needs to block to prevent our win
    /// 3. All blocking positions create double-three for opponent (illegal!)
    /// 4. Result: Opponent is paralyzed - can't ignore, can't respond legally
    /// 
    /// Returns: Large bonus if trap is detected (200+ points)
    fn calculate_double_three_trap_bonus(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let opponent = player.opponent();
        let mut bonus = 0;
        
        // Simulate placing our stone
        let mut test_board = board.clone();
        test_board.place_stone(row, col, player);
        
        // Check each direction for threats we create
        for &(dx, dy) in &DIRECTIONS {
            let backward = PatternAnalyzer::count_consecutive(&test_board, row, col, -dx, -dy, player);
            let forward = PatternAnalyzer::count_consecutive(&test_board, row, col, dx, dy, player);
            let total_stones = backward + forward + 1;
            
            // We need at least 3 in a row to create a forcing threat
            if total_stones >= 3 {
                // Find all positions opponent would want to block
                let mut blocking_positions = Vec::new();
                
                // Check extensions on both ends of our line
                let back_row = row as isize - dx * (backward as isize + 1);
                let back_col = col as isize - dy * (backward as isize + 1);
                if PatternAnalyzer::is_valid_empty(&test_board, back_row, back_col) {
                    blocking_positions.push((back_row as usize, back_col as usize));
                }
                
                let fwd_row = row as isize + dx * (forward as isize + 1);
                let fwd_col = col as isize + dy * (forward as isize + 1);
                if PatternAnalyzer::is_valid_empty(&test_board, fwd_row, fwd_col) {
                    blocking_positions.push((fwd_row as usize, fwd_col as usize));
                }
                
                // Check if ALL blocking positions are illegal for opponent
                if !blocking_positions.is_empty() {
                    let all_illegal = blocking_positions.iter().all(|&(br, bc)| {
                        DoubleThreeDetection::creates_double_three(&test_board, br, bc, opponent)
                    });
                    
                    if all_illegal {
                        // JACKPOT! Opponent can't block without creating double-three!
                        if total_stones == 4 {
                            // 4 in a row with all blocks illegal = GAME OVER (undefendable)
                            // This should beat ANY regular 4-in-a-row because it's undefendable
                            bonus += 50_000;
                        } else if total_stones == 3 {
                            // 3 in a row with all blocks illegal = extremely strong
                            // Better than a defendable 4-in-a-row because it leads to certain win!
                            // The opponent literally cannot stop this without creating illegal double-three
                            bonus += 40_000;
                        }
                    } else {
                        // Partial trap: Some blocks are illegal
                        let illegal_count = blocking_positions.iter()
                            .filter(|&&(br, bc)| {
                                DoubleThreeDetection::creates_double_three(&test_board, br, bc, opponent)
                            })
                            .count();
                        
                        if illegal_count > 0 {
                            // Award proportional bonus
                            let ratio = illegal_count as f32 / blocking_positions.len() as f32;
                            bonus += (100.0 * ratio) as i32;
                        }
                    }
                }
            }
        }
        
        bonus
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
    /// create a capture situation (our stone - opponent - opponent - our stone).
    /// 
    /// SMART RULE AWARENESS: Provides extra bonus if the capture spot would be
    /// illegal for the opponent (e.g., double-three), making it a safe advantage.
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
                                                // Extra bonus if opponent cannot legally capture back
                                                if DoubleThreeDetection::creates_double_three(board, row, col, opponent) {
                                                    bonus += CAPTURE_BONUS_MULTIPLIER / 25;
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
        }
        bonus
    }

    fn find_threat_creating_moves(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut moves = HashSet::new();
        let player_bits = board.get_player_bits(player);
        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward =
                    PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                let total = backward + forward + 1;
                if total >= 2 && total <= 4 {
                    for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                        let r = row as isize + dx * offset;
                        let c = col as isize + dy * offset;
                        if PatternAnalyzer::is_valid_empty(board, r, c) {
                            moves.insert((r as usize, c as usize));
                        }
                    }
                }
            }
        });
        moves
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
