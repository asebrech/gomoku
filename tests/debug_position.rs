use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::ai::heuristic::Heuristic;

#[test]
fn debug_position_analysis() {
    let mut state = GameState::new(19, 5);
    
    // Recreate the exact position from the test
    state.board.place_stone(9, 9, Player::Max);   // Center white stone
    state.board.place_stone(10, 10, Player::Min); // Black response
    state.board.place_stone(8, 8, Player::Max);   // White
    state.board.place_stone(11, 11, Player::Min); // Black
    state.board.place_stone(7, 9, Player::Max);   // White
    
    // Critical diagonal threat (3 in a row with both ends open)
    state.board.place_stone(8, 7, Player::Min);   // Start of diagonal
    state.board.place_stone(9, 8, Player::Min);   // Continue diagonal
    state.board.place_stone(10, 9, Player::Min);  // Third in diagonal
    
    state.board.place_stone(7, 7, Player::Max);
    state.current_player = Player::Max;
    
    println!("Initial position evaluation: {}", Heuristic::evaluate(&state, 0));
    
    // Try the blocking move
    let mut state_block = state.clone();
    state_block.make_move((11, 10));
    println!("After blocking at (11,10): {}", Heuristic::evaluate(&state_block, 0));
    
    // Try the AI's preferred move
    let mut state_ai = state.clone();
    state_ai.make_move((6, 6));
    println!("After move at (6,6): {}", Heuristic::evaluate(&state_ai, 0));
    
    // What happens if Black continues the threat after (6,6)?
    state_ai.current_player = Player::Min;
    state_ai.make_move((11, 10)); // Black completes the diagonal
    println!("After (6,6) followed by Black (11,10): {}", Heuristic::evaluate(&state_ai, 0));
    
    // Check if Black wins
    println!("Does Black win after (11,10)? {}", state_ai.is_terminal());
    if state_ai.is_terminal() {
        println!("Winner: {:?}", state_ai.winner);
    }
    
    // Try another candidate move
    let mut state_other = state.clone();
    state_other.make_move((7, 6));
    println!("After move at (7,6): {}", Heuristic::evaluate(&state_other, 0));
    
    // Print the positions for visual inspection
    println!("\nCritical pieces on board:");
    for row in 6..13 {
        for col in 6..13 {
            match state.board.get_player(row, col) {
                Some(Player::Max) => print!("W "),
                Some(Player::Min) => print!("B "),
                None => print!(". "),
            }
        }
        println!(" <- row {}", row);
    }
    println!("   6 7 8 9 1011");
}
