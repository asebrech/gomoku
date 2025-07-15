use crate::core::board::{Board, Player};
use crate::core::state::GameState;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ThreatLevel {
    Five,        // Immediate win
    OpenFour,    // Unstoppable threat
    Four,        // Threat that can be blocked
    OpenThree,   // Strong threat
    Three,       // Moderate threat
    OpenTwo,     // Weak threat
    Two,         // Very weak threat
    None,        // No threat
}

#[derive(Debug, Clone)]
struct ThreatInfo {
    level: ThreatLevel,
    position: (usize, usize),
    direction: (isize, isize),
    winning_moves: Vec<(usize, usize)>,
}

#[derive(Debug)]
struct PatternAnalysis {
    consecutive: usize,
    gaps: usize,
    blocked_ends: usize,
    open_ends: usize,
    total_length: usize,
}

pub struct Heuristic;

impl Heuristic {
    pub fn evaluate(state: &GameState) -> i32 {
        // Check for terminal states first
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => 1000000,
                Player::Min => -1000000,
            };
        }

        // Use threat-based evaluation
        let max_threats = Self::analyze_threats(&state.board, Player::Max);
        let min_threats = Self::analyze_threats(&state.board, Player::Min);

        // Calculate threat scores
        let max_score = Self::calculate_threat_score(&max_threats);
        let min_score = Self::calculate_threat_score(&min_threats);

        // Check for immediate wins and critical threats
        if Self::has_immediate_win(&max_threats) {
            return 500000;
        }
        if Self::has_immediate_win(&min_threats) {
            return -500000;
        }

        // Check for multiple threats (combination attacks)
        let max_combination_bonus = Self::calculate_combination_bonus(&max_threats);
        let min_combination_bonus = Self::calculate_combination_bonus(&min_threats);

        max_score + max_combination_bonus - min_score - min_combination_bonus
    }

    fn analyze_threats(board: &Board, player: Player) -> Vec<ThreatInfo> {
        let mut threats = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col) == Some(player) {
                    for &direction in &directions {
                        if let Some(threat) = Self::analyze_line_threat(board, row, col, direction, player) {
                            threats.push(threat);
                        }
                    }
                }
            }
        }

        // Remove duplicate threats and keep only the strongest
        Self::deduplicate_threats(threats)
    }

    fn analyze_line_threat(
        board: &Board,
        row: usize,
        col: usize,
        direction: (isize, isize),
        player: Player,
    ) -> Option<ThreatInfo> {
        let analysis = Self::analyze_pattern(board, row, col, direction, player);
        
        let (level, winning_moves) = match analysis {
            // Five in a row - immediate win
            PatternAnalysis { consecutive: 5, .. } => (ThreatLevel::Five, vec![]),
            
            // Open four - unstoppable threat
            PatternAnalysis { consecutive: 4, open_ends: 2, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 4);
                (ThreatLevel::OpenFour, moves)
            },
            
            // Four with one open end - can be blocked
            PatternAnalysis { consecutive: 4, open_ends: 1, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 4);
                (ThreatLevel::Four, moves)
            },
            
            // Open three - strong threat
            PatternAnalysis { consecutive: 3, open_ends: 2, gaps: 0, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 3);
                (ThreatLevel::OpenThree, moves)
            },
            
            // Three with gap or one blocked end
            PatternAnalysis { consecutive: 3, gaps: 1, .. } |
            PatternAnalysis { consecutive: 3, open_ends: 1, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 3);
                (ThreatLevel::Three, moves)
            },
            
            // Open two - weak threat
            PatternAnalysis { consecutive: 2, open_ends: 2, gaps: 0, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 2);
                (ThreatLevel::OpenTwo, moves)
            },
            
            // Two with gap or one blocked end
            PatternAnalysis { consecutive: 2, .. } => {
                let moves = Self::find_completion_moves(board, row, col, direction, player, 2);
                (ThreatLevel::Two, moves)
            },
            
            _ => return None,
        };

        if level != ThreatLevel::None {
            Some(ThreatInfo {
                level,
                position: (row, col),
                direction,
                winning_moves,
            })
        } else {
            None
        }
    }

    fn analyze_pattern(
        board: &Board,
        row: usize,
        col: usize,
        direction: (isize, isize),
        player: Player,
    ) -> PatternAnalysis {
        let (dx, dy) = direction;
        let mut _consecutive = 0;
        let mut gaps = 0;

        // Analyze in both directions from the starting position
        let mut positions = Vec::new();
        
        // Go backwards first
        let mut r = row as isize - dx;
        let mut c = col as isize - dy;
        let mut back_positions = Vec::new();
        
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            match board.get_player(r as usize, c as usize) {
                Some(p) if p == player => {
                    back_positions.push((r, c, true));
                    r -= dx;
                    c -= dy;
                },
                None => {
                    back_positions.push((r, c, false));
                    r -= dx;
                    c -= dy;
                },
                _ => break, // Opponent piece
            }
            if back_positions.len() >= 4 { break; }
        }
        
        // Reverse and add to positions
        for pos in back_positions.iter().rev() {
            positions.push(*pos);
        }
        
        // Add current position
        positions.push((row as isize, col as isize, true));
        
        // Go forwards
        let mut r = row as isize + dx;
        let mut c = col as isize + dy;
        
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            match board.get_player(r as usize, c as usize) {
                Some(p) if p == player => {
                    positions.push((r, c, true));
                    r += dx;
                    c += dy;
                },
                None => {
                    positions.push((r, c, false));
                    r += dx;
                    c += dy;
                },
                _ => break, // Opponent piece
            }
            if positions.len() >= 9 { break; }
        }

        // Analyze the pattern
        let mut max_consecutive = 0;
        let mut current_consecutive = 0;
        let mut _found_gap = false;
        
        for &(_, _, is_player) in &positions {
            if is_player {
                current_consecutive += 1;
                _consecutive += 1;
            } else {
                if current_consecutive > 0 {
                    _found_gap = true;
                    gaps += 1;
                }
                max_consecutive = max_consecutive.max(current_consecutive);
                current_consecutive = 0;
            }
        }
        max_consecutive = max_consecutive.max(current_consecutive);
        
        // Count open ends
        let mut open_ends = 0;
        if positions.len() > 0 {
            // Check first position
            if !positions[0].2 {
                open_ends += 1;
            }
            // Check last position
            if !positions[positions.len() - 1].2 {
                open_ends += 1;
            }
        }

        PatternAnalysis {
            consecutive: max_consecutive,
            gaps,
            blocked_ends: 2 - open_ends,
            open_ends,
            total_length: positions.len(),
        }
    }

    fn find_completion_moves(
        board: &Board,
        row: usize,
        col: usize,
        direction: (isize, isize),
        player: Player,
        _consecutive: usize,
    ) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let (dx, dy) = direction;

        // Check positions around the pattern for completion moves
        for offset in -2..=2 {
            let r = row as isize + dx * offset;
            let c = col as isize + dy * offset;
            
            if r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
                if board.get_player(r as usize, c as usize).is_none() {
                    // Check if this move would extend the pattern
                    if Self::would_extend_pattern(board, r as usize, c as usize, direction, player) {
                        moves.push((r as usize, c as usize));
                    }
                }
            }
        }

        moves
    }

    fn would_extend_pattern(
        board: &Board,
        row: usize,
        col: usize,
        direction: (isize, isize),
        player: Player,
    ) -> bool {
        let (dx, dy) = direction;
        
        // Check if there's a friendly piece in the direction
        let r1 = row as isize + dx;
        let c1 = col as isize + dy;
        let r2 = row as isize - dx;
        let c2 = col as isize - dy;
        
        let check_pos = |r: isize, c: isize| -> bool {
            r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize &&
            board.get_player(r as usize, c as usize) == Some(player)
        };
        
        check_pos(r1, c1) || check_pos(r2, c2)
    }

    fn deduplicate_threats(threats: Vec<ThreatInfo>) -> Vec<ThreatInfo> {
        let mut unique_threats = Vec::new();
        
        for threat in threats {
            // Check if we already have a stronger threat in the same area
            let mut should_add = true;
            
            for existing in &unique_threats {
                if Self::threats_overlap(&threat, existing) {
                    // Keep the stronger threat
                    if Self::threat_strength(threat.level) <= Self::threat_strength(existing.level) {
                        should_add = false;
                        break;
                    }
                }
            }
            
            if should_add {
                // Remove weaker overlapping threats
                unique_threats.retain(|existing| {
                    !Self::threats_overlap(&threat, existing) || 
                    Self::threat_strength(existing.level) >= Self::threat_strength(threat.level)
                });
                unique_threats.push(threat);
            }
        }
        
        unique_threats
    }

    fn threats_overlap(threat1: &ThreatInfo, threat2: &ThreatInfo) -> bool {
        // Simple overlap check - threats are considered overlapping if they're in the same area
        let dist = ((threat1.position.0 as isize - threat2.position.0 as isize).abs() +
                   (threat1.position.1 as isize - threat2.position.1 as isize).abs()) as usize;
        dist <= 2
    }

    fn threat_strength(level: ThreatLevel) -> i32 {
        match level {
            ThreatLevel::Five => 1000000,
            ThreatLevel::OpenFour => 100000,
            ThreatLevel::Four => 10000,
            ThreatLevel::OpenThree => 1000,
            ThreatLevel::Three => 100,
            ThreatLevel::OpenTwo => 10,
            ThreatLevel::Two => 1,
            ThreatLevel::None => 0,
        }
    }

    fn calculate_threat_score(threats: &[ThreatInfo]) -> i32 {
        threats.iter().map(|t| Self::threat_strength(t.level)).sum()
    }

    fn has_immediate_win(threats: &[ThreatInfo]) -> bool {
        threats.iter().any(|t| matches!(t.level, ThreatLevel::Five | ThreatLevel::OpenFour))
    }

    fn calculate_combination_bonus(threats: &[ThreatInfo]) -> i32 {
        let mut bonus = 0;
        
        // Multiple open threes = strong combination
        let open_threes = threats.iter().filter(|t| t.level == ThreatLevel::OpenThree).count();
        if open_threes >= 2 {
            bonus += 10000;
        }
        
        // Multiple fours = very strong
        let fours = threats.iter().filter(|t| matches!(t.level, ThreatLevel::Four | ThreatLevel::OpenFour)).count();
        if fours >= 2 {
            bonus += 50000;
        }
        
        // Mix of strong threats
        let strong_threats = threats.iter().filter(|t| 
            matches!(t.level, ThreatLevel::Four | ThreatLevel::OpenFour | ThreatLevel::OpenThree)
        ).count();
        if strong_threats >= 3 {
            bonus += 5000;
        }
        
        bonus
    }

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        // Advanced move ordering based on threat analysis
        let mut move_scores = Vec::new();
        
        for &mv in moves.iter() {
            let score = Self::evaluate_move_urgency(state, mv);
            move_scores.push((mv, score));
        }
        
        // Sort by score (highest first)
        move_scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Update the moves vector
        for (i, (mv, _)) in move_scores.iter().enumerate() {
            moves[i] = *mv;
        }
    }

    fn evaluate_move_urgency(state: &GameState, mv: (usize, usize)) -> i32 {
        let mut score = 0;
        let (row, col) = mv;
        
        // Check if this move creates or blocks threats
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        
        for &direction in &directions {
            // Check threat creation for current player
            let our_threat = Self::simulate_threat_creation(&state.board, row, col, direction, state.current_player);
            score += Self::threat_strength(our_threat) * 2;
            
            // Check threat blocking for opponent
            let opponent_threat = Self::simulate_threat_creation(&state.board, row, col, direction, state.current_player.opponent());
            score += Self::threat_strength(opponent_threat);
        }
        
        // Prefer center moves
        let center = state.board.size / 2;
        let center_bonus = 100 - ((row as isize - center as isize).abs() + (col as isize - center as isize).abs()) as i32;
        score += center_bonus;
        
        // Prefer moves near existing pieces
        let proximity_bonus = Self::calculate_proximity_bonus(&state.board, row, col);
        score += proximity_bonus;
        
        score
    }

    fn simulate_threat_creation(
        board: &Board,
        row: usize,
        col: usize,
        direction: (isize, isize),
        player: Player,
    ) -> ThreatLevel {
        // Simulate placing a piece and check the resulting threat level
        let mut test_board = board.clone();
        test_board.place_stone(row, col, player);
        
        if let Some(threat) = Self::analyze_line_threat(&test_board, row, col, direction, player) {
            threat.level
        } else {
            ThreatLevel::None
        }
    }

    fn calculate_proximity_bonus(board: &Board, row: usize, col: usize) -> i32 {
        let mut bonus = 0;
        let directions = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
        
        for &(dx, dy) in &directions {
            let new_row = row as isize + dx;
            let new_col = col as isize + dy;
            
            if new_row >= 0 && new_row < board.size as isize && 
               new_col >= 0 && new_col < board.size as isize {
                if board.get_player(new_row as usize, new_col as usize).is_some() {
                    bonus += 50;
                }
            }
        }
        
        bonus
    }
}
