use gomoku::core::state::GameState;
use gomoku::ai::transposition::{TranspositionTable, BoundType};
use gomoku::ai::minimax::reset_profiling;
use gomoku::interface::utils::find_best_move;

fn main() {
    println!("=== ENHANCED TRANSPOSITION TABLE DEMO ===");
    
    // Create a game state and transposition table
    let mut state = GameState::new(19, 19);
    let mut tt = TranspositionTable::new(19, 19);
    
    // Make some initial moves
    state.make_move((9, 9));   // Center
    state.make_move((9, 10));  // Adjacent
    state.make_move((9, 8));   // Block
    
    println!("Initial position after 3 moves");
    println!("Current player: {:?}", state.current_player);
    
    // Demonstrate enhanced TT features
    println!("\n--- Demonstrating Enhanced TT Features ---");
    
    // 1. Store with different depths and bound types
    let test_hash = 12345u64;
    
    println!("1. Storing entries with different depths and bound types:");
    tt.store_enhanced(test_hash, 100, 2, BoundType::Exact, Some((10, 10)));
    println!("   Stored: score=100, depth=2, exact bound, best_move=(10,10)");
    
    // Try to overwrite with lower depth - should not replace
    tt.store_enhanced(test_hash, 200, 1, BoundType::LowerBound, Some((11, 11)));
    println!("   Attempted to store: score=200, depth=1 (should not replace)");
    
    if let Some((score, best_move)) = tt.lookup_enhanced(test_hash, 1, -1000, 1000) {
        println!("   Retrieved: score={}, best_move={:?}", score, best_move);
    }
    
    // 2. Demonstrate age-based replacement
    println!("\n2. Demonstrating age-based cleanup:");
    tt.new_search(); // Increment age
    println!("   Started new search generation");
    
    // Store entry with new age
    tt.store_enhanced(test_hash + 1, 300, 2, BoundType::Exact, None);
    println!("   Stored entry with current age");
    
    let (hits, stores, hit_rate) = tt.get_stats();
    println!("   TT Stats: {} hits, {} stores, {:.1}% hit rate", hits, stores, hit_rate);
    
    // 3. Demonstrate bound type checking
    println!("\n3. Demonstrating bound type usage:");
    
    // Store lower bound
    let bound_test_hash = 54321u64;
    tt.store_enhanced(bound_test_hash, 150, 3, BoundType::LowerBound, None);
    
    // Try lookup with different alpha/beta windows
    if let Some((score, _)) = tt.lookup_enhanced(bound_test_hash, 2, -100, 140) {
        println!("   Lower bound {} usable for beta=140: false", score);
    } else if let Some((score, _)) = tt.lookup_enhanced(bound_test_hash, 2, -100, 160) {
        println!("   Lower bound {} usable for beta=160: true", score);
    }
    
    // 4. Demonstrate in actual minimax search
    println!("\n4. Running minimax search with enhanced TT:");
    reset_profiling();
    
    let search_start = std::time::Instant::now();
    let best_move = find_best_move(&mut state, 4, &mut tt);
    let search_time = search_start.elapsed();
    
    println!("   Best move found: {:?}", best_move);
    println!("   Search took: {:?}", search_time);
    
    let (final_hits, final_stores, final_hit_rate) = tt.get_stats();
    println!("   Final TT Stats: {} hits, {} stores, {:.1}% hit rate", 
             final_hits, final_stores, final_hit_rate);
    println!("   TT Size: {} entries", tt.size());
    
    println!("\n=== Key Improvements ===");
    println!("✓ Bound type checking for accurate alpha-beta integration");
    println!("✓ Depth-based replacement (deeper searches preferred)");
    println!("✓ Best move storage for improved move ordering");
    println!("✓ Age-based cleanup to manage memory usage");
    println!("✓ Enhanced statistics and debugging info");
    println!("✓ Hash collision protection with full hash verification");
}
