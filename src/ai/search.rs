use crate::core::state::GameState;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, RwLock};
use std::collections::HashMap;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::{minimax::minimax, move_ordering::MoveOrdering, transposition::TranspositionTable};

#[derive(Debug)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub score: i32,
    pub depth_reached: i32,
    pub nodes_searched: u64,
    pub time_elapsed: Duration,
}

#[derive(Debug, Clone)]
struct ThreadResult {
    thread_id: usize,
    best_move: Option<(usize, usize)>,
    score: i32,
    depth: i32,
    nodes_searched: u64,
    is_complete: bool,
    confidence: f32,
}

#[derive(Clone)]
struct ThreadParams {
    thread_id: usize,
    base_depth: i32,
    depth_offset: i32,
    randomization_seed: u64,
    min_depth: i32,
}

struct SharedSearchState {
    abort_flag: Arc<AtomicBool>,
    thread_results: Arc<RwLock<Vec<Option<ThreadResult>>>>,
    start_time: Instant,
}

pub fn find_best_move(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
) -> SearchResult {
    let thread_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8).max(2);
    println!("🚀 Starting Lazy SMP search with {} threads, max depth {}", thread_count, max_depth);
    lazy_smp_search(state, max_depth, time_limit, tt, thread_count)
}

pub fn find_best_move_with_threads(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
    num_threads: Option<usize>,
) -> SearchResult {
    let thread_count = num_threads.unwrap_or_else(|| {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    }).min(8).max(2);
    
    println!("🚀 Starting Lazy SMP search with {} threads, max depth {}", thread_count, max_depth);
    lazy_smp_search(state, max_depth, time_limit, tt, thread_count)
}



fn lazy_smp_search(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
    num_threads: usize,
) -> SearchResult {
    let start_time = Instant::now();
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

    tt.advance_age();

    let shared_state = SharedSearchState {
        abort_flag: Arc::new(AtomicBool::new(false)),
        thread_results: Arc::new(RwLock::new(vec![None; num_threads])),
        start_time,
    };

    let mut handles = Vec::new();

    // Spawn worker threads
    for thread_id in 0..num_threads {
        let thread_params = create_thread_params(thread_id, max_depth);
        let state_clone = state.clone();
        let tt_clone = tt.clone();
        let shared_clone = SharedSearchState {
            abort_flag: shared_state.abort_flag.clone(),
            thread_results: shared_state.thread_results.clone(),
            start_time: shared_state.start_time,
        };

        let handle = thread::spawn(move || {
            thread_search_worker(
                state_clone,
                thread_params,
                time_limit,
                tt_clone,
                shared_clone,
            )
        });
        
        handles.push(handle);
    }

    // Monitor threads and handle time limits
    if let Some(time_limit) = time_limit {
        let check_interval = Duration::from_millis(10);
        while start_time.elapsed() < time_limit {
            thread::sleep(check_interval);
        }
        shared_state.abort_flag.store(true, Ordering::Relaxed);
    }

    // Wait for all threads to complete
    for handle in handles {
        let _ = handle.join(); // Ignore thread panic results
    }

    // Analyze results and vote for best move
    let results = shared_state.thread_results.read().unwrap();
    let completed_results: Vec<_> = results.iter().filter_map(|r| r.as_ref()).cloned().collect();
    
    if completed_results.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0,
            depth_reached: 0,
            nodes_searched: 0,
            time_elapsed: start_time.elapsed(),
        };
    }

    // Thread voting to select best move
    let best_move = select_best_move_by_voting(&completed_results);
    let best_result = completed_results.iter()
        .find(|r| r.best_move == best_move)
        .or_else(|| completed_results.first())
        .unwrap();

    let total_nodes: u64 = completed_results.iter().map(|r| r.nodes_searched).sum();
    let max_depth_reached = completed_results.iter().map(|r| r.depth).max().unwrap_or(0);

    println!("🎯 Lazy SMP completed: {} threads finished, best move: {:?}, nodes: {}", 
             completed_results.len(), best_move, total_nodes);

    SearchResult {
        best_move,
        score: best_result.score,
        depth_reached: max_depth_reached,
        nodes_searched: total_nodes,
        time_elapsed: start_time.elapsed(),
    }
}

fn create_thread_params(thread_id: usize, base_depth: i32) -> ThreadParams {
    // Depth offset pattern: 0, 1, 0, 1, 2, 0, 1, 2, ...
    let depth_offset = match thread_id {
        0 => 0, // Main thread
        1 => 1,
        2 => 0,
        3 => 1,
        4 => 2,
        _ => (thread_id - 2) % 3,
    };

    let min_depth = if thread_id == 0 { 1 } else { 1i32.max(depth_offset as i32) };

    ThreadParams {
        thread_id,
        base_depth,
        depth_offset: depth_offset as i32,
        randomization_seed: 0x123456789ABCDEF0u64.wrapping_add(thread_id as u64),
        min_depth,
    }
}

