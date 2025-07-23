use gomoku::core::state::GameState;
use gomoku::core::board::Player;

#[test]
fn test_ai_choice_analysis() {
    let mut state = GameState::new(19, 5);
    
    // Recreate the realistic game scenario
    state.board.place_stone(9, 9, Player::Max);   // Center white stone
    state.board.place_stone(10, 10, Player::Min); // Black response
    state.board.place_stone(8, 8, Player::Max);   // White
    state.board.place_stone(11, 11, Player::Min); // Black
    state.board.place_stone(7, 9, Player::Max);   // White
    
    // The critical diagonal threat (3 in a row with both ends open)
    state.board.place_stone(8, 7, Player::Min);   // Start of diagonal
    state.board.place_stone(9, 8, Player::Min);   // Continue diagonal
    state.board.place_stone(10, 9, Player::Min);  // Third in diagonal
    
    state.board.place_stone(7, 7, Player::Max);
    state.current_player = Player::Max;
    
    println!("Board state before AI move:");
    for row in 5..15 {
        for col in 5..15 {
            let symbol = match state.board.get_player(row, col) {
                Some(Player::Max) => "W",
                Some(Player::Min) => "B",
                None => ".",
            };
            print!("{} ", symbol);
        }
        println!();
    }
    
    // Test what happens if AI plays the blocking move (11, 10)
    println!("\n=== Testing blocking move (11, 10) ===");
    let mut blocking_state = state.clone();
    blocking_state.make_move((11, 10));
    blocking_state.current_player = Player::Min;
    
    // Can black still win immediately?
    let can_black_win_at_7_6 = {
        let mut test_state = blocking_state.clone();
        test_state.make_move((7, 6));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    println!("After blocking at (11, 10), can black win at (7, 6)? {}", can_black_win_at_7_6);
    
    // Test what happens if AI plays the AI's choice (6, 6)
    println!("\n=== Testing AI's choice (6, 6) ===");
    let mut ai_choice_state = state.clone();
    ai_choice_state.make_move((6, 6));
    
    println!("Board after AI plays (6, 6):");
    for row in 5..15 {
        for col in 5..15 {
            let symbol = match ai_choice_state.board.get_player(row, col) {
                Some(Player::Max) => "W",
                Some(Player::Min) => "B",
                None => ".",
            };
            print!("{} ", symbol);
        }
        println!();
    }
    
    // Check if this creates a threat for white
    ai_choice_state.current_player = Player::Min;
    
    // Check if black can still make the diagonal threat
    let mut black_diagonal_state = ai_choice_state.clone();
    black_diagonal_state.make_move((7, 6));
    
    println!("Does black's diagonal threat still work after AI plays (6, 6)? {}", 
             black_diagonal_state.is_terminal() && black_diagonal_state.winner == Some(Player::Min));
    
    // Check what white threatens at (6, 6)
    let mut check_white_threat = ai_choice_state.clone();
    check_white_threat.current_player = Player::Max;
    
    // Look for white's threats around (6, 6)
    println!("\nChecking white's potential threats around (6, 6)...");
    
    // Look at the line from (6, 6) that might be threatening
    let directions = [(0i32, 1i32), (1, 0), (1, 1), (1, -1)];
    for &(dr, dc) in &directions {
        let mut white_count = 1; // Count the stone at (6, 6)
        let mut line_desc = String::from("(6,6)");
        
        // Check in positive direction
        for i in 1..5 {
            let r = 6i32 + i * dr;
            let c = 6i32 + i * dc;
            if r >= 0 && c >= 0 && (r as usize) < 19 && (c as usize) < 19 && 
               check_white_threat.board.get_player(r as usize, c as usize) == Some(Player::Max) {
                white_count += 1;
                line_desc.push_str(&format!(" -> ({},{})", r, c));
            } else {
                break;
            }
        }
        
        // Check in negative direction
        for i in 1..5 {
            let r = 6i32 - i * dr;
            let c = 6i32 - i * dc;
            if r >= 0 && c >= 0 && (r as usize) < 19 && (c as usize) < 19 && 
               check_white_threat.board.get_player(r as usize, c as usize) == Some(Player::Max) {
                white_count += 1;
                line_desc = format!("({},{}) -> ", r, c) + &line_desc;
            } else {
                break;
            }
        }
        
        if white_count >= 3 {
            println!("White has {} in a row: {}", white_count, line_desc);
        }
    }
}
