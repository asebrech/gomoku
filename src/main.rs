mod solver {
    pub mod alpha_beta;
    pub mod game_state;
    pub mod minimax;
}
mod game {
    pub mod shell_game;
    pub mod utils;
}

use game::{shell_game::new_game, utils::Algorithm};

fn main() {
    new_game(8, 4, 3, Algorithm::AlphaBeta);
}
