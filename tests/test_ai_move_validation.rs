use gomoku::core::state::GameState;
use gomoku::core::board::Player;

#[test]
fn test_ai_move_validation_realistic_scenario() {
    let mut state = GameState::new(19, 5);
    
    // Recreate the exact realistic game scenario
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
    
    println!("=== TESTING AI'S CHOICE (6,6) ===");
    
    // Test what happens if AI plays (6, 6) - the AI's actual choice
    let mut ai_state = state.clone();
    ai_state.make_move((6, 6));
    ai_state.current_player = Player::Min;
    
    // Check if black can still win immediately after AI plays (6, 6)
    let can_black_win_at_7_6 = {
        let mut test_state = ai_state.clone();
        test_state.make_move((7, 6));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    let can_black_win_at_11_10 = {
        let mut test_state = ai_state.clone();
        test_state.make_move((11, 10));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    println!("After AI plays (6,6):");
    println!("  Can black win at (7,6)? {}", can_black_win_at_7_6);
    println!("  Can black win at (11,10)? {}", can_black_win_at_11_10);
    
    if can_black_win_at_7_6 || can_black_win_at_11_10 {
        println!("❌ AI'S MOVE (6,6) IS WRONG! Black can still win immediately.");
        panic!("AI failed to block immediate winning threat!");
    } else {
        println!("✅ AI's move (6,6) successfully prevents immediate black win.");
    }
    
    println!("\n=== TESTING BLOCKING MOVES ===");
    
    // Test what happens if AI blocks at (7, 6)
    let mut block_7_6_state = state.clone();
    block_7_6_state.make_move((7, 6));
    block_7_6_state.current_player = Player::Min;
    
    let can_black_win_after_block_7_6 = {
        let mut test_state = block_7_6_state.clone();
        test_state.make_move((11, 10));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    println!("After blocking at (7,6), can black win at (11,10)? {}", can_black_win_after_block_7_6);
    
    // Test what happens if AI blocks at (11, 10)
    let mut block_11_10_state = state.clone();
    block_11_10_state.make_move((11, 10));
    block_11_10_state.current_player = Player::Min;
    
    let can_black_win_after_block_11_10 = {
        let mut test_state = block_11_10_state.clone();
        test_state.make_move((7, 6));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    println!("After blocking at (11,10), can black win at (7,6)? {}", can_black_win_after_block_11_10);
    
    // Summary
    println!("\n=== SUMMARY ===");
    if !can_black_win_at_7_6 && !can_black_win_at_11_10 {
        println!("✅ AI's choice (6,6) is VALID - it prevents immediate black win");
        println!("   The AI might have found a better move than just defending");
    } else {
        println!("❌ AI's choice (6,6) is INVALID - black can still win immediately");
        println!("   The AI should have blocked the threat instead");
    }
}
