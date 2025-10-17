use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::ai::heuristic::Heuristic;

#[test]
fn test_ai_recognizes_check_as_dangerous() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check == Some(Player::Max) {
        let eval = Heuristic::evaluate(&state, 0);
        
        assert!(eval < 0, "AI should evaluate check state as dangerous for Max. Got: {}", eval);
        assert!(eval < -40_000, "Check penalty should be significant. Got: {}", eval);
    }
}

#[test]
fn test_ai_recognizes_putting_opponent_in_check_is_good() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check == Some(Player::Max) {
        let eval = Heuristic::evaluate(&state, 0);
        
        assert!(eval > 0, "AI (Min) should evaluate this as favorable when Max is in check. Got: {}", eval);
        assert!(eval > 40_000, "Putting opponent in check should be highly valued. Got: {}", eval);
    }
}

#[test]
fn test_breakable_five_not_scored_as_win() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check == Some(Player::Max) {
        assert_eq!(state.winner, None, "Breakable five should not set a winner");
        
        let eval = Heuristic::evaluate(&state, 0);
        
        assert!(eval.abs() < 1_000_000, "Breakable five should not be scored as winning. Got: {}", eval);
    }
}

#[test]
fn test_unbreakable_five_still_scores_as_win() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((9, 11));
    state.make_move((10, 12));
    state.make_move((9, 12));
    state.make_move((10, 13));
    state.make_move((9, 13));
    state.make_move((10, 14));
    
    assert_eq!(state.winner, Some(Player::Max), "Unbreakable five should set winner");
    assert_eq!(state.player_in_check, None, "Unbreakable five should not create check state");
    
    let eval = Heuristic::evaluate(&state, 0);
    
    assert!(eval > 900_000, "Unbreakable five should score as winning. Got: {}", eval);
}

#[test]
fn test_check_state_evaluation_is_symmetric() {
    let mut state = GameState::new(19, 5);
    
    state.make_move((10, 10));
    state.make_move((9, 10));
    state.make_move((10, 11));
    state.make_move((5, 5));
    state.make_move((10, 12));
    state.make_move((11, 12));
    state.make_move((10, 13));
    state.make_move((5, 6));
    state.make_move((10, 14));
    
    if state.player_in_check == Some(Player::Max) {
        let eval_min_perspective = Heuristic::evaluate(&state, 0);
        
        let mut reversed_state = state.clone();
        reversed_state.player_in_check = Some(Player::Min);
        let eval_max_perspective = Heuristic::evaluate(&reversed_state, 0);
        
        assert_eq!(
            eval_min_perspective, 
            -eval_max_perspective,
            "Check evaluation should be symmetric. Min perspective: {}, Max perspective: {}",
            eval_min_perspective,
            eval_max_perspective
        );
    }
}
