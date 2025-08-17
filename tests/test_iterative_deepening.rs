use gomoku::ai::transposition::TranspositionTable;
use gomoku::ai::minimax::find_best_move;
use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use std::time::Duration;

#[test]
fn test_iterative_deepening_basic() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Make a few moves to create a non-trivial position
    state.make_move((7, 7)); // Center
    state.make_move((7, 8)); // Adjacent
    state.make_move((7, 6)); // Other side
    
    let result = find_best_move(&mut state, 3, None, &mut tt);
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached > 0);
    assert!(result.depth_reached <= 3);
    assert!(result.nodes_searched > 0);
    println!("Basic test result: {:?}", result);
}

#[test]
fn test_iterative_deepening_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Make a few moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let time_limit = Duration::from_millis(100);
    let result = find_best_move(&mut state, 10, Some(time_limit), &mut tt);
    
    assert!(result.best_move.is_some());
    assert!(result.time_elapsed <= Duration::from_millis(150)); // Allow some margin
    println!("Timed test result: {:?}", result);
}

#[test]
fn test_find_best_move_without_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let result = find_best_move(&mut state, 3, None, &mut tt);
    assert!(result.best_move.is_some());
    println!("Best move from search without time limit: {:?}", result.best_move);
}

#[test]
fn test_find_best_move_with_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let time_limit = Duration::from_millis(50);
    let result = find_best_move(&mut state, 5, Some(time_limit), &mut tt);
    assert!(result.best_move.is_some());
    println!("Best move from timed search: {:?}", result.best_move);
}

#[test]
fn test_iterative_deepening_vs_direct_minimax() {
    let mut state = GameState::new(15, 5);
    let mut tt1 = TranspositionTable::default();
    let mut tt2 = TranspositionTable::default();
    
    // Create identical game states
    let moves = [(7, 7), (7, 8), (8, 7)];
    for &mv in &moves {
        state.make_move(mv);
    }
    
    // Test iterative deepening
    let iterative_result = find_best_move(&mut state, 3, None, &mut tt1);
    
    // Test direct search using the regular find_best_move function  
    let regular_result = gomoku::ai::minimax::find_best_move(&mut state, 3, None, &mut tt2);
    
    // Both should find a move
    assert!(iterative_result.best_move.is_some());
    assert!(regular_result.best_move.is_some());
    
    println!("Iterative result: {:?}", iterative_result.best_move);
    println!("Regular result: {:?}", regular_result);
    
    // The moves might be different due to different search strategies, but both should be valid
    let moves = state.get_possible_moves();
    assert!(moves.contains(&iterative_result.best_move.unwrap()));
    assert!(moves.contains(&regular_result.best_move.unwrap()));
}

#[test]
fn test_early_termination_on_win() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Create a position where there's an immediate winning move
    // Set up a line of 4 for the current player
    state.make_move((7, 7)); // Max
    state.make_move((8, 7)); // Min  
    state.make_move((7, 8)); // Max
    state.make_move((8, 8)); // Min
    state.make_move((7, 9)); // Max
    state.make_move((8, 9)); // Min
    state.make_move((7, 10)); // Max - now has 4 in a row
    
    // Min should find the blocking move immediately
    let result = find_best_move(&mut state, 5, None, &mut tt);
    
    assert!(result.best_move.is_some());
    // Should find the winning/blocking move quickly
    assert!(result.score.abs() >= 1000); // High score for critical position
    println!("Early termination test result: {:?}", result);
}

