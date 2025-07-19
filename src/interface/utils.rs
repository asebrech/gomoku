use crate::ai::{minimax::{minimax, reset_profiling, print_profiling}, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;
use std::time::Instant;

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let start_time = Instant::now();
    reset_profiling();
    
    let mut best_move = None;
    let current_player = state.current_player; // Store the player who is making the move
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

	println!(" Transposition Table Size: {}", tt.size());
    
    // Use fast iteration and collect moves into a temporary vec only once at root
    let move_gen_start = Instant::now();
    let mut moves = Vec::with_capacity(64);
    state.for_each_possible_move(|mv| moves.push(mv));
    println!("Root move generation took: {}μs for {} moves", move_gen_start.elapsed().as_micros(), moves.len());
    
    for mv in moves {
        state.make_move(mv);
        // After make_move, the current_player has switched, so we need to use the opposite
        // for maximizing_player parameter
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            current_player == Player::Min, // This is correct because we want to maximize if current_player is Max
            tt,
        );
        state.undo_move(mv);

        if (current_player == Player::Max && score > best_score)
            || (current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    let total_time = start_time.elapsed();
    println!("Total search time: {}ms", total_time.as_millis());
    print_profiling();
    
    best_move
}
