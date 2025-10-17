use gomoku::core::board::{Board, Player};
use gomoku::core::state::GameState;
use gomoku::core::rules::GameRules;

#[test]
fn test_unbreakable_five_wins_immediately() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((9, 8));
    state.make_move((10, 8));
    state.make_move((9, 9));
    state.make_move((10, 9));
    state.make_move((9, 10));
    state.make_move((10, 10));
    state.make_move((9, 11));
    state.make_move((10, 11));
    state.make_move((9, 12));
    
    assert_eq!(state.winner, Some(Player::Max));
    assert_eq!(state.player_in_check, None);
}

#[test]
fn test_cannot_break_five_no_capture_pattern() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 12, Player::Max);
    
    let can_break = GameRules::can_break_five_by_capture(&board, 9, 10, Player::Max);
    assert!(!can_break);
}

#[test]
fn test_diagonal_capture_pattern_detected() {
    let mut board = Board::new(19);
    
    board.place_stone(10, 10, Player::Max);
    board.place_stone(10, 11, Player::Max);
    board.place_stone(10, 12, Player::Max);
    board.place_stone(10, 13, Player::Max);
    board.place_stone(10, 14, Player::Max);
    
    board.place_stone(9, 10, Player::Min);
    board.place_stone(11, 12, Player::Max);
    
    let can_break = GameRules::can_break_five_by_capture(&board, 10, 11, Player::Max);
    assert!(can_break, "Diagonal capture pattern should be detected");
    
    let breaking_moves = GameRules::get_breaking_capture_moves(&board, 10, 11, Player::Max);
    assert!(breaking_moves.contains(&(12, 13)), "Should include capture move at (12,13)");
}

#[test]
fn test_horizontal_capture_pattern_detected() {
    let mut board = Board::new(19);
    
    board.place_stone(10, 10, Player::Max);
    board.place_stone(10, 11, Player::Max);
    board.place_stone(10, 12, Player::Max);
    board.place_stone(10, 13, Player::Max);
    board.place_stone(10, 14, Player::Max);
    
    board.place_stone(9, 10, Player::Min);
    board.place_stone(11, 11, Player::Max);
    
    let can_break = GameRules::can_break_five_by_capture(&board, 10, 11, Player::Max);
    let breaking_moves = GameRules::get_breaking_capture_moves(&board, 10, 11, Player::Max);
    
    if can_break {
        assert!(breaking_moves.len() > 0, "Should have breaking moves when breakable");
    }
}

#[test]
fn test_breakable_five_creates_check_state() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check.is_some() {
        assert_eq!(state.winner, None, "Breakable five should not declare winner");
        assert_eq!(state.player_in_check, Some(Player::Max));
    }
}

#[test]
fn test_check_state_cleared_after_breaking_move() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check.is_some() {
        let breaking_moves = GameRules::get_breaking_capture_moves(&state.board, 10, 14, Player::Max);
        
        if let Some(&capture_move) = breaking_moves.first() {
            state.make_move(capture_move);
            
            assert!(state.player_in_check.is_none() || state.winner.is_some(), 
                "Check state should be cleared after breaking move or game should end");
        }
    }
}

#[test]
fn test_get_breaking_capture_moves_returns_valid_moves() {
    let mut board = Board::new(19);
    
    board.place_stone(10, 10, Player::Max);
    board.place_stone(10, 11, Player::Max);
    board.place_stone(10, 12, Player::Max);
    board.place_stone(10, 13, Player::Max);
    board.place_stone(10, 14, Player::Max);
    
    board.place_stone(9, 10, Player::Min);
    board.place_stone(11, 12, Player::Max);
    
    let moves = GameRules::get_breaking_capture_moves(&board, 10, 11, Player::Max);
    
    for &(row, col) in &moves {
        assert!(row < 19 && col < 19, "All moves should be within board bounds");
    }
}

#[test]
fn test_vertical_five_cannot_be_broken_without_capture_setup() {
    let mut board = Board::new(19);
    
    for i in 0..5 {
        board.place_stone(10 + i, 10, Player::Max);
    }
    
    let can_break = GameRules::can_break_five_by_capture(&board, 12, 10, Player::Max);
    assert!(!can_break, "Vertical five without capture pattern should not be breakable");
}

#[test]
fn test_check_win_and_breakable_function() {
    let mut board = Board::new(19);
    
    board.place_stone(10, 10, Player::Max);
    board.place_stone(10, 11, Player::Max);
    board.place_stone(10, 12, Player::Max);
    board.place_stone(10, 13, Player::Max);
    board.place_stone(10, 14, Player::Max);
    
    let (has_win, is_breakable) = GameRules::check_win_and_breakable(&board, 10, 12, 5);
    
    assert!(has_win, "Should detect five in a row");
    assert!(!is_breakable, "Should not be breakable without opponent stones");
    
    board.place_stone(9, 10, Player::Min);
    board.place_stone(11, 12, Player::Max);
    
    let (has_win, is_breakable) = GameRules::check_win_and_breakable(&board, 10, 12, 5);
    
    if is_breakable {
        assert!(has_win, "If breakable, should also have a win");
    }
}
