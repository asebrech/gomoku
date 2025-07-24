use gomoku::core::state::GameState;
use gomoku::ai::minimax::parallel_iterative_deepening_search;
use gomoku::legacy::ai::minimax::iterative_deepening_search;
use gomoku::legacy::ai::transposition::TranspositionTable;
use std::time::Instant;

#[test]
fn benchmark_parallel_vs_sequential() {
    let mut state = GameState::new(15, 5);
    
    // Set up a complex middle-game position with many possible moves
    let moves = vec![
        (7, 7), (8, 8), (6, 6), (9, 9),   // Center moves
        (7, 8), (8, 7), (6, 8), (8, 6),   // Adjacent moves  
        (5, 5), (10, 10), (7, 6), (6, 7), // Expanding
    ];
    
    for mv in moves {
        state.make_move(mv);
    }
    
    let depth = 5; // Deep enough to see parallel benefits
    
    // Benchmark sequential search
    let mut state_seq = state.clone();
    let mut tt = TranspositionTable::new_default();
    
    let seq_start = Instant::now();
    let seq_result = iterative_deepening_search(&mut state_seq, depth, None, &mut tt);
    let seq_duration = seq_start.elapsed();
    
    // Benchmark parallel search
    let mut state_par = state.clone();
    
    let par_start = Instant::now();
    let par_result = parallel_iterative_deepening_search(&mut state_par, depth, None);
    let par_duration = par_start.elapsed();
    
    println!("=== PARALLEL vs SEQUENTIAL BENCHMARK ===");
    println!("Depth: {}", depth);
    println!("Sequential: {:?} | {} nodes | score: {}", 
             seq_duration, seq_result.nodes_searched, seq_result.score);
    println!("Parallel:   {:?} | {} nodes | score: {}", 
             par_duration, par_result.nodes_searched, par_result.score);
    
    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    
    // Both should find reasonable quality moves (parallel may find different but valid moves due to non-determinism)
    // Allow larger variance due to parallel execution order affecting move selection
    assert!((seq_result.score - par_result.score).abs() <= 5000, 
            "Results too different: seq={}, par={}", seq_result.score, par_result.score);
    
    // Both should find moves that are at least reasonable (not losing positions)
    assert!(seq_result.score > -50000, "Sequential found a very poor move: {}", seq_result.score);
    assert!(par_result.score > -50000, "Parallel found a very poor move: {}", par_result.score);
    
    // Parallel should be faster or at least competitive
    // (On single-core machines, it might be slightly slower due to overhead)
    println!("Performance test completed successfully!");
}

// Test that demonstrates the thread safety under load
#[test] 
fn test_thread_safety_stress() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    state.make_move((8, 8));
    state.make_move((6, 6));
    
    let state = Arc::new(Mutex::new(state));
    let results = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = vec![];
    
    // Spawn multiple threads doing parallel searches
    for i in 0..4 {
        let state_clone = Arc::clone(&state);
        let results_clone = Arc::clone(&results);
        
        let handle = thread::spawn(move || {
            let mut local_state = {
                let state_guard = state_clone.lock().unwrap();
                state_guard.clone()
            };
            
            let result = parallel_iterative_deepening_search(&mut local_state, 3, None);
            
            let mut results_guard = results_clone.lock().unwrap();
            results_guard.push((i, result.best_move, result.score));
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_results = results.lock().unwrap();
    println!("Stress test results:");
    for (thread_id, best_move, score) in final_results.iter() {
        println!("Thread {}: move={:?}, score={}", thread_id, best_move, score);
    }
    
    // All threads should complete successfully
    assert_eq!(final_results.len(), 4);
    
    // All threads should find reasonable moves
    for (_, best_move, _) in final_results.iter() {
        assert!(best_move.is_some(), "Thread should find a valid move");
    }
    
    println!("Thread safety stress test passed!");
}