use gomoku::ai::transposition::{TranspositionTable, SharedTranspositionTable};
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::interface::utils::{find_best_move, find_best_move_parallel};

#[test]
fn test_find_best_move_first_move() {
    let mut state = GameState::new(19, 5);

    // Test both sequential and parallel implementations
    let mut tt = TranspositionTable::new_default();
    let shared_tt = SharedTranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state, 2, None, &mut tt);
    
    let mut state_copy = state.clone();
    let parallel_move = find_best_move_parallel(&mut state_copy, 2, None, &shared_tt);

    // Both should return center move for first move
    assert_eq!(sequential_move, Some((9, 9)));
    assert_eq!(parallel_move, Some((9, 9)));
    assert_eq!(sequential_move, parallel_move);
}

#[test]
fn test_find_best_move_response() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Make first move
    state.make_move((9, 9));

    // Test parallel implementation with deeper search
    let best_move = find_best_move_parallel(&mut state, 5, None, &shared_tt);

    // Should return some adjacent move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    assert!(state.board.is_adjacent_to_stone(row, col));
}

#[test]
fn test_find_best_move_winning_opportunity() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Set up winning opportunity for current player (Max)
    state.board.place_stone(9, 5, Player::Max);
    state.board.place_stone(9, 6, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.current_player = Player::Max;

    // Use parallel search for better analysis
    let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);

    // Should find the winning move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();

    // Should be adjacent to complete the line
    assert!(row == 9 && (col == 4 || col == 9));
}

#[test]
fn test_find_best_move_block_opponent() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Set up threat from opponent (Min has 4 in a row)
    state.board.place_stone(9, 5, Player::Min);
    state.board.place_stone(9, 6, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move_parallel(&mut state, 5, None, &shared_tt);

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
fn test_find_best_move_block_opponent_sequential() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up threat from opponent (Min has 4 in a row)
    state.board.place_stone(9, 5, Player::Min);
    state.board.place_stone(9, 6, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, None,&mut tt);

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

// TODO: Add a test to verify that find_best_move detects a capture opportunity and chooses the correct capturing move for the current player.
#[test]
fn test_find_best_move_capture_opportunity() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();
    // Set up capture opportunity
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2, None,&mut tt);

    // Should find the capturing move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();

    // Should be the capturing position
    assert_eq!((row, col), (9, 12));
}

#[test]
fn test_find_best_move_capture_opportunity_parallel() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    // Set up capture opportunity
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move_parallel(&mut state, 2, None, &shared_tt);

    // Should find the capturing move
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();

    // Should be the capturing position
    assert_eq!((row, col), (9, 12));
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

    let best_move = find_best_move(&mut state, 2, None,&mut tt);

    // Should return None when no moves available
    assert_eq!(best_move, None);
}

#[test]
fn test_find_best_move_no_moves_parallel() {
    let mut state = GameState::new(3, 3);

    // Fill the board
    let shared_tt = SharedTranspositionTable::new_default();
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let best_move = find_best_move_parallel(&mut state, 2, None, &shared_tt);

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

    let move_depth1 = find_best_move(&mut state, 1, None,&mut tt);
    let move_depth3 = find_best_move(&mut state, 3, None,&mut tt);

    // Both should return valid moves
    assert!(move_depth1.is_some());
    assert!(move_depth3.is_some());

    // Moves might be different due to deeper search
    // (but this isn't guaranteed, so we just check validity)
}

#[test]
fn test_find_best_move_different_depths_parallel() {
    let mut state = GameState::new(19, 5);

    // Set up a position
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    let move_depth1 = find_best_move_parallel(&mut state.clone(), 1, None, &shared_tt);
    let move_depth5 = find_best_move_parallel(&mut state.clone(), 5, None, &shared_tt);

    // Both should return valid moves
    assert!(move_depth1.is_some());
    assert!(move_depth5.is_some());

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

    let move_max = find_best_move(&mut state, 2, None,&mut tt);
    assert!(move_max.is_some());

    // Test with Min player
    state.current_player = Player::Min;
    state.board.place_stone(9, 10, Player::Max); // Add opponent stone

    let move_min = find_best_move(&mut state, 2, None,&mut tt);
    assert!(move_min.is_some());
}

#[test]
fn test_find_best_move_player_alternation_parallel() {
    let mut state = GameState::new(19, 5);

    // Test with Max player
    let shared_tt = SharedTranspositionTable::new_default();
    state.current_player = Player::Max;
    state.board.place_stone(9, 9, Player::Min); // Add opponent stone

    let move_max = find_best_move_parallel(&mut state.clone(), 2, None, &shared_tt);
    assert!(move_max.is_some());

    // Test with Min player
    state.current_player = Player::Min;
    state.board.place_stone(9, 10, Player::Max); // Add opponent stone

    let move_min = find_best_move_parallel(&mut state, 2, None, &shared_tt);
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

    let best_move = find_best_move(&mut state, 2, None,&mut tt);

    // Should find some reasonable move
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&(row, col)));
}

#[test]
fn test_find_best_move_complex_position_parallel() {
    let mut state = GameState::new(19, 5);

    // Create a complex position
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(8, 9, Player::Max);
    state.board.place_stone(8, 10, Player::Min);
    state.board.place_stone(10, 9, Player::Max);
    state.board.place_stone(10, 10, Player::Min);
    state.current_player = Player::Max;

    let best_move = find_best_move_parallel(&mut state, 5, None, &shared_tt);

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
    let _best_move = find_best_move(&mut state, 2, None,&mut tt);

    // State should be preserved
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, Player::Min);
    assert_eq!(state.board.get_player(9, 9), Some(Player::Max));
}

