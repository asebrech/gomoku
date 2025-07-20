use crate::core::state::GameState;
use crate::ai::zobrist::ZobristHash;
use crate::ai::transposition_table::{TranspositionTable, EntryType};
use std::cmp::{max, min};
use rayon::prelude::*;

use super::{heuristic::Heuristic, move_ordering::MoveOrdering};

/// Enhanced minimax engine with transposition table and parallel search
pub struct MinimaxEngine {
    zobrist: ZobristHash,
    tt: TranspositionTable,
}

impl MinimaxEngine {
    /// Create a new minimax engine
    pub fn new(board_size: usize) -> Self {
        Self {
            zobrist: ZobristHash::new(board_size),
            tt: TranspositionTable::new_default(),
        }
    }
    
    /// Create a new minimax engine with custom table size
    pub fn new_with_table_size(board_size: usize, table_size: usize) -> Self {
        Self {
            zobrist: ZobristHash::new(board_size),
            tt: TranspositionTable::new(table_size),
        }
    }
    
    /// Find the best move using parallel search at root level
    pub fn find_best_move(&self, state: &GameState, depth: i32) -> Option<(usize, usize)> {
        let moves = state.get_possible_moves();
        if moves.is_empty() {
            return None;
        }
        
        // Advance age for new search
        self.tt.advance_age();
        
        let initial_hash = self.zobrist.compute_hash(state);
        
        // Parallel search at root level
        let result = moves.into_par_iter().map(|mv| {
            let mut state_copy = state.clone();
            state_copy.make_move(mv);
            
            // Calculate new hash incrementally
            let new_hash = self.zobrist.update_hash_make_move(
                initial_hash, 
                mv.0, 
                mv.1, 
                state.current_player
            );
            
            // Each thread gets its own shared reference to TT
            let tt_shared = self.tt.clone_shared();
            let zobrist_copy = ZobristHash::new(self.zobrist.board_size());
            
            let value = -self.minimax(
                &mut state_copy,
                depth - 1,
                i32::MIN + 1,
                i32::MAX - 1,
                false,
                new_hash,
                &zobrist_copy,
                &tt_shared,
            );
            
            (mv, value)
        }).max_by_key(|&(_, value)| value);
        
        result.map(|(mv, _)| mv)
    }
    
    /// Enhanced minimax with transposition table
    fn minimax(
        &self,
        state: &mut GameState,
        depth: i32,
        mut alpha: i32,
        mut beta: i32,
        maximizing_player: bool,
        hash: u64,
        zobrist: &ZobristHash,
        tt: &TranspositionTable,
    ) -> i32 {
        // Check transposition table first
        let tt_result = tt.probe(hash, depth, alpha, beta);
        if tt_result.cutoff {
            if let Some(value) = tt_result.value {
                return value;
            }
        }
        
        // Terminal node check
        if depth == 0 || state.is_terminal() {
            let eval = Heuristic::evaluate(state, depth);
            tt.store(hash, eval, depth, EntryType::Exact, None);
            return eval;
        }
        
        // Get possible moves
        let mut moves = state.get_possible_moves();
        if moves.is_empty() {
            let eval = Heuristic::evaluate(state, depth);
            tt.store(hash, eval, depth, EntryType::Exact, None);
            return eval;
        }
        
        // Move ordering with TT hint
        MoveOrdering::order_moves(state, &mut moves);
        if let Some(tt_move) = tt_result.best_move {
            if let Some(pos) = moves.iter().position(|&m| m == tt_move) {
                moves.swap(0, pos);
            }
        }
        
        let original_alpha = alpha;
        let mut best_move = None;
        let mut best_value = if maximizing_player { i32::MIN } else { i32::MAX };
        
        for mv in moves {
            // Calculate new hash
            let new_hash = zobrist.update_hash_make_move(hash, mv.0, mv.1, state.current_player);
            
            state.make_move(mv);
            let value = self.minimax(
                state, 
                depth - 1, 
                alpha, 
                beta, 
                !maximizing_player, 
                new_hash,
                zobrist,
                tt
            );
            state.undo_move(mv);
            
            if maximizing_player {
                if value > best_value {
                    best_value = value;
                    best_move = Some(mv);
                }
                alpha = max(alpha, value);
                if beta <= alpha {
                    break; // Beta cutoff
                }
            } else {
                if value < best_value {
                    best_value = value;
                    best_move = Some(mv);
                }
                beta = min(beta, value);
                if beta <= alpha {
                    break; // Alpha cutoff
                }
            }
        }
        
        // Store in transposition table
        let entry_type = if best_value <= original_alpha {
            EntryType::UpperBound
        } else if best_value >= beta {
            EntryType::LowerBound
        } else {
            EntryType::Exact
        };
        
        tt.store(hash, best_value, depth, entry_type, best_move);
        best_value
    }
    
    /// Get transposition table statistics
    pub fn get_tt_stats(&self) -> (usize, f64, u64) {
        let (hits, misses, collisions) = self.tt.get_stats();
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (self.tt.size(), hit_rate, collisions)
    }
    
    /// Clear the transposition table
    pub fn clear_tt(&self) {
        self.tt.clear();
    }
    
    /// Get the zobrist hash for a position
    pub fn get_hash(&self, state: &GameState) -> u64 {
        self.zobrist.compute_hash(state)
    }
}

// Keep the original function for backward compatibility
// Note: This is the old minimax without transposition table
// The enhanced version is now in interface/utils.rs and MinimaxEngine above
/*
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
*/
