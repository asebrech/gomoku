use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_horizontal_capture() {
    let mut state = GameState::new(19, 5);

    // Set up: X - O - O - X (horizontal capture pattern)
    // We need to create a pattern where Max captures Min stones
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(5, 6, Player::Min);
    state.board.place_stone(5, 7, Player::Min);
    
    // Check initial capture count
    assert_eq!(state.max_captures, 0);
    
    // Place the capturing stone - this should automatically handle captures
    state.make_move((5, 8)); // Player Max completes the capture pattern Max-Min-Min-Max

    // Test that captures were executed
    assert_eq!(state.max_captures, 1); // One pair captured
    assert_eq!(state.board.get_player(5, 6), None); // Captured stones should be removed
    assert_eq!(state.board.get_player(5, 7), None);
    
    // Original stones should remain
    assert_eq!(state.board.get_player(5, 5), Some(Player::Max));
    assert_eq!(state.board.get_player(5, 8), Some(Player::Max));
}

#[test]
fn test_vertical_capture() {
    let mut state = GameState::new(19, 5);

    // Set up: X - O - O - X (vertical capture pattern)
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(6, 5, Player::Min);
    state.board.place_stone(7, 5, Player::Min);
    
    // Check initial capture count
    assert_eq!(state.max_captures, 0);
    
    // Place the capturing stone
    state.make_move((8, 5)); // Player Max completes the capture pattern

    // Test that captures were executed
    assert_eq!(state.max_captures, 1); // One pair captured
    assert_eq!(state.board.get_player(6, 5), None); // Captured stones should be removed
    assert_eq!(state.board.get_player(7, 5), None);
    
    // Original stones should remain
    assert_eq!(state.board.get_player(5, 5), Some(Player::Max));
    assert_eq!(state.board.get_player(8, 5), Some(Player::Max));
}

#[test]
fn test_diagonal_capture() {
    let mut state = GameState::new(19, 5);

    // Set up diagonal capture pattern
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(6, 6, Player::Min);
    state.board.place_stone(7, 7, Player::Min);
    
    // Check initial capture count
    assert_eq!(state.max_captures, 0);
    
    // Place the capturing stone
    state.make_move((8, 8)); // Player Max completes the capture pattern

    // Test that captures were executed
    assert_eq!(state.max_captures, 1); // One pair captured
    assert_eq!(state.board.get_player(6, 6), None); // Captured stones should be removed
    assert_eq!(state.board.get_player(7, 7), None);
    
    // Original stones should remain
    assert_eq!(state.board.get_player(5, 5), Some(Player::Max));
    assert_eq!(state.board.get_player(8, 8), Some(Player::Max));
}

#[test]
fn test_no_capture_incomplete_pattern() {
    let mut state = GameState::new(19, 5);

    // Set up incomplete pattern (only one opponent stone)
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(5, 6, Player::Min);
    // Missing second opponent stone
    
    // Check initial capture count
    assert_eq!(state.max_captures, 0);
    
    // Place stone - should not trigger capture
    state.make_move((5, 7)); // Player Max - no capture pattern

    // Test that no captures occurred
    assert_eq!(state.max_captures, 0);
    assert_eq!(state.board.get_player(5, 6), Some(Player::Min)); // Stone should remain
}

#[test]
fn test_undo_capture() {
    let mut state = GameState::new(19, 5);

    // Set up capture pattern
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(5, 6, Player::Min);
    state.board.place_stone(5, 7, Player::Min);
    
    // Make the capturing move
    state.make_move((5, 8)); // Player Max captures
    
    // Verify capture occurred
    assert_eq!(state.max_captures, 1);
    assert_eq!(state.board.get_player(5, 6), None);
    assert_eq!(state.board.get_player(5, 7), None);
    
    // Undo the move - we know we just placed at (5, 8)
    state.undo_move((5, 8));
    
    // Verify capture was undone
    assert_eq!(state.max_captures, 0);
    assert_eq!(state.board.get_player(5, 6), Some(Player::Min)); // Stones restored
    assert_eq!(state.board.get_player(5, 7), Some(Player::Min));
    assert_eq!(state.board.get_player(5, 8), None); // Capturing stone removed
}
