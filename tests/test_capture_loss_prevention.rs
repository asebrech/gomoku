use gomoku::ai::lazy_smp::lazy_smp_search;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

#[test]
fn test_ai_doesnt_allow_capture_win() {
    // Setup a scenario where the opponent (Min) has 4 captures
    // and AI (Max) should NOT place a stone that allows Min to win by capture
    let mut state = GameState::new(19, 5);
    
    // Simulate that Min already has 4 captures (one away from winning with 5)
    state.min_captures = 4;
    
    // Create a position where Max has a vulnerable pair that Min could capture
    // If Max places at (10, 10), this could create patterns where Min can capture
    
    // Place Max stones that form a vulnerable pattern
    state.make_move((10, 8));   // Max
    state.make_move((9, 8));    // Min
    state.make_move((10, 9));   // Max - now Max has stones at (10,8) and (10,9)
    state.make_move((9, 9));    // Min
    
    // Now Max is about to move. If Max places at (10, 7) creating a line of 3,
    // Min could place at (10, 10) and capture (10, 8) and (10, 9), winning the game
    
    // Manually set Min captures to 4
    state.min_captures = 4;
    
    // The AI should recognize this danger and NOT play (10, 7)
    // or any move that creates a capturable pair
    let result = lazy_smp_search(&mut state, 5000, 4, Some(1));
    let best_move = result.best_move;
    
    // Verify the move doesn't create a losing capture opportunity
    if let Some((row, col)) = best_move {
        // Simulate Max's move
        state.make_move((row, col));
        
        // Check all possible Min responses for winning captures
        let mut min_can_win = false;
        
        for r in 0..state.board.size {
            for c in 0..state.board.size {
                if state.board.get_player(r, c).is_some() {
                    continue;
                }
                
                // Try Min's move and check captures
                let mut test_state = state.clone();
                test_state.make_move((r, c));
                
                if test_state.winner == Some(Player::Min) {
                    min_can_win = true;
                    println!("AI played {:?} but Min can win by capture at {:?}", (row, col), (r, c));
                    break;
                }
            }
            if min_can_win {
                break;
            }
        }
        
        assert!(!min_can_win, "AI should not play a move that allows opponent to win by capture");
    }
}

#[test]
fn test_ai_defends_against_imminent_capture_win() {
    // Test that AI recognizes when opponent is at 4 captures and plays defensively
    let mut state = GameState::new(19, 5);
    
    // Set up a position where Min has 4 captures
    state.min_captures = 4;
    
    // Create several vulnerable Max pairs
    state.make_move((10, 10));  // Max
    state.make_move((5, 5));    // Min
    state.make_move((10, 11));  // Max - vulnerable pair at (10,10)-(10,11)
    state.make_move((5, 6));    // Min
    
    // Max's turn - should be very careful not to create more capturable pairs
    let result = lazy_smp_search(&mut state, 5000, 4, Some(1));
    let best_move = result.best_move;
    
    if let Some((row, col)) = best_move {
        state.make_move((row, col));
        
        // Verify Min cannot win on next move
        let mut min_can_win = false;
        for r in 0..state.board.size {
            for c in 0..state.board.size {
                if state.board.get_player(r, c).is_some() {
                    continue;
                }
                
                let mut test_state = state.clone();
                test_state.make_move((r, c));
                
                if test_state.winner == Some(Player::Min) {
                    min_can_win = true;
                    break;
                }
            }
            if min_can_win {
                break;
            }
        }
        
        assert!(!min_can_win, "AI failed to defend against imminent capture win");
    }
}

#[test]
fn test_ai_prioritizes_preventing_capture_loss_over_offense() {
    // Test that when opponent can win by capture, AI prioritizes defense
    let mut state = GameState::new(19, 5);
    
    // Min has 4 captures
    state.min_captures = 4;
    
    // Create a tempting offensive move for Max (like a three-in-a-row)
    state.make_move((10, 10));  // Max
    state.make_move((5, 5));    // Min
    state.make_move((10, 11));  // Max
    state.make_move((5, 6));    // Min
    state.make_move((10, 12));  // Max - three in a row
    
    // Now create a vulnerable pair
    state.make_move((8, 8));    // Min
    state.make_move((12, 10));  // Max
    state.make_move((8, 9));    // Min
    state.make_move((12, 11));  // Max - vulnerable pair at (12,10)-(12,11)
    
    // Min's turn, then Max's turn
    state.make_move((6, 6));    // Min
    
    // Max could extend to (10, 13) for four-in-a-row, but if that creates
    // a capturable pair, AI should avoid it
    let result = lazy_smp_search(&mut state, 5000, 4, Some(1));
    let best_move = result.best_move;
    
    if let Some((row, col)) = best_move {
        state.make_move((row, col));
        
        // Verify Min cannot win by capture
        let mut min_can_win = false;
        for r in 0..state.board.size {
            for c in 0..state.board.size {
                if state.board.get_player(r, c).is_some() {
                    continue;
                }
                
                let mut test_state = state.clone();
                test_state.make_move((r, c));
                
                if test_state.winner == Some(Player::Min) {
                    min_can_win = true;
                    break;
                }
            }
            if min_can_win {
                break;
            }
        }
        
        assert!(!min_can_win, "AI should prioritize preventing capture loss over offensive moves");
    }
}
