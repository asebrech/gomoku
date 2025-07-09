use std::cmp::{max, min};

#[derive(Clone, PartialEq)]
enum Player {
    Max,
    Min,
}
struct GameState {
    board: [[Option<Player>; 4]; 4],
    current_player: Player,
}
impl GameState {
    fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j].is_none() {
                    moves.push((i, j));
                }
            }
        }
        moves
    }
    fn make_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = Some(self.current_player.clone());
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
    }
    fn undo_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = None;
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
    }
    fn is_terminal(&self) -> bool {
        // Check for a win or a full board
        // Implementation omitted for brevity
        false
    }
    fn evaluate(&self) -> i32 {
        // Simple evaluation: +1 for MAX win, -1 for MIN win, 0 for draw
        // Implementation omitted for brevity
        0
    }
}
fn minimax(state: &mut GameState, depth: i32, maximizing_player: bool) -> i32 {
    if depth == 0 || state.is_terminal() {
        return state.evaluate();
    }
    if maximizing_player {
        let mut max_eval = i32::MIN;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            let eval = minimax(state, depth - 1, false);
            state.undo_move(move_);
            max_eval = max(max_eval, eval);
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for move_ in state.get_possible_moves() {
            state.make_move(move_);
            let eval = minimax(state, depth - 1, true);
            state.undo_move(move_);
            min_eval = min(min_eval, eval);
        }
        min_eval
    }
}
