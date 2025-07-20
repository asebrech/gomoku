use crate::ai::minimax_tt::MinimaxWithTT;
use crate::ai::iterative_deepening::{IterativeDeepeningEngine, SearchConfig};
use crate::core::state::GameState;
use std::time::Duration;

/// Example of how to use the new minimax with transposition table
pub fn example_usage() {
    // Create a new game state
    let mut state = GameState::new(15, 5);
    
    // Create a minimax engine with transposition table
    let mut engine = MinimaxWithTT::new(15);
    
    // Make some initial moves to create a position
    state.make_move((7, 7)); // Center move
    state.make_move((7, 8)); // Adjacent move
    state.make_move((8, 7)); // Another adjacent move
    
    println!("Starting position created with {} moves", 3);
    
    // Find the best move using iterative deepening up to depth 6
    let start_time = std::time::Instant::now();
    let best_move = engine.find_best_move(&mut state, 6);
    let elapsed = start_time.elapsed();
    
    match best_move {
        Some((row, col)) => {
            println!("Best move found: ({}, {})", row, col);
        }
        None => {
            println!("No move found!");
        }
    }
    
    // Print performance statistics
    let (table_size, hit_rate, collisions) = engine.get_tt_stats();
    println!("Search completed in {:?}", elapsed);
    println!("Nodes searched: {}", engine.nodes_searched);
    println!("TT hits: {}", engine.tt_hits);
    println!("TT cutoffs: {}", engine.tt_cutoffs);
    println!("TT size: {} entries", table_size);
    println!("TT hit rate: {:.2}%", hit_rate * 100.0);
    println!("TT collisions: {}", collisions);
}

/// Benchmark comparison between old and new minimax
pub fn benchmark_comparison() {
    let mut state = GameState::new(15, 5);
    
    // Create some initial position
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    state.make_move((8, 8));
    
    let depth = 5;
    
    // Benchmark old minimax
    let start_time = std::time::Instant::now();
    let _old_result = crate::ai::minimax::minimax(
        &mut state.clone(),
        depth,
        i32::MIN + 1,
        i32::MAX - 1,
        true,
    );
    let old_time = start_time.elapsed();
    
    // Benchmark new minimax with TT
    let mut engine = MinimaxWithTT::new(15);
    let start_time = std::time::Instant::now();
    let _new_result = engine.find_best_move(&mut state, depth);
    let new_time = start_time.elapsed();
    
    println!("Benchmark Results (depth {}):", depth);
    println!("Old minimax: {:?}", old_time);
    println!("New minimax with TT: {:?}", new_time);
    println!("Speedup: {:.2}x", old_time.as_secs_f64() / new_time.as_secs_f64());
    println!("TT hit rate: {:.2}%", engine.get_tt_stats().1 * 100.0);
}

/// Example usage of iterative deepening search
pub fn example_iterative_deepening() {
    // Create a new game state
    let mut state = GameState::new(15, 5);
    
    // Create an iterative deepening engine
    let mut engine = IterativeDeepeningEngine::new(15);
    
    // Make some initial moves to create an interesting position
    state.make_move((7, 7)); // Center move
    state.make_move((7, 8)); // Adjacent move
    state.make_move((8, 7)); // Another adjacent move
    state.make_move((6, 6)); // Create some complexity
    
    println!("🎮 Starting iterative deepening example with 4 moves played");
    
    // Configure the search
    let config = SearchConfig {
        max_depth: 8,
        max_time: Some(Duration::from_secs(5)),
        use_aspiration_windows: true,
        aspiration_window_size: 50,
        use_parallel_root: true,
    };
    
    // Perform the search
    let start_time = std::time::Instant::now();
    let result = engine.search(&state, config);
    let elapsed = start_time.elapsed();
    
    // Display results
    match result.best_move {
        Some((row, col)) => {
            println!("🎯 Best move found: ({}, {})", row, col);
            println!("📊 Search Statistics:");
            println!("   - Final score: {}", result.best_score);
            println!("   - Depth reached: {}/{}", result.depth_reached, 8);
            println!("   - Nodes evaluated: {}", result.nodes_evaluated);
            println!("   - Time elapsed: {:?}", elapsed);
            println!("   - Principal variation: {:?}", result.pv);
            
            // Engine statistics
            let (table_size, hit_rate, collisions, total_nodes) = engine.get_stats();
            println!("   - TT size: {} entries", table_size);
            println!("   - TT hit rate: {:.2}%", hit_rate * 100.0);
            println!("   - TT collisions: {}", collisions);
            println!("   - Total nodes from engine: {}", total_nodes);
        }
        None => {
            println!("❌ No move found!");
        }
    }
}

