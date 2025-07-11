use crate::core::board::{Board, Player};

pub struct CaptureHandler;

impl CaptureHandler {
    pub fn detect_captures(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let opponent = player.opponent();

        for &(dx, dy) in &directions {
            for &direction_multiplier in &[1, -1] {
                let actual_dx = dx * direction_multiplier;
                let actual_dy = dy * direction_multiplier;
                
                let pos1_x = row as isize + actual_dx;
                let pos1_y = col as isize + actual_dy;
                
                if pos1_x >= 0 && pos1_y >= 0 && 
                   pos1_x < board.size as isize && pos1_y < board.size as isize {
                    
                    if board.cells[pos1_x as usize][pos1_y as usize] == Some(opponent) {
                        let pos2_x = pos1_x + actual_dx;
                        let pos2_y = pos1_y + actual_dy;
                        
                        if pos2_x >= 0 && pos2_y >= 0 && 
                           pos2_x < board.size as isize && pos2_y < board.size as isize {
                            
                            if board.cells[pos2_x as usize][pos2_y as usize] == Some(opponent) {
                                let pos3_x = pos2_x + actual_dx;
                                let pos3_y = pos2_y + actual_dy;
                                
                                if pos3_x >= 0 && pos3_y >= 0 && 
                                   pos3_x < board.size as isize && pos3_y < board.size as isize {
                                    
                                    if board.cells[pos3_x as usize][pos3_y as usize] == Some(player) {
                                        captures.push((pos1_x as usize, pos1_y as usize));
                                        captures.push((pos2_x as usize, pos2_y as usize));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        captures
    }

    pub fn execute_captures(board: &mut Board, captures: &[(usize, usize)]) {
        for &(row, col) in captures {
            board.remove_stone(row, col);
        }
    }

    pub fn can_stone_be_captured(board: &Board, row: usize, col: usize, opponent: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let player = match board.get_player(row, col) {
            Some(p) => p,
            None => return false,
        };

        for &(dx, dy) in &directions {
            for multiplier in [-1, 1] {
                let back_x = row as isize - multiplier * dx;
                let back_y = col as isize - multiplier * dy;

                let next_x = row as isize + multiplier * dx;
                let next_y = col as isize + multiplier * dy;

                let capture_x = next_x + multiplier * dx;
                let capture_y = next_y + multiplier * dy;

                if back_x >= 0 && back_y >= 0 &&
                   next_x >= 0 && next_y >= 0 &&
                   capture_x >= 0 && capture_y >= 0 &&
                   back_x < board.size as isize && back_y < board.size as isize &&
                   next_x < board.size as isize && next_y < board.size as isize &&
                   capture_x < board.size as isize && capture_y < board.size as isize {

                    if board.cells[back_x as usize][back_y as usize] == Some(opponent)
                        && board.cells[next_x as usize][next_y as usize] == Some(player)
                        && board.cells[capture_x as usize][capture_y as usize].is_none() {

                        if board.cells[row][col] == board.cells[next_x as usize][next_y as usize] {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn can_capture_from_line(board: &Board, line: &[(usize, usize)], opponent: Player) -> bool {
        line.iter().any(|&(row, col)| Self::can_stone_be_captured(board, row, col, opponent))
    }
}