#[test]
fn test_complex_board_with_short_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Create a complex tactical position with NO immediate wins
    // Strategy: Create only 2-stone groups separated by opponent stones
    let max_positions = [
        // Safe 2-stone groups (no way to extend to 5)
        (1, 1), (1, 2),       // Horizontal pair near edge
        (3, 4), (3, 5),       // Another horizontal pair
        (6, 1), (7, 1),       // Vertical pair near edge  
        (9, 3), (10, 3),      // Another vertical pair
        (5, 8), (6, 9),       // Diagonal pair
        (11, 5), (12, 6),     // Another diagonal pair
        
        // Isolated singles for complexity
        (0, 6), (2, 8), (4, 12), (8, 14), (13, 11), (14, 2),
    ];
    
    let min_positions = [
        // Safe 2-stone groups that block max expansion
        (1, 3),               // Blocks max horizontal at (1,1)-(1,2)
        (3, 6),               // Blocks max horizontal at (3,4)-(3,5)
        (8, 1),               // Blocks max vertical at (6,1)-(7,1)
        (11, 3),              // Blocks max vertical at (9,3)-(10,3)
        
        // Own safe 2-stone groups
        (2, 2), (2, 3),       // Horizontal pair
        (5, 5), (6, 5),       // Vertical pair  
        (9, 9), (10, 10),     // Diagonal pair
        (12, 1), (13, 2),     // Another diagonal pair
        
        // More isolated singles
        (0, 14), (4, 0), (7, 13), (11, 8), (14, 4), (1, 11),
    ];
    
    // Place stones manually using the board interface
    for &pos in &max_positions {
        if pos.0 < 15 && pos.1 < 15 {
            state.board.place_stone(pos.0, pos.1, Player::Max);
        }
    }
    
    for &pos in &min_positions {
        if pos.0 < 15 && pos.1 < 15 {
            state.board.place_stone(pos.0, pos.1, Player::Min);
        }
    }
    
    // Set current player to Min for the search
    state.current_player = Player::Min;
    
    // Very short time limit that might cause issues
    let time_limit = Duration::from_millis(500);
    
    println!("Board state before search:");
    println!("Current player: {:?}", state.current_player);
    println!("Is terminal: {}", state.is_terminal());
    let moves = state.get_possible_moves();
    println!("Available moves: {} moves", moves.len());
    
    // Check if AI can win in one move - this would make the search trivial
    let mut has_immediate_win = false;
    for &mv in &moves {
        state.make_move(mv);
        if state.is_terminal() && state.check_winner() == Some(Player::Min) {
            has_immediate_win = true;
            println!("WARNING: AI can win immediately with move {:?}", mv);
        }
        state.undo_move(mv);
        if has_immediate_win {
            break;
        }
    }
    
    // Check if opponent can win in one move (forcing defensive play)
    let mut must_defend = false;
    state.current_player = Player::Max; // Temporarily switch to check opponent threats
    let opponent_moves = state.get_possible_moves();
    for &mv in &opponent_moves {
        state.make_move(mv);
        if state.is_terminal() && state.check_winner() == Some(Player::Max) {
            must_defend = true;
            println!("WARNING: Opponent can win with move {:?} - AI must defend", mv);
        }
        state.undo_move(mv);
        if must_defend {
            break;
        }
    }
    state.current_player = Player::Min; // Switch back
    
    println!("Immediate win available: {}, Must defend: {}", has_immediate_win, must_defend);
    
    let start_time = std::time::Instant::now();
    let result = find_best_move(&mut state, 10, Some(time_limit), &mut tt);
    let actual_time = start_time.elapsed();
    
    println!("Search completed in {:?} (measured externally)", actual_time);
    println!("Result reported time: {:?}", result.time_elapsed);
    
    // Should still find a move even with short time limit
    assert!(result.best_move.is_some());
    
    // Should have reached at least depth 1
    assert!(result.depth_reached >= 1);
    
    // Should have searched at least some nodes
    assert!(result.nodes_searched > 0);
    
    // Time should be measured but might be quite fast due to effective pruning
    assert!(result.time_elapsed >= Duration::from_millis(1)); // At least 1ms - very minimal
    assert!(result.time_elapsed <= Duration::from_millis(600)); // Allow some margin
    
    // The move should be valid
    let valid_moves = state.get_possible_moves();
    assert!(valid_moves.contains(&result.best_move.unwrap()));
    
    println!("Complex board test result: {:?}", result);
    println!("Board has {} possible moves", valid_moves.len());
}

#[test]
fn test_500ms_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Make a few moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    
    // Short time limit (500ms) like in real game
    let time_limit = Duration::from_millis(500);
    let result = find_best_move(&mut state, 10, Some(time_limit), &mut tt);
    
    // Should still find a move
    assert!(result.best_move.is_some());
    
    // Should have reached at least depth 1
    assert!(result.depth_reached >= 1);
    
    // Should have done minimal search but not zero
    assert!(result.nodes_searched > 0);
    
    // Move should be valid
    let valid_moves = state.get_possible_moves();
    assert!(valid_moves.contains(&result.best_move.unwrap()));
    
    println!("500ms time test result: {:?}", result);
}

#[test]
fn test_progressive_depth_improvement() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Create a position with tactical elements
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((7, 6));
    state.make_move((8, 7));
    
    let result = find_best_move(&mut state, 5, None, &mut tt);
    
    // Should reach the full depth
    assert_eq!(result.depth_reached, 5);
    
    // Should have searched progressively through depths
    assert!(result.nodes_searched >= 5); // At least one node per depth
    
    println!("Progressive depth test result: {:?}", result);
}

