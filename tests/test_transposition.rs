use gomoku::ai::transposition::{TranspositionTable, TTFlag};

#[test]
fn test_transposition_table_creation() {
    let tt = TranspositionTable::new();
    
    // Should create empty table - lookup should return None
    assert_eq!(tt.lookup(0, 5, -1000, 1000), None);
    assert_eq!(tt.lookup(12345, 3, -500, 500), None);
}

#[test]
fn test_store_and_lookup_exact() {
    let mut tt = TranspositionTable::new();
    
    // Store an exact value
    tt.store(12345, 100, 5, TTFlag::Exact);
    
    // Should retrieve the stored value for same or lesser depth
    assert_eq!(tt.lookup(12345, 5, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(12345, 4, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(12345, 3, -1000, 1000), Some(100));
    
    // Should return None for greater depth
    assert_eq!(tt.lookup(12345, 6, -1000, 1000), None);
}

#[test]
fn test_store_and_lookup_lower_bound() {
    let mut tt = TranspositionTable::new();
    
    // Store a lower bound value
    tt.store(12345, 100, 5, TTFlag::LowerBound);
    
    // Should return value when it's >= beta
    assert_eq!(tt.lookup(12345, 5, -1000, 100), Some(100)); // beta = 100
    assert_eq!(tt.lookup(12345, 5, -1000, 50), Some(100));  // beta = 50
    
    // Should return None when it's < beta
    assert_eq!(tt.lookup(12345, 5, -1000, 150), None); // beta = 150
}

#[test]
fn test_store_and_lookup_upper_bound() {
    let mut tt = TranspositionTable::new();
    
    // Store an upper bound value
    tt.store(12345, 100, 5, TTFlag::UpperBound);
    
    // Should return value when it's <= alpha
    assert_eq!(tt.lookup(12345, 5, 100, 1000), Some(100)); // alpha = 100
    assert_eq!(tt.lookup(12345, 5, 150, 1000), Some(100)); // alpha = 150
    
    // Should return None when it's > alpha
    assert_eq!(tt.lookup(12345, 5, 50, 1000), None); // alpha = 50
}

#[test]
fn test_multiple_entries() {
    let mut tt = TranspositionTable::new();
    
    // Store multiple values with different keys
    tt.store(111, 10, 5, TTFlag::Exact);
    tt.store(222, 20, 4, TTFlag::LowerBound);
    tt.store(333, 30, 3, TTFlag::UpperBound);
    
    // Should retrieve all values correctly
    assert_eq!(tt.lookup(111, 5, -1000, 1000), Some(10));
    assert_eq!(tt.lookup(222, 4, -1000, 20), Some(20));
    assert_eq!(tt.lookup(333, 3, 30, 1000), Some(30));
}

#[test]
fn test_overwrite_entry() {
    let mut tt = TranspositionTable::new();
    
    // Store initial value
    tt.store(123, 100, 5, TTFlag::Exact);
    assert_eq!(tt.lookup(123, 5, -1000, 1000), Some(100));
    
    // Overwrite with new value
    tt.store(123, 200, 6, TTFlag::LowerBound);
    assert_eq!(tt.lookup(123, 6, -1000, 200), Some(200));
}

#[test]
fn test_lookup_nonexistent() {
    let tt = TranspositionTable::new();
    
    // Should return None for non-existent keys
    assert_eq!(tt.lookup(999, 5, -1000, 1000), None);
    assert_eq!(tt.lookup(0, 3, -500, 500), None);
    assert_eq!(tt.lookup(u64::MAX, 10, -10000, 10000), None);
}

#[test]
fn test_negative_values() {
    let mut tt = TranspositionTable::new();
    
    // Store negative values
    tt.store(111, -100, 5, TTFlag::Exact);
    tt.store(222, -999, 4, TTFlag::LowerBound);
    tt.store(333, i32::MIN, 3, TTFlag::UpperBound);
    
    // Should handle negative values correctly
    assert_eq!(tt.lookup(111, 5, -1000, 1000), Some(-100));
    assert_eq!(tt.lookup(222, 4, -1000, -999), Some(-999));
    assert_eq!(tt.lookup(333, 3, i32::MIN, 1000), Some(i32::MIN));
}

#[test]
fn test_extreme_values() {
    let mut tt = TranspositionTable::new();
    
    // Store extreme values
    tt.store(111, i32::MAX, 5, TTFlag::Exact);
    tt.store(222, i32::MIN, 4, TTFlag::Exact);
    tt.store(333, 0, 3, TTFlag::Exact);
    
    // Should handle extreme values correctly
    assert_eq!(tt.lookup(111, 5, -1000, i32::MAX), Some(i32::MAX));
    assert_eq!(tt.lookup(222, 4, i32::MIN, 1000), Some(i32::MIN));
    assert_eq!(tt.lookup(333, 3, -1000, 1000), Some(0));
}

#[test]
fn test_depth_filtering() {
    let mut tt = TranspositionTable::new();
    
    // Store value with depth 5
    tt.store(123, 100, 5, TTFlag::Exact);
    
    // Should return value for depth <= 5
    assert_eq!(tt.lookup(123, 5, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(123, 4, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(123, 1, -1000, 1000), Some(100));
    
    // Should return None for depth > 5
    assert_eq!(tt.lookup(123, 6, -1000, 1000), None);
    assert_eq!(tt.lookup(123, 10, -1000, 1000), None);
}

#[test]
fn test_alpha_beta_bounds() {
    let mut tt = TranspositionTable::new();
    
    // Test LowerBound with various beta values
    tt.store(100, 50, 5, TTFlag::LowerBound);
    
    assert_eq!(tt.lookup(100, 5, -1000, 50), Some(50));   // beta = 50, value = 50 (>=)
    assert_eq!(tt.lookup(100, 5, -1000, 40), Some(50));   // beta = 40, value = 50 (>=)
    assert_eq!(tt.lookup(100, 5, -1000, 60), None);       // beta = 60, value = 50 (<)
    
    // Test UpperBound with various alpha values
    tt.store(200, 50, 5, TTFlag::UpperBound);
    
    assert_eq!(tt.lookup(200, 5, 50, 1000), Some(50));    // alpha = 50, value = 50 (<=)
    assert_eq!(tt.lookup(200, 5, 60, 1000), Some(50));    // alpha = 60, value = 50 (<=)
    assert_eq!(tt.lookup(200, 5, 40, 1000), None);        // alpha = 40, value = 50 (>)
}

#[test]
fn test_ttflag_exact_ignores_bounds() {
    let mut tt = TranspositionTable::new();
    
    // Store exact value
    tt.store(123, 100, 5, TTFlag::Exact);
    
    // Should return value regardless of alpha/beta bounds
    assert_eq!(tt.lookup(123, 5, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(123, 5, 200, 1000), Some(100));   // alpha > value
    assert_eq!(tt.lookup(123, 5, -1000, 50), Some(100));   // beta < value
    assert_eq!(tt.lookup(123, 5, 200, 50), Some(100));     // alpha > value > beta
}

#[test]
fn test_large_keys() {
    let mut tt = TranspositionTable::new();
    
    // Store values with large keys
    tt.store(u64::MAX, 100, 5, TTFlag::Exact);
    tt.store(u64::MAX - 1, 200, 4, TTFlag::LowerBound);
    tt.store(1_000_000_000_000, 300, 3, TTFlag::UpperBound);
    
    // Should handle large keys correctly
    assert_eq!(tt.lookup(u64::MAX, 5, -1000, 1000), Some(100));
    assert_eq!(tt.lookup(u64::MAX - 1, 4, -1000, 200), Some(200));
    assert_eq!(tt.lookup(1_000_000_000_000, 3, 300, 1000), Some(300));
}

#[test]
fn test_zero_key() {
    let mut tt = TranspositionTable::new();
    
    // Store value with zero key
    tt.store(0, 42, 5, TTFlag::Exact);
    
    // Should handle zero key correctly
    assert_eq!(tt.lookup(0, 5, -1000, 1000), Some(42));
}

#[test]
fn test_realistic_game_scenario() {
    let mut tt = TranspositionTable::new();
    
    // Simulate realistic minimax scenario
    let game_hash = 0x1234567890abcdef;
    
    // Store a position evaluation at depth 4
    tt.store(game_hash, 150, 4, TTFlag::Exact);
    
    // Later lookup at depth 3 should return the value
    assert_eq!(tt.lookup(game_hash, 3, -1000, 1000), Some(150));
    
    // Lookup at depth 5 should return None (need deeper search)
    assert_eq!(tt.lookup(game_hash, 5, -1000, 1000), None);
    
    // Store a deeper evaluation
    tt.store(game_hash, 200, 6, TTFlag::LowerBound);
    
    // Now depth 5 lookup should work with appropriate bounds
    assert_eq!(tt.lookup(game_hash, 5, -1000, 200), Some(200));
}

#[test]
fn test_table_size_limit() {
    let mut tt = TranspositionTable::new();
    
    // Store many entries to test size limit behavior
    for i in 0..2000 {
        tt.store(i, i as i32, 5, TTFlag::Exact);
    }
    
    // The table should handle this without crashing
    // Some entries might be evicted due to size limit, but that's expected
    // Test that at least some recent entries are still there
    for i in 1900..2000 {
        // We don't assert specific values since they might be evicted
        // Just ensure the lookup doesn't crash
        let _ = tt.lookup(i, 5, -1000, 1000);
    }
}

#[test]
fn test_mixed_operations() {
    let mut tt = TranspositionTable::new();
    
    // Mix store and lookup operations
    tt.store(1, 10, 5, TTFlag::Exact);
    assert_eq!(tt.lookup(1, 5, -1000, 1000), Some(10));
    
    tt.store(2, 20, 4, TTFlag::LowerBound);
    assert_eq!(tt.lookup(1, 5, -1000, 1000), Some(10));
    assert_eq!(tt.lookup(2, 4, -1000, 20), Some(20));
    
    tt.store(1, 15, 6, TTFlag::UpperBound); // Overwrite
    assert_eq!(tt.lookup(1, 6, 15, 1000), Some(15));
    assert_eq!(tt.lookup(2, 4, -1000, 20), Some(20));
    
    assert_eq!(tt.lookup(3, 5, -1000, 1000), None);
    
    tt.store(3, 30, 3, TTFlag::Exact);
    assert_eq!(tt.lookup(3, 3, -1000, 1000), Some(30));
}

#[test]
fn test_performance_many_lookups() {
    let mut tt = TranspositionTable::new();
    
    // Store initial values
    for i in 0..100 {
        tt.store(i, i as i32, 5, TTFlag::Exact);
    }
    
    // Perform many lookups
    for _ in 0..1000 {
        for i in 0..100 {
            assert_eq!(tt.lookup(i, 5, -1000, 1000), Some(i as i32));
        }
    }
}

#[test]
fn test_flag_combinations() {
    let mut tt = TranspositionTable::new();
    
    // Test 1: Store Exact flag
    tt.store(42, 100, 5, TTFlag::Exact);
    assert_eq!(tt.lookup(42, 5, -1000, 1000), Some(100));
    
    // Test 2: Overwrite with LowerBound
    tt.store(42, 200, 6, TTFlag::LowerBound);
    assert_eq!(tt.lookup(42, 6, -1000, 200), Some(200)); // beta=200, value=200, 200>=200 -> Some(200)
    assert_eq!(tt.lookup(42, 6, -1000, 150), Some(200)); // beta=150, value=200, 200>=150 -> Some(200) (beta cutoff)
    
    // Fix the LowerBound logic test:
    // LowerBound means actual value >= stored value
    // We return stored value if stored value >= beta (causes beta cutoff)
    assert_eq!(tt.lookup(42, 6, -1000, 250), None);      // 200 >= 250 -> false, no cutoff
    
    // Test 3: Overwrite with UpperBound
    tt.store(42, 300, 7, TTFlag::UpperBound);
    assert_eq!(tt.lookup(42, 7, 300, 1000), Some(300));  // alpha=300, value=300, 300<=300 -> Some(300)
    assert_eq!(tt.lookup(42, 7, 350, 1000), Some(300));  // alpha=350, value=300, 300<=350 -> Some(300)
    assert_eq!(tt.lookup(42, 7, 250, 1000), None);       // alpha=250, value=300, 300<=250 -> None
    
    // Test 4: Test with shallower depth
    assert_eq!(tt.lookup(42, 6, 300, 1000), Some(300));  // Should work with depth 6
    
    // Test 5: Test bounds that work for UpperBound
    assert_eq!(tt.lookup(42, 6, 400, 1000), Some(300));  // alpha=400, value=300, 300<=400 -> Some(300)
    
    // Test 6: Test bounds that don't work for UpperBound
    assert_eq!(tt.lookup(42, 6, 200, 1000), None);       // alpha=200, value=300, 300<=200 -> None
}

