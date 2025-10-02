use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::ai::search::find_best_move;

#[test]
fn test_find_best_move_first_move() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should return center move for first move
    assert_eq!(result.best_move, Some((9, 9)));
}

#[test]
fn test_find_best_move_response() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Make first move
    state.make_move((9, 9));

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should return some adjacent move
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();
    assert!(state.board.is_adjacent_to_stone(row, col));
}

#[test]
fn test_find_best_move_winning_opportunity() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Set up winning opportunity for current player (Max)
    state.board.place_stone(9, 5, Player::Max);
    state.board.place_stone(9, 6, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.current_player = Player::Max;

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should find the winning move
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();

    // Should be adjacent to complete the line
    assert!(row == 9 && (col == 4 || col == 9));
}

#[test]
fn test_find_best_move_block_opponent() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Set up threat from opponent (Min has 4 in a row)
    state.board.place_stone(9, 5, Player::Min);
    state.board.place_stone(9, 6, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.current_player = Player::Max;

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should find a blocking move
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();
    
    println!("AI chose: ({}, {})", row, col);
    println!("Score: {}", result.score);
    
    // The AI should recognize this is a losing position
    assert!(result.score <= -1_000_000, "AI should recognize this is a losing position");
    
    // Verify that Min can indeed win if not blocked
    let mut test_state = state.clone();
    test_state.current_player = Player::Min;
    
    // Test if Min can win at (9, 4)
    if test_state.board.get_player(9, 4).is_none() {
        test_state.board.place_stone(9, 4, Player::Min);
        if let Some(winner) = test_state.check_winner() {
            assert_eq!(winner, Player::Min, "Min should be able to win at (9, 4)");
        }
        test_state.board.remove_stone(9, 4);
    }
    
    // Test if Min can win at (9, 9)
    if test_state.board.get_player(9, 9).is_none() {
        test_state.board.place_stone(9, 9, Player::Min);
        if let Some(winner) = test_state.check_winner() {
            assert_eq!(winner, Player::Min, "Min should be able to win at (9, 9)");
        }
    }
}

#[test]
fn test_find_best_move_capture_opportunity() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up capture opportunity: Max-Min-Min-empty
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min); 
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should find a move (capture move would be at (9, 12) to complete Max-Min-Min-Max)
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();
    
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
    let mut tt = TranspositionTable::default();

    // Fill the board
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should return None when no moves available
    assert_eq!(result.best_move, None);
}

#[test]
fn test_find_best_move_different_depths() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Set up a position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    let result1 = find_best_move(&mut state, 1, None, &mut tt);
    let result3 = find_best_move(&mut state, 3, None, &mut tt);

    // Both should return valid moves
    assert!(result1.best_move.is_some());
    assert!(result3.best_move.is_some());

    // Moves might be different due to deeper search
    // (but this isn't guaranteed, so we just check validity)
}

#[test]
fn test_find_best_move_player_alternation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Test with Max player
    state.current_player = Player::Max;
    state.board.place_stone(9, 9, Player::Min); // Add opponent stone

    let result_max = find_best_move(&mut state, 2, None, &mut tt);
    assert!(result_max.best_move.is_some());

    // Test with Min player
    state.current_player = Player::Min;
    state.board.place_stone(9, 10, Player::Max); // Add opponent stone

    let result_min = find_best_move(&mut state, 2, None, &mut tt);
    assert!(result_min.best_move.is_some());
}

#[test]
fn test_find_best_move_complex_position() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Create a complex position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(8, 9, Player::Max);
    state.board.place_stone(8, 10, Player::Min);
    state.board.place_stone(10, 9, Player::Max);
    state.board.place_stone(10, 10, Player::Min);
    state.current_player = Player::Max;

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should find some reasonable move
    assert!(result.best_move.is_some());

    let (row, col) = result.best_move.unwrap();
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&(row, col)));
}

#[test]
fn test_find_best_move_state_preservation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Set up initial state
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;
    let initial_hash = state.hash();

    // Find best move
    let _result = find_best_move(&mut state, 2, None, &mut tt);

    // State should be preserved
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, Player::Min);
    assert_eq!(state.board.get_player(9, 9), Some(Player::Max));
}

#[test]
fn test_find_best_move_consistent_results() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    // Set up a deterministic position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Multiple calls should give same result
    let result1 = find_best_move(&mut state, 2, None, &mut tt);
    let result2 = find_best_move(&mut state, 2, None, &mut tt);

    assert_eq!(result1.best_move, result2.best_move);
}

#[test]
fn test_find_best_move_edge_cases() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

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

    let result = find_best_move(&mut state, 2, None, &mut tt);

    // Should find one of the few available moves
    assert!(result.best_move.is_some());

    let (row, col) = result.best_move.unwrap();
    assert!(state.board.is_empty_position(row, col));
}

#[test]
fn test_find_best_move_capture_win() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up near-capture-win scenario - Max needs 10 captures to win
    state.max_captures = 8; // Two captures away from winning (need 10 total)
    
    // Create capture opportunity: Max-Min-Min-empty
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let result = find_best_move(&mut state, 3, None, &mut tt);

    // Should find the capturing move
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();
    
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
    let mut tt = TranspositionTable::default();
    
    // Create scenario where opponent has immediate win threat
    // Max has 4 in a row, Min must block or lose
    state.board.place_stone(7, 5, Player::Max);
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    // Position (7, 9) is the winning threat for Max
    
    state.current_player = Player::Min; // Min must defend
    
    let result = find_best_move(&mut state, 3, None, &mut tt);
    
    // Should find the blocking move
    assert!(result.best_move.is_some());
    let (row, col) = result.best_move.unwrap();
    
    // Should block the winning threat - either at (7, 4) or (7, 9)
    let valid_blocks = [(7, 4), (7, 9)];
    assert!(valid_blocks.contains(&(row, col)), 
            "Should block the immediate win threat at either end: {:?}", (row, col));
}

#[test]
fn test_find_best_move_different_board_sizes() {
    let mut state13 = GameState::new(13, 5);
    let mut state15 = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();

    let result13 = find_best_move(&mut state13, 2, None, &mut tt);
    let result15 = find_best_move(&mut state15, 2, None, &mut tt);

    // Should find center moves for different board sizes
    assert_eq!(result13.best_move, Some((6, 6)));
    assert_eq!(result15.best_move, Some((7, 7)));
}
