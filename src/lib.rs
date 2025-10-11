pub mod ai {
    pub mod heuristic;
    pub mod lazy_smp;
    pub mod minimax;
    pub mod move_generation;
    pub mod pattern_history;
    pub mod transposition;
}

pub mod core {
    pub mod board;
    pub mod captures;
    pub mod patterns;
    pub mod rules;
    pub mod state;
    pub mod zobrist;
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
