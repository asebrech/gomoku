use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::game::game_state::GameState;

use super::game_state::Player;

pub fn find_best_move(state: &mut GameState, depth: i32) -> Option<(usize, usize)> {
    let mut best_move = None;
    let mut best_score = if state.current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut tt = TranspositionTable::new();

    for mv in state.get_possible_moves() {
        state.make_move(mv);
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            state.current_player == Player::Min,
            &mut tt,
        );
        state.undo_move(mv);

        if (state.current_player == Player::Max && score > best_score)
            || (state.current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}
