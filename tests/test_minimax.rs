use gomoku::interface::utils::{find_best_move, init_ai, clear_tt};
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_minimax_terminal_position() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a winning position
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.winner = Some(Player::Max);

    // Test that AI can find a move even in terminal position
    let best_move = find_best_move(&mut state, 3);
    
    // Should handle terminal position gracefully
    assert!(best_move.is_some() || state.get_possible_moves().is_empty());
}

#[test]
fn test_minimax_depth_zero() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let best_move = find_best_move(&mut state, 0);

    // Should handle depth zero gracefully
    assert!(best_move.is_some() || state.get_possible_moves().is_empty());
}

#[test]
fn test_minimax_maximizing_player() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Set up a position where Max has advantage
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.current_player = Player::Max;

    let has_move = find_best_move(&mut state, 2).is_some();

    // Should find a move
    assert!(has_move);
}

#[test]
fn test_minimax_minimizing_player() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Set up a position where Min has advantage
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 7, Player::Min);
    state.current_player = Player::Min;

    let has_move = find_best_move(&mut state, 2).is_some();

    // Should find a move
    assert!(has_move);
}

#[test]
fn test_minimax_alpha_beta_pruning() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a position with multiple moves
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // Run enhanced minimax
    let has_move = find_best_move(&mut state, 2).is_some();

    // Should find a move
    assert!(has_move);
}

#[test]
fn test_minimax_different_depths() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a non-terminal position
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    let has_move_depth1 = find_best_move(&mut state, 1).is_some();
    clear_tt(); // Clear TT between searches
    let has_move_depth3 = find_best_move(&mut state, 3).is_some();

    // Both depths should find moves
    assert!(has_move_depth1);
    assert!(has_move_depth3);
}

#[test]
fn test_minimax_winning_position_detection() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a position where Max can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    state.current_player = Player::Max;

    let best_move = find_best_move(&mut state, 2);

    // Should detect winning opportunity
    assert!(best_move.is_some());
    
    // The move should complete the line
    if let Some((row, col)) = best_move {
        assert!(row == 9 && (col == 4 || col == 9));
    }
}

#[test]
fn test_minimax_losing_position_detection() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a position where Min can win in one move
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Min);
    }
    state.current_player = Player::Min;

    let best_move = find_best_move(&mut state, 2);

    // Should find a winning move for Min
    assert!(best_move.is_some());
    
    // The move should complete the line
    if let Some((row, col)) = best_move {
        assert!(row == 9 && (col == 4 || col == 9));
    }
}

#[test]
fn test_minimax_state_restoration() {
    let mut state = GameState::new(19, 5);
    
    // Initialize AI
    init_ai(19);
    clear_tt();

    // Record initial state
    state.board.place_stone(9, 9, Player::Max);
    let initial_hash = state.hash();
    let initial_player = state.current_player;

    // Run minimax (should restore state)
    let _best_move = find_best_move(&mut state, 2);

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

    // Initialize AI
    init_ai(3);
    clear_tt();

    let result = find_best_move(&mut state, 2);

    // Should handle no moves gracefully - return None
    assert!(result.is_none());
}

#[test]
fn test_minimax_alternating_players() {
    let mut state = GameState::new(19, 5);

    // Initialize AI
    init_ai(19);
    clear_tt();

    // Start with Max to move
    state.board.place_stone(9, 9, Player::Max);
    state.current_player = Player::Min;

    // At depth 2, should consider Min's move then Max's response
    let result = find_best_move(&mut state, 2);

    // Should complete successfully and find a move
    assert!(result.is_some());
}

#[test]
fn test_minimax_pruning_efficiency() {
    let mut state = GameState::new(19, 5);

    // Initialize AI
    init_ai(19);
    clear_tt();

    // Create a position with many possible moves
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;

    // Should complete in reasonable time even with pruning
    let result = find_best_move(&mut state, 3);

    // Should find a move successfully
    assert!(result.is_some());
}
