use gomoku::ai::minimax::{minimax_shared as minimax, parallel_iterative_deepening_search};
use gomoku::legacy::ai::minimax::{minimax as minimax_single};
use gomoku::legacy::ai::transposition::TranspositionTable;
use gomoku::ai::transposition::SharedTranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_minimax_terminal_position() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Create a winning position
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.winner = Some(Player::Max);

    let (score, _nodes) = minimax(&mut state, 3, i32::MIN, i32::MAX, false, &mut tt);

    // Should return winning score
    assert_eq!(score, 1_000_003);
}

#[test]
fn test_minimax_depth_zero() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let (score, _nodes) = minimax(&mut state, 0, i32::MIN, i32::MAX, false, &mut tt);

    // Should return heuristic evaluation
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_maximizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Set up a position where Max has advantage
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should return positive score (favorable for Max)
    assert!(score > 0);
}

#[test]
fn test_minimax_minimizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Set up a position where Min has advantage
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.current_player = Player::Min;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should return negative score (favorable for Min)
    assert!(score < 0);
}

#[test]
fn test_minimax_alpha_beta_pruning() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Create a position with multiple moves
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Run minimax with tight alpha-beta window
    let (score1, _nodes) = minimax(&mut state, 2, -100, 100, false, &mut tt);

    // Should complete without infinite values
    assert!(score1 > i32::MIN && score1 < i32::MAX);
}

#[test]
fn test_minimax_transposition_table_usage() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Make initial move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // First call should populate transposition table
    let (score1, _nodes1) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Second call should use transposition table (same result)
    let (score2, _nodes2) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    assert_eq!(score1, score2);
}

#[test]
fn test_minimax_different_depths() {
    let mut state = GameState::new(19, 5);
    let mut tt1 = SharedTranspositionTable::new_default();
    let mut tt2 = SharedTranspositionTable::new_default();

    // Create a non-terminal position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    let (score_depth1, _nodes1) = minimax(&mut state, 1, i32::MIN, i32::MAX, false, &mut tt1);
    let (score_depth3, _nodes2) = minimax(&mut state, 3, i32::MIN, i32::MAX, false, &mut tt2);

    // Different depths may give different results
    // (not necessarily, but should complete successfully)
    assert!(score_depth1 != i32::MIN && score_depth1 != i32::MAX);
    assert!(score_depth3 != i32::MIN && score_depth3 != i32::MAX);
}

#[test]
fn test_minimax_winning_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Create a position where Max can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should detect winning opportunity
    assert!(score > 900_000); // Close to winning score
}

#[test]
fn test_minimax_losing_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Create a position where Min can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Min);
    }
    state.current_player = Player::Min;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should detect winning opportunity for Min
    assert!(score < -900_000); // Close to losing score
}

#[test]
fn test_minimax_state_restoration() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Make a move using proper state management
    state.make_move((9, 9));
    
    // Record initial state after the move
    let initial_hash = state.hash();
    let initial_player = state.current_player;

    // Run minimax (should restore state)
    let (_score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // State should be restored
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
}

// TODO: Fix minimax evaluation function so that it properly recognizes and scores positions where a capture opportunity exists for the current player. Test currently fails.
#[test]
fn test_minimax_captures_evaluation() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Set up position with capture opportunity
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should recognize capture opportunity
    assert!(score > 0); // Favorable for Max
}

#[test]
fn test_minimax_empty_moves() {
    let mut state = GameState::new(3, 3);
    let mut tt = SharedTranspositionTable::new_default();

    // Fill the board (no moves available)
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should handle no moves gracefully
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_alternating_players() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Start with Max to move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // At depth 2, should consider Min's move then Max's response
    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &mut tt);

    // Should complete successfully
    assert!(score > i32::MIN && score < i32::MAX);
}

#[test]
fn test_minimax_pruning_efficiency() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Create a position with many possible moves
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    // Should complete in reasonable time even with pruning
    let (score, _nodes) = minimax(&mut state, 3, i32::MIN, i32::MAX, true, &mut tt);

    assert!(score > i32::MIN && score < i32::MAX);
}

// TODO: Update minimax to correctly detect capture-win scenarios where a player can win by making a capture. Test currently fails.
#[test]
fn test_minimax_capture_win_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = SharedTranspositionTable::new_default();

    // Set up near-capture-win scenario
    state.max_captures = 4; // One pair away from winning
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &mut tt);

    // Should detect capture win opportunity
    assert!(score > 900_000);
}

// ==================== PARALLEL MINIMAX TESTS ====================

