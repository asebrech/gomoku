use gomoku::core::board::{Player, initialize_zobrist};
use gomoku::core::state::GameState;
use gomoku::ai::heuristic::Heuristic;

#[test]
fn test_heuristic_empty_board() {
    initialize_zobrist();
    let state = GameState::new(19, 5);
    let score = Heuristic::evaluate(&state);

    // Empty board should have neutral score
    assert_eq!(score, 0);
}

#[test]
fn test_heuristic_winner_max() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    state.winner = Some(Player::Max);
    
    let score = Heuristic::evaluate(&state);
    assert_eq!(score, 1_000_000);
}

#[test]
fn test_heuristic_winner_min() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    state.winner = Some(Player::Min);
    
    let score = Heuristic::evaluate(&state);
    assert_eq!(score, -1_000_000);
}

#[test]
fn test_heuristic_line_evaluation() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a line of 3 for Max
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    
    let score = Heuristic::evaluate(&state);
    assert!(score > 0, "Max should have positive score for line of 3");
}

#[test]
fn test_heuristic_blocked_line() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a blocked line: Min X X X Min
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    state.board.place_stone(9, 12, Player::Min);
    
    let score = Heuristic::evaluate(&state);
    
    // The blocked line should have some value but not as much as an open line
    // Just check that the function doesn't crash and returns a reasonable value
    assert!(score.abs() < 100_000, "Blocked line should have limited value, got: {}", score);
}

#[test]
fn test_heuristic_multiple_directions() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create lines in multiple directions
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);  // Horizontal
    state.board.place_stone(10, 9, Player::Max);  // Vertical
    state.board.place_stone(10, 10, Player::Max); // Diagonal
    
    let score = Heuristic::evaluate(&state);
    assert!(score > 0, "Multiple directions should give positive score");
}

#[test]
fn test_heuristic_opponent_advantage() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create advantage for Min
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    
    let score = Heuristic::evaluate(&state);
    assert!(score < 0, "Min should have negative score advantage");
}

#[test]
fn test_heuristic_symmetry() {
    initialize_zobrist();
    let mut state1 = GameState::new(19, 5);
    let mut state2 = GameState::new(19, 5);
    
    // Create symmetric positions
    state1.board.place_stone(9, 9, Player::Max);
    state1.board.place_stone(9, 10, Player::Max);
    
    state2.board.place_stone(9, 9, Player::Min);
    state2.board.place_stone(9, 10, Player::Min);
    
    let score1 = Heuristic::evaluate(&state1);
    let score2 = Heuristic::evaluate(&state2);
    
    assert_eq!(score1, -score2, "Symmetric positions should have opposite scores");
}

#[test]
fn test_heuristic_no_moves_draw() {
    initialize_zobrist();
    let mut state = GameState::new(3, 3);
    
    // Fill board without winner
    state.board.place_stone(0, 0, Player::Max);
    state.board.place_stone(0, 1, Player::Min);
    state.board.place_stone(0, 2, Player::Max);
    state.board.place_stone(1, 0, Player::Min);
    state.board.place_stone(1, 1, Player::Max);
    state.board.place_stone(1, 2, Player::Min);
    state.board.place_stone(2, 0, Player::Max);
    state.board.place_stone(2, 1, Player::Min);
    state.board.place_stone(2, 2, Player::Max);
    
    let score = Heuristic::evaluate(&state);
    // Should not crash and return some value
    assert!(score.abs() < 1_000_000);
}

#[test]
fn test_heuristic_winning_line_excluded() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a winning line but don't set winner
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    
    // Don't set winner - this tests the heuristic's pattern recognition
    let score = Heuristic::evaluate(&state);
    assert!(score > 50_000, "5-in-a-row should have very high value");
}

#[test]
fn test_heuristic_open_line_vs_blocked() {
    initialize_zobrist();
    let mut state1 = GameState::new(19, 5);
    let mut state2 = GameState::new(19, 5);
    
    // Open line of 3
    state1.board.place_stone(9, 9, Player::Max);
    state1.board.place_stone(9, 10, Player::Max);
    state1.board.place_stone(9, 11, Player::Max);
    
    // Blocked line of 3
    state2.board.place_stone(9, 8, Player::Min);
    state2.board.place_stone(9, 9, Player::Max);
    state2.board.place_stone(9, 10, Player::Max);
    state2.board.place_stone(9, 11, Player::Max);
    state2.board.place_stone(9, 12, Player::Min);
    
    let score1 = Heuristic::evaluate(&state1);
    let score2 = Heuristic::evaluate(&state2);
    
    assert!(score1 > score2, "Open line should be better than blocked line");
}

#[test]
fn test_heuristic_different_line_lengths() {
    initialize_zobrist();
    let mut state1 = GameState::new(19, 5);
    let mut state2 = GameState::new(19, 5);
    
    // Line of 2
    state1.board.place_stone(9, 9, Player::Max);
    state1.board.place_stone(9, 10, Player::Max);
    
    // Line of 3
    state2.board.place_stone(9, 9, Player::Max);
    state2.board.place_stone(9, 10, Player::Max);
    state2.board.place_stone(9, 11, Player::Max);
    
    let score1 = Heuristic::evaluate(&state1);
    let score2 = Heuristic::evaluate(&state2);
    
    assert!(score2 > score1, "Longer line should have higher value");
}

#[test]
fn test_heuristic_edge_cases() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Single stone
    state.board.place_stone(9, 9, Player::Max);
    let score = Heuristic::evaluate(&state);
    assert!(score >= 0, "Single stone should not give negative score");
    
    // Corner position
    state.board.place_stone(0, 0, Player::Max);
    let score = Heuristic::evaluate(&state);
    assert!(score.abs() < 1_000_000, "Corner position should not crash");
}
