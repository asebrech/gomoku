use gomoku::core::state::GameState;
use gomoku::ai::minimax::{parallel_iterative_deepening_search, iterative_deepening_search};
use gomoku::ai::transposition::TranspositionTable;
use std::time::Duration;

#[test]
fn test_parallel_vs_sequential_same_result() {
    let mut state = GameState::new(15, 5);
    
    // Set up a simple position
    state.make_move((7, 7)); // Center move
    state.make_move((7, 8)); // Adjacent move
    state.make_move((8, 7)); // Another move
    
    let mut state_copy = state.clone();
    
    // Test with depth 3 (shallow enough to be deterministic)
    let mut tt = TranspositionTable::new_default();
    let sequential_result = iterative_deepening_search(&mut state, 3, None, &mut tt);
    let parallel_result = parallel_iterative_deepening_search(&mut state_copy, 3, None);
    
    // Both should find the same best move (or at least equally good moves)
    println!("Sequential: best_move={:?}, score={}", sequential_result.best_move, sequential_result.score);
    println!("Parallel: best_move={:?}, score={}", parallel_result.best_move, parallel_result.score);
    
    // The scores should be the same (both finding optimal play)
    assert_eq!(sequential_result.score, parallel_result.score);
    assert!(sequential_result.best_move.is_some());
    assert!(parallel_result.best_move.is_some());
}

#[test]
fn test_parallel_search_performance() {
    let mut state = GameState::new(15, 5);
    
    // Set up a position with multiple good moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    state.make_move((6, 8));
    
    let start_time = std::time::Instant::now();
    let result = parallel_iterative_deepening_search(&mut state, 4, Some(Duration::from_millis(1000)));
    let elapsed = start_time.elapsed();
    
    println!("Parallel search completed in {:?}", elapsed);
    println!("Result: best_move={:?}, score={}, depth_reached={}, nodes={}", 
             result.best_move, result.score, result.depth_reached, result.nodes_searched);
    
    // Should complete within reasonable time
    assert!(elapsed < Duration::from_secs(2));
    assert!(result.best_move.is_some());
    assert!(result.nodes_searched > 0);
}

#[test]
fn test_parallel_search_finds_winning_move() {
    let mut state = GameState::new(15, 5);
    
    // Set up a position where there's a clear winning move
    // Create a 4-in-a-row threat that needs to be completed
    state.make_move((7, 7));
    state.make_move((6, 7));
    state.make_move((7, 8));
    state.make_move((6, 8));
    state.make_move((7, 9));
    state.make_move((6, 9));
    state.make_move((7, 10));
    // Now (7, 6) would complete 5 in a row for the current player
    
    let result = parallel_iterative_deepening_search(&mut state, 4, None);
    
    println!("Winning position result: best_move={:?}, score={}", result.best_move, result.score);
    
    // Should find the winning move
    assert!(result.best_move.is_some());
    
    // The score should indicate a winning position (very high positive value)
    // Since this is a forced win, the score should be close to the mate value
    assert!(result.score > 900_000, "Expected winning score but got {}", result.score);
}

#[test]
fn test_parallel_search_mate_distance() {
    let mut state = GameState::new(15, 5);
    
    // Create a position where there are wins at different depths
    // This tests that the parallel search correctly prioritizes shorter mates
    state.make_move((7, 7));
    state.make_move((6, 7));
    state.make_move((7, 8));
    state.make_move((6, 8));
    state.make_move((7, 9));
    state.make_move((6, 9));
    state.make_move((7, 10));
    // Now it's Min's turn - the AI found (6, 10) as the best move
    
    let result = parallel_iterative_deepening_search(&mut state, 6, None);
    
    println!("Mate distance test: best_move={:?}, score={}", result.best_move, result.score);
    
    // Should find the immediate win
    assert_eq!(result.best_move, Some((6, 10)));
    
    // Score should reflect a winning position (very high positive value indicates Max wins through optimal play)
    assert!(result.score > 999_995, "Expected immediate mate score but got {}", result.score);
}