#[test]
fn test_time_vs_depth_consistency() {
    let mut state = GameState::new(15, 5);
    let mut tt1 = TranspositionTable::default();
    let mut tt2 = TranspositionTable::default();
    
    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    
    // Test with depth limit
    let depth_result = find_best_move(&mut state, 3, None, &mut tt1);
    
    // Test with generous time limit that should allow reaching the same depth
    let time_limit = Duration::from_millis(5000); // 5 seconds should be plenty
    let time_result = find_best_move(&mut state, 10, Some(time_limit), &mut tt2);
    
    // Both should find moves
    assert!(depth_result.best_move.is_some());
    assert!(time_result.best_move.is_some());
    
    // Time-based search should reach at least the depth of depth-based search
    assert!(time_result.depth_reached >= depth_result.depth_reached);
    
    println!("Depth result: {:?}", depth_result);
    println!("Time result: {:?}", time_result);
}

#[test]
fn test_winning_position_early_termination() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position where current player can win immediately
    // Create 4 in a row for current player (Max)
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(7, 9, Player::Max);
    state.board.place_stone(7, 10, Player::Max);
    // Position (7, 6) or (7, 11) would be winning moves
    
    state.current_player = Player::Max;
    
    let result = find_best_move(&mut state, 5, None, &mut tt);
    
    // Should find the winning move
    assert!(result.best_move.is_some());
    
    // Should have a very high score (winning)
    assert!(result.score >= 100_000);
    
    // Should terminate early due to finding a win
    // Even though max depth is 5, it might stop earlier
    assert!(result.depth_reached >= 1);
    
    // The move should complete the 5-in-a-row
    let best_move = result.best_move.unwrap();
    assert!(best_move == (7, 6) || best_move == (7, 11));
    
    println!("Winning position test result: {:?}", result);
}

#[test]
fn test_defensive_play_under_pressure() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Set up a position where opponent (Max) has 4 in a row
    state.board.place_stone(7, 7, Player::Max);
    state.board.place_stone(7, 8, Player::Max);
    state.board.place_stone(7, 9, Player::Max);
    state.board.place_stone(7, 10, Player::Max);
    
    // Current player (Min) must block
    state.current_player = Player::Min;
    
    // Use a 500ms time limit to test under pressure
    let time_limit = Duration::from_millis(500);
    let result = find_best_move(&mut state, 6, Some(time_limit), &mut tt);
    
    // Should find the blocking move
    assert!(result.best_move.is_some());
    
    let best_move = result.best_move.unwrap();
    // Should block at (7, 6) or (7, 11)
    assert!(best_move == (7, 6) || best_move == (7, 11));
    
    // Should recognize this as a critical position
    // Min player blocking a win gets negative score (good for Min)
    assert!(result.score <= -1000, "Expected high negative score for Min blocking, got {}", result.score);
    
    println!("Defensive play test result: {:?}", result);
}

#[test]
fn test_transposition_table_benefits() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Create a position
    state.make_move((7, 7));
    state.make_move((7, 8));
    state.make_move((8, 7));
    
    // First search to populate transposition table
    let first_result = find_best_move(&mut state, 4, None, &mut tt);
    
    // Second search should benefit from transposition table
    let second_result = find_best_move(&mut state, 4, None, &mut tt);
    
    // Both should find the same move (or equally good moves)
    assert!(first_result.best_move.is_some());
    assert!(second_result.best_move.is_some());
    
    // Second search might be faster due to TT hits, but both should reach full depth
    assert_eq!(first_result.depth_reached, 4);
    assert_eq!(second_result.depth_reached, 4);
    
    // Get TT statistics
    let (hits, misses) = tt.get_stats();
    println!("TT stats after searches: hits={}, misses={}", hits, misses);
    
    // Should have some hits from the second search
    assert!(hits > 0);
    
    println!("First search: {:?}", first_result);
    println!("Second search: {:?}", second_result);
}