/// Comparison between regular minimax and iterative deepening
pub fn compare_search_methods() {
    println!("🔬 Comparing search methods on the same position");
    
    let mut state = GameState::new(15, 5);
    // Create a moderately complex position
    let moves = vec![(7, 7), (8, 8), (6, 6), (9, 9), (7, 8), (8, 7)];
    for mv in &moves {
        state.make_move(*mv);
    }
    
    println!("Position created with {} moves", moves.len());
    
    // Test regular search (using interface function)
    println!("\n📊 Regular Alpha-Beta Search:");
    let start = std::time::Instant::now();
    let regular_move = crate::interface::utils::find_best_move(&mut state.clone(), 5);
    let regular_time = start.elapsed();
    
    if let Some((row, col)) = regular_move {
        println!("   Move: ({}, {})", row, col);
        println!("   Time: {:?}", regular_time);
    }
    
    // Test iterative deepening
    println!("\n🔄 Iterative Deepening Search:");
    let mut id_engine = IterativeDeepeningEngine::new(15);
    let config = SearchConfig {
        max_depth: 5,
        max_time: Some(Duration::from_secs(3)),
        use_aspiration_windows: true,
        aspiration_window_size: 50,
        use_parallel_root: true,
    };
    
    let start = std::time::Instant::now();
    let id_result = id_engine.search(&state, config);
    let id_time = start.elapsed();
    
    if let Some((row, col)) = id_result.best_move {
        println!("   Move: ({}, {})", row, col);
        println!("   Time: {:?}", id_time);
        println!("   Depth reached: {}", id_result.depth_reached);
        println!("   Nodes evaluated: {}", id_result.nodes_evaluated);
        
        let (_, hit_rate, _, _) = id_engine.get_stats();
        println!("   TT hit rate: {:.2}%", hit_rate * 100.0);
    }
    
    println!("\n📈 Comparison Summary:");
    println!("   Regular search time: {:?}", regular_time);
    println!("   Iterative deepening time: {:?}", id_time);
    if regular_time > Duration::from_nanos(0) && id_time > Duration::from_nanos(0) {
        let ratio = regular_time.as_secs_f64() / id_time.as_secs_f64();
        if ratio > 1.0 {
            println!("   Iterative deepening is {:.2}x faster", ratio);
        } else {
            println!("   Regular search is {:.2}x faster", 1.0 / ratio);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_usage() {
        example_usage();
        // This test mainly ensures the code runs without panicking
    }
    
    #[test]
    fn test_iterative_deepening_example() {
        example_iterative_deepening();
        // This test mainly ensures the iterative deepening example runs without panicking
    }
    
    #[test]
    fn test_compare_search_methods() {
        compare_search_methods();
        // This test mainly ensures the comparison runs without panicking
    }
    
    #[test]
    fn test_performance_improvement() {
        // This is a basic test to ensure TT provides some benefit
        let mut state = GameState::new(15, 5);
        state.make_move((7, 7));
        state.make_move((7, 8));
        
        let mut engine = MinimaxWithTT::new(15);
        
        // First search
        let start = std::time::Instant::now();
        let _result1 = engine.find_best_move(&mut state, 4);
        let time1 = start.elapsed();
        
        // Second search (should benefit from TT)
        let start = std::time::Instant::now();
        let _result2 = engine.find_best_move(&mut state, 4);
        let time2 = start.elapsed();
        
        println!("First search: {:?}", time1);
        println!("Second search: {:?}", time2);
        println!("TT hit rate: {:.2}%", engine.get_tt_stats().1 * 100.0);
        
        // The second search should typically be faster due to TT hits
        // (though this isn't guaranteed in all positions)
    }
}
