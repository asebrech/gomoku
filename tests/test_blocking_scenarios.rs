use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::{find_best_move, find_best_move_parallel};
use gomoku::ai::transposition::{TranspositionTable, SharedTranspositionTable};

#[test]
fn test_simple_blocking_scenario() {
    let mut state = GameState::new(19, 5);
    
    // Create a simpler scenario - Black has 4 in a row with only ONE way to win
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 10, Player::Min);
    // Black can win at (9, 6) or (9, 11) but let's block one end
    state.board.place_stone(9, 6, Player::Max); // Block one end
    
    // Now Black can ONLY win at (9, 11)
    state.current_player = Player::Max;
    
    let mut tt = TranspositionTable::new_default();
    let shared_tt = SharedTranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state.clone(), 6, None, &mut tt);
    let parallel_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("Simple blocking - Sequential: {:?}", sequential_move);
    println!("Simple blocking - Parallel: {:?}", parallel_move);
    
    // Both should block at (9, 11)
    if let Some((row, col)) = sequential_move {
        assert_eq!((row, col), (9, 11), "Sequential should block at (9, 11)");
    }
    
    if let Some((row, col)) = parallel_move {
        assert_eq!((row, col), (9, 11), "Parallel should block at (9, 11)");
    }
}

#[test]
fn test_double_threat_recognition() {
    let mut state = GameState::new(19, 5);
    
    // The original double threat scenario
    state.board.place_stone(8, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(10, 9, Player::Min);
    state.board.place_stone(11, 10, Player::Min);
    state.current_player = Player::Max;
    
    let mut tt = TranspositionTable::new_default();
    let shared_tt = SharedTranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state.clone(), 8, None, &mut tt); // Deeper search
    let parallel_move = find_best_move_parallel(&mut state, 8, None, &shared_tt);
    
    println!("Double threat (depth 8) - Sequential: {:?}", sequential_move);
    println!("Double threat (depth 8) - Parallel: {:?}", parallel_move);
    
    // In a double threat, the AI might recognize it's hopeless and play a different strategy
    // But both sequential and parallel should behave the same way
    
    // Let's check if they make the same decision
    if sequential_move.is_some() && parallel_move.is_some() {
        let seq_move = sequential_move.unwrap();
        let par_move = parallel_move.unwrap();
        
        if seq_move != par_move {
            println!("🚨 INCONSISTENCY: Sequential chose {:?}, Parallel chose {:?}", seq_move, par_move);
            
            // This would indicate a problem with the parallel implementation
            panic!("Sequential and parallel searches gave different results for the same position!");
        }
    }
}

#[test] 
fn test_forced_block_scenario() {
    let mut state = GameState::new(19, 5);
    
    // Create a scenario where there's only one correct blocking move
    state.board.place_stone(9, 7, Player::Min);
    state.board.place_stone(9, 8, Player::Min);
    state.board.place_stone(9, 9, Player::Min);
    state.board.place_stone(9, 10, Player::Min);
    
    // Block one end so there's only one threat
    state.board.place_stone(9, 11, Player::Max);
    
    // Now only (9, 6) can complete the line for Black
    state.current_player = Player::Max;
    
    let mut tt = TranspositionTable::new_default();
    let shared_tt = SharedTranspositionTable::new_default();
    let sequential_move = find_best_move(&mut state.clone(), 6, None, &mut tt);
    let parallel_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    
    println!("Forced block - Sequential: {:?}", sequential_move);
    println!("Forced block - Parallel: {:?}", parallel_move);
    
    // Both must block at (9, 6)
    assert!(sequential_move.is_some());
    assert!(parallel_move.is_some());
    
    let seq_move = sequential_move.unwrap();
    let par_move = parallel_move.unwrap();
    
    assert_eq!(seq_move, (9, 6), "Sequential must block the threat");
    assert_eq!(par_move, (9, 6), "Parallel must block the threat");
    assert_eq!(seq_move, par_move, "Sequential and parallel must agree on forced moves");
}
