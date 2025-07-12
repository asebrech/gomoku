use crate::{core::board::Player, core::state::GameState, interface::utils::find_best_move};
use std::io;

pub fn new_game(board_size: usize, winning_condition: usize, depth: i32) {
    let mut state = GameState::new(board_size, winning_condition);
    let (human, opponent, is_human_opponent) = choose_sides();

    loop {
        print_board(&state);

        if state.is_terminal() {
            if is_human_opponent {
                print_game_result(&state, human, opponent.unwrap());
            } else {
                print_game_result(&state, human, Player::Min);
            }
            break;
        }

        if state.current_player == human {
            handle_human_move(&mut state);
        } else if is_human_opponent {
            handle_human_move(&mut state); // Alternate human player
        } else {
            handle_ai_move(&mut state, depth);
        }
    }
}

pub fn choose_sides() -> (Player, Option<Player>, bool) {
    println!("Do you want to play against another human? (y/n)");
    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input).unwrap();
    let mode_input = mode_input.trim().to_lowercase();
    let is_human_opponent = mode_input == "y";

    println!("Do you want to play as X (Max) or O (Min)?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_uppercase();

    let human = if input == "X" {
        Player::Max
    } else {
        Player::Min
    };

    let opponent = if is_human_opponent {
        Some(if human == Player::Max {
            Player::Min
        } else {
            Player::Max
        })
    } else {
        None
    };

    (human, opponent, is_human_opponent)
}

pub fn print_board(state: &GameState) {
    let n = state.board.size;
    let possible_moves = state.get_possible_moves();

    println!(
        "Captures: X = {} pairs, O = {} pairs",
        state.max_captures, state.min_captures
    );
    println!();

    print!("   ");
    for j in 0..n {
        print!("{:^3}", j);
    }
    println!();

    for i in 0..n {
        print!("{:>2} ", i);
        for j in 0..n {
            match state.board.get_player(i, j) {
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

fn print_game_result(state: &GameState, human: Player, ai: Player) {
    match state.check_winner() {
        Some(p) if p == human => println!("🎉 You win!"),
        Some(p) if p == ai => println!("💀 AI wins!"),
        _ => println!("🤝 It's a draw."),
    }
}

fn handle_human_move(state: &mut GameState) {
    loop {
        println!("Your move (row and col, e.g. `7 7`): ");
        let mut move_input = String::new();
        io::stdin().read_line(&mut move_input).unwrap();

        let parts: Vec<_> = move_input
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .collect();

        if parts.len() == 2 {
            let mv = (parts[0], parts[1]);

            if let Some(error) = validate_human_move(state, mv) {
                println!("❌ {}", error);
                continue;
            }

            state.make_move_with_actions(mv);
            break;
        } else {
            println!("❌ Invalid input format. Type two numbers like `7 7`.");
        }
    }
}

fn validate_human_move(state: &GameState, mv: (usize, usize)) -> Option<String> {
    if mv.0 >= state.board.size || mv.1 >= state.board.size {
        return Some(format!(
            "Move out of bounds. Please enter numbers between 0 and {}.",
            state.board.size - 1
        ));
    }

    let possible_moves = state.get_possible_moves();
    if possible_moves.contains(&mv) {
        return None;
    }

    if state.board.get_player(mv.0, mv.1).is_some() {
        return Some("That cell is already occupied.".to_string());
    }

    if state.board.is_empty() {
        return Some(format!(
            "First move must be at the center ({}, {}).",
            state.board.size / 2,
            state.board.size / 2
        ));
    }

    if !state.board.is_adjacent_to_stone(mv.0, mv.1) {
        return Some("Move must be adjacent to an existing piece.".to_string());
    }

    if crate::core::moves::RuleValidator::creates_double_three(
        &state.board,
        mv.0,
        mv.1,
        state.current_player,
    ) {
        return Some("This move would create a double-three, which is forbidden.".to_string());
    }

    Some("Invalid move. This move is not allowed by the game rules.".to_string())
}

fn handle_ai_move(state: &mut GameState, depth: i32) {
    println!("🤖 AI is thinking...");
    if let Some(mv) = find_best_move(state, depth) {
        println!("AI chooses: {:?}", mv);
        state.make_move_with_actions(mv);
    } else {
        println!("AI has no valid moves.");
    }
}
