use crate::core::board::{Board, Player};

pub struct WinChecker;

impl WinChecker {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        let player_opt = board.get_player(row, col);
        if player_opt.is_none() {
            return false;
        }
        let player = player_opt.unwrap();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in directions.iter() {
            let mut count = 1;

            // Forward direction
            let mut step = 1;
            loop {
                let x = row as isize + dx as isize * step;
                let y = col as isize + dy as isize * step;
                if x < 0 || y < 0 || x >= board.size as isize || y >= board.size as isize {
                    break;
                }
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                } else {
                    break;
                }
                step += 1;
            }

            // Backward direction
            let mut step = 1;
            loop {
                let x = row as isize - dx as isize * step;
                let y = col as isize - dy as isize * step;
                if x < 0 || y < 0 || x >= board.size as isize || y >= board.size as isize {
                    break;
                }
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                } else {
                    break;
                }
                step += 1;
            }

            if count >= win_condition {
                return true;
            }
        }

        false
    }

    pub fn check_capture_win(max_captures: usize, min_captures: usize) -> Option<Player> {
        if max_captures >= 5 {
            Some(Player::Max)
        } else if min_captures >= 5 {
            Some(Player::Min)
        } else {
            None
        }
    }
}
