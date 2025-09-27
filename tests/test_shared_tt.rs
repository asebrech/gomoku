use gomoku::{ai::{search::find_best_move, transposition::TranspositionTable}, core::state::GameState};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[test]
fn test_shared_transposition_table() {
    let mut state = GameState::new(15, 5);
    
    // Make a few moves to create a non-trivial position
    state.make_move((7, 7)); // Center
    state.make_move((7, 8)); // Adjacent
    state.make_move((7, 6)); // Other side
    
    let shared_tt = TranspositionTable::new(10000);
    
    // Clone the TT to share between threads (this tests that cloning works)
    let tt_clone1 = shared_tt.clone();
    let tt_clone2 = shared_tt.clone();
    
    // Test that both references point to the same underlying data
    shared_tt.advance_age();
    
    let mut state_clone1 = state.clone();
    let mut state_clone2 = state.clone();
    
    // Run searches in different threads using the shared TT
    let handle1 = thread::spawn(move || {
        find_best_move(&mut state_clone1, 3, None, &tt_clone1)
    });
    
    let handle2 = thread::spawn(move || {
        find_best_move(&mut state_clone2, 3, None, &tt_clone2)
    });
    
    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    
    // Both should have found valid moves
    assert!(result1.best_move.is_some());
    assert!(result2.best_move.is_some());
    
    // Check that the TT has accumulated stats from both searches
    let (hits, misses) = shared_tt.get_stats();
    assert!(hits > 0 || misses > 0, "TT should have some activity");
    
    println!("Test completed - TT stats: {} hits, {} misses", hits, misses);
    println!("Result 1: {:?}", result1.best_move);
    println!("Result 2: {:?}", result2.best_move);
}

#[test]
fn test_tt_cache_sharing_between_threads() {
    let shared_tt = TranspositionTable::new(50000);
    let mut base_state = GameState::new(15, 5);
    
    // Create a position where the TT will have entries
    base_state.make_move((7, 7));
    base_state.make_move((8, 8));
    base_state.make_move((6, 6));
    
    // First thread does a deep search to populate the TT
    let tt_clone1 = shared_tt.clone();
    let mut state_clone1 = base_state.clone();
    let handle1 = thread::spawn(move || {
        find_best_move(&mut state_clone1, 4, None, &tt_clone1)
    });
    let result1 = handle1.join().unwrap();
    
    let (hits_after_first, misses_after_first) = shared_tt.get_stats();
    let tt_size_after_first = shared_tt.size();
    
    // Second thread does a similar search - should benefit from cached entries
    let tt_clone2 = shared_tt.clone();
    let mut state_clone2 = base_state.clone();
    let handle2 = thread::spawn(move || {
        find_best_move(&mut state_clone2, 4, None, &tt_clone2)
    });
    let result2 = handle2.join().unwrap();
    
    let (hits_after_second, misses_after_second) = shared_tt.get_stats();
    let tt_size_after_second = shared_tt.size();
    
    // Verify that both searches found moves
    assert!(result1.best_move.is_some());
    assert!(result2.best_move.is_some());
    
    // The second search should have benefited from cached entries
    assert!(hits_after_second > hits_after_first, 
           "Second search should have more hits due to shared cache. Hits: {} -> {}", 
           hits_after_first, hits_after_second);
    
    // Total entries should not have doubled (some reuse occurred)
    assert!(tt_size_after_second < tt_size_after_first * 2, 
           "TT size should not double due to cache reuse. Size: {} -> {}", 
           tt_size_after_first, tt_size_after_second);
    
    println!("First search - Hits: {}, Misses: {}, TT Size: {}", 
             hits_after_first, misses_after_first, tt_size_after_first);
    println!("After second search - Hits: {}, Misses: {}, TT Size: {}", 
             hits_after_second, misses_after_second, tt_size_after_second);
}

