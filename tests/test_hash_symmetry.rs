#[cfg(test)]
mod tests {
    use gomoku::core::state::GameState;

    #[test]
    fn test_hash_symmetry() {
        let mut state = GameState::new(19, 5);
        
        // Record initial state
        let initial_hash = state.hash();
        println!("Initial hash: {}", initial_hash);
        println!("Initial player: {:?}", state.current_player);
        
        // Make a move
        let move_pos = (9, 9);
        state.make_move(move_pos);
        let after_move_hash = state.hash();
        println!("After move hash: {}", after_move_hash);
        println!("After move player: {:?}", state.current_player);
        
        // Undo the move
        state.undo_move(move_pos);
        let after_undo_hash = state.hash();
        println!("After undo hash: {}", after_undo_hash);
        println!("After undo player: {:?}", state.current_player);
        
        // Check symmetry
        assert_eq!(initial_hash, after_undo_hash, "Hash symmetry is broken!");
        assert_eq!(state.current_player, gomoku::core::board::Player::Max, "Player not restored!");
    }
}