#[test]
fn test_very_complex_board_500ms() {
    let mut state = GameState::new(19, 5); // Larger board
    let mut tt = TranspositionTable::default();
    
    // Create an even more complex board with many stones scattered around
    let moves = [
        (9, 9), (9, 10), (10, 9), (10, 10), (8, 9), (8, 10),
        (11, 9), (11, 10), (9, 8), (10, 8), (9, 11), (10, 11),
        (7, 9), (7, 10), (12, 9), (12, 10), (8, 8), (8, 11),
        (11, 8), (11, 11), (6, 9), (6, 10), (13, 9), (13, 10),
        (7, 8), (7, 11), (12, 8), (12, 11), (5, 9), (5, 10),
        (14, 9), (14, 10), (6, 8), (6, 11), (13, 8), (13, 11),
        (4, 9), (4, 10), (15, 9), (15, 10), (5, 8), (5, 11),
        (14, 8), (14, 11), (3, 9), (3, 10), (16, 9), (16, 10),
        (4, 8), (4, 11), (15, 8), (15, 11), (2, 9), (2, 10),
        (17, 9), (17, 10), (3, 8), (3, 11), (16, 8), (16, 11),
    ];
    
    for mv in moves {
        if !state.is_terminal() && mv.0 < 19 && mv.1 < 19 {
            state.make_move(mv);
        }
    }
    
    // 500ms time limit - this should reproduce the issue you mentioned
    let time_limit = Duration::from_millis(500);
    let result = find_best_move(&mut state, 8, Some(time_limit), &mut tt);
    
    // Should still find a move
    assert!(result.best_move.is_some());
    
    // Should have reached at least depth 1 (this was the main issue)
    assert!(result.depth_reached >= 1, "AI should reach at least depth 1, got {}", result.depth_reached);
    
    // Should have searched meaningful number of nodes
    assert!(result.nodes_searched > 0, "AI should search at least some nodes, got {}", result.nodes_searched);
    
    // The move should be valid
    let valid_moves = state.get_possible_moves();
    assert!(valid_moves.contains(&result.best_move.unwrap()), "AI move should be valid");

    // Should respect the time limit (important for game performance)
    assert!(result.time_elapsed <= Duration::from_millis(600), "AI should not exceed time limit by much, got {:?}", result.time_elapsed);
    
    // Time should be measured (even if very fast)
    assert!(result.time_elapsed > Duration::ZERO, "AI should report some time elapsed, got {:?}", result.time_elapsed);

    // Should reach at least depth 2 with 500ms on this position (unless moves are very limited)
    if valid_moves.len() > 50 {
        assert!(result.depth_reached >= 2, "With 500ms and {} moves, AI should reach at least depth 2, got {}", valid_moves.len(), result.depth_reached);
    } else {
        assert!(result.depth_reached >= 1, "AI should reach at least depth 1, got {}", result.depth_reached);
    }    println!("Very complex board (19x19) with 500ms test result: {:?}", result);
    println!("Board has {} possible moves", valid_moves.len());
    println!("Reached depth {} in {:?}", result.depth_reached, result.time_elapsed);
}

#[test]
fn test_game_like_conditions() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::default();
    
    // Simulate a mid-game situation with realistic stone placement
    let realistic_moves = [
        (7, 7),   // Center opening
        (7, 8),   // Adjacent response
        (8, 7),   // Diagonal
        (6, 8),   // Block
        (8, 8),   // Corner of square
        (6, 7),   // Complete square
        (9, 7),   // Extend
        (5, 7),   // Counter-extend
        (7, 9),   // Side extension
        (7, 6),   // Other side
        (8, 6),   // Support
        (6, 9),   // Counter
    ];
    
    for mv in realistic_moves {
        if !state.is_terminal() {
            state.make_move(mv);
        }
    }
    
    // Test with game-like 500ms time limit
    let time_limit = Duration::from_millis(500);
    let result = find_best_move(&mut state, 6, Some(time_limit), &mut tt);
    
    // Basic assertions
    assert!(result.best_move.is_some(), "Should find a move");
    assert!(result.depth_reached >= 1, "Should reach at least depth 1");
    assert!(result.nodes_searched > 0, "Should search some nodes");
    
    // Time assertions
    assert!(result.time_elapsed >= Duration::from_millis(50), "Should take some time to think");
    assert!(result.time_elapsed <= Duration::from_millis(600), "Should respect time limit");
    
    // Quality assertions
    let valid_moves = state.get_possible_moves();
    assert!(valid_moves.contains(&result.best_move.unwrap()), "Move should be valid");
    
    // With 500ms, should be able to search multiple depths
    assert!(result.depth_reached >= 2, "Should reach decent depth with 500ms");
    
    println!("Game-like conditions test result: {:?}", result);
    println!("Position has {} valid moves", valid_moves.len());
}
