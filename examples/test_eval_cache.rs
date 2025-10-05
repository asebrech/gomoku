use gomoku::ai::lazy_smp::lazy_smp_search;
use gomoku::core::state::GameState;
use std::time::Duration;

fn main() {
    println!("=== Testing Heuristic Evaluation Caching ===\n");
    
    // Create a game state with some moves
    let mut state = GameState::new(19, 5);
    
    // Place some stones to create a non-trivial position
    state.make_move((9, 9));   // Center
    state.make_move((9, 10));
    state.make_move((10, 9));
    state.make_move((10, 10));
    state.make_move((8, 9));
    state.make_move((8, 10));
    
    println!("Testing position with {} moves played", state.move_history.len());
    println!("Current player: {:?}\n", state.current_player);
    
    // Test different depths with 500ms time limit
    let time_limit = Some(Duration::from_millis(500));
    
    for depth in [5, 6, 7, 8, 9, 10] {
        let mut test_state = state.clone();
        
        println!("Testing depth {}...", depth);
        let result = lazy_smp_search(&mut test_state, depth, time_limit, Some(4));
        
        println!("  Depth reached: {}", result.depth_reached);
        println!("  Nodes searched: {}", result.nodes_searched);
        println!("  Time elapsed: {:?}", result.time_elapsed);
        println!("  Score: {}", result.score);
        if let Some(best_move) = result.best_move {
            println!("  Best move: ({}, {})", best_move.0, best_move.1);
        }
        println!("  Nodes/second: {:.0}", 
            result.nodes_searched as f64 / result.time_elapsed.as_secs_f64());
        println!();
        
        if result.time_elapsed > Duration::from_millis(550) {
            println!("  ⚠️  Warning: Exceeded time limit!");
        }
    }
    
    println!("\n=== Testing Complete ===");
    println!("\nKey optimizations implemented:");
    println!("  ✓ Incremental pattern evaluation");
    println!("  ✓ Evaluation cache (100k entries per worker)");
    println!("  ✓ Lightweight tactical evaluation for depth > 7");
    println!("  ✓ Historical bonus only computed at shallow depths");
}
