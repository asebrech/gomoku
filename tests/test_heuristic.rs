use gomoku::ai::heuristic::Heuristic;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_heuristic_empty_board() {
    let state = GameState::new(19, 5, 5);
    let score = Heuristic::evaluate(&state, 1);

    // Empty board should have neutral score
    assert_eq!(score, 0);
}

#[test]
fn test_heuristic_winner_max() {
    let mut state = GameState::new(19, 5, 5);
    state.winner = Some(Player::Max);

    let score = Heuristic::evaluate(&state, 1);
    assert_eq!(score, 1_000_001);
}

#[test]
fn test_heuristic_winner_min() {
    let mut state = GameState::new(19, 5, 5);
    state.winner = Some(Player::Min);

    let score = Heuristic::evaluate(&state, 1);
    assert_eq!(score, -1_000_001);
}

#[test]
fn test_heuristic_capture_win_max() {
    let mut state = GameState::new(19, 5, 5);
    state.max_captures = 5; // 5 pairs captured = win

    let score = Heuristic::evaluate(&state, 1);
    assert_eq!(score, 1_000_001);
}

#[test]
fn test_heuristic_capture_win_min() {
    let mut state = GameState::new(19, 5, 5);
    state.min_captures = 5; // 5 pairs captured = win

    let score = Heuristic::evaluate(&state, 1);
    assert_eq!(score, -1_000_001);
}

#[test]
fn test_heuristic_no_moves_draw() {
    let mut state = GameState::new(3, 3, 5);

    // Fill board with no winner
    state.board.place_stone(0, 0, Player::Max);
    state.board.place_stone(0, 1, Player::Min);
    state.board.place_stone(0, 2, Player::Max);
    state.board.place_stone(1, 0, Player::Min);
    state.board.place_stone(1, 1, Player::Max);
    state.board.place_stone(1, 2, Player::Min);
    state.board.place_stone(2, 0, Player::Max);
    state.board.place_stone(2, 1, Player::Min);
    state.board.place_stone(2, 2, Player::Max);

    let score = Heuristic::evaluate(&state, 1);
    assert_eq!(score, 0); // Draw
}

#[test]
fn test_heuristic_capture_advantage() {
    let mut state = GameState::new(19, 5, 5);

    // Give Max capture advantage
    state.max_captures = 3;
    state.min_captures = 1;

    state.board.place_stone(9, 9, Player::Max); // Add some stones to avoid empty board

    let score = Heuristic::evaluate(&state, 1);

    // Should favor Max due to capture advantage
    assert!(score > 0);

    // Should include capture bonus (3-1)*1000 = 2000
    assert!(score >= 2000);
}

#[test]
fn test_heuristic_line_evaluation() {
    let mut state = GameState::new(19, 5, 5);

    // Create a line of 3 stones for Max
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 9, Player::Max);

    let score = Heuristic::evaluate(&state, 1);

    // Should be positive (favoring Max)
    assert!(score > 0);
}

#[test]
fn test_heuristic_blocked_line() {
    let mut state = GameState::new(19, 5, 5);

    // Create a line of 3 stones for Max, blocked on both sides
    state.board.place_stone(9, 6, Player::Min); // Block left
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min); // Block right

    let score = Heuristic::evaluate(&state, 1);

    // Should be less favorable than open line
    // Since blocked lines get score 0, other factors determine the score
    assert!(score != 0); // Not zero due to other evaluations
}

#[test]
fn test_heuristic_open_line_vs_blocked() {
    let mut state1 = GameState::new(19, 5, 5);
    let mut state2 = GameState::new(19, 5, 5);

    // State 1: Open line (both ends open)
    state1.board.place_stone(9, 7, Player::Max);
    state1.board.place_stone(9, 8, Player::Max);
    state1.board.place_stone(9, 9, Player::Max);

    // State 2: Semi-open line (one end blocked)
    state2.board.place_stone(9, 6, Player::Min); // Block one end
    state2.board.place_stone(9, 7, Player::Max);
    state2.board.place_stone(9, 8, Player::Max);
    state2.board.place_stone(9, 9, Player::Max);

    let score1 = Heuristic::evaluate(&state1, 1);
    let score2 = Heuristic::evaluate(&state2, 1);

    // Open line should be better than semi-open
    assert!(score1 > score2);
}

