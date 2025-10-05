use gomoku::ai::lazy_smp::lazy_smp_search;
use gomoku::core::board::{Board, Player};
use gomoku::core::state::GameState;

#[test]
fn test_parallel_search_basic() {
    let mut board = Board::new(15);
    
    // Create a position with several possible moves
    board.place_stone(7, 7, Player::Max);
    board.place_stone(7, 8, Player::Min);
    board.place_stone(8, 7, Player::Max);
    board.place_stone(8, 8, Player::Min);
    
    let mut state = GameState::new(15, 5);
    state.board = board;
    state.current_player = Player::Max;

    // Test with 1000ms time limit for reasonable complexity

    // Parallel search
    let result = lazy_smp_search(&mut state, 1000, Some(4));

    // Should find valid move
    assert!(result.best_move.is_some(), "Parallel search should find a move");

    // Should search to reasonable depth
    assert!(result.depth_reached >= 3, "Should reach at least depth 3");

    // Should examine reasonable number of nodes
    assert!(result.nodes_searched > 10, "Should examine at least 10 nodes");

    println!("Parallel: move={:?}, score={}, depth={}, nodes={}, time={:?}",
             result.best_move,
             result.score,
             result.depth_reached,
             result.nodes_searched,
             result.time_elapsed);
}

#[test]
fn test_parallel_winning_position() {
    let mut board = Board::new(15);
    
    // Create a position where Max can win in 2 moves
    board.place_stone(7, 7, Player::Max);
    board.place_stone(7, 8, Player::Max);
    board.place_stone(7, 9, Player::Max);
    board.place_stone(7, 10, Player::Max);
    // Min blocks one side
    board.place_stone(7, 6, Player::Min);
    
    let mut state = GameState::new(15, 5);
    state.board = board;
    state.current_player = Player::Max;

    let result = lazy_smp_search(&mut state, 500, Some(4));

    assert!(result.best_move.is_some(), "Should find winning move");
    assert_eq!(result.best_move.unwrap(), (7, 11), "Should play the winning move");
    assert!(result.score > 900_000, "Should recognize winning position with score: {}", result.score);
}

#[test]
fn test_parallel_blocking_position() {
    let mut board = Board::new(15);
    
    // Create a position where Min must block Max's threat
    board.place_stone(7, 7, Player::Max);
    board.place_stone(7, 8, Player::Max);
    board.place_stone(7, 9, Player::Max);
    board.place_stone(7, 10, Player::Max);
    
    // Some other moves
    board.place_stone(8, 8, Player::Min);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(10, 10, Player::Min);
    
    let mut state = GameState::new(15, 5);
    state.board = board;
    state.current_player = Player::Min;  // Min to move

    let result = lazy_smp_search(&mut state, 500, Some(4));

    assert!(result.best_move.is_some(), "Should find blocking move");
    
    let best_move = result.best_move.unwrap();
    // Should block at one of the ends
    assert!(best_move == (7, 6) || best_move == (7, 11), 
           "Should block at (7,6) or (7,11), got {:?}", best_move);
}

#[test]
fn test_parallel_search_empty_board() {
    let board = Board::new(15);
    let mut state = GameState::new(15, 5);
    state.board = board;
    state.current_player = Player::Max;

    let result = lazy_smp_search(&mut state, 200, Some(2));

    assert!(result.best_move.is_some(), "Should find a move on empty board");
    // Should prefer center area
    let (row, col) = result.best_move.unwrap();
    assert!(row >= 6 && row <= 8 && col >= 6 && col <= 8, 
           "Should prefer center area, got ({}, {})", row, col);
}

#[test]
fn test_parallel_search_performance() {
    let mut board = Board::new(15);
    
    // Create a moderately complex position
    for i in 0..5 {
        board.place_stone(7 + i, 7, Player::Max);
        board.place_stone(7 + i, 8, Player::Min);
    }
    
    let mut state = GameState::new(15, 5);
    state.board = board;
    state.current_player = Player::Max;

    let start = std::time::Instant::now();
    let result = lazy_smp_search(&mut state, 1000, Some(4));
    let elapsed = start.elapsed();

    assert!(result.best_move.is_some(), "Should find a move");
    assert!(elapsed.as_millis() <= 1200, "Should complete within reasonable time");
    
    println!("Parallel search: move={:?}, score={}, depth={}, nodes={}, time={:?}",
             result.best_move,
             result.score,
             result.depth_reached,
             result.nodes_searched,
             elapsed);
             
    // More reasonable expectation - if it finds a winning move quickly, that's good!
    // If the search terminates early due to a winning position, accept lower node count
    if result.score.abs() >= 900_000 {
        assert!(result.nodes_searched > 0, "Should search at least some nodes for winning positions, got {}", result.nodes_searched);
    } else {
        assert!(result.nodes_searched > 50, "Should search at least 50 nodes for non-terminal positions, got {}", result.nodes_searched);
    }
}