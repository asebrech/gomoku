use wasm_bindgen::prelude::*;

// Set up panic hook to log panics to console
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

pub mod ai {
    pub mod heuristic;
    pub mod lazy_smp;
    pub mod minimax;
    pub mod move_ordering;
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

/// Initialize the thread pool for parallel search in WASM
#[wasm_bindgen]
pub fn init_thread_pool(num_threads: usize) -> js_sys::Promise {
    wasm_bindgen_rayon::init_thread_pool(num_threads)
}