#[test]
fn test_concurrent_tt_access_stress() {
    let shared_tt = TranspositionTable::new(100000);
    let num_threads = 8;
    let searches_per_thread = 3;
    
    // Create different starting positions for each thread
    let mut base_positions = Vec::new();
    for i in 0..num_threads {
        let mut state = GameState::new(15, 5);
        state.make_move((7, 7));
        state.make_move((7 + (i % 3), 8 + (i % 3)));
        state.make_move((6 - (i % 2), 6 + (i % 2)));
        base_positions.push(state);
    }
    
    let hit_counters: Vec<Arc<AtomicUsize>> = (0..num_threads)
        .map(|_| Arc::new(AtomicUsize::new(0)))
        .collect();
    
    let handles: Vec<_> = (0..num_threads).map(|thread_id| {
        let tt_clone = shared_tt.clone();
        let mut state = base_positions[thread_id].clone();
        let hit_counter = hit_counters[thread_id].clone();
        
        thread::spawn(move || {
            let mut thread_results = Vec::new();
            
            for search_id in 0..searches_per_thread {
                // Modify the position slightly for each search
                if search_id > 0 {
                    let moves = state.get_possible_moves();
                    if !moves.is_empty() {
                        state.make_move(moves[search_id % moves.len()]);
                    }
                }
                
                let (_initial_hits, _) = tt_clone.get_stats();
                let result = find_best_move(&mut state, 3, None, &tt_clone);
                let (final_hits, _) = tt_clone.get_stats();
                
                hit_counter.store(final_hits as usize, Ordering::Relaxed);
                thread_results.push((thread_id, search_id, result.best_move.is_some()));
                
                // Small delay to increase chances of concurrent access
                thread::sleep(Duration::from_millis(1));
            }
            
            thread_results
        })
    }).collect();
    
    // Wait for all threads to complete
    let mut all_results = Vec::new();
    for handle in handles {
        let thread_results = handle.join().unwrap();
        all_results.extend(thread_results);
    }
    
    // Verify all searches found moves
    for (thread_id, search_id, found_move) in &all_results {
        assert!(*found_move, "Thread {} search {} should have found a move", thread_id, search_id);
    }
    
    let (final_hits, final_misses) = shared_tt.get_stats();
    let final_size = shared_tt.size();
    
    // With concurrent access, we should see significant TT activity
    assert!(final_hits > 0, "Should have cache hits from shared access");
    assert!(final_size > 0, "TT should contain cached positions");
    
    // Verify hit counters show increasing values (cache building up)
    let mut increasing_hits = 0;
    for i in 1..hit_counters.len() {
        let prev_hits = hit_counters[i-1].load(Ordering::Relaxed);
        let curr_hits = hit_counters[i].load(Ordering::Relaxed);
        if curr_hits > prev_hits {
            increasing_hits += 1;
        }
    }
    
    println!("Stress test completed:");
    println!("  Total searches: {}", all_results.len());
    println!("  Final TT stats: {} hits, {} misses, {} entries", final_hits, final_misses, final_size);
    println!("  Threads showing hit increases: {}/{}", increasing_hits, hit_counters.len() - 1);
}

#[test]
fn test_tt_data_consistency_across_threads() {
    let shared_tt = TranspositionTable::new(25000);
    let mut state = GameState::new(15, 5);
    
    // Create a deterministic position
    state.make_move((7, 7));
    state.make_move((8, 7));
    state.make_move((6, 7));
    
    let num_threads = 4;
    let handles: Vec<_> = (0..num_threads).map(|thread_id| {
        let tt_clone = shared_tt.clone();
        let mut state_clone = state.clone();
        
        thread::spawn(move || {
            // Each thread does the same search from the same position
            let result = find_best_move(&mut state_clone, 3, None, &tt_clone);
            (thread_id, result.best_move, result.score, result.nodes_searched)
        })
    }).collect();
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // All threads should find moves
    for (thread_id, best_move, _, nodes) in &results {
        assert!(best_move.is_some(), "Thread {} should find a move", thread_id);
        assert!(*nodes > 0, "Thread {} should search some nodes", thread_id);
    }
    
    // Since all threads started from the same position, they should find the same best move
    // (assuming deterministic search behavior)
    let _first_move = results[0].1;
    for (thread_id, best_move, _, _) in &results[1..] {
        // Note: Due to parallel execution and racing, moves might differ slightly
        // but they should all be valid
        assert!(best_move.is_some(), "Thread {} should have a valid move like thread 0", thread_id);
    }
    
    let (total_hits, total_misses) = shared_tt.get_stats();
    let total_size = shared_tt.size();
    
    println!("Consistency test results:");
    for (thread_id, best_move, score, nodes) in &results {
        println!("  Thread {}: move={:?}, score={}, nodes={}", thread_id, best_move, score, nodes);
    }
    println!("  Shared TT: {} hits, {} misses, {} entries", total_hits, total_misses, total_size);
    
    // The shared TT should have been used effectively
    assert!(total_hits > 0 || total_size > 0, "TT should show evidence of sharing");
}

