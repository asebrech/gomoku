use gomoku::ai::minimax::minimax;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_minimax_terminal_position() {
    let mut state = GameState::new(19, 5);

    // Create a winning position
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.winner = Some(Player::Max);

    let score = minimax(&mut state, 3, i32::MIN, i32::MAX, false);

    // Should return winning score
    assert_eq!(score, 1_000_003);
}

#[test]
fn test_minimax_depth_zero() {
    let mut state = GameState::new(19, 5);

    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let score = minimax(&mut state, 0, i32::MIN, i32::MAX, false);

    // Should return heuristic evaluation
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_maximizing_player() {
    let mut state = GameState::new(19, 5);

    // Set up a position where Max has advantage
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.current_player = Player::Max;

    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, true);

    // Should return positive score (favorable for Max)
    assert!(score > 0);
}

#[test]
fn test_minimax_minimizing_player() {
    let mut state = GameState::new(19, 5);

    // Set up a position where Min has advantage
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.current_player = Player::Min;

    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, false);

    // Should return negative score (favorable for Min)
    assert!(score < 0);
}

#[test]
fn test_minimax_alpha_beta_pruning() {
    let mut state = GameState::new(19, 5);

    // Create a position with multiple moves
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Run minimax with tight alpha-beta window
    let score1 = minimax(&mut state, 2, -100, 100, false);

    // Should complete without infinite values
    assert!(score1 > i32::MIN && score1 < i32::MAX);
}

#[test]
fn test_minimax_different_depths() {
    let mut state = GameState::new(19, 5);

    // Create a non-terminal position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    let score_depth1 = minimax(&mut state, 1, i32::MIN, i32::MAX, false);
    let score_depth3 = minimax(&mut state, 3, i32::MIN, i32::MAX, false);

    // Different depths may give different results
    // (not necessarily, but should complete successfully)
    assert!(score_depth1 != i32::MIN && score_depth1 != i32::MAX);
    assert!(score_depth3 != i32::MIN && score_depth3 != i32::MAX);
}

#[test]
fn test_minimax_winning_position_detection() {
    let mut state = GameState::new(19, 5);

    // Create a position where Max can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.current_player = Player::Max;

    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, true);

    // Should detect winning opportunity
    assert!(score > 900_000); // Close to winning score
}

#[test]
fn test_minimax_losing_position_detection() {
    let mut state = GameState::new(19, 5);

    // Create a position where Min can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Min);
    }
    state.current_player = Player::Min;

    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, false);

    // Should detect winning opportunity for Min
    assert!(score < -900_000); // Close to losing score
}

#[test]
fn test_minimax_state_restoration() {
    let mut state = GameState::new(19, 5);

    // Record initial state
    state.board.place_stone(9, 9, Player::Max);
    let initial_hash = state.hash();
    let initial_player = state.current_player;

    // Run minimax (should restore state)
    minimax(&mut state, 2, i32::MIN, i32::MAX, false);

    // State should be restored
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
}

#[test]
fn test_minimax_empty_moves() {
    let mut state = GameState::new(3, 3);

    // Fill the board (no moves available)
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, Player::Max);
        }
    }

    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, false);

    // Should handle no moves gracefully
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_minimax_alternating_players() {
    let mut state = GameState::new(19, 5);

    // Start with Max to move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // At depth 2, should consider Min's move then Max's response
    let score = minimax(&mut state, 2, i32::MIN, i32::MAX, false);

    // Should complete successfully
    assert!(score > i32::MIN && score < i32::MAX);
}

#[test]
fn test_minimax_pruning_efficiency() {
    let mut state = GameState::new(19, 5);

    // Create a position with many possible moves
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    // Should complete in reasonable time even with pruning
    let score = minimax(&mut state, 3, i32::MIN, i32::MAX, true);

    assert!(score > i32::MIN && score < i32::MAX);
}