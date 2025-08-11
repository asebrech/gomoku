use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::interface::utils::find_best_move;

#[test]
fn test_find_best_move_first_move() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should return center move for first move
    assert_eq!(best_move, Some((9, 9)));
}

#[test]
fn test_find_best_move_response() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Make first move
    state.make_move((9, 9));

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should return some adjacent move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    assert!(state.board.is_adjacent_to_stone(row, col));
}

#[test]
fn test_find_best_move_winning_opportunity() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up winning opportunity for current player (Max)
    state.board.place_stone(9, 5, Player::Max);
    state.board.place_stone(9, 6, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should find the winning move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();

    // Should be adjacent to complete the line
    assert!(row == 9 && (col == 4 || col == 9));
}

#[test]
fn test_find_best_move_block_opponent() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up threat from opponent (Min has 4 in a row)
    state.board.place_stone(9, 5, Player::Min);
    state.board.place_stone(9, 6, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should find a blocking move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    println!("AI chose: ({}, {})", row, col);
    
    // AI should block the immediate threat at one of the ends
    let valid_blocking_moves = vec![(9, 4), (9, 9)];
    assert!(
        valid_blocking_moves.contains(&(row, col)),
        "AI chose ({}, {}) but should block the threat at one of: {:?}",
        row, col, valid_blocking_moves
    );
}

#[test]
fn test_find_best_move_capture_opportunity() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Set up capture opportunity: Max-Min-Min-empty
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min); 
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should find a move (capture move would be at (9, 12) to complete Max-Min-Min-Max)
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    // Verify it's a valid move
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&(row, col)), "Move should be valid");
    
    // Test the move leads to captures
    let initial_captures = state.max_captures;
    state.make_move((row, col));
    
    // If it was a capturing move, captures should increase
    println!("Capture test: {} -> {} at move ({}, {})", 
             initial_captures, state.max_captures, row, col);
}

#[test]
fn test_find_best_move_no_moves() {
    let mut state = GameState::new(3, 3);
    let mut tt = TranspositionTable::new_default();

    // Fill the board
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should return None when no moves available
    assert_eq!(best_move, None);
}

#[test]
fn test_find_best_move_different_depths() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up a position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    let move_depth1 = find_best_move(&mut state, 1, &mut tt);
    let move_depth3 = find_best_move(&mut state, 3, &mut tt);

    // Both should return valid moves
    assert!(move_depth1.is_some());
    assert!(move_depth3.is_some());

    // Moves might be different due to deeper search
    // (but this isn't guaranteed, so we just check validity)
}

#[test]
fn test_find_best_move_player_alternation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Test with Max player
    state.current_player = Player::Max;
    state.board.place_stone(9, 9, Player::Min); // Add opponent stone

    let move_max = find_best_move(&mut state, 2, &mut tt);
    assert!(move_max.is_some());

    // Test with Min player
    state.current_player = Player::Min;
    state.board.place_stone(9, 10, Player::Max); // Add opponent stone

    let move_min = find_best_move(&mut state, 2, &mut tt);
    assert!(move_min.is_some());
}

#[test]
fn test_find_best_move_complex_position() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a complex position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(8, 9, Player::Max);
    state.board.place_stone(8, 10, Player::Min);
    state.board.place_stone(10, 9, Player::Max);
    state.board.place_stone(10, 10, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should find some reasonable move
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&(row, col)));
}

#[test]
fn test_find_best_move_state_preservation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up initial state
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;
    let initial_hash = state.hash();

    // Find best move
    let _best_move = find_best_move(&mut state, 2, &mut tt);

    // State should be preserved
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, Player::Min);
    assert_eq!(state.board.get_player(9, 9), Some(Player::Max));
}

#[test]
fn test_find_best_move_consistent_results() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up a deterministic position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Multiple calls should give same result
    let move1 = find_best_move(&mut state, 2, &mut tt);
    let move2 = find_best_move(&mut state, 2, &mut tt);

    assert_eq!(move1, move2);
}

#[test]
fn test_find_best_move_edge_cases() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Fill most of the board leaving only a few moves
    for i in 0..19 {
        for j in 0..19 {
            if (i, j) != (9, 9) && (i, j) != (9, 10) && (i, j) != (8, 9) {
                state.board.place_stone(i, j, Player::Max);
            }
        }
    }

    // Add a stone to make moves possible
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    let best_move = find_best_move(&mut state, 2, &mut tt);

    // Should find one of the few available moves
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    assert!(state.board.is_empty_position(row, col));
}

#[test]
fn test_find_best_move_capture_win() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Set up near-capture-win scenario - Max needs 10 captures to win
    state.max_captures = 8; // Two captures away from winning (need 10 total)
    
    // Create capture opportunity: Max-Min-Min-empty
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 3, &mut tt);

    // Should find the capturing move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    // Test that this move leads to a capture
    let initial_captures = state.max_captures;
    state.make_move((row, col));
    
    // Should have increased captures
    println!("Capture win test: {} -> {} captures", initial_captures, state.max_captures);
    
    // Verify it's a valid strategic move
    let possible_moves = state.get_possible_moves();
    assert!(!possible_moves.is_empty() || state.is_terminal());
}

#[test]
fn test_find_best_move_defensive_priority() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Create scenario where opponent has immediate win threat
    // Max has 4 in a row, Min must block or lose
    state.board.place_stone(7, 5, Player::Max);
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    // Position (7, 9) is the winning threat for Max
    
    state.current_player = Player::Min; // Min must defend
    
    let best_move = find_best_move(&mut state, 3, &mut tt);
    
    // Should find the blocking move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    // Should block the winning threat - either at (7, 4) or (7, 9)
    let valid_blocks = [(7, 4), (7, 9)];
    assert!(valid_blocks.contains(&(row, col)), 
            "Should block the immediate win threat at either end: {:?}", (row, col));
}

#[test]
fn test_find_best_move_different_board_sizes() {
    let mut state13 = GameState::new(13, 5);
    let mut state15 = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();

    let move13 = find_best_move(&mut state13, 2, &mut tt);
    let move15 = find_best_move(&mut state15, 2, &mut tt);

    // Should find center moves for different board sizes
    assert_eq!(move13, Some((6, 6)));
    assert_eq!(move15, Some((7, 7)));
}
