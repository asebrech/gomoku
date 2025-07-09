mod solver {
    pub mod alpha_beta;
    pub mod alpha_beta_transposition;
    pub mod game_state;
    pub mod minimax;
    pub mod transposition;
}
mod game {
    pub mod shell_game;
    pub mod utils;
}

use game::{shell_game::new_game, utils::Algorithm};

fn main() {
    new_game(8, 4, 4, Algorithm::AlphaBetaTransposition);
}
