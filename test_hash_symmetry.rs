use gomoku::core::state::GameState;

fn main() {
    let mut state = GameState::new(19, 5);
    
    // Record initial state
    let initial_hash = state.hash();
    println!("Initial hash: {}", initial_hash);
    println!("Initial player: {:?}", state.current_player);
    
    // Make a move
    state.make_move((9, 9));
    let after_move_hash = state.hash();
    println!("After move hash: {}", after_move_hash);
    println!("After move player: {:?}", state.current_player);
    
    // Undo the move
    state.undo_move();
    let after_undo_hash = state.hash();
    println!("After undo hash: {}", after_undo_hash);
    println!("After undo player: {:?}", state.current_player);
    
    // Check symmetry
    if initial_hash == after_undo_hash {
        println!("✅ Hash symmetry is correct!");
    } else {
        println!("❌ Hash symmetry is broken!");
        println!("  Expected: {}", initial_hash);
        println!("  Actual:   {}", after_undo_hash);
    }
}
