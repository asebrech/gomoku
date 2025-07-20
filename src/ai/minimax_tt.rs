use crate::core::state::GameState;
use crate::ai::zobrist::ZobristHash;
use crate::ai::transposition_table::{TranspositionTable, EntryType};
use std::cmp::{max, min};

use super::{heuristic::Heuristic, move_ordering::MoveOrdering};

/// Enhanced minimax implementation with transposition table support
pub struct MinimaxWithTT {
    /// Zobrist hashing for position identification
    zobrist: ZobristHash,
    /// Transposition table for caching evaluations
    tt: TranspositionTable,
    /// Statistics
    pub nodes_searched: u64,
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
}

impl MinimaxWithTT {
    /// Create a new minimax engine with transposition table
    pub fn new(board_size: usize) -> Self {
        Self {
            zobrist: ZobristHash::new(board_size),
            tt: TranspositionTable::new_default(),
            nodes_searched: 0,
            tt_hits: 0,
            tt_cutoffs: 0,
        }
    }
    
    /// Create a new minimax engine with custom table size
    pub fn new_with_table_size(board_size: usize, table_size: usize) -> Self {
        Self {
            zobrist: ZobristHash::new(board_size),
            tt: TranspositionTable::new(table_size),
            nodes_searched: 0,
            tt_hits: 0,
            tt_cutoffs: 0,
        }
    }
    
    /// Reset statistics for a new search
    pub fn reset_stats(&mut self) {
        self.nodes_searched = 0;
        self.tt_hits = 0;
        self.tt_cutoffs = 0;
        self.tt.advance_age();
    }
    
    /// Get the current hash of a position
    pub fn get_hash(&self, state: &GameState) -> u64 {
        self.zobrist.compute_hash(state)
    }
    
    /// Main minimax search with transposition table
    pub fn minimax(
        &mut self,
        state: &mut GameState,
        depth: i32,
        mut alpha: i32,
        mut beta: i32,
        maximizing_player: bool,
        hash: u64,
    ) -> i32 {
        self.nodes_searched += 1;
        
        // Check transposition table first
        let tt_result = self.tt.probe(hash, depth, alpha, beta);
        if tt_result.cutoff {
            self.tt_hits += 1;
            if let Some(value) = tt_result.value {
                self.tt_cutoffs += 1;
                return value;
            }
        }
        
        // Get TT move for move ordering
        let tt_best_move = tt_result.best_move;
        
        // Terminal node check
        if depth == 0 || state.is_terminal() {
            let eval = Heuristic::evaluate(state, depth);
            
            // Store exact evaluation in transposition table
            self.tt.store(hash, eval, depth, EntryType::Exact, None);
            return eval;
        }
        
        // Get possible moves
        let mut moves = state.get_possible_moves();
        if moves.is_empty() {
            let eval = Heuristic::evaluate(state, depth);
            self.tt.store(hash, eval, depth, EntryType::Exact, None);
            return eval;
        }
        
        // Move ordering (enhanced with TT move)
        MoveOrdering::order_moves(state, &mut moves);
        
        // Put TT move first if available
        if let Some(tt_move) = tt_best_move {
            if let Some(pos) = moves.iter().position(|&m| m == tt_move) {
                moves.swap(0, pos);
            }
        }
        
        let original_alpha = alpha;
        let mut best_move = None;
        let mut best_value = if maximizing_player { i32::MIN } else { i32::MAX };
        
        // Search all moves
        for &move_pos in &moves {
            // Make the move and update hash
            let new_hash = self.zobrist.update_hash_make_move(hash, move_pos.0, move_pos.1, state.current_player);
            state.make_move(move_pos);
            
            // Recursive call
            let value = self.minimax(state, depth - 1, alpha, beta, !maximizing_player, new_hash);
            
            // Undo the move
            state.undo_move(move_pos);
            
            // Update best value and alpha-beta bounds
            if maximizing_player {
                if value > best_value {
                    best_value = value;
                    best_move = Some(move_pos);
                }
                alpha = max(alpha, value);
                if beta <= alpha {
                    break; // Beta cutoff
                }
            } else {
                if value < best_value {
                    best_value = value;
                    best_move = Some(move_pos);
                }
                beta = min(beta, value);
                if beta <= alpha {
                    break; // Alpha cutoff
                }
            }
        }
        
        // Determine the type of entry to store
        let entry_type = if best_value <= original_alpha {
            EntryType::UpperBound
        } else if best_value >= beta {
            EntryType::LowerBound
        } else {
            EntryType::Exact
        };
        
        // Store in transposition table
        self.tt.store(hash, best_value, depth, entry_type, best_move);
        
        best_value
    }
    
