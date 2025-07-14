use gomoku::core::board::{Player, initialize_zobrist};
use gomoku::core::state::GameState;
use gomoku::interface::utils::find_best_move;

#[test]
fn test_ai_blocks_immediate_win() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a scenario where the human player (Min) is about to win
    // We'll create a situation where Min has 4 in a row and needs to be blocked
    state.make_move((9, 9));   // Max
    state.make_move((10, 9));  // Min
    state.make_move((8, 8));   // Max
    state.make_move((10, 10)); // Min
    state.make_move((7, 7));   // Max
    state.make_move((10, 11)); // Min
    state.make_move((6, 6));   // Max
    state.make_move((10, 12)); // Min - now Min has 4 in a row horizontally
    
    // Now it's Max's turn (AI) and it should block at (10, 13) or (10, 8)
    println!("Current board state:");
    print_board(&state);
    
    let best_move = find_best_move(&mut state, 4);
    assert!(best_move.is_some(), "AI should find a blocking move");
    
    let (row, col) = best_move.unwrap();
    println!("AI chose move: ({}, {})", row, col);
    
    // The AI should block at either end of the 4-in-a-row
    let blocking_moves = vec![(10, 8), (10, 13)];
    assert!(blocking_moves.contains(&(row, col)), 
            "AI should block at ({}, {}) or ({}, {}) but chose ({}, {})", 
            10, 8, 10, 13, row, col);
}

#[test]
fn test_ai_blocks_diagonal_win() {
    initialize_zobrist();
    let mut state = GameState::new(19, 5);
    
    // Create a diagonal threat
    state.make_move((9, 9));   // Max
    state.make_move((10, 10)); // Min
    state.make_move((8, 8));   // Max
    state.make_move((11, 11)); // Min
    state.make_move((7, 7));   // Max
    state.make_move((12, 12)); // Min
    state.make_move((6, 6));   // Max
    state.make_move((13, 13)); // Min - now Min has 4 in a diagonal
    
    println!("Current board state (diagonal threat):");
    print_board(&state);
    
    let best_move = find_best_move(&mut state, 4);
    assert!(best_move.is_some(), "AI should find a blocking move");
    
    let (row, col) = best_move.unwrap();
    println!("AI chose move: ({}, {})", row, col);
    
    // The AI should block at either end of the diagonal
    let blocking_moves = vec![(14, 14), (9, 9)]; // Note: (9,9) is already occupied by Max
    // So it should block at (14, 14)
    assert!(blocking_moves.contains(&(row, col)), 
            "AI should block at (14, 14) or (9, 9) but chose ({}, {})", row, col);
}

fn print_board(state: &GameState) {
    println!("   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8");
    for row in 0..19 {
        print!("{:2} ", row);
        for col in 0..19 {
            match state.board.get_player(row, col) {
                Some(Player::Max) => print!("X "),
                Some(Player::Min) => print!("O "),
                None => print!(". "),
            }
        }
        println!();
    }
    println!();
}
