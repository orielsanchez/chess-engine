use chess_engine::distance_to_mate::DistanceToMateAnalyzer;
use chess_engine::position::Position;
use chess_engine::tablebase::TablebaseResult;

/// Tests for Distance-to-Mate (DTM) visualization functionality
///
/// This module tests the ability to:
/// - Calculate precise distance-to-mate from tablebase positions
/// - Generate optimal mate sequences showing best play
/// - Visualize mate paths for endgame study
/// - Provide move-by-move mate analysis

#[test]
fn test_distance_to_mate_analyzer_creation() {
    // Test: DTM analyzer should be creatable and functional
    // Expected: Can create analyzer instance for mate analysis
    let analyzer = DistanceToMateAnalyzer::new();

    assert!(
        analyzer.is_ready(),
        "DTM analyzer should be ready for analysis"
    );
}

#[test]
fn test_calculate_distance_to_mate_for_tablebase_position() {
    // Test: Should calculate exact DTM for positions in tablebase
    // Expected: Returns precise DTM matching tablebase result
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position - tablebase shows mate in 10
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let dtm_result = analyzer.calculate_distance_to_mate(&position);

    assert!(
        dtm_result.is_ok(),
        "Should calculate DTM for tablebase position"
    );
    let dtm = dtm_result.unwrap();

    assert_eq!(dtm.distance(), 10, "Should return correct distance to mate");
    assert_eq!(
        dtm.result(),
        TablebaseResult::Win(10),
        "Should indicate winning position"
    );
    assert!(dtm.is_winning(), "Should identify as winning position");
}

#[test]
fn test_generate_optimal_mate_sequence() {
    // Test: Should generate complete sequence of moves leading to mate
    // Expected: Returns move-by-move path to mate with evaluations
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position - should show optimal mate sequence
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let sequence = analyzer.generate_mate_sequence(&position);

    assert!(sequence.is_ok(), "Should generate mate sequence");
    let mate_seq = sequence.unwrap();

    assert_eq!(mate_seq.length(), 10, "Should show 10-move mate sequence");
    assert!(!mate_seq.moves().is_empty(), "Should contain actual moves");
    assert!(mate_seq.is_forced_mate(), "Should indicate forced mate");

    // First move should be the best tablebase move
    let first_move = &mate_seq.moves()[0];
    assert!(
        first_move.evaluation() > 19000,
        "First move should have tablebase mate score"
    );
    assert_eq!(
        first_move.distance_to_mate(),
        10,
        "First move should show DTM=10"
    );
}

#[test]
fn test_mate_sequence_contains_position_evaluations() {
    // Test: Each move in sequence should show evaluation and DTM
    // Expected: Sequence provides move-by-move analysis data
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position for mate sequence analysis
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let sequence = analyzer.generate_mate_sequence(&position).unwrap();

    // Each move should have decreasing DTM
    let moves = sequence.moves();
    for (i, mate_move) in moves.iter().enumerate() {
        let expected_dtm = 10 - i; // DTM should decrease each move
        assert_eq!(
            mate_move.distance_to_mate(),
            expected_dtm,
            "Move {} should have DTM={}",
            i,
            expected_dtm
        );
        assert!(
            mate_move.evaluation() > 19000,
            "Move {} should have mate score",
            i
        );
        assert!(
            mate_move.is_best_move(),
            "Move {} should be marked as best",
            i
        );
    }
}

#[test]
fn test_mate_sequence_shows_both_sides_moves() {
    // Test: Sequence should show optimal moves for both White and Black
    // Expected: Complete game tree showing best defense and attack
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position - should show White's winning moves and Black's best defense
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let sequence = analyzer.generate_mate_sequence(&position).unwrap();
    let moves = sequence.moves();

    // Should have moves for both sides
    assert!(moves.len() >= 2, "Should contain moves for both sides");

    // First move should be White's (side to move)
    assert_eq!(
        moves[0].side_to_move(),
        chess_engine::types::Color::White,
        "First move should be White's"
    );

    // Moves should alternate between sides
    for i in 1..moves.len() {
        let previous_side = moves[i - 1].side_to_move();
        let current_side = moves[i].side_to_move();
        assert_ne!(
            previous_side, current_side,
            "Moves should alternate between sides"
        );
    }
}

