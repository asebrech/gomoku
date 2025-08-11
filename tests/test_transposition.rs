use gomoku::ai::transposition::{TranspositionTable, EntryType};

#[test]
fn test_transposition_table_creation() {
    let mut tt = TranspositionTable::new_default();
    
    // Should create empty table
    let result1 = tt.probe(0, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(12345, 1, i32::MIN, i32::MAX);
    assert!(!result1.cutoff);
    assert!(!result2.cutoff);
}

#[test]
fn test_store_and_lookup() {
    let mut tt = TranspositionTable::new_default();
    
    // Store a value
    tt.store(12345, 100, 1, EntryType::Exact, None);
    
    // Should retrieve the stored value
    let result = tt.probe(12345, 1, i32::MIN, i32::MAX);
    assert!(result.cutoff);
    assert_eq!(result.value, Some(100));
}

#[test]
fn test_multiple_entries() {
    let mut tt = TranspositionTable::new_default();
    
    // Store multiple values
    tt.store(111, 10, 1, EntryType::Exact, None);
    tt.store(222, 20, 1, EntryType::Exact, None);
    tt.store(333, 30, 1, EntryType::Exact, None);
    
    // Should retrieve all values correctly
    let result1 = tt.probe(111, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(222, 1, i32::MIN, i32::MAX);
    let result3 = tt.probe(333, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(10));
    assert!(result2.cutoff && result2.value == Some(20));
    assert!(result3.cutoff && result3.value == Some(30));
}

#[test]
fn test_overwrite_entry() {
    let mut tt = TranspositionTable::new_default();
    
    // Store initial value
    tt.store(123, 100, 1, EntryType::Exact, None);
    let result1 = tt.probe(123, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(100));
    
    // Overwrite with new value
    tt.store(123, 200, 1, EntryType::Exact, None);
    let result2 = tt.probe(123, 1, i32::MIN, i32::MAX);
    assert!(result2.cutoff && result2.value == Some(200));
}

#[test]
fn test_lookup_nonexistent() {
    let mut tt = TranspositionTable::new_default();
    
    // Should return None for non-existent keys
    let result1 = tt.probe(999, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(0, 1, i32::MIN, i32::MAX);
    let result3 = tt.probe(u64::MAX, 1, i32::MIN, i32::MAX);
    assert!(!result1.cutoff);
    assert!(!result2.cutoff);
    assert!(!result3.cutoff);
}

#[test]
fn test_negative_values() {
    let mut tt = TranspositionTable::new_default();
    
    // Store negative values
    tt.store(111, -100, 1, EntryType::Exact, None);
    tt.store(222, -999, 1, EntryType::Exact, None);
    tt.store(333, i32::MIN, 1, EntryType::Exact, None);
    
    // Should handle negative values correctly
    let result1 = tt.probe(111, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(222, 1, i32::MIN, i32::MAX);
    let result3 = tt.probe(333, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(-100));
    assert!(result2.cutoff && result2.value == Some(-999));
    assert!(result3.cutoff && result3.value == Some(i32::MIN));
}

#[test]
fn test_extreme_values() {
    let mut tt = TranspositionTable::new_default();
    
    // Store extreme values
    tt.store(111, i32::MAX, 1, EntryType::Exact, None);
    tt.store(222, i32::MIN, 1, EntryType::Exact, None);
    tt.store(333, 0, 1, EntryType::Exact, None);
    
    // Should handle extreme values correctly
    let result1 = tt.probe(111, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(222, 1, i32::MIN, i32::MAX);
    let result3 = tt.probe(333, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(i32::MAX));
    assert!(result2.cutoff && result2.value == Some(i32::MIN));
    assert!(result3.cutoff && result3.value == Some(0));
}

#[test]
fn test_large_keys() {
    let mut tt = TranspositionTable::new_default();
    
    // Store values with large keys
    tt.store(u64::MAX, 100, 1, EntryType::Exact, None);
    tt.store(u64::MAX - 1, 200, 1, EntryType::Exact, None);
    tt.store(1_000_000_000_000, 300, 1, EntryType::Exact, None);
    
    // Should handle large keys correctly
    let result1 = tt.probe(u64::MAX, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(u64::MAX - 1, 1, i32::MIN, i32::MAX);
    let result3 = tt.probe(1_000_000_000_000, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(100));
    assert!(result2.cutoff && result2.value == Some(200));
    assert!(result3.cutoff && result3.value == Some(300));
}

#[test]
fn test_zero_key() {
    let mut tt = TranspositionTable::new_default();
    
    // Store value with zero key
    tt.store(0, 42, 1, EntryType::Exact, None);
    
    // Should handle zero key correctly
    let result = tt.probe(0, 1, i32::MIN, i32::MAX);
    assert!(result.cutoff && result.value == Some(42));
}

#[test]
fn test_collision_behavior() {
    let mut tt = TranspositionTable::new_default();
    
    // Store two different values with same key (simulating collision)
    tt.store(123, 100, 1, EntryType::Exact, None);
    tt.store(123, 200, 1, EntryType::Exact, None);
    
    // Should return the latest stored value
    let result = tt.probe(123, 1, i32::MIN, i32::MAX);
    assert!(result.cutoff && result.value == Some(200));
}

#[test]
fn test_many_entries() {
    let mut tt = TranspositionTable::new_default();
    
    // Store many entries
    for i in 0..1000 {
        tt.store(i, i as i32 * 10, 1, EntryType::Exact, None);
    }
    
    // Should retrieve all entries correctly
    for i in 0..1000 {
        let result = tt.probe(i, 1, i32::MIN, i32::MAX);
        assert!(result.cutoff && result.value == Some(i as i32 * 10));
    }
}

#[test]
fn test_clear_and_reuse() {
    let mut tt = TranspositionTable::new_default();
    
    // Store some values
    tt.store(111, 100, 1, EntryType::Exact, None);
    tt.store(222, 200, 1, EntryType::Exact, None);
    
    // Create new table (effectively clearing)
    tt = TranspositionTable::new_default();
    
    // Should be empty again
    let result1 = tt.probe(111, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(222, 1, i32::MIN, i32::MAX);
    assert!(!result1.cutoff);
    assert!(!result2.cutoff);
    
    // Should work with new values
    tt.store(333, 300, 1, EntryType::Exact, None);
    let result3 = tt.probe(333, 1, i32::MIN, i32::MAX);
    assert!(result3.cutoff && result3.value == Some(300));
}

#[test]
fn test_realistic_game_hashes() {
    let mut tt = TranspositionTable::new_default();
    
    // Simulate realistic game state hashes
    let game_hashes = vec![
        0x1234567890abcdef,
        0xfedcba0987654321,
        0x1111111111111111,
        0xaaaaaaaaaaaaaaaa,
        0x5555555555555555,
    ];
    
    let scores = vec![150, -200, 0, 1000, -500];
    
    // Store all hash-score pairs
    for (hash, score) in game_hashes.iter().zip(scores.iter()) {
        tt.store(*hash, *score, 1, EntryType::Exact, None);
    }
    
    // Verify all can be retrieved
    for (hash, expected_score) in game_hashes.iter().zip(scores.iter()) {
        let result = tt.probe(*hash, 1, i32::MIN, i32::MAX);
        assert!(result.cutoff && result.value == Some(*expected_score));
    }
}

#[test]
fn test_performance_many_lookups() {
    let mut tt = TranspositionTable::new_default();
    
    // Store initial values
    for i in 0..100 {
        tt.store(i, i as i32, 1, EntryType::Exact, None);
    }
    
    // Perform many lookups
    for _ in 0..1000 {
        for i in 0..100 {
            let result = tt.probe(i, 1, i32::MIN, i32::MAX);
            assert!(result.cutoff && result.value == Some(i as i32));
        }
    }
}

#[test]
fn test_mixed_operations() {
    let mut tt = TranspositionTable::new_default();
    
    // Mix store and lookup operations
    tt.store(1, 10, 1, EntryType::Exact, None);
    let result1 = tt.probe(1, 1, i32::MIN, i32::MAX);
    assert!(result1.cutoff && result1.value == Some(10));
    
    tt.store(2, 20, 1, EntryType::Exact, None);
    let result1b = tt.probe(1, 1, i32::MIN, i32::MAX);
    let result2 = tt.probe(2, 1, i32::MIN, i32::MAX);
    assert!(result1b.cutoff && result1b.value == Some(10));
    assert!(result2.cutoff && result2.value == Some(20));
    
    tt.store(1, 15, 1, EntryType::Exact, None); // Overwrite
    let result1c = tt.probe(1, 1, i32::MIN, i32::MAX);
    let result2b = tt.probe(2, 1, i32::MIN, i32::MAX);
    assert!(result1c.cutoff && result1c.value == Some(15));
    assert!(result2b.cutoff && result2b.value == Some(20));
    
    let result3 = tt.probe(3, 1, i32::MIN, i32::MAX);
    assert!(!result3.cutoff);
    
    tt.store(3, 30, 1, EntryType::Exact, None);
    let result3b = tt.probe(3, 1, i32::MIN, i32::MAX);
    assert!(result3b.cutoff && result3b.value == Some(30));
}

#[test]
fn test_hash_collision_resistance() {
    let mut tt = TranspositionTable::new_default();
    let mut successful_stores = 0;
    let total_attempts = 500;
    
    // Try to force collisions by using keys that might collide
    for i in 0..total_attempts {
        let base_key = i as u64;
        let collision_key = base_key ^ 0x8000000000000000_u64; // Flip top bit
        
        tt.store(base_key, i as i32, 1, EntryType::Exact, None);
        tt.store(collision_key, (i + 1000) as i32, 1, EntryType::Exact, None);
        
        // Check if we can retrieve the correct values
        let result1 = tt.probe(base_key, 1, i32::MIN, i32::MAX);
        let result2 = tt.probe(collision_key, 1, i32::MIN, i32::MAX);
        
        if result1.cutoff && result2.cutoff {
            if result1.value == Some(i as i32) && result2.value == Some((i + 1000) as i32) {
                successful_stores += 1;
            }
        }
    }
    
    let success_rate = successful_stores as f64 / total_attempts as f64;
    
    // Should handle most entries correctly even with potential collisions
    assert!(success_rate > 0.90, "Success rate too low: {:.2}%", success_rate * 100.0);
}

#[test]
fn test_aging_mechanism() {
    let mut tt = TranspositionTable::new_default();
    
    // Store entry with current age
    tt.store(12345, 500, 10, EntryType::Exact, None);
    let initial_probe = tt.probe(12345, 10, i32::MIN, i32::MAX);
    assert!(initial_probe.cutoff && initial_probe.value == Some(500));
    
    // Advance age and add more entries
    tt.advance_age();
    tt.store(67890, 300, 8, EntryType::Exact, None);
    
    // Old entry should still be there initially
    assert!(tt.probe(12345, 10, i32::MIN, i32::MAX).cutoff);
    
    // Advance age multiple times
    for _ in 0..10 {
        tt.advance_age();
    }
    
    // Table should still function after aging
    tt.store(11111, 700, 12, EntryType::Exact, None);
    assert!(tt.probe(11111, 12, i32::MIN, i32::MAX).cutoff);
}

#[test] 
fn test_table_capacity_management() {
    let mut tt = TranspositionTable::new(50); // Smaller for testing
    
    // Fill beyond capacity with varied keys
    for i in 0..200 {
        let key = (i as u64).wrapping_mul(0x9e3779b97f4a7c15); // Better distribution
        tt.store(key, i as i32 * 10, 8, EntryType::Exact, None);
    }
    
    // Recent entries should still be accessible  
    let last_key = (199_u64).wrapping_mul(0x9e3779b97f4a7c15);
    let result = tt.probe(last_key, 8, i32::MIN, i32::MAX);
    assert!(result.cutoff && result.value == Some(1990));
}

