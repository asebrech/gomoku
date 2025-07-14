use gomoku::core::board::{Player, initialize_zobrist};
use gomoku::core::state::GameState;

#[test]
fn test_game_state_creation() {
    let state = GameState::new(19, 5);
    assert_eq!(state.board.size, 19);
    assert_eq!(state.win_condition, 5);
    assert_eq!(state.current_player, Player::Max);
    assert_eq!(state.winner, None);
    assert_eq!(state.max_captures, 0);
    assert_eq!(state.min_captures, 0);
    assert!(state.board.is_empty());
}

#[test]
fn test_make_move_basic() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.make_move((9, 9));
    
    assert_eq!(state.board.get_player(9, 9), Some(Player::Max));
    assert_eq!(state.current_player, Player::Min);
    assert!(!state.board.is_empty());
}

#[test]
fn test_undo_move_basic() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.make_move((9, 9));
    state.undo_move((9, 9));
    
    assert_eq!(state.board.get_player(9, 9), None);
    assert_eq!(state.current_player, Player::Max);
    assert!(state.board.is_empty());
}

#[test]
fn test_make_move_with_capture() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Set up capture scenario
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    
    state.current_player = Player::Max;
    state.make_move((9, 12));
    
    // Verify capture occurred
    assert_eq!(state.board.get_player(9, 10), None);
    assert_eq!(state.board.get_player(9, 11), None);
    assert_eq!(state.max_captures, 1);
}

#[test]
fn test_undo_move_with_capture() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Set up and execute capture
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    
    state.current_player = Player::Max;
    state.make_move((9, 12));
    
    // Undo the capture
    state.undo_move((9, 12));
    
    // Verify restoration
    assert_eq!(state.board.get_player(9, 10), Some(Player::Min));
    assert_eq!(state.board.get_player(9, 11), Some(Player::Min));
    assert_eq!(state.board.get_player(9, 12), None);
    assert_eq!(state.min_captures, 0);
}

#[test]
fn test_is_terminal_winner_exists() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.winner = Some(Player::Max);
    assert!(state.is_terminal());
}

#[test]
fn test_is_terminal_no_moves() {
    initialize_zobrist();
    let mut state = GameState::new(3, 3);
    
    // Fill the board
    for i in 0..3 {
        for j in 0..3 {
            state.board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
        }
    }
    
    assert!(state.is_terminal());
}

#[test]
fn test_check_winner() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    assert_eq!(state.check_winner(), None);
    
    state.winner = Some(Player::Max);
    assert_eq!(state.check_winner(), Some(Player::Max));
}

#[test]
fn test_winning_by_line() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create 4 in a row
    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }
    
    // Make the winning move
    state.current_player = Player::Max;
    state.make_move((9, 9));
    
    assert_eq!(state.winner, Some(Player::Max));
}

#[test]
fn test_capture_win_detection() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    state.max_captures = 5;
    assert_eq!(state.check_capture_win(), Some(Player::Max));
    
    state.max_captures = 0;
    state.min_captures = 5;
    assert_eq!(state.check_capture_win(), Some(Player::Min));
}

#[test]
fn test_capture_history_tracking() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Set up capture scenario
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    
    state.current_player = Player::Max;
    state.make_move((9, 12));
    
    // Check capture history
    assert_eq!(state.capture_history.len(), 1);
    assert_eq!(state.capture_history[0].len(), 2);
    assert!(state.capture_history[0].contains(&(9, 10)));
    assert!(state.capture_history[0].contains(&(9, 11)));
}

#[test]
fn test_hash_consistency() {
    initialize_zobrist();
    let mut state1 = GameState::new(19, 5);
    let mut state2 = GameState::new(19, 5);
    
    // Same sequences should produce same hash
    state1.make_move((9, 9));
    state1.make_move((9, 10));
    
    state2.make_move((9, 9));
    state2.make_move((9, 10));
    
    assert_eq!(state1.hash(), state2.hash());
    
    // Different sequences should produce different hashes
    state1.make_move((8, 8));
    state2.make_move((8, 9));
    
    assert_ne!(state1.hash(), state2.hash());
}

#[test]
fn test_complex_game_sequence() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    let moves = vec![
        (9, 9), (9, 10), (8, 9), (10, 10),
        (7, 9), (11, 11), (6, 9), (12, 12),
    ];
    
    for &mv in &moves {
        let possible_moves = state.get_possible_moves();
        assert!(possible_moves.contains(&mv));
        state.make_move(mv);
    }
    
    // Should still be playable
    assert!(!state.is_terminal());
    assert!(state.get_possible_moves().len() > 0);
}

#[test]
fn test_game_state_different_sizes() {
    initialize_zobrist();
    
    for &size in &[13, 15, 19] {
        let state = GameState::new(size, 5);
        assert_eq!(state.board.size, size);
        assert_eq!(state.board.center(), (size / 2, size / 2));
    }
}

#[test]
fn test_first_move_only_center() {
    initialize_zobrist();
    let state = GameState::new(19, 5);
    
    let possible_moves = state.get_possible_moves();
    assert_eq!(possible_moves.len(), 1);
    assert_eq!(possible_moves[0], (9, 9));
}
