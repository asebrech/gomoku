// filepath: c:\Users\furie\OneDrive\Documents\Dev\gomoku\src\ui\app.rs
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme};
use bevy::color::palettes::css::CRIMSON;
use gomoku::core::board::{initialize_zobrist, Player};
use gomoku::core::state::GameState;
use gomoku::interface::utils::find_best_move;

#[test]
fn test_find_best_move_first_move() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    let best_move = find_best_move(&mut state, 3);
    
    assert!(best_move.is_some());
    assert_eq!(best_move.unwrap(), (9, 9)); // Should be center
}

#[test]
fn test_find_best_move_response() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Make first move
    state.make_move((9, 9));
    
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    let (row, col) = best_move.unwrap();
    assert!(state.board.is_adjacent_to_stone(row, col));
}

#[test]
fn test_find_best_move_block_opponent() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a threat that needs blocking
    state.make_move((9, 9));  // Max
    state.make_move((9, 10)); // Min
    state.make_move((9, 11)); // Max
    state.make_move((9, 12)); // Min - creates threat
    
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    let (row, col) = best_move.unwrap();
    
    // Should be a reasonable move (valid and adjacent)
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&(row, col)), "AI should make valid move: ({}, {})", row, col);
}

#[test]
fn test_find_best_move_winning_opportunity() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a winning opportunity by making moves properly
    // This creates a situation where Max has 4 in a row and needs to complete it
    state.make_move((9, 9));  // Max
    state.make_move((10, 9)); // Min
    state.make_move((9, 10)); // Max
    state.make_move((10, 10)); // Min
    state.make_move((9, 11)); // Max
    state.make_move((10, 11)); // Min
    state.make_move((9, 12)); // Max - now Max has 4 in a row horizontally
    state.make_move((10, 12)); // Min
    
    // Now it's Max's turn and should complete the 5-in-a-row
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    let (row, col) = best_move.unwrap();
    
    // Should complete the winning line (either at (9, 8) or (9, 13))
    let winning_moves = vec![(9, 8), (9, 13)];
    assert!(winning_moves.contains(&(row, col)), "Should play winning move at ({}, {}), but got ({}, {})", 9, 8, row, col);
}

#[test]
fn test_find_best_move_no_moves() {
    initialize_zobrist();
    let mut state = GameState::new(3, 3);
    
    // Fill the board
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
        }
    }
    
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_none());
}

#[test]
fn test_find_best_move_different_depths() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Make some moves
    state.make_move((9, 9));
    state.make_move((9, 10));
    
    let move_depth_1 = find_best_move(&mut state, 1);
    let move_depth_3 = find_best_move(&mut state, 3);
    
    assert!(move_depth_1.is_some());
    assert!(move_depth_3.is_some());
    
    // Both should be valid moves
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&move_depth_1.unwrap()));
    assert!(possible_moves.contains(&move_depth_3.unwrap()));
}

#[test]
fn test_find_best_move_complex_position() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a complex position
    let moves = vec![
        (9, 9), (9, 10), (8, 9), (10, 10),
        (7, 9), (11, 11), (6, 9), (12, 12),
    ];
    
    for &mv in &moves {
        state.make_move(mv);
    }
    
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    let possible_moves = state.get_possible_moves();
    assert!(possible_moves.contains(&best_move.unwrap()));
}

#[test]
fn test_find_best_move_player_alternation() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Test that AI works for both players
    state.current_player = Player::Max;
    let max_move = find_best_move(&mut state, 2);
    assert!(max_move.is_some());
    
    state.make_move(max_move.unwrap());
    
    state.current_player = Player::Min;
    let min_move = find_best_move(&mut state, 2);
    assert!(min_move.is_some());
}

#[test]
fn test_find_best_move_consistent_results() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.make_move((9, 9));
    state.make_move((9, 10));
    
    // Multiple calls should return valid moves
    for _ in 0..5 {
        let best_move = find_best_move(&mut state, 2);
        assert!(best_move.is_some());
        
        let possible_moves = state.get_possible_moves();
        assert!(possible_moves.contains(&best_move.unwrap()));
    }
}

#[test]
fn test_find_best_move_state_preservation() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.make_move((9, 9));
    state.make_move((9, 10));
    
    let initial_hash = state.hash();
    let initial_player = state.current_player;
    
    let best_move = find_best_move(&mut state, 3);
    assert!(best_move.is_some());
    
    // State should be preserved
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
}

#[test]
fn test_find_best_move_different_board_sizes() {
    initialize_zobrist();
    
    for &size in &[13, 15, 19] {
        let mut state = GameState::new(size, 5);
        let best_move = find_best_move(&mut state, 2);
        
        assert!(best_move.is_some());
        let (row, col) = best_move.unwrap();
        assert_eq!((row, col), (size / 2, size / 2)); // Should be center
    }
}

#[test]
fn test_find_best_move_edge_cases() {
    initialize_zobrist();
    
    // Test with capture win condition
    let mut state = GameState::new(19, 5);
    state.make_move((9, 9));
    state.max_captures = 5; // Set captures to winning amount
    
    let best_move = find_best_move(&mut state, 3);
    // Should return None since game is won by captures
    assert!(best_move.is_none());
    
    // Test with winner set
    let mut state2 = GameState::new(19, 5);
    state2.winner = Some(Player::Max);
    let best_move = find_best_move(&mut state2, 3);
    assert!(best_move.is_none());
    
    // Test with normal position (should work)
    let mut state3 = GameState::new(19, 5);
    state3.make_move((9, 9));
    let best_move = find_best_move(&mut state3, 3);
    assert!(best_move.is_some());
}
