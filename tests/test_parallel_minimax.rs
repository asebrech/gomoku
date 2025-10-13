use gomoku::ai::lazy_smp::lazy_smp_search;
use gomoku::core::state::GameState;

#[test]
fn test_parallel_search_functionality() {
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    
    
    let result = lazy_smp_search(&mut state, 300, 10, Some(4));
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached >= 1);
    assert!(result.nodes_searched > 0);
}

#[test]
fn test_parallel_search_with_time_limit() {
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    
    
    
    let result = lazy_smp_search(&mut state, 50, 10, Some(4));
    
    // Allow some overhead for parallelization and first-move complexity
    assert!(result.time_elapsed.as_millis() <= 500, "Time elapsed: {}ms", result.time_elapsed.as_millis());
    assert!(result.best_move.is_some());
    // Just verify it finds a move within reasonable time
}

#[test]
fn test_parallel_search_consistency() {
    let mut state1 = GameState::new(15, 5);
    let mut state2 = state1.clone();
    
    for &mv in &[(7, 7), (8, 8), (6, 6)] {
        state1.make_move(mv);
        state2.make_move(mv);
    }
    
    
    
    
    let result1 = lazy_smp_search(&mut state1, 200, 10, Some(4));
    let result2 = lazy_smp_search(&mut state2, 200, 10, Some(4));
    
    // Parallel searches should find valid moves
    assert!(result1.best_move.is_some());
    assert!(result2.best_move.is_some());
    
    // Both should reach similar depths (within 1)
    assert!((result1.depth_reached as i32 - result2.depth_reached as i32).abs() <= 1,
            "Depths should be similar: {} vs {}", result1.depth_reached, result2.depth_reached);
    
    // Note: exact move/score might differ due to parallel search non-determinism,
    // but both should be reasonable moves
}
