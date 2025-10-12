//! Minimax search with alpha-beta pruning and MTDF driver.
//!
//! This module contains an alpha-beta implementation augmented with a
//! transposition table (memory) and an MTDF driver. The implementation is
//! intentionally small and focused on clarity rather than squeezing every
//! last micro-optimization.
//!
//! Key algorithms explained:
//! - Alpha-Beta pruning: standard minimax pruning to reduce the number of
//!   nodes visited. (<https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning>)
//! - Transposition table: stores previously computed results keyed by a
//!   Zobrist hash to avoid re-searching identical positions.
//! - MTDF: a memory-enhanced driver around zero-window alpha-beta calls to
//!   converge quickly to the minimax value. See: <https://www.researchgate.net/publication/220728459_MTD-f_A_New_Optimal_Minimax_Search_Algorithm>
//!
//! Notes on integration:
//! - The transposition table returns either exact values or bounds which are
//!   used to cut off search early where possible.

use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::{Duration, Instant};

use super::{heuristic::Heuristic, transposition::{TranspositionTable, EntryType}};

/// Alpha-beta search with transposition table (memory) support.
///
/// This function implements a standard minimax search with alpha-beta
/// pruning. It also probes a transposition table (TT) to find cached
/// results and stores the final value back into the TT. The returned
/// tuple is (value, nodes_visited).
///
/// Important behaviour:
/// - If the TT contains a cutoff (exact value or an applicable bound)
///   the function returns immediately without expanding children.
/// - At leaf nodes the heuristic is used and the value is stored as
///   an Exact entry in the TT.
///
/// See: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
fn alpha_beta_with_memory(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
    start_time: &Instant,
    time_limit: Option<Duration>,
) -> (i32, u64) {
    let original_alpha = alpha;
    let original_beta = beta;
    let hash_key = state.hash();
    let mut nodes_visited = 1u64;
    
    if let Some(limit) = time_limit {
        if start_time.elapsed() >= limit {
            return (0, nodes_visited);
        }
    }
    
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    if tt_result.cutoff {
        return (tt_result.value.unwrap(), nodes_visited);
    }

    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state, depth);
        tt.store(hash_key, eval, depth, EntryType::Exact, None);
        return (eval, nodes_visited);
    }

    let mut moves = state.get_candidate_moves();
    
    if let Some(best_move) = tt_result.best_move {
        if let Some(pos) = moves.iter().position(|&m| m == best_move) {
            moves.swap(0, pos);
        }
    }

    let mut best_move = None;
    let mut value;

    if maximizing_player {
        value = i32::MIN;
        for move_ in moves {
            state.make_move(move_);
            let (eval, child_nodes) = alpha_beta_with_memory(
                state, depth - 1, alpha, beta, false, tt, start_time, time_limit
            );
            state.undo_move(move_);
            nodes_visited += child_nodes;
            
            if eval > value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value >= beta {
                break;
            }
            alpha = max(alpha, value);
        }
    } else {
        value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            let (eval, child_nodes) = alpha_beta_with_memory(
                state, depth - 1, alpha, beta, true, tt, start_time, time_limit
            );
            state.undo_move(move_);
            nodes_visited += child_nodes;
            
            if eval < value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value <= alpha {
                break;
            }
            beta = min(beta, value);
        }
    }

    let entry_type = if value <= original_alpha {
        EntryType::UpperBound
    } else if value >= original_beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };
    
    tt.store(hash_key, value, depth, entry_type, best_move);
    (value, nodes_visited)
}

/// MTDF driver around zero-window alpha-beta searches.
///
/// MTDF repeatedly calls a zero-width alpha-beta (beta-1, beta) to
/// converge on the minimax value using bounds. It requires a
/// transposition table to be efficient since it relies heavily on
/// stored information to avoid re-searching.
///
/// Reference: "MTD(f) - A New Optimal Minimax Search Algorithm".
pub fn mtdf(
    state: &mut GameState,
    first_guess: i32,
    depth: i32,
    tt: &mut TranspositionTable,
    start_time: &Instant,
    time_limit: Option<Duration>,
) -> (i32, u64, Option<(usize, usize)>) {
    let mut g = first_guess;
    let mut upper_bound = i32::MAX;
    let mut lower_bound = i32::MIN;
    let mut total_nodes = 0u64;
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    
    while lower_bound < upper_bound {
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }
        
        let beta = if g == lower_bound { g + 1 } else { g };
        
        let (value, nodes) = alpha_beta_with_memory(
            state,
            depth,
            beta - 1,
            beta,
            is_maximizing,
            tt,
            start_time,
            time_limit,
        );
        
        total_nodes += nodes;
        
        if value < beta {
            upper_bound = value;
        } else {
            lower_bound = value;
        }
        
        g = value;
    }
    
    let hash_key = state.hash();
    let best_move = tt.get_best_move(hash_key);
    
    (g, total_nodes, best_move)
}