#[test]
fn test_distance_to_mate_for_losing_position() {
    // Test: Should calculate DTM for losing positions (opponent mates us)
    // Expected: Returns negative DTM indicating we're getting mated
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position with Black to move (Black is losing)
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 b - - 0 1").unwrap();

    let dtm_result = analyzer.calculate_distance_to_mate(&position);

    assert!(
        dtm_result.is_ok(),
        "Should calculate DTM for losing position"
    );
    let dtm = dtm_result.unwrap();

    assert!(dtm.is_losing(), "Should identify as losing position");
    assert_eq!(
        dtm.result(),
        TablebaseResult::Loss(9),
        "Should show loss in 9 moves"
    );
    assert_eq!(
        dtm.distance(),
        9,
        "Should return distance to mate (being mated)"
    );
}

#[test]
fn test_distance_to_mate_for_drawn_position() {
    // Test: Should identify drawn positions correctly
    // Expected: Returns draw result with no mate distance
    let analyzer = DistanceToMateAnalyzer::new();

    // KBvK position (typically drawn with best play)
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1B1k4 w - - 0 1").unwrap();

    let dtm_result = analyzer.calculate_distance_to_mate(&position);

    assert!(dtm_result.is_ok(), "Should analyze drawn position");
    let dtm = dtm_result.unwrap();

    assert!(dtm.is_draw(), "Should identify as drawn position");
    assert_eq!(
        dtm.result(),
        TablebaseResult::Draw,
        "Should return draw result"
    );
    assert_eq!(dtm.distance(), 0, "Should have no distance to mate in draw");
}

#[test]
fn test_visualize_mate_path_with_board_positions() {
    // Test: Should generate visual representation of mate sequence
    // Expected: Returns formatted output showing board after each move
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position for visualization
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let visualization = analyzer.visualize_mate_path(&position);

    assert!(
        visualization.is_ok(),
        "Should generate mate path visualization"
    );
    let visual = visualization.unwrap();

    assert!(visual.contains("Move 1"), "Should show move numbers");
    assert!(visual.contains("DTM: 10"), "Should show distance to mate");
    assert!(visual.contains("White to move"), "Should show side to move");
    assert!(visual.len() > 100, "Should generate substantial output");
}

#[test]
fn test_mate_sequence_performance() {
    // Test: DTM analysis should be reasonably fast for UI use
    // Expected: Analysis completes within acceptable time for real-time use
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let start = std::time::Instant::now();
    let _sequence = analyzer.generate_mate_sequence(&position).unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "DTM analysis should complete quickly for UI responsiveness"
    );
}

#[test]
fn test_distance_to_mate_with_dtz_considerations() {
    // Test: Should integrate DTZ data for 50-move rule awareness
    // Expected: Analysis considers both DTM and DTZ for complete evaluation
    let analyzer = DistanceToMateAnalyzer::new();

    // Position approaching 50-move rule
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 40 1").unwrap();

    let dtm_result = analyzer.calculate_distance_to_mate(&position);

    assert!(dtm_result.is_ok(), "Should handle DTZ considerations");
    let dtm = dtm_result.unwrap();

    // Should account for 50-move rule in analysis
    assert!(
        dtm.considers_fifty_move_rule(),
        "Should factor in 50-move rule"
    );
    assert!(
        dtm.is_winning(),
        "Should still be winning despite high halfmove clock"
    );
}

#[test]
fn test_interactive_mate_study_mode() {
    // Test: Should support interactive study mode for endgame learning
    // Expected: Can step through mate sequence move by move
    let analyzer = DistanceToMateAnalyzer::new();

    // KQvK position for study mode
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let study_mode = analyzer.create_study_session(&position);

    assert!(study_mode.is_ok(), "Should create study session");
    let mut session = study_mode.unwrap();

    // Should be able to step through moves
    assert!(session.has_next_move(), "Should have moves to study");

    let first_move = session.next_move().unwrap();
    assert!(first_move.is_optimal(), "Should show optimal moves");
    assert!(
        first_move.has_explanation(),
        "Should provide move explanations"
    );

    // Should continue until mate
    let mut move_count = 1;
    while session.has_next_move() {
        let _move = session.next_move().unwrap();
        move_count += 1;
        if move_count > 20 {
            break;
        } // Safety check
    }

    assert!(session.is_mate_reached(), "Study session should reach mate");
    assert_eq!(move_count, 10, "Should have exactly 10 moves to mate");
}
