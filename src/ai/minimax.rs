use crate::core::state::GameState;
use std::cmp::{max, min};

use super::{heuristic::Heuristic, move_ordering::MoveOrdering};

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
