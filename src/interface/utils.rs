use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;
use crate::core::rules::WinChecker;

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let mut best_move = None;
    let current_player = state.current_player;
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let mut moves = state.get_possible_moves();
    prioritize_defensive_moves(state, &mut moves);

    for mv in moves {
        state.make_move(mv);
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            current_player == Player::Min,
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

    best_move
}

fn prioritize_defensive_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
    let opponent = state.current_player.opponent();
    let mut threat_blocking_moves = Vec::new();
    let mut other_moves = Vec::new();
    
    for &mv in moves.iter() {
        if blocks_immediate_threat(state, mv, opponent) {
            threat_blocking_moves.push(mv);
        } else {
            other_moves.push(mv);
        }
    }
    
    moves.clear();
    moves.extend(threat_blocking_moves);
    moves.extend(other_moves);
}

fn blocks_immediate_threat(state: &GameState, mv: (usize, usize), opponent: Player) -> bool {
    let mut test_state = state.clone();
    test_state.board.place_stone(mv.0, mv.1, opponent);
    
    WinChecker::check_win_around(&test_state.board, mv.0, mv.1, state.win_condition)
}
