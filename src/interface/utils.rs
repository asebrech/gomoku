use crate::ai::search::iterative_deepening_search;
use crate::core::state::GameState;
// use std::time::Duration;

/// Find the best move for the current player in the given game state.
/// Returns None if the game is terminal (won/draw) or no valid moves exist.
pub fn find_best_move(state: &mut GameState, max_depth: i32) -> Option<(usize, usize)> {
    // Check if game is terminal first
    if state.is_terminal() {
        return None;
    }
    
    // Check if there are any possible moves
    let possible_moves = state.get_possible_moves();
    if possible_moves.is_empty() {
        return None;
    }
    
    // No time limit, just search to max_depth
    iterative_deepening_search(state, max_depth)
}
