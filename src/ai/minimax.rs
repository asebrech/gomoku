use crate::core::state::GameState;
use std::cmp::{max, min};

use super::{heuristic::Heuristic, transposition::TranspositionTable, transposition::TTFlag};

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
        tt.store(state.hash(), eval, depth, TTFlag::Exact);
        return eval;
    }

    let key = state.hash();
    if let Some(score) = tt.lookup(key, depth, alpha, beta) {
        return score;
    }

    let mut moves = state.get_possible_moves();
    if moves.is_empty() {
        let eval = Heuristic::evaluate(state);
        tt.store(key, eval, depth, TTFlag::Exact);
        return eval;
    }

    Heuristic::order_moves(state, &mut moves);

    if maximizing_player {
        let mut value = i32::MIN;
        let original_alpha = alpha;

        for move_ in moves {
            state.make_move(move_);
            let score = minimax(state, depth - 1, alpha, beta, false, tt);
            state.undo_move(move_);

            value = max(value, score);
            alpha = max(alpha, value);

            if beta <= alpha {
                break; // Beta cutoff
            }
        }

        // Store in transposition table with appropriate flag
        let flag = if value <= original_alpha {
            TTFlag::UpperBound
        } else if value >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        tt.store(key, value, depth, flag);
        value
    } else {
        let mut value = i32::MAX;
        let original_beta = beta;

        for move_ in moves {
            state.make_move(move_);
            let score = minimax(state, depth - 1, alpha, beta, true, tt);
            state.undo_move(move_);

            value = min(value, score);
            beta = min(beta, value);

            if beta <= alpha {
                break; // Alpha cutoff
            }
        }

        // Store in transposition table with appropriate flag
        let flag = if value <= alpha {
            TTFlag::UpperBound
        } else if value >= original_beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        tt.store(key, value, depth, flag);
        value
    }
}
