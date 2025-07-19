use crate::core::board::{Board, Player};

pub struct CaptureHandler;

impl CaptureHandler {
    pub fn detect_captures(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
    ) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let opponent = player.opponent();

        for &(dx, dy) in &directions {
            for &direction_multiplier in &[1, -1] {
                let actual_dx = dx * direction_multiplier;
                let actual_dy = dy * direction_multiplier;

                let pos1_x = row as isize + actual_dx;
                let pos1_y = col as isize + actual_dy;

                if pos1_x >= 0
                    && pos1_y >= 0
                    && pos1_x < board.size as isize
                    && pos1_y < board.size as isize
                {
                    if board.get_player(pos1_x as usize, pos1_y as usize) == Some(opponent) {
                        let pos2_x = pos1_x + actual_dx;
                        let pos2_y = pos1_y + actual_dy;

                        if pos2_x >= 0
                            && pos2_y >= 0
                            && pos2_x < board.size as isize
                            && pos2_y < board.size as isize
                        {
                            if board.get_player(pos2_x as usize, pos2_y as usize) == Some(opponent) {
                                let pos3_x = pos2_x + actual_dx;
                                let pos3_y = pos2_y + actual_dy;

                                if pos3_x >= 0
                                    && pos3_y >= 0
                                    && pos3_x < board.size as isize
                                    && pos3_y < board.size as isize
                                {
                                    if board.get_player(pos3_x as usize, pos3_y as usize) == Some(player)
                                    {
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
}
