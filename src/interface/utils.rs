use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;

pub fn find_best_move(state: &mut GameState, depth: i32) -> Option<(usize, usize)> {
    let current_player = state.current_player;
    let mut tt = TranspositionTable::new_default();
    let mut best_move = None;
    
    // Iterative deepening: start from depth 1 and go up to target depth
    for current_depth in 1..=depth {
        let mut current_best_move = None;
        let mut best_score = if current_player == Player::Max {
            i32::MIN
        } else {
            i32::MAX
        };
        
        let mut moves = state.get_possible_moves();
        
        // Use previous iteration's best move for better move ordering
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        for mv in moves {
            state.make_move(mv);
            // After make_move, the current_player has switched, so we need to use the opposite
            // for maximizing_player parameter
            let score = minimax(
                state,
                current_depth - 1,
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
                current_best_move = Some(mv);
            }
        }
        
        // Update best move from this iteration
        if current_best_move.is_some() {
            best_move = current_best_move;
        }
    }

    best_move
}
