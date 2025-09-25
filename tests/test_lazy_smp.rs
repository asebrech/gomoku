use gomoku::ai::search::find_best_move;
use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use std::sync::Arc;
use std::thread;

#[test]
fn test_shared_transposition_table() {
    // Create a shared transposition table
    let tt = Arc::new(TranspositionTable::new(10_000));
    
    // Create test state
    let mut state = GameState::new(19, 5);
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    
    // Test that multiple threads can share the same TT
    let tt1 = tt.clone();
    let tt2 = tt.clone();
    let state1 = state.clone();
    let state2 = state.clone();
    
    let handle1 = thread::spawn(move || {
        let mut state_copy = state1;
        find_best_move(&mut state_copy, 2, None, &tt1)
    });
    
    let handle2 = thread::spawn(move || {
        let mut state_copy = state2;
        find_best_move(&mut state_copy, 2, None, &tt2)
    });
    
    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    
    // Both should return valid moves
    assert!(result1.best_move.is_some());
    assert!(result2.best_move.is_some());
    
    // Check that the shared TT has accumulated statistics from both searches
    let (hits, misses) = tt.get_stats();
    println!("Shared TT stats: {} hits, {} misses", hits, misses);
    assert!(hits + misses > 0, "Shared TT should have been used");
}

#[test]
fn test_shared_vs_separate_tt_performance() {
    // Create shared TT
    let shared_tt = Arc::new(TranspositionTable::new(50_000));
    
    // Create a moderately complex position
    let mut base_state = GameState::new(19, 5);
    base_state.board.place_stone(9, 9, Player::Max);
    base_state.board.place_stone(9, 10, Player::Min);
    base_state.board.place_stone(8, 9, Player::Max);
    base_state.board.place_stone(8, 10, Player::Min);
    
    // Test with shared TT
    let start_time = std::time::Instant::now();
    let tt_ref1 = shared_tt.clone();
    let tt_ref2 = shared_tt.clone();
    
    let mut state1 = base_state.clone();
    let mut state2 = base_state.clone();
    
    let handle1 = thread::spawn(move || {
        find_best_move(&mut state1, 3, None, &tt_ref1)
    });
    
    let handle2 = thread::spawn(move || {
        find_best_move(&mut state2, 3, None, &tt_ref2)  
    });
    
    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();
    let shared_time = start_time.elapsed();
    
    let (shared_hits, shared_misses) = shared_tt.get_stats();
    let shared_hit_rate = shared_tt.hit_rate();
    
    println!("Shared TT: {} nodes (r1: {}, r2: {}), {:.1}ms, hit rate: {:.2}%", 
             result1.nodes_searched + result2.nodes_searched, 
             result1.nodes_searched, result2.nodes_searched,
             shared_time.as_millis(), 
             shared_hit_rate * 100.0);
    
    // Test with separate TTs
    let start_time = std::time::Instant::now();
    let separate_tt1 = TranspositionTable::new(25_000);
    let separate_tt2 = TranspositionTable::new(25_000);
    
    let mut state1 = base_state.clone();
    let mut state2 = base_state;
    
    let handle1 = thread::spawn(move || {
        find_best_move(&mut state1, 3, None, &separate_tt1)
    });
    
    let handle2 = thread::spawn(move || {
        find_best_move(&mut state2, 3, None, &separate_tt2)
    });
    
    let sep_result1 = handle1.join().unwrap();
    let sep_result2 = handle2.join().unwrap();
    let separate_time = start_time.elapsed();
    
    println!("Separate TT: {} nodes (r1: {}, r2: {}), {:.1}ms", 
             sep_result1.nodes_searched + sep_result2.nodes_searched,
             sep_result1.nodes_searched, sep_result2.nodes_searched,
             separate_time.as_millis());
    
    // Verify both approaches work
    assert!(result1.best_move.is_some());
    assert!(result2.best_move.is_some());
    assert!(sep_result1.best_move.is_some());
    assert!(sep_result2.best_move.is_some());
    
    // The shared TT should generally have better hit rates
    println!("Shared TT stats: {} hits, {} misses, hit rate: {:.2}%", 
             shared_hits, shared_misses, shared_hit_rate * 100.0);
}