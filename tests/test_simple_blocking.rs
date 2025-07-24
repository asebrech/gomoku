use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move as find_best_move_parallel;
use gomoku::legacy::interface::utils::find_best_move;
use gomoku::legacy::ai::transposition::TranspositionTable;
use gomoku::ai::transposition::SharedTranspositionTable;

#[test]
fn test_simple_immediate_threat() {
    let mut state = GameState::new(19, 5);
    
    // Create a simple immediate threat - Black has 4 in a row and will win next move
    state.board.place_stone(10, 10, Player::Min); // Black stone
    state.board.place_stone(10, 11, Player::Min); // Black stone
    state.board.place_stone(10, 12, Player::Min); // Black stone
    state.board.place_stone(10, 13, Player::Min); // Black stone
    // Position (10, 14) is open - Black can win there
    // Position (10, 9) is also open - Black can win there too
    
    // Add some white pieces elsewhere
    state.board.place_stone(9, 9, Player::Max);   // White stone
    state.board.place_stone(8, 8, Player::Max);   // White stone
    
    // It's white's turn and they MUST block at (10, 14) or (10, 9)
    state.current_player = Player::Max;
    
    // Test both search methods
    let mut state_seq = state.clone();
    let mut tt = TranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state_seq, 4, None, &mut tt);
    
    let mut state_par = state.clone();
    let shared_tt = SharedTranspositionTable::new_default();
    let parallel_move = find_best_move_parallel(&mut state_par, 4, None, &shared_tt);
    
    println!("Sequential search result: {:?}", sequential_move);
    println!("Parallel search result: {:?}", parallel_move);
    
    // They should give the same result
    assert_eq!(sequential_move, parallel_move, 
              "Sequential and parallel search should give the same result!");
    
    // And both should block the immediate threat
    assert!(sequential_move.is_some());
    let (row, col) = sequential_move.unwrap();
    
    let blocking_moves = vec![(10, 9), (10, 14)];
    assert!(blocking_moves.contains(&(row, col)), 
           "AI should block immediate threat at (10,9) or (10,14) but chose ({},{})", row, col);
}
