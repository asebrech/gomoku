// Example usage of the enhanced minimax with transposition table

use crate::ai::minimax::{init_minimax, minimax, clear_tt};
use crate::core::state::GameState;

fn example_usage() {
// Initialize the transposition table and Zobrist hashing once
init_minimax(15); // for a 15x15 board

    // Create your game state
    let mut state = GameState::new(15, 5);

    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));

    // Use minimax as before - it now automatically uses the transposition table
    let evaluation = minimax(&mut state, 6, i32::MIN + 1, i32::MAX - 1, true);

    println!("Position evaluation: {}", evaluation);

    // Optional: clear the transposition table between games
    clear_tt();

}