    /// Find the best move using iterative deepening
    pub fn find_best_move(&mut self, state: &mut GameState, max_depth: i32) -> Option<(usize, usize)> {
        self.reset_stats();
        
        let mut best_move = None;
        let initial_hash = self.get_hash(state);
        
        // Iterative deepening
        for depth in 1..=max_depth {
            let maximizing = match state.current_player {
                crate::core::board::Player::Max => true,
                crate::core::board::Player::Min => false,
            };
            
            let _value = self.minimax(state, depth, i32::MIN + 1, i32::MAX - 1, maximizing, initial_hash);
            
            // Get the best move from the transposition table
            if let Some(move_from_tt) = self.tt.get_best_move(initial_hash) {
                best_move = Some(move_from_tt);
            }
            
            // Print search statistics
            println!(
                "Depth {}: Nodes: {}, TT Hits: {}, TT Cutoffs: {}, Hit Rate: {:.2}%",
                depth,
                self.nodes_searched,
                self.tt_hits,
                self.tt_cutoffs,
                self.tt.hit_rate() * 100.0
            );
        }
        
        best_move
    }
    
    /// Clear the transposition table
    pub fn clear_table(&mut self) {
        self.tt.clear();
    }
    
    /// Get transposition table statistics
    pub fn get_tt_stats(&self) -> (usize, f64, u64) {
        (self.tt.size(), self.tt.hit_rate(), self.tt.collisions)
    }
}

// Keep the original function for backward compatibility
pub fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
) -> i32 {
    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state, depth);
        return eval;
    }

    let mut moves = state.get_possible_moves();
    MoveOrdering::order_moves(state, &mut moves);

    if maximizing_player {
        let mut value = i32::MIN;
        for move_ in moves {
            state.make_move(move_);
            value = max(value, minimax(state, depth - 1, alpha, beta, false));
            state.undo_move(move_);
            if value >= beta {
                break;
            }
            alpha = max(alpha, value);
        }
        value
    } else {
        let mut value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            value = min(value, minimax(state, depth - 1, alpha, beta, true));
            state.undo_move(move_);
            if value <= alpha {
                break;
            }
            beta = min(beta, value);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::GameState;
    
    #[test]
    fn test_minimax_with_tt_basic() {
        let mut engine = MinimaxWithTT::new(15);
        let mut state = GameState::new(15, 5);
        
        // Make a few moves to create a non-trivial position
        state.make_move((7, 7));
        state.make_move((7, 8));
        state.make_move((8, 7));
        
        let best_move = engine.find_best_move(&mut state, 3);
        assert!(best_move.is_some());
        
        // Check that we got some transposition table hits
        println!("TT stats: {:?}", engine.get_tt_stats());
        assert!(engine.tt_hits > 0);
    }
    
    #[test]
    fn test_hash_consistency() {
        let engine = MinimaxWithTT::new(15);
        let mut state = GameState::new(15, 5);
        
        let initial_hash = engine.get_hash(&state);
        
        // Make and undo a move
        state.make_move((7, 7));
        let after_move_hash = engine.get_hash(&state);
        state.undo_move((7, 7));
        let after_undo_hash = engine.get_hash(&state);
        
        assert_ne!(initial_hash, after_move_hash);
        assert_eq!(initial_hash, after_undo_hash);
    }
}
