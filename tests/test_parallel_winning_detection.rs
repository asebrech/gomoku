use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move;
use gomoku::ai::transposition::SharedTranspositionTable;

#[test]
fn test_parallel_winning_move_detection() {
    let mut state = GameState::new(15, 5);
    
    // Set up a position where there's an immediate winning move
    // Max has 4 in a row and can complete 5 in a row
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(7, 9, Player::Max);
    
    // Add some Min pieces to make it not completely trivial
    state.board.place_stone(6, 7, Player::Min);
    state.board.place_stone(8, 8, Player::Min);
    
    state.current_player = Player::Max;
    
    let best_move = find_best_move(&mut state, 6, None, &shared_tt);
    
    println!("Position: Max has 4 in a row at (7,6-9), needs to complete at (7,5) or (7,10)");
    println!("Best move found: {:?}", best_move);
    
    // Should find the winning move - either (7, 5) or (7, 10)
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let winning_moves = vec![(7, 5), (7, 10)];
    assert!(winning_moves.contains(&(row, col)), 
           "Expected winning move (7,5) or (7,10) but got ({},{})", row, col);
    
    // Verify it's actually a winning move by making it
    state.make_move((row, col));
    assert!(state.is_terminal(), "Move should result in a win");
    assert_eq!(state.winner, Some(Player::Max), "Max should win");
}

#[test]
fn test_parallel_blocking_move_detection() {
    let mut state = GameState::new(15, 5);
    
    // Set up a position where Min has 4 in a row and Max must block
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(7, 6, Player::Min);
    state.board.place_stone(7, 7, Player::Min);
    state.board.place_stone(7, 8, Player::Min);
    state.board.place_stone(7, 9, Player::Min);
    
    // Add some Max pieces
    state.board.place_stone(6, 7, Player::Max);
    state.board.place_stone(8, 8, Player::Max);
    
    state.current_player = Player::Max;
    
    let best_move = find_best_move(&mut state, 6, None, &shared_tt);
    
    println!("Position: Min has 4 in a row at (7,6-9), Max must block at (7,5) or (7,10)");
    println!("Best move found: {:?}", best_move);
    
    // Should find the blocking move - either (7, 5) or (7, 10)
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    let blocking_moves = vec![(7, 5), (7, 10)];
    assert!(blocking_moves.contains(&(row, col)), 
           "Expected blocking move (7,5) or (7,10) but got ({},{})", row, col);
}

#[test]
fn test_parallel_race_condition_stress() {
    // Run multiple parallel searches with winning positions to try to trigger race conditions
    for i in 0..10 {
        // Use a fresh transposition table for each iteration to avoid interference
        let shared_tt = SharedTranspositionTable::new_default();
        let mut state = GameState::new(15, 5);
        
        // Create different winning positions each iteration
        let base_row = 6 + (i % 3);
        state.board.place_stone(base_row, 6, Player::Max);
        state.board.place_stone(base_row, 7, Player::Max);
        state.board.place_stone(base_row, 8, Player::Max);
        state.board.place_stone(base_row, 9, Player::Max);
        
        // Add some noise
        state.board.place_stone(base_row + 1, 7, Player::Min);
        state.board.place_stone(base_row - 1, 8, Player::Min);
        
        state.current_player = Player::Max;
        
        let best_move = find_best_move(&mut state, 6, None, &shared_tt);
        
        // Should always find a winning move
        assert!(best_move.is_some(), "Iteration {}: Should find a winning move", i);
        let (row, col) = best_move.unwrap();
        
        let winning_moves = vec![(base_row, 5), (base_row, 10)];
        assert!(winning_moves.contains(&(row, col)), 
               "Iteration {}: Expected winning move ({},5) or ({},10) but got ({},{})", 
               i, base_row, base_row, row, col);
        
        // Verify it's actually a winning move
        state.make_move((row, col));
        assert!(state.is_terminal(), "Iteration {}: Move should result in a win", i);
        assert_eq!(state.winner, Some(Player::Max), "Iteration {}: Max should win", i);
    }
}

#[test]
fn test_parallel_multiple_winning_moves() {
    let mut state = GameState::new(15, 5);
    
    // Create a position with multiple winning moves to see if parallel search is consistent
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(7, 9, Player::Max);
    state.board.place_stone(7, 10, Player::Max);
    
    // Also create another potential winning line
    state.board.place_stone(6, 7, Player::Max);
    state.board.place_stone(8, 7, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(10, 7, Player::Max);
    
    state.current_player = Player::Max;
    
    // Run multiple times to check for consistency
    let mut moves = Vec::new();
    for _ in 0..5 {
        let best_move = find_best_move(&mut state.clone(), 6, None, &shared_tt);
        assert!(best_move.is_some(), "Should always find a winning move");
        moves.push(best_move.unwrap());
    }
    
    // All moves should be winning moves
    let valid_winning_moves = vec![(7, 6), (5, 7), (11, 7)];
    for (i, &mv) in moves.iter().enumerate() {
        assert!(valid_winning_moves.contains(&mv), 
               "Run {}: Move {:?} should be a winning move", i, mv);
    }
    
    println!("Parallel search returned moves: {:?}", moves);
}
