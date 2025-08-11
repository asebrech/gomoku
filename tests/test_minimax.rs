use gomoku::ai::minimax::minimax;
use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_minimax_terminal_position() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a winning position
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.winner = Some(Player::Max);

    let (score, nodes) = minimax(&mut state, 3, i32::MIN, i32::MAX, false, &mut tt);

    // Should return winning score
    assert_eq!(score, 1_000_003);
    assert!(nodes > 0);
}

#[test]
fn test_minimax_depth_zero() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let (score, nodes) = minimax(&mut state, 0, i32::MIN, i32::MAX, false, &mut tt);

    // Should return heuristic evaluation
    assert!(score != i32::MIN && score != i32::MAX);
    assert!(nodes > 0);
}

#[test]
fn test_minimax_maximizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up a position where Max has advantage
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.current_player = Player::Max;

    let (score, nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should return positive score (favorable for Max)
    assert!(score > 0);
    assert!(nodes > 0);
}

#[test]
fn test_minimax_minimizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up a position where Min has advantage
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.current_player = Player::Min;

    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should return negative score (favorable for Min)
    assert!(score < 0);
}

#[test]
fn test_minimax_alpha_beta_pruning() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a position with multiple moves
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Run minimax with tight alpha-beta window
    let (score1, _) = minimax(&mut state, 2, -100, 100, false, &mut tt);

    // Should complete without infinite values
    assert!(score1 > i32::MIN && score1 < i32::MAX);
}

#[test]
fn test_minimax_transposition_table_usage() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Make initial move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // First call should populate transposition table
    let (score1, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Second call should use transposition table (same result)
    let (score2, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    assert_eq!(score1, score2);
}

#[test]
fn test_minimax_different_depths() {
    let mut state = GameState::new(19, 5);
    let mut tt1 = TranspositionTable::new_default();
    let mut tt2 = TranspositionTable::new_default();

    // Create a non-terminal position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    let (score_depth1, _) = minimax(&mut state, 1, i32::MIN, i32::MAX, false, &mut tt1);
    let (score_depth3, _) = minimax(&mut state, 3, i32::MIN, i32::MAX, false, &mut tt2);

    // Different depths may give different results
    // (not necessarily, but should complete successfully)
    assert!(score_depth1 != i32::MIN && score_depth1 != i32::MAX);
    assert!(score_depth3 != i32::MIN && score_depth3 != i32::MAX);
}

#[test]
fn test_minimax_winning_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a position where Max can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.current_player = Player::Max;

    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should detect winning opportunity
    assert!(score > 900_000); // Close to winning score
}

#[test]
fn test_minimax_losing_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a position where Min can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Min);
    }
    state.current_player = Player::Min;

    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should detect winning opportunity for Min
    assert!(score < -900_000); // Close to losing score
}

#[test]
fn test_minimax_state_restoration() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Make a move using proper state management
    state.make_move((9, 9));
    
    // Record initial state after the move
    let initial_hash = state.hash();
    let initial_player = state.current_player;

    // Run minimax (should restore state)
    minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // State should be restored
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
}

#[test]
fn test_minimax_captures_evaluation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up position with capture opportunity
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should recognize capture opportunity
    assert!(score > 0); // Favorable for Max
}

#[test]
fn test_minimax_empty_moves() {
    let mut state = GameState::new(3, 3);
    let mut tt = TranspositionTable::new_default();

    // Fill the board (no moves available)
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should handle no moves gracefully
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_alternating_players() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Start with Max to move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // At depth 2, should consider Min's move then Max's response
    let (score, _) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should complete successfully
    assert!(score > i32::MIN && score < i32::MAX);
}

#[test]
fn test_minimax_pruning_efficiency() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Create a position with many possible moves
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    // Should complete in reasonable time even with pruning
    let (score, _) = minimax(&mut state, 3, i32::MIN, i32::MAX, true, &mut tt);

    assert!(score > i32::MIN && score < i32::MAX);
}

#[test]
fn test_minimax_capture_win_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up capture win scenario where Max can win by capturing
    state.max_captures = 8; // Need 10 to win, so 1 capture away
    
    // Create a capture opportunity: Max-Min-Min-empty
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should detect capture opportunity and give high score
    assert!(score > 500_000, "Should detect capture win opportunity, got score: {}", score);
}

#[test] 
fn test_minimax_immediate_win_detection() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();

    // Set up 4-in-a-row with one empty spot to complete win
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(7, 9, Player::Max);
    // Empty at (7, 10) for 5-in-a-row win
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 1, i32::MIN, i32::MAX, true, &mut tt);

    // Should detect immediate win
    assert!(score > 900_000, "Should detect immediate win, got score: {}", score);
}

#[test]
fn test_find_immediate_win_or_block() {
    use gomoku::ai::minimax::iterative_deepening_search;
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Create immediate win scenario for current player
    state.board.place_stone(7, 5, Player::Max);
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.current_player = Player::Max; // Max can win by playing (7,9)
    
    let result = iterative_deepening_search(&mut state, 3, None, &mut tt);
    
    // Should find immediate win (either end of the line is valid)
    assert!(result.best_move.is_some());
    let chosen_move = result.best_move.unwrap();
    assert!(chosen_move == (7, 4) || chosen_move == (7, 9), 
            "Should choose either end of winning line, got {:?}", chosen_move);
    assert!(result.score > 900_000);
    assert_eq!(result.depth_reached, 1); // Should terminate early
}

#[test]
fn test_iterative_deepening_progressive_improvement() {
    use gomoku::ai::minimax::iterative_deepening_search;
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Create complex position
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(8, 8, Player::Min);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(8, 7, Player::Min);
    state.current_player = Player::Max;
    
    let result = iterative_deepening_search(&mut state, 4, None, &mut tt);
    
    // Should reach reasonable depth
    assert!(result.depth_reached >= 3, "Should reach depth 3+, got: {}", result.depth_reached);
    assert!(result.best_move.is_some());
    assert!(result.nodes_searched > 0);
}
