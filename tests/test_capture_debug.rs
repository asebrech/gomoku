use gomoku::core::state::GameState;
use gomoku::core::captures::CaptureHandler;

#[test]
fn debug_capture_scenario() {
    let mut state = GameState::new(19, 5, 5);

    println!("=== Setting up proper capture scenario ===");
    
    // Create a clear capture pattern: O-X-X-? where Min will place at ? to capture X-X
    state.make_move((9, 8)); // Max at (9,8)
    println!("After Max (9,8): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);
    
    state.make_move((9, 6)); // Min at (9,6) - this will be the first O
    println!("After Min (9,6): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);
    
    state.make_move((9, 7)); // Max at (9,7) - this will be captured (first X)
    println!("After Max (9,7): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);
    
    state.make_move((8, 8)); // Min somewhere else
    println!("After Min (8,8): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);
    
    // state.make_move((9, 8)); // Max already there - this will be captured (second X)
    state.make_move((10, 10)); // Min somewhere else
    println!("After Min (10,10): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);

    println!("\n=== Before capture move ===");
    println!("Board state around target area:");
    for row in 6..13 {
        print!("Row {}: ", row);
        for col in 4..12 {
            let cell = if let Some(player) = state.board.get_player(row, col) {
                match player {
                    gomoku::core::board::Player::Max => "X",
                    gomoku::core::board::Player::Min => "O",
                }
            } else {
                "."
            };
            print!("{} ", cell);
        }
        println!();
    }

    // Now we should have: O . X X . . .
    // If Min plays at (9,9), we get: O . X X O which creates O-X-X-O capture pattern
    let potential_captures = CaptureHandler::detect_captures(&state.board, 9, 9, gomoku::core::board::Player::Min);
    println!("Potential captures at (9,9) for Min: {:?}", potential_captures);

    // Min should capture by playing at (9,9)
    state.make_move((9, 9)); // Min - this should trigger capture of (9,7) and (9,8)
    println!("After Min (9,9): Max captures: {}, Min captures: {}", state.max_captures, state.min_captures);
    
    println!("\n=== Final board state ===");
    for row in 6..13 {
        print!("Row {}: ", row);
        for col in 4..12 {
            let cell = if let Some(player) = state.board.get_player(row, col) {
                match player {
                    gomoku::core::board::Player::Max => "X",
                    gomoku::core::board::Player::Min => "O",
                }
            } else {
                "."
            };
            print!("{} ", cell);
        }
        println!();
    }
    
    println!("\n=== Final verification ===");
    println!("Total captures: Max={}, Min={}", state.max_captures, state.min_captures);
    assert!(state.min_captures > 0 || state.max_captures > 0, "No captures occurred!");
}
