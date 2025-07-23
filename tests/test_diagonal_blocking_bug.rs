use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::{find_best_move, find_best_move_parallel};
use gomoku::ai::transposition::{TranspositionTable, SharedTranspositionTable};

#[test]
fn test_diagonal_blocking_bug_sequential() {
    let mut state = GameState::new(19, 5);
    
    // Recreate a 3-in-a-row diagonal that needs to be blocked
    // Black has 3 stones in diagonal with both ends open
    state.board.place_stone(8, 7, Player::Min); // Black stone (start of diagonal)
    state.board.place_stone(9, 8, Player::Min); // Black stone
    state.board.place_stone(10, 9, Player::Min); // Black stone (3 in a row)
    // Positions (7,6) and (11,10) are open and need to be blocked
    
    // Add some white pieces that were on the board
    state.board.place_stone(7, 7, Player::Max); // White stone (the one that was placed)
    
    // It's white's turn and they MUST block the diagonal at (7,6) or (11,10)
    state.current_player = Player::Max;
    
    let mut tt = TranspositionTable::new_default();
    let best_move = find_best_move(&mut state, 6, None, &mut tt);
    
    println!("Sequential search result: {:?}", best_move);
    
    // The AI should find one of the blocking moves
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    assert!(blocking_moves.contains(&(row, col)), 
           "Sequential: Expected blocking move (7,6) or (11,10) but got ({},{})", row, col);
    
    // Verify it actually blocks the threat
    state.make_move((row, col));
    
    // Black should not be able to win immediately after this block
    let mut test_state = state.clone();
    test_state.current_player = Player::Min;
    
    // Try the other diagonal position
    for &test_move in &[(7, 6), (11, 10)] {
        if test_move != (row, col) {
            if test_state.board.get_player(test_move.0, test_move.1).is_none() {
                test_state.make_move(test_move);
                // This should NOT be a winning position for black if we blocked correctly
                if test_state.is_terminal() && test_state.winner == Some(Player::Min) {
                    panic!("Failed to block diagonal threat! Black can still win at {:?}", test_move);
                }
                test_state.undo_move(test_move);
            }
        }
    }
}

#[test]
fn test_diagonal_blocking_bug_parallel() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    // Recreate a 3-in-a-row diagonal that needs to be blocked
    // Black has 3 stones in diagonal with both ends open
    state.board.place_stone(8, 7, Player::Min); // Black stone (start of diagonal)
    state.board.place_stone(9, 8, Player::Min); // Black stone
    state.board.place_stone(10, 9, Player::Min); // Black stone (3 in a row)
    // Positions (7,6) and (11,10) are open and need to be blocked
    
    // Add some white pieces that were on the board
    state.board.place_stone(7, 7, Player::Max); // White stone (the one that was placed)
    
    // It's white's turn and they MUST block the diagonal
    state.current_player = Player::Max;
    
    let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("Parallel search result: {:?}", best_move);
    
    // The AI should find one of the blocking moves
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    assert!(blocking_moves.contains(&(row, col)), 
           "Parallel: Expected blocking move (7,6) or (11,10) but got ({},{})", row, col);
    
    // Verify it actually blocks the threat
    state.make_move((row, col));
    
    // Black should not be able to win immediately after this block
    let mut test_state = state.clone();
    test_state.current_player = Player::Min;
    
    // Try the other diagonal position
    for &test_move in &[(7, 6), (11, 10)] {
        if test_move != (row, col) {
            if test_state.board.get_player(test_move.0, test_move.1).is_none() {
                test_state.make_move(test_move);
                // This should NOT be a winning position for black if we blocked correctly
                if test_state.is_terminal() && test_state.winner == Some(Player::Min) {
                    panic!("Failed to block diagonal threat! Black can still win at {:?}", test_move);
                }
                test_state.undo_move(test_move);
            }
        }
    }
}

#[test]
fn test_sequential_vs_parallel_consistency_diagonal() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    // Same position
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    state.board.place_stone(7, 7, Player::Max);
    state.current_player = Player::Max;
    
    let mut tt = TranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state.clone(), 6, None, &mut tt);
    let parallel_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("Sequential result: {:?}", sequential_move);
    println!("Parallel result: {:?}", parallel_move);
    
    // Both should find valid blocking moves
    assert!(sequential_move.is_some());
    assert!(parallel_move.is_some());
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    let seq_move = sequential_move.unwrap();
    let par_move = parallel_move.unwrap();
    
    assert!(blocking_moves.contains(&seq_move), 
           "Sequential didn't find blocking move: {:?}", seq_move);
    assert!(blocking_moves.contains(&par_move), 
           "Parallel didn't find blocking move: {:?}", par_move);
    
    // They should find the same move (or at least equally good moves)
    if seq_move != par_move {
        println!("WARNING: Sequential and parallel found different moves!");
        println!("Sequential: {:?}, Parallel: {:?}", seq_move, par_move);
        
        // Both should be equally good blocking moves, so this is acceptable
        // as long as both are valid blocks
    }
}