#[test]
fn test_find_best_move_state_preservation_parallel() {
    let mut state = GameState::new(19, 5);

    // Set up initial state
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;
    let initial_hash = state.hash();

    // Find best move
    let _best_move = find_best_move_parallel(&mut state, 2, None, &shared_tt);

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
    let move1 = find_best_move(&mut state, 2, None,&mut tt);
    let move2 = find_best_move(&mut state, 2, None,&mut tt);

    assert_eq!(move1, move2);
}

#[test]
fn test_find_best_move_consistent_results_parallel() {
    let mut state = GameState::new(19, 5);

    // Set up a deterministic position
    let shared_tt = SharedTranspositionTable::new_default();
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Multiple calls should give same result
    let move1 = find_best_move_parallel(&mut state.clone(), 2, None, &shared_tt);
    let move2 = find_best_move_parallel(&mut state, 2, None, &shared_tt);

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

    let best_move = find_best_move(&mut state, 2, None,&mut tt);

    // Should find one of the few available moves
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    assert!(state.board.is_empty_position(row, col));
}

#[test]
fn test_find_best_move_edge_cases_parallel() {
    let mut state = GameState::new(19, 5);

    // Fill most of the board leaving only a few moves
    let shared_tt = SharedTranspositionTable::new_default();
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

    let best_move = find_best_move_parallel(&mut state, 2, None, &shared_tt);

    // Should find one of the few available moves
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    assert!(state.board.is_empty_position(row, col));
}

// TODO: Add a test to confirm that find_best_move selects the correct move to win the game via a capture when one pair away from a capture-win.
#[test]
fn test_find_best_move_capture_win() {
    let mut state = GameState::new(19, 5);

    // Set up near-capture-win scenario
    state.max_captures = 4; // One away from winning
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;
    let mut tt = TranspositionTable::new_default();
    let best_move = find_best_move(&mut state, 2, None,&mut tt);

    // Should find the winning capture
    assert_eq!(best_move, Some((9, 12)));
}

#[test]
fn test_find_best_move_capture_win_parallel() {
    let mut state = GameState::new(19, 5);

    // Set up near-capture-win scenario
    let shared_tt = SharedTranspositionTable::new_default();
    state.max_captures = 4; // One away from winning
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;
    
    let best_move = find_best_move_parallel(&mut state, 2, None, &shared_tt);

    // Should find the winning capture
    assert_eq!(best_move, Some((9, 12)));
}

#[test]
fn test_find_best_move_different_board_sizes() {
    let mut state13 = GameState::new(13, 5);
    let mut state15 = GameState::new(15, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    let mut tt = TranspositionTable::new_default();

    let move13 = find_best_move(&mut state13, 2, None,&mut tt);
    let move15 = find_best_move(&mut state15, 2, None,&mut tt);

    // Should find center moves for different board sizes
    assert_eq!(move13, Some((6, 6)));
    assert_eq!(move15, Some((7, 7)));
}

#[test]
fn test_find_best_move_different_board_sizes_parallel() {
    let mut state13 = GameState::new(13, 5);
    let mut state15 = GameState::new(15, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    let move13 = find_best_move_parallel(&mut state13, 2, None, &shared_tt);
    let move15 = find_best_move_parallel(&mut state15, 2, None, &shared_tt);

    // Should find center moves for different board sizes
    assert_eq!(move13, Some((6, 6)));
    assert_eq!(move15, Some((7, 7)));
}

#[test]
fn test_parallel_vs_sequential_consistency() {
    let mut state = GameState::new(15, 5);
    
    // Set up a complex middle-game position
    let shared_tt = SharedTranspositionTable::new_default();
    let moves = vec![
        (7, 7), (8, 8), (6, 6), (9, 9), 
        (7, 8), (8, 7), (6, 8), (8, 6),
        (5, 5), (10, 10)
    ];
    
    for mv in moves {
        state.make_move(mv);
    }
    
    // Test both implementations
    let mut state_seq = state.clone();
    let mut state_par = state.clone();
    
    let mut tt = TranspositionTable::new_default();
    let sequential_result = find_best_move(&mut state_seq, 4, None, &mut tt);
    let parallel_result = find_best_move_parallel(&mut state_par, 5, None, &shared_tt);
    
    // Both should find valid moves
    assert!(sequential_result.is_some());
    assert!(parallel_result.is_some());
    
    println!("Sequential (depth 4): {:?}", sequential_result);
    println!("Parallel (depth 5): {:?}", parallel_result);
    
    // Moves should be reasonable (in the active area)
    if let Some((row, col)) = parallel_result {
        assert!(row >= 4 && row <= 11);
        assert!(col >= 4 && col <= 11);
    }
}

#[test]
fn test_parallel_performance_on_complex_position() {
    let mut state = GameState::new(15, 5);
    
    // Create a complex tactical position
    let shared_tt = SharedTranspositionTable::new_default();
    let setup_moves = vec![
        (7, 7), (7, 8), (8, 7), (8, 8),
        (6, 6), (6, 9), (9, 6), (9, 9),
        (5, 7), (10, 8), (7, 5), (8, 10),
    ];
    
    for mv in setup_moves {
        state.make_move(mv);
    }
    
    let start_time = std::time::Instant::now();
    let result = find_best_move_parallel(&mut state, 6, Some(std::time::Duration::from_millis(2000)), &shared_tt);
    let elapsed = start_time.elapsed();
    
    println!("Parallel search on complex position:");
    println!("Time: {:?}", elapsed);
    println!("Result: {:?}", result);
    
    // Should complete within time limit and find a move
    assert!(elapsed <= std::time::Duration::from_millis(2500)); // Small buffer for overhead
    assert!(result.is_some());
}
