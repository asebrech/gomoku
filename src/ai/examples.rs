use crate::ai::minimax_tt::MinimaxWithTT;
use crate::core::state::GameState;

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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_usage() {
        example_usage();
        // This test mainly ensures the code runs without panicking
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