#[test] 
fn test_three_in_a_row_diagonal_blocking_sequential() {
    let mut state = GameState::new(19, 5);
    
    // Create position where black has 3 in diagonal with both ends open
    // This is when AI should have blocked you!
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    // Both ends (7,6) and (11,10) are open - AI must block one!
    
    state.current_player = Player::Max; // White's turn
    
    let mut tt = TranspositionTable::new_default();
    let best_move = find_best_move(&mut state, 6, None, &mut tt);
    
    println!("Sequential search for 3-in-a-row threat: {:?}", best_move);
    
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    assert!(blocking_moves.contains(&(row, col)), 
           "Sequential: Should block 3-in-a-row at (7,6) or (11,10) but got ({},{})", row, col);
}

#[test] 
fn test_three_in_a_row_diagonal_blocking_parallel() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    // Create position where black has 3 in diagonal with both ends open
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    // Both ends (7,6) and (11,10) are open - AI must block one!
    
    state.current_player = Player::Max; // White's turn
    
    let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("Parallel search for 3-in-a-row threat: {:?}", best_move);
    
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    assert!(blocking_moves.contains(&(row, col)), 
           "Parallel: Should block 3-in-a-row at (7,6) or (11,10) but got ({},{})", row, col);
}

#[test]
fn test_realistic_game_scenario() {
    let mut state = GameState::new(19, 5);
    
    // Create a more realistic game scenario with multiple pieces on the board
    // This might better replicate what happened in your actual game
    
    // Add some earlier moves (simulate a real game)
    state.board.place_stone(9, 9, Player::Max);   // Center white stone
    state.board.place_stone(10, 10, Player::Min); // Black response
    state.board.place_stone(8, 8, Player::Max);   // White
    state.board.place_stone(11, 11, Player::Min); // Black
    state.board.place_stone(7, 9, Player::Max);   // White
    
    // Now add the critical diagonal threat (3 in a row with both ends open)
    state.board.place_stone(8, 7, Player::Min);   // Start of diagonal
    state.board.place_stone(9, 8, Player::Min);   // Continue diagonal
    state.board.place_stone(10, 9, Player::Min);  // Third in diagonal
    
    // Add the white stone that was actually played in your game
    state.board.place_stone(7, 7, Player::Max);
    
    // Now it's white's turn and they should block the diagonal threat
    state.current_player = Player::Max;
    
    // Test both sequential and parallel search
    let mut state_seq = state.clone();
    let mut tt = TranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state_seq, 6, None, &mut tt);
    
    let mut state_par = state.clone();
    let shared_tt = SharedTranspositionTable::new_default();
    let parallel_move = find_best_move_parallel(&mut state_par, 6, None, &shared_tt);
    
    println!("Sequential search result: {:?}", sequential_move);
    println!("Parallel search result: {:?}", parallel_move);
    
    // They should give the same result!
    assert_eq!(sequential_move, parallel_move, 
              "Sequential and parallel search should give the same result!");
    
    let best_move = parallel_move;
    
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    // Verify that the AI's move prevents black from winning immediately
    let mut validation_state = state.clone();
    validation_state.make_move((row, col));
    validation_state.current_player = Player::Min;
    
    // Check if black can still win at either diagonal position
    let can_black_win_at_7_6 = {
        let mut test_state = validation_state.clone();
        test_state.make_move((7, 6));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    let can_black_win_at_11_10 = {
        let mut test_state = validation_state.clone();
        test_state.make_move((11, 10));
        test_state.is_terminal() && test_state.winner == Some(Player::Min)
    };
    
    assert!(!can_black_win_at_7_6 && !can_black_win_at_11_10, 
           "AI's move ({},{}) failed to prevent immediate black win! Black can still win at (7,6): {} or (11,10): {}", 
           row, col, can_black_win_at_7_6, can_black_win_at_11_10);
}

#[test]
fn test_with_more_noise() {
    let mut state = GameState::new(19, 5);
    
    // Add lots of pieces to create "noise" that might confuse the AI
    state.board.place_stone(5, 5, Player::Max);
    state.board.place_stone(6, 6, Player::Min);
    state.board.place_stone(14, 14, Player::Max);
    state.board.place_stone(15, 15, Player::Min);
    state.board.place_stone(3, 12, Player::Max);
    state.board.place_stone(4, 13, Player::Min);
    state.board.place_stone(16, 3, Player::Max);
    state.board.place_stone(17, 4, Player::Min);
    
    // Add the critical diagonal threat
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    
    state.current_player = Player::Max;
    
    let sequential_move = {
        let mut tt = TranspositionTable::new_default();
        find_best_move(&mut state.clone(), 6, None, &mut tt)
    };
    let shared_tt = SharedTranspositionTable::new_default();
    let parallel_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("With noise - Sequential: {:?}, Parallel: {:?}", sequential_move, parallel_move);
    
    let blocking_moves = vec![(7, 6), (11, 10)];
    
    assert!(sequential_move.is_some());
    assert!(parallel_move.is_some());
    
    let seq_move = sequential_move.unwrap();
    let par_move = parallel_move.unwrap();
    
    assert!(blocking_moves.contains(&seq_move), 
           "Sequential with noise: should block at (7,6) or (11,10) but chose ({},{})", seq_move.0, seq_move.1);
    assert!(blocking_moves.contains(&par_move), 
           "Parallel with noise: should block at (7,6) or (11,10) but chose ({},{})", par_move.0, par_move.1);
    
    // This is the key test - they should choose the same move
    if seq_move != par_move {
        println!("🚨 INCONSISTENCY DETECTED: Sequential chose {:?}, Parallel chose {:?}", seq_move, par_move);
        panic!("Sequential and parallel searches gave different results with board noise!");
    }
}
