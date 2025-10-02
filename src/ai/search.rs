use crate::core::state::GameState;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

use super::{minimax::minimax, move_ordering::MoveOrdering, transposition::TranspositionTable};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub score: i32,
    pub depth_reached: i32,
    pub nodes_searched: u64,
    pub time_elapsed: Duration,
}

impl Default for SearchResult {
    fn default() -> Self {
        Self {
            best_move: None,
            score: i32::MIN,
            depth_reached: 0,
            nodes_searched: 0,
            time_elapsed: Duration::new(0, 0),
        }
    }
}

#[derive(Debug, Clone)]
struct SearchThread {
    depth_offset: i32,
    move_ordering_variant: MoveOrderingType,
    reduction_factor: f32,
    aspiration_window: Option<(i32, i32)>,
}

#[derive(Debug, Clone)]
enum MoveOrderingType {
    Standard,
    ThreatFirst,
    CenterFirst,
    HistoryBased,
}

pub fn find_best_move(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
) -> SearchResult {
    lazy_smp_search(state, max_depth, time_limit, tt, 4) // Use 4 threads by default
}

pub fn lazy_smp_search(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
    num_threads: usize,
) -> SearchResult {
    let start_time = Instant::now();
    
    tt.advance_age();

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

    let shared_result = Arc::new(Mutex::new(None));
    let should_stop = Arc::new(AtomicBool::new(false));
    
    // Configure different search threads
    let thread_configs = create_thread_configs(num_threads, max_depth);
    
    // Spawn worker threads
    let handles: Vec<_> = thread_configs.into_iter().map(|config| {
        let state_clone = state.clone();
        let tt_clone = tt.clone();
        let shared_result_clone = shared_result.clone();
        let should_stop_clone = should_stop.clone();
        
        thread::spawn(move || {
            search_worker(
                state_clone,
                config,
                time_limit,
                tt_clone,
                shared_result_clone,
                should_stop_clone,
                start_time,
            )
        })
    }).collect();
    
    // Wait for threads and collect results
    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    // Select best result
    select_best_result(results, start_time.elapsed())
}

fn create_thread_configs(num_threads: usize, _max_depth: i32) -> Vec<SearchThread> {
    let mut configs = Vec::new();
    
    for i in 0..num_threads {
        let config = match i {
            0 => SearchThread {
                depth_offset: 0,
                move_ordering_variant: MoveOrderingType::Standard,
                reduction_factor: 1.0,
                aspiration_window: None,
            },
            1 if num_threads > 1 => SearchThread {
                depth_offset: 1,
                move_ordering_variant: MoveOrderingType::ThreatFirst,
                reduction_factor: 0.8,
                aspiration_window: Some((-50, 50)),
            },
            2 if num_threads > 2 => SearchThread {
                depth_offset: 0,
                move_ordering_variant: MoveOrderingType::CenterFirst,
                reduction_factor: 0.6,
                aspiration_window: Some((-100, 100)),
            },
            _ => SearchThread {
                depth_offset: (i % 3) as i32,
                move_ordering_variant: MoveOrderingType::HistoryBased,
                reduction_factor: 0.4,
                aspiration_window: Some((-200, 200)),
            },
        };
        configs.push(config);
    }
    
    configs
}

fn search_worker(
    mut state: GameState,
    config: SearchThread,
    time_limit: Option<Duration>,
    tt: TranspositionTable,
    shared_result: Arc<Mutex<Option<SearchResult>>>,
    should_stop: Arc<AtomicBool>,
    start_time: Instant,
) -> SearchResult {
    let mut best_result = SearchResult::default();
    let thread_time_limit = calculate_thread_time_limit(time_limit, config.reduction_factor);
    
    let start_depth = 1 + config.depth_offset;
    let effective_max_depth = 50; // Reasonable maximum
    
    for depth in start_depth..=effective_max_depth {
        // Check global stop condition
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        // Check thread-specific time limit
        if let Some(limit) = thread_time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }
        
        // Perform search with thread-specific configuration
        let result = thread_specific_search(
            &mut state,
            depth,
            &config,
            &tt,
            &start_time,
            thread_time_limit,
            &should_stop,
        );
        
        // Update local best result
        if result.best_move.is_some() && (result.score > best_result.score || best_result.best_move.is_none()) {
            best_result = result;
            
            // Share promising results early
            if should_share_result(&best_result) {
                update_shared_result(&shared_result, &best_result, &should_stop);
            }
        }
        
        // Early termination for mate scores
        if best_result.score.abs() >= 1_000_000 {
            should_stop.store(true, Ordering::Relaxed);
            break;
        }
    }
    
    best_result
}

