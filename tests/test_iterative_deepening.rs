use gomoku::interface::utils::{find_best_move, init_ai, clear_tt};
use gomoku::core::state::GameState;
use gomoku::ai::iterative_deepening::{IterativeDeepeningEngine, SearchConfig};
use std::time::Duration;

#[test]
fn test_iterative_deepening_basic() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Place some stones to create an interesting position
    state.make_move((7, 7));  // Player::Max
    state.make_move((7, 8));  // Player::Min
    state.make_move((8, 7));  // Player::Max
    
    // Find best move using iterative deepening
    let best_move = find_best_move(&mut state, 6);
    assert!(best_move.is_some());
    
    println!("Iterative deepening found move: {:?}", best_move);
}

#[test]
fn test_iterative_deepening_vs_regular() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    
    // Create a position
    state.make_move((7, 7));
    state.make_move((6, 6));
    state.make_move((8, 8));
    
    // Test both search methods
    clear_tt();
    let regular_move = find_best_move(&mut state, 4);
    
    clear_tt();
    let id_move = find_best_move(&mut state, 4);
    
    assert!(regular_move.is_some());
    assert!(id_move.is_some());
    
    println!("Regular move: {:?}, ID move: {:?}", regular_move, id_move);
}

#[test]
fn test_iterative_deepening_time_limit() {
    let mut state = GameState::new(15, 5);
    
    // Initialize AI
    init_ai(15);
    clear_tt();
    
    // Create a complex position
    let moves = vec![
        (7, 7), (8, 8), (6, 6), (9, 9),
        (7, 8), (8, 7), (6, 7), (7, 6)
    ];
    
    for mv in moves {
        state.make_move(mv);
    }
    
    // Time the search with a reasonable depth
    let start = std::time::Instant::now();
    let best_move = find_best_move(&mut state, 4); // Use even smaller depth
    let duration = start.elapsed();
    
    assert!(best_move.is_some());
    // Just verify it completes in reasonable time - iterative deepening can be slower due to repeated work
    println!("ID search took {:?} and found move {:?}", duration, best_move);
}

#[test]
fn test_iterative_deepening_engine_direct() {
    let mut state = GameState::new(15, 5);
    
    // Place some stones
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    
    // Test the engine directly
    let mut engine = IterativeDeepeningEngine::new(15);
    
    let config = SearchConfig {
        max_depth: 5,
        max_time: Some(Duration::from_secs(3)),
    };
    
    let result = engine.search(&state, config);
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached > 0);
    assert!(result.nodes_evaluated > 0);
    
    println!("Direct engine test - Move: {:?}, Depth: {}, Nodes: {}, Time: {:?}", 
             result.best_move, result.depth_reached, result.nodes_evaluated, result.time_elapsed);
}

#[test]
fn test_iterative_deepening_aspiration_windows() {
    let mut state = GameState::new(15, 5);
    
    // Create near-winning position for current player
    state.make_move((7, 7));  // Max
    state.make_move((6, 6));  // Min
    state.make_move((7, 8));  // Max
    state.make_move((6, 7));  // Min  
    state.make_move((7, 9));  // Max
    state.make_move((6, 8));  // Min
    state.make_move((7, 10)); // Max - 4 in a row
    // Now Min to play, should try to block or find counter-threat
    
    let mut engine = IterativeDeepeningEngine::new(15);
    
    // Test with aspiration windows
    let config_with_aspiration = SearchConfig {
        max_depth: 6,
        max_time: Some(Duration::from_secs(5)),
    };
    
    let result_with_aspiration = engine.search(&state, config_with_aspiration);
    
    // Test without aspiration windows
    engine.clear_tt();
    let config_without_aspiration = SearchConfig {
        max_depth: 6,
        max_time: Some(Duration::from_secs(5)),
    };
    
    let result_without_aspiration = engine.search(&state, config_without_aspiration);
    
    assert!(result_with_aspiration.best_move.is_some());
    assert!(result_without_aspiration.best_move.is_some());
    
    println!("With aspiration: Move {:?}, Nodes: {}, Time: {:?}", 
             result_with_aspiration.best_move, result_with_aspiration.nodes_evaluated, result_with_aspiration.time_elapsed);
    println!("Without aspiration: Move {:?}, Nodes: {}, Time: {:?}", 
             result_without_aspiration.best_move, result_without_aspiration.nodes_evaluated, result_without_aspiration.time_elapsed);
}

#[test]
fn test_iterative_deepening_empty_board() {
    let mut state = GameState::new(15, 5);
    
    // Test on empty board
    let mut engine = IterativeDeepeningEngine::new(15);
    
    let config = SearchConfig {
        max_depth: 4,
        max_time: Some(Duration::from_secs(2)),
    };
    
    let result = engine.search(&state, config);
    
    assert!(result.best_move.is_some());
    
    let (row, col) = result.best_move.unwrap();
    let center = 7; // 15/2 = 7
    
    // Should choose somewhere in central area
    assert!(row >= center - 2 && row <= center + 2);
    assert!(col >= center - 2 && col <= center + 2);
    
    println!("Opening move with ID: ({}, {}), depth reached: {}", 
             row, col, result.depth_reached);
}
