use gomoku::core::state::GameState;

#[test]
fn test_hash_stability_and_uniqueness() {
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    state.make_move((8, 8));
    
    let initial_hash = state.hash();
    let moves = vec![(7, 8), (8, 7), (6, 9), (9, 6)];
    
    for &mv in &moves {
        let before_hash = state.hash();
        state.make_move(mv);
        let after_move = state.hash();
        state.undo_move(mv);
        let after_undo = state.hash();
        
        assert_eq!(before_hash, after_undo, "Hash not restored after undo");
        assert_ne!(before_hash, after_move, "Hash should change after move");
    }
    
    assert_eq!(state.hash(), initial_hash, "Final hash should match initial");
    
    // Test position uniqueness
    let mut test_state = GameState::new(15, 5);
    let empty_hash = test_state.hash();
    
    test_state.make_move((0, 0));
    let corner_hash = test_state.hash();
    
    assert_ne!(empty_hash, corner_hash, "Empty and corner positions should have different hashes");
}

#[test]
fn test_player_turn_affects_hash() {
    let mut state1 = GameState::new(15, 5);
    let mut state2 = GameState::new(15, 5);
    
    state1.make_move((7, 7));
    state1.make_move((8, 8));
    
    state2.make_move((7, 7));
    state2.make_move((8, 8));
    state2.make_move((7, 8));
    state2.undo_move((7, 8));
    
    assert_eq!(state1.hash(), state2.hash(), "Same position should have same hash");
    
    state2.make_move((7, 8));
    assert_ne!(state1.hash(), state2.hash(), "Different players to move should have different hashes");
}
