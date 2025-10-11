//! Lightweight pattern history analyzer used to bias the heuristic based on
//! recent capture momentum or initiative.
//!
//! This is not a full Monte-Carlo history heuristic; it's a simple history
//! tracking of the last few moves used to prefer moves for the player who
//! has recent initiative (captures or aggressive play). It produces a small
//! bonus/penalty applied to the heuristic evaluation.

use crate::core::board::Player;

const CAPTURE_MOMENTUM_BONUS: i32 = 200;
const INITIATIVE_BONUS: i32 = 100;
const HISTORY_WINDOW: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveRecord {
    pub player: Player,
    pub captures_made: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternHistoryAnalyzer {
    recent_moves: Vec<MoveRecord>,
    current_initiative: Option<Player>,
}

impl PatternHistoryAnalyzer {
    pub fn new() -> Self {
        Self {
            recent_moves: Vec::new(),
            current_initiative: None,
        }
    }

    pub fn analyze_move(&mut self, player: Player, captures_made: usize) {
        let move_record = MoveRecord {
            player,
            captures_made,
        };

        self.recent_moves.push(move_record);
        
        if self.recent_moves.len() > HISTORY_WINDOW {
            self.recent_moves.drain(0..1);
        }

        self.update_initiative();
    }

    pub fn calculate_historical_bonus(&self, current_player: Player) -> i32 {
        let mut bonus = 0;

        if let Some(initiative_player) = self.current_initiative {
            if initiative_player == current_player {
                bonus += INITIATIVE_BONUS;
            } else {
                bonus -= INITIATIVE_BONUS;
            }
        }

        bonus += self.calculate_capture_momentum_bonus(current_player);

        bonus
    }

    pub fn reset(&mut self) {
        self.recent_moves.clear();
        self.current_initiative = None;
    }

    pub fn move_count(&self) -> usize {
        self.recent_moves.len()
    }

    pub fn latest_move(&self) -> Option<&MoveRecord> {
        self.recent_moves.last()
    }

    fn update_initiative(&mut self) {
        if self.recent_moves.len() < 2 {
            self.current_initiative = None;
            return;
        }

        let recent_window = self.recent_moves.iter().rev().take(HISTORY_WINDOW);
        let mut max_captures = 0;
        let mut min_captures = 0;

        for move_record in recent_window {
            if move_record.captures_made > 0 {
                match move_record.player {
                    Player::Max => max_captures += move_record.captures_made,
                    Player::Min => min_captures += move_record.captures_made,
                }
            }
        }

        if max_captures > min_captures {
            self.current_initiative = Some(Player::Max);
        } else if min_captures > max_captures {
            self.current_initiative = Some(Player::Min);
        } else {
            self.current_initiative = None;
        }
    }

    fn calculate_capture_momentum_bonus(&self, player: Player) -> i32 {
        let recent_captures: usize = self.recent_moves
            .iter()
            .rev()
            .take(3) 
            .filter(|m| m.player == player && m.captures_made > 0)
            .map(|m| m.captures_made)
            .sum();

        if recent_captures > 0 {
            (CAPTURE_MOMENTUM_BONUS as f32 * (recent_captures as f32).sqrt()) as i32
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