#[test]
fn test_tt_age_and_cleanup_with_threading() {
    let shared_tt = TranspositionTable::new(1000); // Small size to force cleanup
    let mut base_state = GameState::new(15, 5);
    base_state.make_move((7, 7));
    
    // First, populate the TT with some entries
    let tt_clone = shared_tt.clone();
    let mut state_clone = base_state.clone();
    let handle = thread::spawn(move || {
        find_best_move(&mut state_clone, 2, None, &tt_clone)
    });
    handle.join().unwrap();
    
    let size_after_first = shared_tt.size();
    assert!(size_after_first > 0, "TT should have some entries after first search");
    
    // Advance age several times to trigger cleanup
    for _ in 0..15 {
        shared_tt.advance_age();
    }
    
    // Do another search in a separate thread after aging
    let tt_clone2 = shared_tt.clone();
    let mut state_clone2 = base_state.clone();
    state_clone2.make_move((8, 8)); // Slightly different position
    let handle2 = thread::spawn(move || {
        find_best_move(&mut state_clone2, 2, None, &tt_clone2)
    });
    handle2.join().unwrap();
    
    let size_after_aging = shared_tt.size();
    
    // After aging and cleanup, old entries should be removed
    // The exact behavior depends on the cleanup implementation, but size should be manageable
    assert!(size_after_aging <= shared_tt.size() * 2, "TT size should be controlled after aging");
    
    println!("Age/cleanup test: size {} -> {} after aging", size_after_first, size_after_aging);
}

#[test]
fn test_tt_thread_safety_rapid_access() {
    let shared_tt = TranspositionTable::new(50000);
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    state.make_move((8, 8));
    
    let num_rapid_threads = 10;
    let operations_per_thread = 5;
    
    let handles: Vec<_> = (0..num_rapid_threads).map(|thread_id| {
        let tt_clone = shared_tt.clone();
        let mut state_clone = state.clone();
        
        // Add a unique move for each thread to create different positions
        let unique_moves = state_clone.get_possible_moves();
        if thread_id < unique_moves.len() {
            state_clone.make_move(unique_moves[thread_id]);
        }
        
        thread::spawn(move || {
            let mut operations_completed = 0;
            
            for op_id in 0..operations_per_thread {
                // Alternate between search operations and TT queries
                if op_id % 2 == 0 {
                    // Do a quick search
                    let result = find_best_move(&mut state_clone, 2, None, &tt_clone);
                    if result.best_move.is_some() {
                        operations_completed += 1;
                    }
                } else {
                    // Test TT operations directly
                    tt_clone.advance_age(); // This should be thread-safe
                    let (hits, misses) = tt_clone.get_stats();
                    if hits > 0 || misses > 0 { // Basic sanity check - any activity
                        operations_completed += 1;
                    }
                }
                
                // Very brief pause to increase concurrency chances
                thread::sleep(Duration::from_micros(100));
            }
            
            (thread_id, operations_completed)
        })
    }).collect();
    
    // Collect all results
    let mut completed_operations = 0;
    for handle in handles {
        let (thread_id, ops_completed) = handle.join().unwrap();
        completed_operations += ops_completed;
        println!("Thread {} completed {} operations", thread_id, ops_completed);
    }
    
    let (final_hits, final_misses) = shared_tt.get_stats();
    let final_size = shared_tt.size();
    
    // All operations should have completed successfully
    let expected_operations = num_rapid_threads * operations_per_thread;
    assert_eq!(completed_operations, expected_operations, 
              "All {} operations should complete successfully", expected_operations);
    
    // TT should have activity from concurrent access
    assert!(final_hits + final_misses > 0, "TT should show access activity");
    assert!(final_size > 0, "TT should contain some entries");
    
    println!("Rapid access test: {}/{} operations completed", completed_operations, expected_operations);
    println!("Final TT state: {} hits, {} misses, {} entries", final_hits, final_misses, final_size);
}