#[test]
fn test_heuristic_opponent_advantage() {
    let mut state = GameState::new(19, 5, 5);

    // Create advantage for Min
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 9, Player::Min);

    let score = Heuristic::evaluate(&state, 1);

    // Should be negative (favoring Min)
    assert!(score < 0);
}

#[test]
fn test_heuristic_multiple_directions() {
    let mut state = GameState::new(19, 5, 5);

    // Create lines in multiple directions for Max
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 8, Player::Max); // Horizontal
    state.board.place_stone(8, 9, Player::Max); // Vertical
    state.board.place_stone(8, 8, Player::Max); // Diagonal

    let score = Heuristic::evaluate(&state, 1);

    // Should be strongly positive due to multiple threats
    assert!(score > 100);
}

#[test]
fn test_heuristic_winning_line_excluded() {
    let mut state = GameState::new(19, 5, 5);

    // Create a winning line (5 in a row)
    for i in 0..5 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }

    let score = Heuristic::evaluate(&state, 1);

    // Winning lines should not contribute to line evaluation
    // Score should be based on other factors
    assert!(score >= 0);
}

#[test]
fn test_heuristic_different_line_lengths() {
    let mut state2 = GameState::new(19, 5, 5);
    let mut state3 = GameState::new(19, 5, 5);
    let mut state4 = GameState::new(19, 5, 5);

    // 2 in a row (open)
    state2.board.place_stone(9, 8, Player::Max);
    state2.board.place_stone(9, 9, Player::Max);

    // 3 in a row (open)
    state3.board.place_stone(9, 7, Player::Max);
    state3.board.place_stone(9, 8, Player::Max);
    state3.board.place_stone(9, 9, Player::Max);

    // 4 in a row (open)
    state4.board.place_stone(9, 6, Player::Max);
    state4.board.place_stone(9, 7, Player::Max);
    state4.board.place_stone(9, 8, Player::Max);
    state4.board.place_stone(9, 9, Player::Max);

    let score2 = Heuristic::evaluate(&state2, 1);
    let score3 = Heuristic::evaluate(&state3, 1);
    let score4 = Heuristic::evaluate(&state4, 1);

    // Longer lines should be more valuable
    assert!(score2 < score3);
    assert!(score3 < score4);
}

#[test]
fn test_heuristic_edge_cases() {
    let mut state = GameState::new(19, 5, 5);

    // Test evaluation near board edges
    state.board.place_stone(0, 0, Player::Max);
    state.board.place_stone(0, 1, Player::Max);
    state.board.place_stone(0, 2, Player::Max);

    let score = Heuristic::evaluate(&state, 1);

    // Should handle edge cases without crashing
    assert!(score != 0);
}

#[test]
fn test_heuristic_symmetry() {
    let mut state_max = GameState::new(19, 5, 5);
    let mut state_min = GameState::new(19, 5, 5);

    // Create identical patterns for both players
    state_max.board.place_stone(9, 7, Player::Max);
    state_max.board.place_stone(9, 8, Player::Max);
    state_max.board.place_stone(9, 9, Player::Max);

    state_min.board.place_stone(9, 7, Player::Min);
    state_min.board.place_stone(9, 8, Player::Min);
    state_min.board.place_stone(9, 9, Player::Min);

    let score_max = Heuristic::evaluate(&state_max, 1);
    let score_min = Heuristic::evaluate(&state_min, 1);

    // Should be symmetric (opposite signs)
    assert_eq!(score_max, -score_min);
}
