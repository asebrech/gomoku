use crate::ai::{
    minimax::{minimax, reset_profiling, print_profiling}, 
    transposition::TranspositionTable,
    advanced_search::{iterative_deepening_search, SearchContext, reset_profiling as reset_advanced_profiling, print_profiling as print_advanced_profiling}
};
use crate::core::state::GameState;
use crate::core::board::Player;
use std::time::{Instant, Duration};

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let start_time = Instant::now();
    reset_profiling();
    
    // Start a new search generation for better transposition table management
    //tt.new_search();
    
    let mut best_move = None;
    let current_player = state.current_player; // Store the player who is making the move
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let (tt_hits, tt_stores, hit_rate) = tt.get_stats();
    println!("TT Size: {} entries, Hits: {}, Stores: {}, Hit rate: {:.1}%", 
             tt.size(), tt_hits, tt_stores, hit_rate);
    
    // Use fast iteration and collect moves into a temporary vec only once at root
    let move_gen_start = Instant::now();
    let mut moves = Vec::with_capacity(64);
    state.for_each_possible_move(|mv| moves.push(mv));
    println!("Root move generation took: {}μs for {} moves", move_gen_start.elapsed().as_micros(), moves.len());
    
    for mv in moves {
        state.make_move(mv);
        // After make_move, the current_player has switched, so we need to use the opposite
        // for maximizing_player parameter
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            current_player == Player::Min, // This is correct because we want to maximize if current_player is Max
            tt,
        );
        state.undo_move(mv);

        if (current_player == Player::Max && score > best_score)
            || (current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    let total_time = start_time.elapsed();
    println!("Total search time: {}ms", total_time.as_millis());
    print_profiling();

    best_move
}

// Advanced AI function using iterative deepening and all optimizations
// Returns (best_move, depth_reached)
pub fn find_best_move_advanced_with_depth(state: &mut GameState, time_limit_ms: Option<u64>, max_depth: Option<i32>) -> (Option<(usize, usize)>, i32) {
    let start_time = Instant::now();
    reset_advanced_profiling();
    
    // Create a new TT for this search - in a real game, you'd want to persist this
    let mut tt = TranspositionTable::new(state.board.size(), state.board.size());
    tt.new_search();
    
    let time_limit = time_limit_ms.map(|ms| Duration::from_millis(ms));
    let max_search_depth = max_depth.unwrap_or(20); // Default to depth 20 if not specified
    let mut search_context = SearchContext::new();
    
    let (tt_hits, tt_stores, hit_rate) = tt.get_stats();
    println!("=== ADVANCED AI SEARCH ===");
    println!("TT Size: {} entries, Hits: {}, Stores: {}, Hit rate: {:.1}%", 
             tt.size(), tt_hits, tt_stores, hit_rate);
    
    if let Some(limit) = time_limit {
        println!("Time limit: {:?}", limit);
    } else {
        println!("No time limit");
    }
    println!("Max depth: {}", max_search_depth);
    
    let (best_move, depth_reached) = iterative_deepening_search(
        state, 
        max_search_depth, 
        time_limit, 
        &mut tt, 
        &mut search_context
    );
    
    let total_time = start_time.elapsed();
    println!("Total advanced search time: {:?}", total_time);
    println!("Depth reached: {}", depth_reached);
    print_advanced_profiling();
    
    let (final_hits, final_stores, final_hit_rate) = tt.get_stats();
    println!("Final TT Stats: {} hits, {} stores, {:.1}% hit rate", 
             final_hits, final_stores, final_hit_rate);
    
    (best_move, depth_reached)
}

// Legacy function - now calls advanced search but only returns the move
pub fn find_best_move_advanced(state: &mut GameState, max_depth: i32, time_limit_ms: Option<u64>, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let start_time = Instant::now();
    reset_advanced_profiling();
    
    // Start a new search generation for better transposition table management
    tt.new_search();
    
    let time_limit = time_limit_ms.map(|ms| Duration::from_millis(ms));
    let mut search_context = SearchContext::new();
    
    let (tt_hits, tt_stores, hit_rate) = tt.get_stats();
    println!("=== ADVANCED AI SEARCH ===");
    println!("TT Size: {} entries, Hits: {}, Stores: {}, Hit rate: {:.1}%", 
             tt.size(), tt_hits, tt_stores, hit_rate);
    
    if let Some(limit) = time_limit {
        println!("Time limit: {:?}", limit);
    }
    println!("Max depth: {}", max_depth);
    
    let (best_move, depth_reached) = iterative_deepening_search(
        state, 
        max_depth, 
        time_limit, 
        tt, 
        &mut search_context
    );
    
    let total_time = start_time.elapsed();
    println!("Total advanced search time: {:?}", total_time);
    println!("Depth reached: {}", depth_reached);
    print_advanced_profiling();
    
    let (final_hits, final_stores, final_hit_rate) = tt.get_stats();
    println!("Final TT Stats: {} hits, {} stores, {:.1}% hit rate", 
             final_hits, final_stores, final_hit_rate);
    
    best_move
}

// Wrapper for backward compatibility - uses advanced search by default
pub fn find_best_move_with_time_limit(state: &mut GameState, time_limit_ms: u64, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    find_best_move_advanced(state, 12, Some(time_limit_ms), tt) // Max depth 12 with time limit
}

// New convenient functions focused on time-based search
pub fn find_best_move_timed(state: &mut GameState, time_limit_ms: u64) -> (Option<(usize, usize)>, i32) {
    find_best_move_advanced_with_depth(state, Some(time_limit_ms), None)
}

pub fn find_best_move_unlimited(state: &mut GameState, max_depth: i32) -> (Option<(usize, usize)>, i32) {
    find_best_move_advanced_with_depth(state, None, Some(max_depth))
}

pub fn find_best_move_quick(state: &mut GameState) -> (Option<(usize, usize)>, i32) {
    find_best_move_advanced_with_depth(state, Some(1000), None) // 1 second default
}
