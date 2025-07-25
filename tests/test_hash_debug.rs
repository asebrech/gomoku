use gomoku::core::state::GameState;

#[test]
fn test_hash_stability() {
    let mut state = GameState::new(15, 5, 5);
    
    // Place some initial stones using proper move mechanics
    state.make_move((7, 7));  // Max
    state.make_move((8, 8));  // Min
    // Now Max is to move again
    
    let initial_hash = state.hash();
    println!("Initial hash: {}", initial_hash);
    
    // Test move/undo cycle stability
    let moves = vec![(7, 8), (8, 7), (6, 9), (9, 6)];
    
    for (i, &mv) in moves.iter().enumerate() {
        println!("\n--- Testing move {} at {:?} ---", i + 1, mv);
        
        let before_move_hash = state.hash();
        let before_player = state.current_player;
        println!("Before move - Hash: {}, Player: {:?}", before_move_hash, before_player);
        
        // Make move
        state.make_move(mv);
        let after_move_hash = state.hash();
        let after_player = state.current_player;
        println!("After move - Hash: {}, Player: {:?}", after_move_hash, after_player);
        
        // Undo move
        state.undo_move(mv);
        let after_undo_hash = state.hash();
        let after_undo_player = state.current_player;
        println!("After undo - Hash: {}, Player: {:?}", after_undo_hash, after_undo_player);
        
        // Check if hash is restored
        if before_move_hash != after_undo_hash {
            panic!(
                "Hash not restored! Before: {}, After undo: {}", 
                before_move_hash, after_undo_hash
            );
        }
        
        if before_player != after_undo_player {
            panic!(
                "Player not restored! Before: {:?}, After undo: {:?}",
                before_player, after_undo_player
            );
        }
        
        println!("✓ Hash and player correctly restored");
    }
    
    let final_hash = state.hash();
    if initial_hash != final_hash {
        panic!("Final hash {} doesn't match initial hash {}", final_hash, initial_hash);
    }
    
    println!("\n✅ All hash stability tests passed!");
}

#[test]
fn test_position_uniqueness() {
    println!("\n🔍 Testing position hash uniqueness...");
    
    let mut state = GameState::new(15, 5, 5);
    let mut hashes = std::collections::HashSet::new();
    
    // Initial state
    let initial_hash = state.hash();
    println!("Empty board hash: {}", initial_hash);
    hashes.insert(initial_hash);
    
    let mut collisions = 0;
    let mut total_positions = 1;  // Count the initial empty board
    
    // Generate different positions using proper move mechanics
    let test_positions = vec![
        vec![(7, 7)],                    // Single stone
        vec![(7, 7), (8, 8)],           // Two stones
        vec![(7, 7), (8, 8), (7, 8)],  // Three stones
        vec![(7, 7), (8, 8), (7, 8), (8, 7)], // Four stones
        vec![(0, 0)],                    // Corner stone
        vec![(14, 14)],                  // Opposite corner
        vec![(7, 0)],                    // Edge stone
        vec![(0, 7)],                    // Different edge
        vec![(7, 7), (7, 8), (7, 9)],  // Horizontal line
        vec![(7, 7), (8, 7), (9, 7)],  // Vertical line
        vec![(6, 6), (7, 7), (8, 8)],  // Diagonal line
    ];
    
    for (i, position) in test_positions.iter().enumerate() {
        // Reset to empty board
        let mut temp_state = GameState::new(15, 5, 5);
        
        // Apply moves
        for &mv in position {
            temp_state.make_move(mv);
        }
        
        let hash = temp_state.hash();
        total_positions += 1;
        
        if hashes.contains(&hash) {
            collisions += 1;
            println!("❌ COLLISION at position {}: Hash {} already seen", i, hash);
            println!("   Position: {:?}", position);
        } else {
            hashes.insert(hash);
            println!("✓ Position {}: Hash {} (unique)", i, hash);
        }
    }
    
    let collision_rate = (collisions as f64 / total_positions as f64) * 100.0;
    println!("\n📊 Hash uniqueness results:");
    println!("Total positions: {}", total_positions);
    println!("Unique hashes: {}", hashes.len());
    println!("Collisions: {}", collisions);
    println!("Collision rate: {:.2}%", collision_rate);
    
    if collision_rate > 10.0 {
        panic!("Hash collision rate too high: {:.2}%", collision_rate);
    }
    
    println!("✅ Position uniqueness test passed!");
}

#[test]
fn test_player_perspective_in_hash() {
    println!("\n--- Testing Player Perspective in Hash ---");
    
    let mut state1 = GameState::new(15, 5, 5);
    let mut state2 = GameState::new(15, 5, 5);
    
    // Create identical board positions but different current players
    state1.make_move((7, 7));  // Max plays
    state1.make_move((8, 8));  // Min plays, now Max to move
    
    state2.make_move((7, 7));  // Max plays
    state2.make_move((8, 8));  // Min plays, now Max to move
    state2.make_move((7, 8));  // Max plays, now Min to move
    state2.undo_move((7, 8));  // Back to Max to move - should be same as state1
    
    let hash1 = state1.hash();
    let hash2 = state2.hash();
    
    println!("State1 hash (Max to move): {}", hash1);
    println!("State2 hash (Max to move after undo): {}", hash2);
    
    if hash1 != hash2 {
        panic!("⚠️  ERROR: Same position should have same hash after undo!");
    }
    
    // Now test different players to move
    state2.make_move((7, 8));  // Now Min to move
    let hash3 = state2.hash();
    
    println!("State2 hash (Min to move): {}", hash3);
    
    if hash1 == hash3 {
        println!("⚠️  WARNING: Same hash for different players to move!");
        println!("This could explain low TT hit rates - positions with different players");
        println!("to move should have different hashes for proper transposition table usage.");
    } else {
        println!("✓ Different players to move produce different hashes");
    }
}
