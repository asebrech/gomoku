pub mod ai {
    pub mod heuristic;
    pub mod minimax;
    pub mod transposition;
}

pub mod core {
    pub mod board;
    pub mod captures;
    pub mod moves;
    pub mod rules;
    pub mod state;
}

pub mod interface {
    pub mod shell_game;
    pub mod utils;
}

