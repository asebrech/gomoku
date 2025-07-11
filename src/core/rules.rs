use crate::core::board::{Board, Player};

pub struct WinChecker;

impl WinChecker {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        let player = board.get_player(row, col).unwrap();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in directions.iter() {
            let mut count = 1;

            let mut x = row as isize + dx as isize;
            let mut y = col as isize + dy as isize;
            while x >= 0 && y >= 0 && x < board.size as isize && y < board.size as isize {
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                    x += dx as isize;
                    y += dy as isize;
                } else {
                    break;
                }
            }

            let mut x = row as isize - dx as isize;
            let mut y = col as isize - dy as isize;
            while x >= 0 && y >= 0 && x < board.size as isize && y < board.size as isize {
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                    x -= dx as isize;
                    y -= dy as isize;
                } else {
                    break;
                }
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

    pub fn find_five_in_a_row_lines(board: &Board, player: Player, win_condition: usize) -> Vec<Vec<(usize, usize)>> {
        let mut lines = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for i in 0..board.size {
            for j in 0..board.size {
                if board.get_player(i, j) == Some(player) {
                    for &(dx, dy) in &directions {
                        let line = Self::get_line_from_position(board, i, j, dx, dy, player);
                        if line.len() >= win_condition {
                            lines.push(line);
                        }
                    }
                }
            }
        }

        lines
    }

    fn get_line_from_position(board: &Board, start_row: usize, start_col: usize, dx: isize, dy: isize, player: Player) -> Vec<(usize, usize)> {
        let mut line = Vec::new();
        
        let prev_x = start_row as isize - dx;
        let prev_y = start_col as isize - dy;
        
        if prev_x >= 0 && prev_y >= 0 && 
           prev_x < board.size as isize && prev_y < board.size as isize {
            if board.get_player(prev_x as usize, prev_y as usize) == Some(player) {
                return line;
            }
        }

        let mut x = start_row as isize;
        let mut y = start_col as isize;
        
        while x >= 0 && y >= 0 && 
              x < board.size as isize && y < board.size as isize &&
              board.get_player(x as usize, y as usize) == Some(player) {
            line.push((x as usize, y as usize));
            x += dx;
            y += dy;
        }

        line
    }

    pub fn can_break_five_by_capture(board: &Board, player: Player, win_condition: usize) -> bool {
        let opponent = player.opponent();

        let five_lines = Self::find_five_in_a_row_lines(board, player, win_condition);
        
        for line in five_lines {
            if crate::core::captures::CaptureHandler::can_capture_from_line(board, &line, opponent) {
                return true;
            }
        }

        false
    }

    pub fn is_about_to_lose_by_capture(max_captures: usize, min_captures: usize, player: Player) -> bool {
        match player {
            Player::Max => max_captures >= 4,
            Player::Min => min_captures >= 4,
        }
    }

    pub fn can_capture_to_win(board: &Board, max_captures: usize, min_captures: usize, player: Player) -> bool {
        let opponent = player.opponent();
        
        if !Self::is_about_to_lose_by_capture(max_captures, min_captures, opponent) {
            return false;
        }

        for i in 0..board.size {
            for j in 0..board.size {
                if board.is_empty_position(i, j) {
                    let captures = crate::core::captures::CaptureHandler::detect_captures(board, i, j, player);
                    if !captures.is_empty() {
                        return true;
                    }
                }
            }
        }

        false
    }
}

