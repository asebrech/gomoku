use crate::core::state::GameState;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use rayon::prelude::*;

use super::{minimax::mtdf, transposition::TranspositionTable};

/// Search result structure
#[derive(Debug)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub score: i32,
    pub depth_reached: i32,
    pub nodes_searched: u64,
    pub time_elapsed: Duration,
}

/// Shared search state for Lazy SMP
pub struct SharedSearchState {
    pub best_move: Mutex<Option<(usize, usize)>>,
    pub best_score: AtomicI32,
    pub nodes_searched: AtomicU64,
    pub depth_reached: AtomicI32,
    pub stop_search: AtomicBool,
}

impl SharedSearchState {
    pub fn new() -> Self {
        Self {
            best_move: Mutex::new(None),
            best_score: AtomicI32::new(0),
            nodes_searched: AtomicU64::new(0),
            depth_reached: AtomicI32::new(0),
            stop_search: AtomicBool::new(false),
        }
    }

    pub fn update_best(&self, score: i32, mv: Option<(usize, usize)>, depth: i32) -> bool {
        let current_score = self.best_score.load(Ordering::Relaxed);
        
        // Try to update if this is a better score
        if score > current_score {
            if self.best_score.compare_exchange_weak(
                current_score, 
                score, 
                Ordering::Relaxed, 
                Ordering::Relaxed
            ).is_ok() {
                if let Some(mv) = mv {
                    *self.best_move.lock().unwrap() = Some(mv);
                }
                self.depth_reached.store(depth, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    pub fn should_stop(&self) -> bool {
        self.stop_search.load(Ordering::Relaxed)
    }

    pub fn signal_stop(&self) {
        self.stop_search.store(true, Ordering::Relaxed);
    }

    pub fn add_nodes(&self, nodes: u64) {
        self.nodes_searched.fetch_add(nodes, Ordering::Relaxed);
    }
}

/// Lazy SMP worker that runs MTD(f) with slightly different parameters
fn lazy_smp_worker(
    state: &GameState,
    max_depth: i32,
    shared_state: Arc<SharedSearchState>,
    worker_id: usize,
    start_time: Instant,
    time_limit: Option<Duration>,
) -> (i32, Option<(usize, usize)>, i32, u64) {
    let mut local_state = state.clone();
    let mut tt = TranspositionTable::new(1_000_000); // Each worker gets its own TT
    
    let mut best_move = None;
    let mut best_score = 0;
    let mut depth_reached = 0;
    let mut total_nodes = 0;

    // Lazy SMP parameters: different workers use slightly different search parameters
    let depth_offset = match worker_id {
        0 => 0,  // Main worker searches at requested depth
        1 => -1, // Worker 1 searches one depth shallower
        2 => 1,  // Worker 2 searches one depth deeper
        3 => -2, // Worker 3 searches two depths shallower
        _ => (worker_id as i32 - 2) % 3 - 1, // Other workers vary between -1, 0, 1
    };

    // Different aspiration window sizes
    let aspiration_offset = match worker_id % 4 {
        0 => 0,
        1 => 50,
        2 => -50,
        _ => 0,
    };

    for depth in 1..=max_depth {
        if shared_state.should_stop() {
            break;
        }

        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                shared_state.signal_stop();
                break;
            }
        }

        // Apply depth offset for this worker
        let search_depth = (depth + depth_offset).max(1);

        // Use shared best score as first guess, with aspiration offset
        let first_guess = shared_state.best_score.load(Ordering::Relaxed) + aspiration_offset;

        let (score, nodes, mv) = mtdf(
            &mut local_state,
            first_guess,
            search_depth,
            &mut tt,
            &start_time,
            time_limit,
        );

        total_nodes += nodes;
        shared_state.add_nodes(nodes);

        if mv.is_some() {
            best_move = mv;
            best_score = score;
            depth_reached = search_depth;

            // Try to update shared state
            shared_state.update_best(score, mv, search_depth);

            // Stop if we found a winning position
            if score.abs() >= 1_000_000 {
                shared_state.signal_stop();
                break;
            }
        }

        // Check if another worker found a better result
        if shared_state.should_stop() {
            break;
        }
    }

    (best_score, best_move, depth_reached, total_nodes)
}

/// Parallel search using Lazy SMP with time limit
/// 
/// Searches iteratively deeper until time runs out, going as deep as possible.
/// This is the recommended function for gameplay.
/// 
/// # Arguments
/// * `state` - The game state to search from
/// * `time_limit_ms` - Time limit for the search in milliseconds (e.g., 500 for 500ms)
/// * `num_threads` - Number of worker threads (None = auto-detect CPU cores, capped at 8)
pub fn lazy_smp_search(
    state: &mut GameState,
    time_limit_ms: u64,
    num_threads: Option<usize>,
) -> SearchResult {
    lazy_smp_search_internal(state, 100, Some(Duration::from_millis(time_limit_ms)), num_threads)
}

/// Parallel search using Lazy SMP with depth limit (for testing)
/// 
/// Searches up to a specific depth. Useful for tests that need predictable behavior.
/// 
/// # Arguments
/// * `state` - The game state to search from
/// * `max_depth` - Maximum depth to search
/// * `time_limit` - Optional time limit
/// * `num_threads` - Number of worker threads
pub fn lazy_smp_search_depth(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    num_threads: Option<usize>,
) -> SearchResult {
    lazy_smp_search_internal(state, max_depth, time_limit, num_threads)
}

/// Internal search implementation used by both public functions
fn lazy_smp_search_internal(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    num_threads: Option<usize>,
) -> SearchResult {
    let start_time = Instant::now();
    
    // Use number of CPU cores if not specified
    let threads = num_threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8) // Cap at 8 threads for diminishing returns
    });

    let initial_moves = state.get_possible_moves();
    if initial_moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0,
            depth_reached: 0,
            nodes_searched: 0,
            time_elapsed: start_time.elapsed(),
        };
    }

    let shared_state = Arc::new(SharedSearchState::new());
    
    // Launch worker threads
    let workers: Vec<_> = (0..threads).into_par_iter().map(|worker_id| {
        let state_clone = state.clone();
        let shared_state_clone = Arc::clone(&shared_state);
        
        lazy_smp_worker(
            &state_clone,
            max_depth,
            shared_state_clone,
            worker_id,
            start_time,
            time_limit,
        )
    }).collect();

    // Wait for all workers to complete and get the best result
    let mut best_score = i32::MIN;
    let mut best_move = None;
    let mut max_depth_reached = 0;

    for (score, mv, depth, _nodes) in workers {
        if score > best_score && mv.is_some() {
            best_score = score;
            best_move = mv;
        }
        max_depth_reached = max_depth_reached.max(depth);
    }

    // Use shared state results if they're better
    let shared_score = shared_state.best_score.load(Ordering::Relaxed);
    let shared_move = *shared_state.best_move.lock().unwrap();
    
    if shared_score > best_score && shared_move.is_some() {
        best_score = shared_score;
        best_move = shared_move;
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached: shared_state.depth_reached.load(Ordering::Relaxed).max(max_depth_reached),
        nodes_searched: shared_state.nodes_searched.load(Ordering::Relaxed),
        time_elapsed: start_time.elapsed(),
    }
}