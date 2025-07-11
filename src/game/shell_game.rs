use crate::{
    game::utils::find_best_move,
    solver::game_state::{GameState, Player},
};
use std::io;

pub fn print_board(state: &GameState) {
    let n = state.board.len();
    let possible_moves = state.get_possible_moves();

    // Print column headers
    print!("   ");
    for j in 0..n {
        print!("{:^3}", j);
    }
    println!();

    for i in 0..n {
        print!("{:>2} ", i); // row index
        for j in 0..n {
            match state.board[i][j] {
                Some(Player::Max) => print!(" X "),
                Some(Player::Min) => print!(" O "),
                None => {
                    let mv = (i, j);
                    if possible_moves.contains(&mv) {
                        print!(" + ");
                    } else {
                        print!(" . ");
                    }
                }
            }
        }
        println!();
    }
}

pub fn new_game(board_size: usize, winning_condition: usize, depth: i32) {
    let mut state = GameState::new(board_size, winning_condition);
    // Choose sides
    println!("Do you want to play as X (Max) or O (Min)?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_uppercase();

    let human = if input == "X" {
        Player::Max
    } else {
        Player::Min
    };
    let ai = if human == Player::Max {
        Player::Min
    } else {
        Player::Max
    };

    loop {
        print_board(&state);
        if state.is_terminal() {
            match state.check_winner() {
                Some(p) if p == human => println!("🎉 You win!"),
                Some(p) if p == ai => println!("💀 AI wins!"),
                _ => println!("🤝 It's a draw."),
            }
            break;
        }

        if state.current_player == human {
            // Human move
            loop {
                println!("Your move (row and col, e.g. `7 7`): ");
                let mut move_input = String::new();
                io::stdin().read_line(&mut move_input).unwrap();

                let parts: Vec<_> = move_input
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();

                if parts.len() == 2 {
                    let mv = (parts[0], parts[1]);

                    if mv.0 >= state.board.len() || mv.1 >= state.board.len() {
                        println!(
                            "❌ Move out of bounds. Please enter numbers between 0 and {}.",
                            state.board.len() - 1
                        );
                        continue;
                    }

                    let possible_moves = state.get_possible_moves();
                    if possible_moves.contains(&mv) {
                        state.make_move(mv);
                        break; // Move accepted
                    } else if state.board[mv.0][mv.1].is_some() {
                        println!("❌ That cell is already occupied.");
                    } else if state.is_board_empty() {
                        println!("❌ First move must be at the center ({}, {}).", state.board.len() / 2, state.board.len() / 2);
                    } else if !state.is_move_adjacent(mv) {
                        println!("❌ Move must be adjacent to an existing piece.");
                    } else if state.creates_double_three(mv.0, mv.1, state.current_player) {
                        println!("❌ This move would create a double-three, which is forbidden.");
                    } else {
                        println!("❌ Invalid move. This move is not allowed by the game rules.");
                    }
                } else {
                    println!("❌ Invalid input format. Type two numbers like `7 7`.");
                }
            }
        } else {
            println!("🤖 AI is thinking...");
            if let Some(mv) = find_best_move(&mut state, depth) {
                println!("AI chooses: {:?}", mv);
                state.make_move(mv);
            } else {
                println!("AI has no valid moves.");
                break;
            }
        }
    }
}