#[test]
fn test_minimax_shared_terminal_position() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Create a winning position
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.winner = Some(Player::Max);

    let (score, _nodes) = minimax(&mut state, 3, i32::MIN, i32::MAX, false, &shared_tt);

    // Should return winning score
    assert_eq!(score, 1_000_003);
}

#[test]
fn test_minimax_shared_depth_zero() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let (score, _nodes) = minimax(&mut state, 0, i32::MIN, i32::MAX, false, &shared_tt);

    // Should return heuristic evaluation
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_shared_maximizing_player() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Set up a position where Max has advantage
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &shared_tt);

    // Should return positive score (favorable for Max)
    assert!(score > 0);
}

#[test]
fn test_minimax_shared_minimizing_player() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Set up a position where Min has advantage
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.current_player = Player::Min;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &shared_tt);

    // Should return negative score (favorable for Min)
    assert!(score < 0);
}

#[test]
fn test_parallel_iterative_deepening_basic() {
    let mut state = GameState::new(15, 5);

    // Set up a simple position
    state.make_move((7, 7)); // Center move
    state.make_move((7, 8)); // Adjacent move

    let result = parallel_iterative_deepening_search(&mut state, 3, None);

    // Should find a valid move
    assert!(result.best_move.is_some());
    assert!(result.score != i32::MIN && result.score != i32::MAX);
    assert!(result.depth_reached > 0);
    assert!(result.nodes_searched > 0);
}

#[test]
fn test_parallel_iterative_deepening_winning_position() {
    let mut state = GameState::new(15, 5);

    // Create a position where there's a clear winning move
    state.make_move((7, 7));
    state.make_move((6, 7));
    state.make_move((7, 8));
    state.make_move((6, 8));
    state.make_move((7, 9));
    state.make_move((6, 9));
    state.make_move((7, 10));
    // Now current player (Min) has a winning move

    let result = parallel_iterative_deepening_search(&mut state, 4, None);

    // Should find the winning move
    assert!(result.best_move.is_some());
    assert!(result.score.abs() > 900_000); // Close to mate value
}

#[test]
fn test_parallel_iterative_deepening_time_limit() {
    let mut state = GameState::new(15, 5);

    // Set up a complex position
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    state.make_move((6, 8));

    let time_limit = std::time::Duration::from_millis(100);
    let start_time = std::time::Instant::now();
    let result = parallel_iterative_deepening_search(&mut state, 10, Some(time_limit));
    let elapsed = start_time.elapsed();

    // Should respect time limit
    assert!(elapsed <= std::time::Duration::from_millis(200)); // Small buffer
    assert!(result.best_move.is_some());
    assert!(result.depth_reached > 0);
}

#[test]
fn test_minimax_shared_vs_sequential_consistency() {
    let mut state = GameState::new(15, 5);

    // Set up a deterministic position
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Min);
    state.current_player = Player::Max;

    // Test sequential minimax
    let mut tt_seq = TranspositionTable::new_default();
    let (score_seq, _nodes_seq) = minimax_single(&mut state.clone(), 2, i32::MIN, i32::MAX, true, &mut tt_seq);

    // Test shared minimax
    let shared_tt = SharedTranspositionTable::new_default();
    let (score_shared, _nodes_shared) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &shared_tt);

    // Should give same results
    assert_eq!(score_seq, score_shared);
}

#[test]
fn test_minimax_shared_state_preservation() {
    let mut state = GameState::new(15, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Make a move using proper state management
    state.make_move((7, 7));
    
    // Record initial state after the move
    let initial_hash = state.hash();
    let initial_player = state.current_player;

    // Run minimax_shared (should restore state)
    let (_score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, false, &shared_tt);

    // State should be preserved
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
}

#[test]
fn test_parallel_iterative_deepening_empty_moves() {
    let mut state = GameState::new(3, 3);

    // Fill the board (no moves available)
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let result = parallel_iterative_deepening_search(&mut state, 2, None);

    // Should handle no moves gracefully
    assert_eq!(result.best_move, None);
    assert_eq!(result.score, 0); // Draw score
    assert_eq!(result.depth_reached, 0);
    assert_eq!(result.nodes_searched, 0);
}

#[test]
fn test_minimax_shared_captures_evaluation() {
    let mut state = GameState::new(19, 5);
    let shared_tt = SharedTranspositionTable::new_default();

    // Set up position with capture opportunity
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.current_player = Player::Max;

    let (score, _nodes) = minimax(&mut state, 2, i32::MIN, i32::MAX, true, &shared_tt);

    // Should recognize capture opportunity
    assert!(score > 0); // Favorable for Max
}
