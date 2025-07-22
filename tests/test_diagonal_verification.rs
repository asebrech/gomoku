use gomoku::core::state::GameState;
use gomoku::core::board::Player;

#[test]
fn test_diagonal_setup_verification() {
    let mut state = GameState::new(19, 5);
    
    // Set up the diagonal
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    state.board.place_stone(11, 10, Player::Min);
    
    // Test if Black can win by playing at (7, 6)
    let mut test_state = state.clone();
    test_state.current_player = Player::Min;
    test_state.make_move((7, 6));
    
    println!("After Black plays (7, 6): terminal={}, winner={:?}", 
             test_state.is_terminal(), test_state.winner);
    
    if test_state.is_terminal() && test_state.winner == Some(Player::Min) {
        println!("✓ (7, 6) is indeed a winning move for Black");
    }
    
    // Test if Black can win by playing at (12, 11)
    let mut test_state2 = state.clone();
    test_state2.current_player = Player::Min;
    test_state2.make_move((12, 11));
    
    println!("After Black plays (12, 11): terminal={}, winner={:?}", 
             test_state2.is_terminal(), test_state2.winner);
    
    if test_state2.is_terminal() && test_state2.winner == Some(Player::Min) {
        println!("✓ (12, 11) is indeed a winning move for Black");
    }
    
    // Now test what happens if White blocks (7, 6)
    let mut blocked_state = state.clone();
    blocked_state.current_player = Player::Max;
    blocked_state.make_move((7, 6)); // White blocks
    
    blocked_state.current_player = Player::Min;
    blocked_state.make_move((12, 11)); // Black tries the other end
    
    println!("After White blocks (7, 6) and Black plays (12, 11): terminal={}, winner={:?}", 
             blocked_state.is_terminal(), blocked_state.winner);
             
    // This should still be a win for Black!
    assert!(blocked_state.is_terminal());
    assert_eq!(blocked_state.winner, Some(Player::Min));
    
    println!("🚨 CONFIRMED: Blocking (7, 6) doesn't prevent Black from winning at (12, 11)!");
    
    // So the AI needs to block (12, 11) instead, or both ends are threats
}
