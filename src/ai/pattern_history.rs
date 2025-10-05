use crate::core::board::{Board, Player};
use crate::core::state::GameState;
use crate::ai::heuristic::{Heuristic, PatternFreedom};

const TEMPO_BONUS: i32 = 300;
const PATTERN_DEVELOPMENT_BONUS: i32 = 150;
const RECENT_CAPTURE_BONUS: i32 = 400;
const DEFENSIVE_SEQUENCE_PENALTY: i32 = -100;
const HISTORY_WINDOW: usize = 8;
const TEMPO_WINDOW: usize = 6;
const PATTERN_DEVELOPMENT_WINDOW: usize = 4;
const CAPTURE_MOMENTUM_WINDOW: usize = 3;

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

    /// Analyze patterns around a position and count threats created and blocked
    /// Returns (threats_created, threats_blocked)
    fn analyze_move_threats(
        &self,
        board: &Board,
        position: (usize, usize),
        player: Player,
        win_condition: usize,
    ) -> (usize, usize) {
        const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut threats_created = 0;
        let mut threats_blocked = 0;
        let opponent = player.opponent();

        for &(dx, dy) in &DIRECTIONS {
            // Analyze patterns created by this move
            let (start_row, start_col) = Heuristic::find_pattern_start(board, position.0, position.1, dx, dy, player);
            let length = Heuristic::count_consecutive(board, start_row, start_col, dx, dy, player);

            if length >= 2 && Heuristic::has_sufficient_space(board, start_row, start_col, dx, dy, length, player, win_condition) {
                let freedom = Heuristic::analyze_pattern_freedom(board, start_row, start_col, dx, dy, length);
                
                // Count as threat based on pattern strength
                threats_created += match (length, freedom) {
                    (4, PatternFreedom::Free) => 3,        // Live four - immediate win threat
                    (4, PatternFreedom::HalfFree) => 2,    // Half-free four - strong threat
                    (3, PatternFreedom::Free) => 2,        // Live three - can become four
                    (3, PatternFreedom::HalfFree) => 1,    // Half-free three - moderate threat
                    _ => 0,
                };
            }

            // Analyze opponent patterns blocked by this move
            // We need to check what opponent pattern existed before this position was taken
            // Count from adjacent positions in both directions
            let before_row = position.0 as isize - dx;
            let before_col = position.1 as isize - dy;
            let after_row = position.0 as isize + dx;
            let after_col = position.1 as isize + dy;
            
            // Only count if adjacent positions have opponent stones (is_position_empty checks boundaries)
            let before_length = if !Heuristic::is_position_empty(board, before_row, before_col) {
                let idx = board.index(before_row as usize, before_col as usize);
                let opponent_bits = match opponent {
                    Player::Max => &board.max_bits,
                    Player::Min => &board.min_bits,
                };
                if Board::is_bit_set(opponent_bits, idx) {
                    Heuristic::count_consecutive(board, before_row as usize, before_col as usize, -dx, -dy, opponent)
                } else {
                    0
                }
            } else {
                0
            };
            
            let after_length = if !Heuristic::is_position_empty(board, after_row, after_col) {
                let idx = board.index(after_row as usize, after_col as usize);
                let opponent_bits = match opponent {
                    Player::Max => &board.max_bits,
                    Player::Min => &board.min_bits,
                };
                if Board::is_bit_set(opponent_bits, idx) {
                    Heuristic::count_consecutive(board, after_row as usize, after_col as usize, dx, dy, opponent)
                } else {
                    0
                }
            } else {
                0
            };
            
            let total_blocked = before_length + after_length + 1; // +1 for the current position connecting them

            if total_blocked >= 3 { // At least a three was blocked
                // Only count if the blocked pattern had sufficient space
                if before_length > 0 {
                    let (opp_start_row, opp_start_col) = Heuristic::find_pattern_start(
                        board, before_row as usize, before_col as usize, -dx, -dy, opponent
                    );
                    if Heuristic::has_sufficient_space(board, opp_start_row, opp_start_col, dx, dy, before_length, opponent, win_condition) {
                        threats_blocked += match before_length {
                            n if n >= 4 => 3, // Blocked a four (would have been five)
                            3 => 2,           // Blocked a three (would have been four)
                            2 => 1,           // Blocked a two (would have been three)
                            _ => 0,
                        };
                    }
                }
                
                // Also check the pattern on the other side if different
                if after_length > 0 && (before_length == 0 || after_length != before_length) {
                    let (opp_start_row, opp_start_col) = Heuristic::find_pattern_start(
                        board, after_row as usize, after_col as usize, dx, dy, opponent
                    );
                    if Heuristic::has_sufficient_space(board, opp_start_row, opp_start_col, dx, dy, after_length, opponent, win_condition) {
                        threats_blocked += match after_length {
                            n if n >= 4 => 3,
                            3 => 2,
                            2 => 1,
                            _ => 0,
                        };
                    }
                }
            }
        }

        (threats_created, threats_blocked)
    }

    pub fn analyze_move_simple(&mut self, position: (usize, usize), player: Player, captures_made: usize) {
        let move_analysis = MoveAnalysis {
            position,
            player,
            move_type: if captures_made > 0 { MoveType::Capture } else { MoveType::Positional },
            captures_made,
            threats_created: 0,
            threats_blocked: 0,
        };

        self.move_history.push(move_analysis);
        
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

        // Analyze threats once
        let (threats_created, threats_blocked) = self.analyze_move_threats(
            &state.board,
            last_move,
            move_player,
            state.win_condition,
        );

        let move_analysis = MoveAnalysis {
            position: last_move,
            player: move_player,
            move_type: self.classify_move_with_threats(state, threats_created, threats_blocked),
            captures_made,
            threats_created,
            threats_blocked,
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

    fn classify_move_with_threats(&self, state: &GameState, threats_created: usize, threats_blocked: usize) -> MoveType {
        if let Some(last_captures) = state.capture_history.last() {
            if !last_captures.is_empty() {
                return MoveType::Capture;
            }
        }

        if threats_created > 0 && threats_created > threats_blocked {
            MoveType::Aggressive
        } else if threats_blocked > 0 && threats_blocked >= threats_created {
            MoveType::Defensive
        } else {
            MoveType::Positional
        }
    }

    fn update_tempo_and_initiative(&mut self) {
        if self.move_history.is_empty() {
            return;
        }

        let recent_moves = self.move_history.iter().rev().take(TEMPO_WINDOW);
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
            .take(PATTERN_DEVELOPMENT_WINDOW)
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
            .take(CAPTURE_MOMENTUM_WINDOW)
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
            .take(CAPTURE_MOMENTUM_WINDOW)
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