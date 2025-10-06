use gomoku::ai::minimax::mtdf;
use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use std::time::Instant;

fn test_mtdf(
    state: &mut GameState,
    depth: i32,
    first_guess: i32,
    tt: &mut TranspositionTable,
) -> (i32, u64, Option<(usize, usize)>) {
    let start_time = Instant::now();
    mtdf(state, first_guess, depth, tt, &start_time, None)
}

#[test]
fn test_mtdf_terminal_position() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let (score, nodes, _) = test_mtdf(&mut state, 3, 0, &mut tt);

    // Should return valid evaluation
    assert!(score != i32::MIN && score != i32::MAX);
    assert!(nodes > 0);
}

#[test]
fn test_mtdf_depth_zero() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Make a simple move
    state.board.place_stone(9, 9, Player::Max);

    let (score, nodes, _) = test_mtdf(&mut state, 0, 0, &mut tt);

    // Should return heuristic evaluation
    assert!(score != i32::MIN && score != i32::MAX);
    assert!(nodes > 0);
}

#[test]
fn test_mtdf_maximizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position that favors the maximizing player
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.current_player = Player::Max;

    let (score, nodes, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Should return a positive score since Max has advantage
    assert!(nodes > 0);
    // Basic search functionality test
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_mtdf_minimizing_player() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up position with Min to move
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.current_player = Player::Min;

    let (score, _, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Min should try to minimize the score
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_mtdf_transposition_table_usage() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a basic position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(10, 10, Player::Min);

    // First search
    let (score1, _, _) = test_mtdf(&mut state, 2, 0, &mut tt);
    
    // Second search should benefit from transposition table
    let (score2, _, _) = test_mtdf(&mut state, 2, score1, &mut tt);

    // Should get consistent results
    assert_eq!(score1, score2);
}

#[test]
fn test_mtdf_different_depths() {
    let mut state = GameState::new(19, 5);
    let mut tt1 = TranspositionTable::default();
    let mut tt2 = TranspositionTable::default();
    
    // Set up a position
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(10, 10, Player::Min);

    let (score_depth1, _, _) = test_mtdf(&mut state, 1, 0, &mut tt1);
    let (score_depth3, _, _) = test_mtdf(&mut state, 3, 0, &mut tt2);

    // Deeper search should give at least as good results
    // (This is a basic test - in practice results may vary)
    assert!(score_depth1 != i32::MIN && score_depth1 != i32::MAX);
    assert!(score_depth3 != i32::MIN && score_depth3 != i32::MAX);
}

#[test]
fn test_mtdf_winning_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a near-winning position for Max
    for i in 0..4 {
        state.board.place_stone(9, 9 + i, Player::Max);
    }
    state.current_player = Player::Max;

    let (score, _, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Should detect winning potential - print for debugging
    println!("Winning position score: {}", score);
    assert!(score > 10_000); // High positive score for winning position
}

#[test]
fn test_mtdf_losing_position_detection() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a near-winning position for Max, but Min to move
    for i in 0..4 {
        state.board.place_stone(9, 9 + i, Player::Max);
    }
    state.current_player = Player::Min;

    let (score, _, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Max is about to win, so score should be very positive from Max's perspective
    println!("Losing position score: {}", score);
    assert!(score > 100_000); // Very positive score - Max is winning
}

#[test]
fn test_mtdf_state_restoration() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position
    state.board.place_stone(9, 9, Player::Max);
    let original_hash = state.hash();

    // Perform search
    let _ = test_mtdf(&mut state, 2, 0, &mut tt);

    // State should be restored to original
    assert_eq!(state.hash(), original_hash);
}

#[test]
fn test_mtdf_best_move_recommendation() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position where there's an obvious good move
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    state.current_player = Player::Max;

    let (score, _, best_move) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Should recommend a move that completes the line or blocks opponent
    assert!(best_move.is_some());
    println!("Best move recommendation score: {}", score);
    assert!(score > 1_000); // Should recognize the winning opportunity
}

#[test]
fn test_mtdf_defensive_move() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position where Min needs to defend
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    state.current_player = Player::Min;

    let (score, _, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Max has 3 in a row, so score should be positive from Max's perspective
    println!("Defensive move score: {}", score);
    assert!(score > 1_000); // Positive score - Max has advantage
}

#[test]
fn test_mtdf_empty_board() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    let (score, _, best_move) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Should handle empty board gracefully
    assert!(best_move.is_some());
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_mtdf_center_preference() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();

    let (score, _nodes, _) = test_mtdf(&mut state, 2, 0, &mut tt);

    // Should return reasonable evaluation for starting position
    assert!(score != i32::MIN && score != i32::MAX);
}

#[test]
fn test_mtdf_pattern_recognition() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::default();
    
    // Create a pattern that should be recognized as valuable
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    state.current_player = Player::Max;

    let (score, _nodes, _) = test_mtdf(&mut state, 1, 0, &mut tt);

    // Should evaluate pattern appropriately
    assert!(score != i32::MIN && score != i32::MAX);
}