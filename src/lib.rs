pub mod ai {
    pub mod heuristic;
    #[cfg(feature = "rayon")]
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
