pub mod ai {
    pub mod heuristic;
    pub mod minimax;
    pub mod move_ordering;
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

pub mod ui {
    pub mod app;
    pub mod display {
        pub mod display;
    }
    pub mod screens {
        pub mod game {
            pub mod board;
            pub mod game;
            pub mod settings;
        }
        pub mod menu;
        pub mod splash;
        pub mod utils;
    }
}
