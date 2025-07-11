use crate::game::state::GameState;
use std::cmp::{max, min};

use super::{heuristic::Heuristic, transposition::TranspositionTable};

/// Minimax algorithm with alpha-beta pruning and transposition table
/// 
/// # Arguments
/// 
/// * `state` - The current game state
/// * `depth` - How deep to search (0 = evaluate current position)
/// * `alpha` - Alpha value for pruning
/// * `beta` - Beta value for pruning
/// * `maximizing_player` - True if current player is maximizing
/// * `tt` - Transposition table for memoization
/// 
/// # Returns
/// 
/// The evaluation score for the current position
pub fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
) -> i32 {
    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state);
        tt.store(state.hash(), eval);
        return eval;
    }

    let key = state.hash();
    if let Some(score) = tt.lookup(key) {
        return score;
    }

    if maximizing_player {
        let mut value = i32::MIN;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            value = max(value, minimax(state, depth - 1, alpha, beta, false, tt));
            state.undo_move(move_);
            if value >= beta {
                break;
            }
            alpha = max(alpha, value);
        }
        tt.store(key, value);
        value
    } else {
        let mut value = i32::MAX;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            value = min(value, minimax(state, depth - 1, alpha, beta, true, tt));
            state.undo_move(move_);
            if value <= alpha {
                break;
            }
            beta = min(beta, value);
        }
        tt.store(key, value);
        value
    }
}
