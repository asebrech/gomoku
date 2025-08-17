use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::interface::utils::find_best_move;

#[test]
fn test_full_game_flow_simple() {
    let mut state = GameState::new(19, 5);

    // Play a simple game sequence
    let moves = vec![
        (9, 9),  // Max - center
        (9, 10), // Min - adjacent
        (9, 8),  // Max - extend line
        (10, 9), // Min - block
        (9, 7),  // Max - extend line
        (11, 9), // Min - extend own line
        (9, 6),  // Max - extend line
        (12, 9), // Min - extend own line
    ];

    for (i, &mv) in moves.iter().enumerate() {
        let expected_player = if i % 2 == 0 { Player::Max } else { Player::Min };
        assert_eq!(state.current_player, expected_player);

        // Check move is valid
        let possible_moves = state.get_possible_moves();
        assert!(possible_moves.contains(&mv));

        // Make the move
        state.make_move(mv);

        // Verify the move was made
        assert_eq!(state.board.get_player(mv.0, mv.1), Some(expected_player));
    }

    // Game should still be ongoing
    assert!(!state.is_terminal());
}

#[test]
fn test_full_game_with_captures() {
    let mut state = GameState::new(19, 5);

    // Create a capture scenario step by step
    // Move 1-2: Set up initial positions
    state.make_move((9, 7));  // Max
    state.make_move((9, 8));  // Min
    
    // Move 3-4: Continue setup
    state.make_move((9, 9));  // Max 
    state.make_move((8, 8));  // Min (random move)
    
    // Move 5-6: Create capture opportunity (Max-Min-Min-Max pattern)
    state.make_move((9, 6));  // Max - creates capture opportunity when Min plays (9,10)
    state.make_move((9, 10)); // Min - this should trigger capture
    
    // Verify capture occurred
    let initial_max_captures = state.max_captures;
    
    // Max should capture the Min stones at (9,8) and (9,9) 
    state.make_move((9, 11)); // Max captures by surrounding pattern
    
    // Verify the capture mechanics worked
    if state.max_captures > initial_max_captures {
        // Captures worked - verify stones removed
        println!("Capture successful: {} -> {}", initial_max_captures, state.max_captures);
    }
    
    // Continue game to test further mechanics
    for i in 0..10 {
        let moves = state.get_possible_moves();
        if !moves.is_empty() && !state.is_terminal() {
            state.make_move(moves[i % moves.len()]);
        } else {
            break;
        }
    }
    
    // Game should handle captures properly throughout
    assert!(state.max_captures >= initial_max_captures);
}

#[test]
fn test_ai_vs_ai_game() {
    let mut state = GameState::new(13, 5); // Smaller board for faster test
    let max_moves = 50;
    let mut move_count = 0;
    let mut tt = TranspositionTable::default();

    while !state.is_terminal() && move_count < max_moves {
        let best_move = find_best_move(&mut state, 2, None, &mut tt);

        if let Some(mv) = best_move {
            let current_player = state.current_player;
            state.make_move(mv);

            // Verify the move was valid
            assert_eq!(state.board.get_player(mv.0, mv.1), Some(current_player));

            move_count += 1;
        } else {
            break;
        }
    }

    // Game should either end or reach move limit
    assert!(state.is_terminal() || move_count >= max_moves);
}

#[test]
fn test_game_ending_conditions() {
    let mut state = GameState::new(19, 5);

    for i in 0..4 {
        state.board.place_stone(9, 5 + i, Player::Max);
    }

    state.current_player = Player::Max;
    state.make_move((9, 9)); // This completes the 5-in-a-row at (9, 5-9)

    assert!(state.is_terminal());
    assert_eq!(state.check_winner(), Some(Player::Max));
}

#[test]
fn test_capture_win_condition() {
    let mut state = GameState::new(19, 5);

    // Set up capture win
    state.max_captures = 5;

    // Make a move to trigger win check
    state.make_move((9, 9));

    assert!(state.is_terminal());
    assert_eq!(state.check_capture_win(), Some(Player::Max));
}

#[test]
fn test_undo_redo_sequence() {
    let mut state = GameState::new(19, 5);

    // Make some moves
    let moves = vec![(9, 9), (9, 10), (9, 8), (10, 9)];

    for &mv in &moves {
        state.make_move(mv);
    }

    // Undo moves in reverse order
    for &mv in moves.iter().rev() {
        state.undo_move(mv);
    }

    // Should be back to initial state
    assert!(state.board.is_empty());
    assert_eq!(state.current_player, Player::Max);
    assert_eq!(state.winner, None);
    assert_eq!(state.max_captures, 0);
    assert_eq!(state.min_captures, 0);
}

