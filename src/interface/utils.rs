use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;

pub fn find_best_move(state: &mut GameState, depth: i32) -> Option<(usize, usize)> {
    let mut best_move = None;
    let current_player = state.current_player; // Store the player who is making the move
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut tt = TranspositionTable::new_default();

    for mv in state.get_possible_moves() {
        state.make_move(mv);
        // After make_move, the current_player has switched, so we need to use the opposite
        // for maximizing_player parameter
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            current_player == Player::Min, // This is correct because we want to maximize if current_player is Max
            &mut tt,
        );
        state.undo_move(mv);

        if (current_player == Player::Max && score > best_score)
            || (current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}
