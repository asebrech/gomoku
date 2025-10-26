pub mod ai {
    pub mod heuristic;
    #[cfg(feature = "rayon")]
    pub mod lazy_smp;
    pub mod minimax;
    pub mod move_generation;
    pub mod pattern_history;
    pub mod transposition;
}

#[cfg(feature = "bevy")]
pub mod audio;

pub mod core {
    pub mod board;
    pub mod captures;
    pub mod patterns;
    pub mod rules;
    pub mod state;
    pub mod zobrist;
}

#[cfg(feature = "bevy")]
pub mod ui {
    pub mod app;
    pub mod config;
    pub mod theme;
    pub mod components {
        pub mod button;
    }
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
        pub mod tutorial;
        pub mod utils;
    }
}

// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm;