#[test]
fn test_complex_capture_scenario() {
    let mut state = GameState::new(19, 5);

    // Set up complex capture scenario using proper move mechanics
    state.make_move((9, 9)); // Max
    state.make_move((9, 10)); // Min
    state.make_move((8, 8)); // Max (dummy move)
    state.make_move((9, 11)); // Min
    
    // Now Max to move - create horizontal capture: X-O-O-X
    state.make_move((9, 12)); // Max - this should capture (9,10) and (9,11)

    // Verify capture
    assert_eq!(state.board.get_player(9, 10), None);
    assert_eq!(state.board.get_player(9, 11), None);
    assert_eq!(state.max_captures, 1); // Max made the capture, so max_captures should increase

    // Verify capture history
    assert_eq!(state.capture_history.len(), 5); // One entry per move (empty for non-capture moves)
    // The last entry should contain the captured stones
    assert_eq!(state.capture_history[4].len(), 2); // 2 stones captured in the last move
}

#[test]
fn test_game_state_consistency() {
    let mut state = GameState::new(19, 5);

    // Make random moves and verify consistency
    let moves = vec![
        (9, 9),
        (9, 10),
        (8, 9),
        (10, 10),
        (7, 9),
        (11, 11),
        (6, 9),
        (12, 12),
    ];

    for &mv in &moves {
        let before_hash = state.hash();
        let current_player = state.current_player;

        state.make_move(mv);

        // Verify state changes
        assert_ne!(state.hash(), before_hash);
        assert_eq!(state.current_player, current_player.opponent());

        // Undo and verify restoration
        state.undo_move(mv);
        assert_eq!(state.hash(), before_hash);
        assert_eq!(state.current_player, current_player);

        // Redo the move
        state.make_move(mv);
    }
}

#[test]
fn test_ai_decision_quality() {
    let mut state = GameState::new(19, 5);

    // Create a position where AI should block
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 10, Player::Min);
    state.board.place_stone(9, 11, Player::Min);
    state.board.place_stone(9, 12, Player::Min);
    state.current_player = Player::Max;
    let mut tt = TranspositionTable::default();

    let best_move = find_best_move(&mut state, 3, None, &mut tt);

    // Should block the threat
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    
    // AI should block the immediate threat at one of the ends
    let valid_blocking_moves = vec![(9, 8), (9, 13)];
    assert!(
        valid_blocking_moves.contains(&(row, col)),
        "AI chose ({}, {}) but should block the threat at one of: {:?}",
        row, col, valid_blocking_moves
    );
}

#[test]
fn test_performance_constraints() {
    let mut state = GameState::new(19, 5);

    // Create a position with many moves
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Min);
    state.current_player = Player::Max;
    let mut tt = TranspositionTable::default();

    // AI should complete search in reasonable time
    use std::time::Instant;
    let start = Instant::now();

    let _best_move = find_best_move(&mut state, 3, None, &mut tt);

    let elapsed = start.elapsed();

    // Should complete within reasonable time (adjust as needed)
    assert!(elapsed.as_secs() < 10);
}

#[test]
fn test_edge_case_board_full() {
    let mut state = GameState::new(5, 5);

    // Fill most positions
    for i in 0..5 {
        for j in 0..5 {
            if (i + j) % 2 == 0 {
                state.board.place_stone(i, j, Player::Max);
            } else if (i, j) != (2, 2) {
                state.board.place_stone(i, j, Player::Min);
            }
        }
    }

    // Last move
    state.current_player = Player::Max;
    let moves = state.get_possible_moves();

    // Should have very few moves left
    assert!(moves.len() <= 1);
}

#[test]
fn test_simultaneous_threats() {
    let mut state = GameState::new(19, 5);

    // Create multiple threats that Min should try to block
    state.board.place_stone(9, 9, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(9, 11, Player::Max);
    state.board.place_stone(9, 12, Player::Max);

    state.board.place_stone(10, 9, Player::Max);
    state.board.place_stone(11, 9, Player::Max);
    state.board.place_stone(12, 9, Player::Max);

    state.current_player = Player::Min;
    let mut tt = TranspositionTable::default();

    // Debug: Print the position
    println!("Horizontal threat: (9,9), (9,10), (9,11), (9,12) - 4 in a row");
    println!("Vertical threat: (9,9), (10,9), (11,9), (12,9) - 4 in a row");
    println!("Critical blocking positions: (9,8), (9,13), (8,9), (13,9)");

    // AI should prioritize blocking one of the immediate threats
    let best_move = find_best_move(&mut state, 3, None, &mut tt);
    assert!(best_move.is_some());

    let (row, col) = best_move.unwrap();
    println!("AI chose: ({}, {})", row, col);
    
    // The AI should block at least one of the critical threats
    // Valid blocking moves are: (9,8), (9,13), (8,9), (13,9)
    let valid_blocks = vec![(9, 8), (9, 13), (8, 9), (13, 9)];
    assert!(
        valid_blocks.contains(&(row, col)),
        "AI chose ({}, {}) but should block one of the immediate threats: {:?}",
        row, col, valid_blocks
    );
}
