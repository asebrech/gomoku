use gomoku::core::board::{Player, initialize_zobrist};
use gomoku::core::state::GameState;
use gomoku::interface::utils::find_best_move;

#[test]
fn test_ai_debug_blocking() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a scenario where the human player (Min) is about to win
    state.make_move((9, 9));   // Max
    state.make_move((10, 9));  // Min
    state.make_move((8, 8));   // Max
    state.make_move((10, 10)); // Min
    state.make_move((7, 7));   // Max
    state.make_move((10, 11)); // Min
    state.make_move((6, 6));   // Max
    state.make_move((10, 12)); // Min - now Min has 4 in a row horizontally
    
    println!("Current board state:");
    print_board(&state);
    
    // Check what moves are available
    let possible_moves = state.get_possible_moves();
    println!("Possible moves: {:?}", possible_moves);
    
    // Check if blocking moves are in the possible moves
    let blocking_moves = vec![(10, 8), (10, 13)];
    for &mv in &blocking_moves {
        if possible_moves.contains(&mv) {
            println!("Blocking move {:?} is available", mv);
            
            // Test what happens if we make this move
            let mut temp_state = state.clone();
            temp_state.current_player = Player::Min;
            temp_state.make_move(mv);
            if temp_state.check_winner() == Some(Player::Min) {
                println!("Move {:?} would result in Min winning!", mv);
            }
        } else {
            println!("Blocking move {:?} is NOT available", mv);
        }
    }
    
    // Check current player
    println!("Current player: {:?}", state.current_player);
    
    let best_move = find_best_move(&mut state, 4);
    println!("AI chose move: {:?}", best_move);
}

fn print_board(state: &GameState) {
    println!("   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8");
    for row in 0..19 {
        print!("{:2} ", row);
        for col in 0..19 {
            match state.board.get_player(row, col) {
                Some(Player::Max) => print!("X "),
                Some(Player::Min) => print!("O "),
                None => print!(". "),
            }
        }
        println!();
    }
    println!();
}
