use crate::core::board::Player;
use crate::core::state::GameState;

const TEMPO_BONUS: i32 = 300;
const PATTERN_DEVELOPMENT_BONUS: i32 = 150;
const RECENT_CAPTURE_BONUS: i32 = 400;
const DEFENSIVE_SEQUENCE_PENALTY: i32 = -100;
const HISTORY_WINDOW: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveType {
    Aggressive,
    Defensive,
    Positional,
    Capture,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveAnalysis {
    pub position: (usize, usize),
    pub player: Player,
    pub move_type: MoveType,
    pub captures_made: usize,
    pub threats_created: usize,
    pub threats_blocked: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternHistoryAnalyzer {
    move_history: Vec<MoveAnalysis>,
    tempo_score: i32,
    initiative_player: Option<Player>,
}

impl PatternHistoryAnalyzer {
    pub fn new() -> Self {
        Self {
            move_history: Vec::new(),
            tempo_score: 0,
            initiative_player: None,
        }
    }

    /// Undo the last move analysis (for search rollback)
    pub fn undo_last_move(&mut self) {
        if !self.move_history.is_empty() {
            self.move_history.pop();
            self.update_tempo_and_initiative();
        }
    }

    pub fn analyze_move_simple(&mut self, position: (usize, usize), player: Player, captures_made: usize) {
        let _move_analysis = MoveAnalysis {
            position,
            player,
            move_type: if captures_made > 0 { MoveType::Capture } else { MoveType::Positional },
            captures_made,
            threats_created: 0,
            threats_blocked: 0,
        };

        self.move_history.push(_move_analysis);
        
        if self.move_history.len() > HISTORY_WINDOW * 2 {
            self.move_history.drain(0..HISTORY_WINDOW);
        }

        self.update_tempo_and_initiative();
    }

    pub fn analyze_move(&mut self, state: &GameState, last_move: (usize, usize)) {
        let move_player = state.current_player.opponent();
        let captures_made = if let Some(last_captures) = state.capture_history.last() {
            last_captures.len() / 2
        } else {
            0
        };

        let move_analysis = MoveAnalysis {
            position: last_move,
            player: move_player,
            move_type: self.classify_move(state, last_move, move_player),
            captures_made,
            threats_created: self.count_threats_created(state, last_move, move_player),
            threats_blocked: self.count_threats_blocked(state, last_move, move_player),
        };

        self.move_history.push(move_analysis);
        
        if self.move_history.len() > HISTORY_WINDOW * 2 {
            self.move_history.drain(0..HISTORY_WINDOW);
        }

        self.update_tempo_and_initiative();
    }

    pub fn calculate_historical_bonus(&self, state: &GameState) -> i32 {
        let mut bonus = 0;

        if let Some(initiative_player) = self.initiative_player {
            if initiative_player == state.current_player {
                bonus += TEMPO_BONUS;
            } else {
                bonus -= TEMPO_BONUS;
            }
        }

        bonus += self.calculate_pattern_development_bonus(state.current_player);
        bonus += self.calculate_capture_momentum_bonus(state.current_player);
        bonus += self.calculate_defensive_sequence_penalty(state.current_player);

        bonus
    }

    pub fn reset(&mut self) {
        self.move_history.clear();
        self.tempo_score = 0;
        self.initiative_player = None;
    }

    pub fn get_recent_patterns(&self) -> Vec<&MoveAnalysis> {
        self.move_history.iter().rev().take(HISTORY_WINDOW).collect()
    }

    fn classify_move(&self, state: &GameState, position: (usize, usize), player: Player) -> MoveType {
        if let Some(last_captures) = state.capture_history.last() {
            if !last_captures.is_empty() {
                return MoveType::Capture;
            }
        }

        let threats_created = self.count_threats_created(state, position, player);
        let threats_blocked = self.count_threats_blocked(state, position, player);

        if threats_created > 0 && threats_created > threats_blocked {
            MoveType::Aggressive
        } else if threats_blocked > 0 && threats_blocked >= threats_created {
            MoveType::Defensive
        } else {
            MoveType::Positional
        }
    }

    fn count_threats_created(&self, state: &GameState, position: (usize, usize), player: Player) -> usize {
        // Count threats created by placing a stone at this position
        // This is a simplified version that counts immediate pattern formations
        let board = &state.board;
        let mut threats = 0;
        
        // Check all directions for new threat patterns
        for &(dx, dy) in &[(1, 0), (0, 1), (1, 1), (1, -1)] {
            let backward = crate::core::patterns::PatternAnalyzer::count_consecutive(
                board, position.0, position.1, -dx, -dy, player
            );
            let forward = crate::core::patterns::PatternAnalyzer::count_consecutive(
                board, position.0, position.1, dx, dy, player
            );
            let total = backward + forward + 1;
            
            // Count as threat if it forms 3+ in a row
            if total >= 3 {
                threats += 1;
            }
        }
        
        threats
    }

    fn count_threats_blocked(&self, state: &GameState, position: (usize, usize), player: Player) -> usize {
        // Count opponent threats that would be blocked by this move
        // Check if opponent would have formed threats at this position
        let opponent = player.opponent();
        let board = &state.board;
        let mut blocked = 0;
        
        for &(dx, dy) in &[(1, 0), (0, 1), (1, 1), (1, -1)] {
            let backward = crate::core::patterns::PatternAnalyzer::count_consecutive(
                board, position.0, position.1, -dx, -dy, opponent
            );
            let forward = crate::core::patterns::PatternAnalyzer::count_consecutive(
                board, position.0, position.1, dx, dy, opponent
            );
            let total = backward + forward + 1;
            
            // Count as blocked threat if opponent would have formed 3+ in a row
            if total >= 3 {
                blocked += 1;
            }
        }
        
        blocked
    }

    fn update_tempo_and_initiative(&mut self) {
        if self.move_history.is_empty() {
            return;
        }

        let recent_moves = self.move_history.iter().rev().take(6);
        let mut max_score = 0i32;
        let mut min_score = 0i32;

        for move_analysis in recent_moves {
            let move_score = match move_analysis.move_type {
                MoveType::Aggressive => 3,
                MoveType::Capture => 4,
                MoveType::Defensive => -1,
                MoveType::Positional => 0,
            };

            match move_analysis.player {
                Player::Max => max_score += move_score,
                Player::Min => min_score += move_score,
            }
        }

        self.tempo_score = max_score - min_score;
        
        // Determine who has initiative
        if self.tempo_score > 2 {
            self.initiative_player = Some(Player::Max);
        } else if self.tempo_score < -2 {
            self.initiative_player = Some(Player::Min);
        } else {
            self.initiative_player = None;
        }
    }

    fn calculate_pattern_development_bonus(&self, player: Player) -> i32 {
        let recent_moves: Vec<_> = self.move_history
            .iter()
            .rev()
            .take(4)
            .filter(|m| m.player == player)
            .collect();

        if recent_moves.len() < 2 {
            return 0;
        }

        // Bonus for consistent aggressive play
        let aggressive_count = recent_moves
            .iter()
            .filter(|m| matches!(m.move_type, MoveType::Aggressive | MoveType::Capture))
            .count();

        if aggressive_count >= recent_moves.len() / 2 {
            PATTERN_DEVELOPMENT_BONUS
        } else {
            0
        }
    }

    fn calculate_capture_momentum_bonus(&self, player: Player) -> i32 {
        let recent_captures: usize = self.move_history
            .iter()
            .rev()
            .take(3)
            .filter(|m| m.player == player && m.move_type == MoveType::Capture)
            .map(|m| m.captures_made)
            .sum();

        if recent_captures > 0 {
            RECENT_CAPTURE_BONUS * recent_captures as i32
        } else {
            0
        }
    }

    fn calculate_defensive_sequence_penalty(&self, player: Player) -> i32 {
        let recent_defensive = self.move_history
            .iter()
            .rev()
            .take(3)
            .filter(|m| m.player == player && m.move_type == MoveType::Defensive)
            .count();

        if recent_defensive >= 2 {
            DEFENSIVE_SEQUENCE_PENALTY * recent_defensive as i32
        } else {
            0
        }
    }
}

impl Default for PatternHistoryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}