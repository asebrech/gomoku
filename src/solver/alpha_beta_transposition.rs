use crate::solver::game_state::GameState;
use std::cmp::{max, min};

use super::transposition::TranspositionTable;

pub fn alpha_beta_transposition(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
) -> i32 {
    if depth == 0 || state.is_terminal() {
        let eval = state.evaluate();
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
            value = max(
                value,
                alpha_beta_transposition(state, depth - 1, alpha, beta, false, tt),
            );
            state.undo_move(move_);
            if value >= beta {
                break; // Beta cutoff
            }
            alpha = max(alpha, value);
        }
        tt.store(key, value);
        value
    } else {
        let mut value = i32::MAX;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            value = min(
                value,
                alpha_beta_transposition(state, depth - 1, alpha, beta, true, tt),
            );
            state.undo_move(move_);
            if value <= alpha {
                break; // Alpha cutoff
            }
            beta = min(beta, value);
        }
        tt.store(key, value);
        value
    }
}
