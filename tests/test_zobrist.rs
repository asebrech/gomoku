use gomoku::ai::zobrist::ZobristHash;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use std::collections::HashSet;

#[test]
fn test_zobrist_initialization() {
    let zobrist = ZobristHash::new(15);
    assert_eq!(zobrist.board_size(), 15);
    
    // Test different board sizes
    let zobrist_19 = ZobristHash::new(19);
    assert_eq!(zobrist_19.board_size(), 19);
    
    let zobrist_13 = ZobristHash::new(13);
    assert_eq!(zobrist_13.board_size(), 13);
}

#[test]
fn test_zobrist_deterministic_initialization() {
    // Multiple instances should generate identical hash keys (same seed)
    let zobrist1 = ZobristHash::new(15);
    let zobrist2 = ZobristHash::new(15);
    
    let mut state = GameState::new(15, 5);
    state.board.place_stone(7, 7, Player::Max);
    
    let hash1 = zobrist1.compute_hash(&state);
    let hash2 = zobrist2.compute_hash(&state);
    
    assert_eq!(hash1, hash2, "Zobrist hashes should be deterministic");
}

#[test]
fn test_zobrist_hash_uniqueness() {
    let zobrist = ZobristHash::new(15);
    let mut hashes = HashSet::new();
    
    // Test multiple different board positions
    for row in 0..5 {
        for col in 0..5 {
            let mut state = GameState::new(15, 5);
            state.board.place_stone(row, col, Player::Max);
            
            let hash = zobrist.compute_hash(&state);
            assert!(hashes.insert(hash), "Hash collision detected for position ({}, {})", row, col);
        }
    }
    
    // Should have 25 unique hashes
    assert_eq!(hashes.len(), 25);
}

#[test]
fn test_zobrist_player_differentiation() {
    let zobrist = ZobristHash::new(15);
    
    let mut state_max = GameState::new(15, 5);
    state_max.board.place_stone(7, 7, Player::Max);
    state_max.current_player = Player::Max;
    
    let mut state_min = GameState::new(15, 5);
    state_min.board.place_stone(7, 7, Player::Min);
    state_min.current_player = Player::Max;
    
    let hash_max = zobrist.compute_hash(&state_max);
    let hash_min = zobrist.compute_hash(&state_min);
    
    assert_ne!(hash_max, hash_min, "Different players should produce different hashes");
}

#[test]
fn test_zobrist_current_player_effect() {
    let zobrist = ZobristHash::new(15);
    
    let mut state = GameState::new(15, 5);
    state.board.place_stone(7, 7, Player::Max);
    
    // Same board, different current players
    state.current_player = Player::Max;
    let hash_max_turn = zobrist.compute_hash(&state);
    
    state.current_player = Player::Min;
    let hash_min_turn = zobrist.compute_hash(&state);
    
    assert_ne!(hash_max_turn, hash_min_turn, "Current player should affect hash");
}

#[test]
fn test_zobrist_incremental_updates() {
    let zobrist = ZobristHash::new(15);
    
    // Start with empty board
    let state = GameState::new(15, 5);
    let empty_hash = zobrist.compute_hash(&state);
    
    // Make a move incrementally
    let updated_hash = zobrist.update_hash_make_move(empty_hash, 7, 7, Player::Max);
    
    // Make the same move on actual board and compute hash
    let mut actual_state = GameState::new(15, 5);
    actual_state.board.place_stone(7, 7, Player::Max);
    actual_state.current_player = Player::Min; // Player switches after move
    let actual_hash = zobrist.compute_hash(&actual_state);
    
    assert_eq!(updated_hash, actual_hash, "Incremental update should match full computation");
}

#[test]
fn test_zobrist_move_undo_symmetry() {
    let zobrist = ZobristHash::new(15);
    
    let mut state = GameState::new(15, 5);
    state.board.place_stone(5, 5, Player::Max);
    state.current_player = Player::Min;
    
    let original_hash = zobrist.compute_hash(&state);
    
    // Make move incrementally
    let after_move = zobrist.update_hash_make_move(original_hash, 8, 8, Player::Min);
    
    // Undo move incrementally
    let after_undo = zobrist.update_hash_undo_move(after_move, 8, 8, Player::Min);
    
    assert_eq!(original_hash, after_undo, "Move and undo should be symmetric");
}

#[test]
fn test_zobrist_capture_updates() {
    let zobrist = ZobristHash::new(15);
    
    let mut state = GameState::new(15, 5);
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Min);
    state.board.place_stone(7, 9, Player::Min);
    
    let before_capture = zobrist.compute_hash(&state);
    
    // Simulate capture of two Min stones
    let captured_positions = vec![(7, 8), (7, 9)];
    let after_capture = zobrist.update_hash_capture(before_capture, &captured_positions, Player::Min);
    
    // Manually remove stones and compute hash
    state.board.remove_stone(7, 8);
    state.board.remove_stone(7, 9);
    let manual_hash = zobrist.compute_hash(&state);
    
    assert_eq!(after_capture, manual_hash, "Capture update should match manual computation");
}

#[test]
fn test_zobrist_multiple_moves() {
    let zobrist = ZobristHash::new(15);
    
    let mut incremental_hash = zobrist.compute_hash(&GameState::new(15, 5));
    let mut actual_state = GameState::new(15, 5);
    
    let moves = vec![
        (7, 7, Player::Max),
        (7, 8, Player::Min), 
        (8, 7, Player::Max),
        (6, 7, Player::Min),
        (9, 7, Player::Max),
    ];
    
    for (row, col, player) in moves {
        // Update incrementally
        incremental_hash = zobrist.update_hash_make_move(incremental_hash, row, col, player);
        
        // Update actual state
        actual_state.board.place_stone(row, col, player);
        actual_state.current_player = match player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };
    }
    
    let actual_hash = zobrist.compute_hash(&actual_state);
    assert_eq!(incremental_hash, actual_hash, "Multiple incremental updates should match full computation");
}

#[test]
fn test_zobrist_empty_board_hash() {
    let zobrist = ZobristHash::new(15);
    
    let empty_state1 = GameState::new(15, 5);
    let empty_state2 = GameState::new(15, 5);
    
    let hash1 = zobrist.compute_hash(&empty_state1);
    let hash2 = zobrist.compute_hash(&empty_state2);
    
    assert_eq!(hash1, hash2, "Empty boards should have identical hashes");
    
    // Empty board with different current player
    let mut empty_min = GameState::new(15, 5);
    empty_min.current_player = Player::Min;
    let hash_min = zobrist.compute_hash(&empty_min);
    
    assert_ne!(hash1, hash_min, "Empty boards with different current players should differ");
}

#[test]
fn test_zobrist_position_independence() {
    let zobrist = ZobristHash::new(15);
    
    // Two positions with same pieces but different locations
    let mut state1 = GameState::new(15, 5);
    state1.board.place_stone(0, 0, Player::Max);
    state1.board.place_stone(1, 1, Player::Min);
    
    let mut state2 = GameState::new(15, 5);
    state2.board.place_stone(5, 5, Player::Max);
    state2.board.place_stone(6, 6, Player::Min);
    
    let hash1 = zobrist.compute_hash(&state1);
    let hash2 = zobrist.compute_hash(&state2);
    
    assert_ne!(hash1, hash2, "Different positions should have different hashes");
}