fn thread_search_worker(
    mut state: GameState,
    params: ThreadParams,
    time_limit: Option<Duration>,
    tt: TranspositionTable,
    shared_state: SharedSearchState,
) {
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    let mut best_move = None;
    let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;
    let mut rng = ChaCha8Rng::seed_from_u64(params.randomization_seed);

    let effective_max_depth = params.base_depth + params.depth_offset;
    let start_depth = params.min_depth;

    for depth in start_depth..=effective_max_depth {
        if shared_state.abort_flag.load(Ordering::Relaxed) {
            break;
        }

        if let Some(limit) = time_limit {
            if shared_state.start_time.elapsed() >= limit {
                break;
            }
        }

        let mut iteration_best_move = None;
        let mut iteration_best_score = if is_maximizing { i32::MIN } else { i32::MAX };

        let mut moves = state.get_possible_moves();
        apply_thread_move_ordering(&mut moves, &mut state, &params, &mut rng);
        
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        for mv in moves {
            if shared_state.abort_flag.load(Ordering::Relaxed) {
                break;
            }

            if let Some(limit) = time_limit {
                if shared_state.start_time.elapsed() >= limit {
                    break;
                }
            }
            
            state.make_move(mv);
            let (score, child_nodes) = minimax(
                &mut state,
                depth - 1,
                i32::MIN,
                i32::MAX,
                !is_maximizing,
                &tt,
                &shared_state.start_time,
                time_limit,
            );
            state.undo_move(mv);
            nodes_searched += child_nodes;

            let is_better = if is_maximizing {
                score > iteration_best_score
            } else {
                score < iteration_best_score
            };

            if is_better {
                iteration_best_score = score;
                iteration_best_move = Some(mv);
            }
        }
        
        best_move = iteration_best_move;
        best_score = iteration_best_score;
        depth_reached = depth;
        
        if best_score.abs() >= 1_000_000 {
            break;
        }
    }

    // Calculate confidence based on depth completion
    let target_depth = effective_max_depth;
    let confidence = if target_depth > 0 {
        (depth_reached as f32 / target_depth as f32).min(1.0)
    } else {
        1.0
    };

    let result = ThreadResult {
        thread_id: params.thread_id,
        best_move,
        score: best_score,
        depth: depth_reached,
        nodes_searched,
        is_complete: depth_reached >= target_depth,
        confidence,
    };

    // Store result
    if let Ok(mut results) = shared_state.thread_results.write() {
        if params.thread_id < results.len() {
            results[params.thread_id] = Some(result);
        }
    }
}

fn apply_thread_move_ordering(
    moves: &mut Vec<(usize, usize)>,
    state: &mut GameState,
    params: &ThreadParams,
    rng: &mut ChaCha8Rng,
) {
    // Apply base move ordering
    MoveOrdering::order_moves(state, moves);
    
    // Add thread-specific randomization for helper threads
    if params.thread_id > 0 && moves.len() > 1 {
        let randomization_level = 0.1; // 10% randomization
        let num_to_randomize = ((moves.len() as f32 * randomization_level) as usize).max(1).min(3);
        
        // Randomly swap some moves from the front with moves from later
        for i in 1..=num_to_randomize {
            if i < moves.len() {
                let range_size = moves.len() - i;
                let random_offset = (rng.next_u32() as usize) % range_size;
                let swap_target = i + random_offset;
                moves.swap(i, swap_target);
            }
        }
    }
}

fn select_best_move_by_voting(results: &[ThreadResult]) -> Option<(usize, usize)> {
    if results.is_empty() {
        return None;
    }

    let mut vote_map: HashMap<(usize, usize), f32> = HashMap::new();
    let worst_score = results.iter().map(|r| r.score).min().unwrap_or(0);
    
    // Collect votes from all threads
    for result in results {
        if let Some(best_move) = result.best_move {
            let vote_weight = calculate_vote_weight(result, worst_score);
            *vote_map.entry(best_move).or_insert(0.0) += vote_weight;
        }
    }
    
    if vote_map.is_empty() {
        return None;
    }
    
    // Find move with highest vote total
    vote_map.into_iter()
        .max_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(mv, _)| mv)
}

fn calculate_vote_weight(result: &ThreadResult, worst_score: i32) -> f32 {
    let score_component = (result.score - worst_score + 10) as f32;
    let depth_component = result.depth as f32;
    let completion_component = if result.is_complete { 1.2 } else { 1.0 };
    let confidence_component = result.confidence;
    
    score_component * depth_component * completion_component * confidence_component
}