fn calculate_thread_time_limit(time_limit: Option<Duration>, reduction_factor: f32) -> Option<Duration> {
    time_limit.map(|limit| {
        Duration::from_millis((limit.as_millis() as f32 * reduction_factor) as u64)
    })
}

fn thread_specific_search(
    state: &mut GameState,
    depth: i32,
    config: &SearchThread,
    tt: &TranspositionTable,
    start_time: &Instant,
    time_limit: Option<Duration>,
    should_stop: &Arc<AtomicBool>,
) -> SearchResult {
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    let mut best_move = None;
    let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
    let mut nodes_searched = 0u64;

    let mut moves = state.get_possible_moves();
    apply_thread_specific_ordering(&mut moves, state, config);

    for move_ in moves {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }

        state.make_move(move_);
        
        let (score, child_nodes) = if let Some((alpha, beta)) = config.aspiration_window {
            minimax(state, depth - 1, alpha, beta, !is_maximizing, tt, start_time, time_limit)
        } else {
            minimax(state, depth - 1, i32::MIN, i32::MAX, !is_maximizing, tt, start_time, time_limit)
        };
        
        state.undo_move(move_);
        nodes_searched += child_nodes;

        let is_better = if is_maximizing {
            score > best_score
        } else {
            score < best_score
        };

        if is_better {
            best_score = score;
            best_move = Some(move_);
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached: depth,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}

fn apply_thread_specific_ordering(
    moves: &mut [(usize, usize)],
    state: &GameState,
    config: &SearchThread,
) {
    match config.move_ordering_variant {
        MoveOrderingType::Standard => {
            MoveOrdering::order_moves(state, moves);
        },
        MoveOrderingType::ThreatFirst => {
            MoveOrdering::order_moves(state, moves);
            // Additional threat-based reordering could be added here
        },
        MoveOrderingType::CenterFirst => {
            let center = state.board.size / 2;
            moves.sort_unstable_by_key(|&(row, col)| {
                ((row as i32 - center as i32).abs() + (col as i32 - center as i32).abs()) as i32
            });
        },
        MoveOrderingType::HistoryBased => {
            MoveOrdering::order_moves(state, moves);
            // Could add history table ordering here
        },
    }
}

fn should_share_result(result: &SearchResult) -> bool {
    // Share results that are winning or at reasonable depth
    result.score >= 1_000_000 || result.depth_reached >= 4
}

fn update_shared_result(
    shared_result: &Arc<Mutex<Option<SearchResult>>>,
    new_result: &SearchResult,
    should_stop: &Arc<AtomicBool>,
) {
    if let Ok(mut shared) = shared_result.try_lock() {
        let should_update = match &*shared {
            None => true,
            Some(current) => is_better_result(new_result, current),
        };
        
        if should_update {
            *shared = Some(new_result.clone());
            
            // Stop all threads if we found a winning move
            if new_result.score >= 1_000_000 {
                should_stop.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn select_best_result(results: Vec<SearchResult>, total_time: Duration) -> SearchResult {
    // Aggregate node counts from all threads first
    let total_nodes: u64 = results.iter().map(|r| r.nodes_searched).sum();
    
    let mut best_result = results.into_iter()
        .filter(|r| r.best_move.is_some())
        .max_by(|a, b| {
            // First compare by winning scores
            match (a.score >= 1_000_000, b.score >= 1_000_000) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => {
                    // Then by depth, then by score
                    a.depth_reached.cmp(&b.depth_reached)
                        .then(a.score.cmp(&b.score))
                }
            }
        })
        .unwrap_or_default();
    
    best_result.nodes_searched = total_nodes;
    best_result.time_elapsed = total_time;
    
    best_result
}

fn is_better_result(candidate: &SearchResult, current_best: &SearchResult) -> bool {
    // Winning moves are always preferred
    if candidate.score >= 1_000_000 && current_best.score < 1_000_000 {
        return true;
    }
    
    // At same depth, prefer better score
    if candidate.depth_reached == current_best.depth_reached {
        return candidate.score > current_best.score;
    }
    
    // Prefer deeper search, but not if score is significantly worse
    candidate.depth_reached > current_best.depth_reached && 
    candidate.score >= current_best.score - 1000
}
