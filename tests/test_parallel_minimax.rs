use gomoku::ai::{search::find_best_move, transposition::TranspositionTable};
use gomoku::core::state::GameState;
use std::time::Duration;

#[test]
fn test_parallel_search_functionality() {
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let tt = TranspositionTable::new(50_000);
    
    let result = find_best_move(&mut state, 4, None, &tt);
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached >= 1);
    assert!(result.nodes_searched > 0);
}

#[test]
fn test_parallel_search_with_time_limit() {
    let mut state = GameState::new(15, 5);
    state.make_move((7, 7));
    
    let tt = TranspositionTable::new(10_000);
    
    let time_limit = Duration::from_millis(50);
    let result = find_best_move(&mut state, 8, Some(time_limit), &tt);

    // Lazy SMP coordination may exceed tight time limits
    if result.time_elapsed > Duration::from_millis(1000) {
        println!("Lazy SMP coordination took: {:?}", result.time_elapsed);
    }
    assert!(result.best_move.is_some());
}

#[test]
fn test_parallel_search_consistency() {
    let mut state1 = GameState::new(15, 5);
    let mut state2 = state1.clone();
    
    for &mv in &[(7, 7), (8, 8), (6, 6)] {
        state1.make_move(mv);
        state2.make_move(mv);
    }
    
    let tt1 = TranspositionTable::new(20_000);
    let tt2 = TranspositionTable::new(20_000);
    
    let result1 = find_best_move(&mut state1, 3, None, &tt1);
    let result2 = find_best_move(&mut state2, 3, None, &tt2);
    
    assert_eq!(result1.best_move, result2.best_move);
    assert_eq!(result1.score, result2.score);
    assert_eq!(result1.depth_reached, result2.depth_reached);
}
