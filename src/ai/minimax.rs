use crate::core::state::GameState;
use std::cmp::{max, min};

use super::{heuristic::Heuristic, move_ordering::MoveOrdering, transposition::{TranspositionTable, EntryType}};

pub fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
) -> i32 {
    let original_alpha = alpha;
    let hash_key = state.hash();
    
    // Probe transposition table
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    if tt_result.cutoff {
        return tt_result.value.unwrap();
    }

    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state, depth);
        tt.store(hash_key, eval, depth, EntryType::Exact, None);
        return eval;
    }

    let mut moves = state.get_possible_moves();
    MoveOrdering::order_moves(state, &mut moves);
    
    // Try the best move from TT first if available
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
            let eval = minimax(state, depth - 1, alpha, beta, false, tt);
            state.undo_move(move_);
            
            if eval > value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value >= beta {
                break; // Beta cutoff
            }
            alpha = max(alpha, value);
        }
    } else {
        value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            let eval = minimax(state, depth - 1, alpha, beta, true, tt);
            state.undo_move(move_);
            
            if eval < value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value <= alpha {
                break; // Alpha cutoff
            }
            beta = min(beta, value);
        }
    }

    // Store in transposition table with appropriate entry type
    let entry_type = if value <= original_alpha {
        EntryType::UpperBound
    } else if value >= beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };
    
    tt.store(hash_key, value, depth, entry_type, best_move);
    value
}
