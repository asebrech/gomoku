use crate::solver::game_state::GameState;
use std::cmp::{max, min};

pub fn alpha_beta(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
) -> i32 {
    if depth == 0 || state.is_terminal() {
        return state.evaluate();
    }

    if maximizing_player {
        let mut value = i32::MIN;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            value = max(value, alpha_beta(state, depth - 1, alpha, beta, false));
            state.undo_move(move_);
            if value >= beta {
                break; // Beta cutoff
            }
            alpha = max(alpha, value);
        }
        value
    } else {
        let mut value = i32::MAX;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            value = min(value, alpha_beta(state, depth - 1, alpha, beta, true));
            state.undo_move(move_);
            if value <= alpha {
                break; // Alpha cutoff
            }
            beta = min(beta, value);
        }
        value
    }
}
