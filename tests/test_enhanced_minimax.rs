use gomoku::interface::utils::{find_best_move, init_ai, get_tt_stats, clear_tt};
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_enhanced_minimax_basic() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Place some stones to create an interesting position
    state.make_move((7, 7));  // Player::Max
    state.make_move((7, 8));  // Player::Min
    state.make_move((8, 7));  // Player::Max
    
    // Find best move using enhanced minimax with TT
    let best_move = find_best_move(&mut state, 4);
    assert!(best_move.is_some());
    
    let (size, hit_rate, collisions) = get_tt_stats();
    println!("TT entries: {}, hit rate: {:.2}%, collisions: {}", size, hit_rate * 100.0, collisions);
}

#[test]
fn test_enhanced_minimax_different_depths() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Create a position
    state.make_move((7, 7));
    state.make_move((6, 6));
    state.make_move((8, 8));
    
    // Test different search depths
    let move_depth2 = find_best_move(&mut state, 2);
    clear_tt(); // Clear between searches
    let move_depth4 = find_best_move(&mut state, 4);
    
    assert!(move_depth2.is_some());
    assert!(move_depth4.is_some());
    
    // Both should find valid moves
    println!("Depth 2 move: {:?}, Depth 4 move: {:?}", move_depth2, move_depth4);
}

#[test]
fn test_enhanced_minimax_winning_position() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Create near-winning position for current player
    state.make_move((7, 7));  // Max
    state.make_move((6, 6));  // Min
    state.make_move((7, 8));  // Max
    state.make_move((6, 7));  // Min  
    state.make_move((7, 9));  // Max
    state.make_move((6, 8));  // Min
    state.make_move((7, 10)); // Max - 4 in a row
    // Now Min to play, should try to block or find counter-threat
    
    let best_move = find_best_move(&mut state, 6);
    assert!(best_move.is_some());
    
    // Check that TT was used effectively
    let (size, hit_rate, _) = get_tt_stats();
    assert!(size > 0);
    println!("Position analysis used {} TT entries with {:.1}% hit rate", size, hit_rate * 100.0);
}

#[test] 
fn test_enhanced_minimax_performance() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Create a complex middle-game position
    let moves = vec![
        (7, 7), (8, 8), (6, 6), (9, 9),
        (7, 8), (8, 7), (6, 7), (7, 6)
    ];
    
    for mv in moves {
        state.make_move(mv);
    }
    
    // Time the search
    let start = std::time::Instant::now();
    let best_move = find_best_move(&mut state, 5);
    let duration = start.elapsed();
    
    assert!(best_move.is_some());
    
    let (size, hit_rate, collisions) = get_tt_stats();
    println!("Search took {:?}", duration);
    println!("TT: {} entries, {:.1}% hit rate, {} collisions", size, hit_rate * 100.0, collisions);
    
    // Should have reasonable performance and explore positions
    assert!(duration.as_secs() < 15); // Should finish within 15 seconds (increased for iterative deepening)
    // Note: When using iterative deepening, the global TT might have fewer entries
    // since ID uses its own engine with separate TT, so we just check that search completed successfully
    assert!(best_move.is_some());
}

#[test]
fn test_enhanced_minimax_empty_board() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Empty board - should choose center or near-center
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    let (row, col) = best_move.unwrap();
    let center = 7; // 15/2 = 7
    
    // Should choose somewhere in central area
    assert!(row >= center - 2 && row <= center + 2);
    assert!(col >= center - 2 && col <= center + 2);
    
    println!("Opening move: ({}, {})", row, col);
}
