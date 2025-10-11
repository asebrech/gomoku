use gomoku::ai::move_ordering::MoveOrdering;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_move_ordering_prioritizes_center() {
    let mut state = GameState::new(15, 5);
    
    // Place one stone to enable adjacent moves
    state.board.place_stone(7, 7, Player::Max);
    
    let mut moves = state.get_candidate_moves();
    let original_moves = moves.clone();
    
    MoveOrdering::order_moves(&state, &mut moves);
    
    // Should have same moves, just reordered
    assert_eq!(moves.len(), original_moves.len());
    assert!(moves.iter().all(|m| original_moves.contains(m)));
    
    // Center-adjacent moves should be prioritized
    let center = state.board.size / 2;
    let first_move = moves[0];
    let last_move = moves[moves.len() - 1];
    
    let first_distance = ((first_move.0 as isize - center as isize).abs() + 
                         (first_move.1 as isize - center as isize).abs()) as usize;
    let last_distance = ((last_move.0 as isize - center as isize).abs() + 
                        (last_move.1 as isize - center as isize).abs()) as usize;
    
    assert!(first_distance <= last_distance, "First move should be closer to center");
}

#[test]
fn test_move_ordering_prioritizes_threats() {
    let mut state = GameState::new(15, 5);
    
    // Create threat scenario: XXX.
    state.board.place_stone(7, 5, Player::Max);
    state.board.place_stone(7, 6, Player::Max);
    state.board.place_stone(7, 7, Player::Max);
    
    // Add some other random stones to create more move options
    state.board.place_stone(5, 5, Player::Min);
    state.board.place_stone(9, 9, Player::Min);
    
    let mut moves = state.get_candidate_moves();
    MoveOrdering::order_moves(&state, &mut moves);
    
    // Move that completes the threat (7,8) or blocks it (7,4) should be first
    let first_few_moves = &moves[0..3.min(moves.len())];
    assert!(first_few_moves.contains(&(7, 8)) || first_few_moves.contains(&(7, 4)),
            "Threat completion/blocking should be prioritized: {:?}", first_few_moves);
}

#[test]
fn test_move_ordering_adjacency_bonus() {
    let mut state = GameState::new(15, 5);
    
    // Place stones that don't create threats (scattered pattern)
    // This avoids triggering threat detection in move generation
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(9, 9, Player::Min);  // Far from Max stone
    state.board.place_stone(5, 5, Player::Max);  // Far from first Max stone
    
    let mut moves = state.get_candidate_moves();
    MoveOrdering::order_moves(&state, &mut moves);
    
    // Moves adjacent to stones should be prioritized
    // Positions like (7,6), (7,8), (6,7), (8,7) are adjacent to (7,7)
    // Positions like (5,4), (5,6), (4,5), (6,5) are adjacent to (5,5)
    let high_priority_moves = &moves[0..10.min(moves.len())];
    
    // Should prioritize moves adjacent to existing stones
    // Check if any of the top moves are adjacent to (7,7)
    let adjacent_to_77: Vec<_> = high_priority_moves.iter()
        .filter(|(r, c)| {
            (*r as isize - 7).abs() <= 1 && (*c as isize - 7).abs() <= 1 && !(*r == 7 && *c == 7)
        })
        .collect();
    
    assert!(!adjacent_to_77.is_empty(), "Should prioritize moves adjacent to stones");
}

#[test]
fn test_move_ordering_consistency() {
    let mut state = GameState::new(15, 5);
    
    // Create same position twice
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(8, 8, Player::Min);
    
    let mut moves1 = state.get_candidate_moves();
    let mut moves2 = state.get_candidate_moves();
    
    MoveOrdering::order_moves(&state, &mut moves1);
    MoveOrdering::order_moves(&state, &mut moves2);
    
    // Should produce identical ordering for identical positions
    assert_eq!(moves1, moves2, "Move ordering should be deterministic");
}

#[test]
fn test_move_ordering_performance() {
    let mut state = GameState::new(19, 5);
    
    // Create complex position with many possible moves
    for i in 5..15 {
        for j in 5..15 {
            if (i + j) % 3 == 0 {
                state.board.place_stone(i, j, Player::Max);
            } else if (i + j) % 5 == 0 {
                state.board.place_stone(i, j, Player::Min);
            }
        }
    }
    
    let mut moves = state.get_candidate_moves();
    let start = std::time::Instant::now();
    
    MoveOrdering::order_moves(&state, &mut moves);
    
    let elapsed = start.elapsed();
    
    // Should complete quickly even with many moves
    assert!(elapsed.as_millis() < 10, "Move ordering should be fast: {:?}", elapsed);
    assert!(!moves.is_empty(), "Should have moves to order");
}
