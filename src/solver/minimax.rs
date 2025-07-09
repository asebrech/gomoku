use crate::solver::game_state::GameState;
use std::cmp::{max, min};

pub fn minimax(state: &mut GameState, depth: i32, maximizing_player: bool) -> i32 {
    if depth == 0 || state.is_terminal() {
        return state.evaluate();
    }
    if maximizing_player {
        let mut max_eval = i32::MIN;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            let eval = minimax(state, depth - 1, false);
            state.undo_move(move_);
            max_eval = max(max_eval, eval);
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            let eval = minimax(state, depth - 1, true);
            state.undo_move(move_);
            min_eval = min(min_eval, eval);
        }
        min_eval
    }
}
