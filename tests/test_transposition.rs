use gomoku::ai::transposition::TranspositionTable;

#[test]
fn test_transposition_table_creation() {
    let tt = TranspositionTable::new(19, 19);
    
    // Should create empty table
    assert_eq!(tt.lookup_raw(0), None);
    assert_eq!(tt.lookup_raw(12345), None);
}

#[test]
fn test_store_and_lookup() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store a value
    tt.store_raw(12345, 100);
    
    // Should retrieve the stored value
    assert_eq!(tt.lookup_raw(12345), Some(100));
}

#[test]
fn test_multiple_entries() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store multiple values
    tt.store_raw(111, 10);
    tt.store_raw(222, 20);
    tt.store_raw(333, 30);
    
    // Should retrieve all values correctly
    assert_eq!(tt.lookup_raw(111), Some(10));
    assert_eq!(tt.lookup_raw(222), Some(20));
    assert_eq!(tt.lookup_raw(333), Some(30));
}

#[test]
fn test_overwrite_entry() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store initial value
    tt.store_raw(123, 100);
    assert_eq!(tt.lookup_raw(123), Some(100));
    
    // Overwrite with new value
    tt.store_raw(123, 200);
    assert_eq!(tt.lookup_raw(123), Some(200));
}

#[test]
fn test_lookup_nonexistent() {
    let tt = TranspositionTable::new(19, 19);
    
    // Should return None for non-existent keys
    assert_eq!(tt.lookup_raw(999), None);
    assert_eq!(tt.lookup_raw(0), None);
    assert_eq!(tt.lookup_raw(u64::MAX), None);
}

#[test]
fn test_negative_values() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store negative values
    tt.store_raw(111, -100);
    tt.store_raw(222, -999);
    tt.store_raw(333, i32::MIN);
    
    // Should handle negative values correctly
    assert_eq!(tt.lookup_raw(111), Some(-100));
    assert_eq!(tt.lookup_raw(222), Some(-999));
    assert_eq!(tt.lookup_raw(333), Some(i32::MIN));
}

#[test]
fn test_extreme_values() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store extreme values
    tt.store_raw(111, i32::MAX);
    tt.store_raw(222, i32::MIN);
    tt.store_raw(333, 0);
    
    // Should handle extreme values correctly
    assert_eq!(tt.lookup_raw(111), Some(i32::MAX));
    assert_eq!(tt.lookup_raw(222), Some(i32::MIN));
    assert_eq!(tt.lookup_raw(333), Some(0));
}

#[test]
fn test_large_keys() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store values with large keys
    tt.store_raw(u64::MAX, 100);
    tt.store_raw(u64::MAX - 1, 200);
    tt.store_raw(1_000_000_000_000, 300);
    
    // Should handle large keys correctly
    assert_eq!(tt.lookup_raw(u64::MAX), Some(100));
    assert_eq!(tt.lookup_raw(u64::MAX - 1), Some(200));
    assert_eq!(tt.lookup_raw(1_000_000_000_000), Some(300));
}

#[test]
fn test_zero_key() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store value with zero key
    tt.store_raw(0, 42);
    
    // Should handle zero key correctly
    assert_eq!(tt.lookup_raw(0), Some(42));
}

#[test]
fn test_collision_behavior() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store two different values with same key (simulating collision)
    tt.store_raw(123, 100);
    tt.store_raw(123, 200);
    
    // Should return the latest stored value
    assert_eq!(tt.lookup_raw(123), Some(200));
}

#[test]
fn test_many_entries() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store many entries
    for i in 0..1000 {
        tt.store_raw(i, i as i32 * 10);
    }
    
    // Should retrieve all entries correctly
    for i in 0..1000 {
        assert_eq!(tt.lookup_raw(i), Some(i as i32 * 10));
    }
}

#[test]
fn test_clear_and_reuse() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store some values
    tt.store_raw(111, 100);
    tt.store_raw(222, 200);
    
    // Create new table (effectively clearing)
    tt = TranspositionTable::new(19, 19);
    
    // Should be empty again
    assert_eq!(tt.lookup_raw(111), None);
    assert_eq!(tt.lookup_raw(222), None);
    
    // Should work with new values
    tt.store_raw(333, 300);
    assert_eq!(tt.lookup_raw(333), Some(300));
}

#[test]
fn test_realistic_game_hashes() {
    let mut tt = TranspositionTable::new(19, 19);
    
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
        tt.store_raw(*hash, *score);
    }
    
    // Verify all can be retrieved
    for (hash, expected_score) in game_hashes.iter().zip(scores.iter()) {
        assert_eq!(tt.lookup_raw(*hash), Some(*expected_score));
    }
}

#[test]
fn test_performance_many_lookups() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Store initial values
    for i in 0..100 {
        tt.store_raw(i, i as i32);
    }
    
    // Perform many lookups
    for _ in 0..1000 {
        for i in 0..100 {
            assert_eq!(tt.lookup_raw(i), Some(i as i32));
        }
    }
}

#[test]
fn test_mixed_operations() {
    let mut tt = TranspositionTable::new(19, 19);
    
    // Mix store and lookup operations
    tt.store_raw(1, 10);
    assert_eq!(tt.lookup_raw(1), Some(10));
    
    tt.store_raw(2, 20);
    assert_eq!(tt.lookup_raw(1), Some(10));
    assert_eq!(tt.lookup_raw(2), Some(20));
    
    tt.store_raw(1, 15); // Overwrite
    assert_eq!(tt.lookup_raw(1), Some(15));
    assert_eq!(tt.lookup_raw(2), Some(20));
    
    assert_eq!(tt.lookup_raw(3), None);
    
    tt.store_raw(3, 30);
    assert_eq!(tt.lookup_raw(3), Some(30));
}

