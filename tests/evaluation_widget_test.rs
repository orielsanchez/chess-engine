use chess_engine::moves::{Move, MoveType};
use chess_engine::search::SearchResult;
use chess_engine::tui::{EvaluationWidget, TuiApp};
use chess_engine::types::Square;

#[test]
fn test_evaluation_widget_creation() {
    let search_result = create_mock_search_result(123, 15);

    let widget = EvaluationWidget::new(&search_result);

    // Should have title
    assert!(widget.title().is_some());
    assert_eq!(widget.title().unwrap(), "Evaluation");

    // Should have borders
    assert!(widget.has_borders());
}

#[test]
fn test_evaluation_widget_positive_score_formatting() {
    let search_result = create_mock_search_result(150, 12); // +1.50

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should format centipaws to decimal
    assert!(content.contains("+1.50"));

    // Should show positive evaluation styling
    assert!(content.contains("Score: +1.50"));
}

#[test]
fn test_evaluation_widget_negative_score_formatting() {
    let search_result = create_mock_search_result(-89, 10); // -0.89

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should format negative centipawns correctly
    assert!(content.contains("-0.89"));
    assert!(content.contains("Score: -0.89"));
}

#[test]
fn test_evaluation_widget_zero_score_formatting() {
    let search_result = create_mock_search_result(0, 8);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should format zero score correctly
    assert!(content.contains("0.00"));
    assert!(content.contains("Score: 0.00"));
}

#[test]
fn test_evaluation_widget_large_score_formatting() {
    let search_result = create_mock_search_result(2500, 15); // +25.00

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should handle large scores
    assert!(content.contains("+25.00"));
    assert!(content.contains("Score: +25.00"));
}

#[test]
fn test_evaluation_widget_depth_display() {
    let search_result = create_mock_search_result(100, 18);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should display search depth
    assert!(content.contains("Depth: 18"));
}

#[test]
fn test_evaluation_widget_best_move_display() {
    let search_result = create_mock_search_result(200, 12);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should display best move (from mock: e2e4)
    assert!(content.contains("Best: e2e4"));
}

#[test]
fn test_evaluation_widget_component_breakdown() {
    let search_result = create_mock_search_result(150, 12);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should show evaluation components
    assert!(content.contains("Material:"));
    assert!(content.contains("Position:"));
    assert!(content.contains("Pawns:"));

    // Component values should be displayed with signs
    assert!(content.contains("+") || content.contains("-"));
}

#[test]
fn test_evaluation_widget_integration_with_position() {
    let app = TuiApp::new().unwrap();
    let search_result = create_mock_search_result(75, 10);

    // Test that widget can be created from TUI app context
    let widget = app.create_evaluation_widget(&search_result);

    // Should have proper styling
    assert!(widget.title().is_some());
    assert!(widget.has_borders());

    let content = widget.content();
    assert!(content.contains("Score:"));
    assert!(content.contains("Depth:"));
}

#[test]
fn test_evaluation_widget_content_formatting() {
    let search_result = create_mock_search_result(100, 12);
    let widget = EvaluationWidget::new(&search_result);

    // Test widget content formatting
    let content = widget.content();

    // Should contain all expected sections
    assert!(content.contains("Score: +1.00"));
    assert!(content.contains("Depth: 12"));
    assert!(content.contains("Best: e2e4"));
    assert!(content.contains("Material:"));
    assert!(content.contains("Position:"));
    assert!(content.contains("Pawns:"));
}

#[test]
fn test_evaluation_widget_dynamic_content_updates() {
    // Test with different search results
    let results = vec![
        create_mock_search_result(50, 8),
        create_mock_search_result(-120, 15),
        create_mock_search_result(0, 12),
        create_mock_search_result(999, 20),
    ];

    for result in results {
        let widget = EvaluationWidget::new(&result);
        let content = widget.content();

        // Each widget should have proper content
        assert!(content.contains("Score:"));
        assert!(content.contains("Depth:"));
        assert!(content.contains("Best:"));

        // Content should update based on search result
        let expected_score = format!("{:.2}", result.evaluation as f64 / 100.0);
        assert!(content.contains(&expected_score));
    }
}

// Phase 3 TDD Tests: Enhanced Evaluation Statistics

#[test]
fn test_evaluation_widget_search_performance_metrics() {
    let search_result = create_mock_search_result_with_metrics(150, 12, 25000, 500);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should display nodes per second calculation (25000 nodes / 500ms = 50000 nps)
    assert!(content.contains("NPS: 50000"));

    // Should display nodes searched
    assert!(content.contains("Nodes: 25000"));

    // Should display search time
    assert!(content.contains("Time: 500ms"));
}

#[test]
fn test_evaluation_widget_advantage_indicators() {
    let test_cases = vec![
        (25, "slight advantage"),     // +0.25
        (100, "advantage"),           // +1.00
        (300, "winning"),             // +3.00
        (-50, "slight disadvantage"), // -0.50
        (-200, "disadvantage"),       // -2.00
        (-500, "losing"),             // -5.00
        (0, "equal"),                 // 0.00
    ];

    for (eval_cp, expected_indicator) in test_cases {
        let search_result = create_mock_search_result(eval_cp, 10);
        let widget = EvaluationWidget::new(&search_result);
        let content = widget.content();

        assert!(
            content.contains(expected_indicator),
            "Expected '{}' for evaluation {}, but content was: {}",
            expected_indicator,
            eval_cp,
            content
        );
    }
}

#[test]
fn test_evaluation_widget_transposition_table_statistics() {
    let search_result = create_mock_search_result(100, 12);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should display TT hit rate as percentage
    assert!(content.contains("TT Hit: 85%"));

    // Should display TT hits and stores
    assert!(content.contains("TT Hits: 8500"));
    assert!(content.contains("TT Stores: 1500"));
}

#[test]
fn test_evaluation_widget_detailed_evaluation_breakdown() {
    let search_result = create_mock_search_result(150, 12);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should show detailed evaluation components with actual values, not placeholders
    assert!(!content.contains("Material: +0.00")); // Should NOT have placeholder values
    assert!(!content.contains("Position: +0.00"));
    assert!(!content.contains("Pawns: +0.00"));

    // Should contain realistic evaluation breakdown
    assert!(content.contains("Material:"));
    assert!(content.contains("Position:"));
    assert!(content.contains("Pawns:"));

    // Should show values that add up to total evaluation
    // This will be implemented to actually calculate the breakdown
}

#[test]
fn test_evaluation_widget_aspiration_window_statistics() {
    let search_result = create_mock_search_result(100, 15);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should display aspiration window failures and re-searches
    assert!(content.contains("Asp Fails: 2"));
    assert!(content.contains("Asp Research: 1"));
    assert!(content.contains("Asp Window: 50"));
}

#[test]
fn test_evaluation_widget_iterative_deepening_progress() {
    let search_result = create_mock_search_result(200, 15);

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should show iterative deepening progress
    assert!(content.contains("Iterations: 15"));
    assert!(content.contains("Target Depth: 15"));
    assert!(content.contains("Completed: 15"));
}

#[test]
fn test_evaluation_widget_time_management_indicators() {
    let mut search_result = create_mock_search_result(100, 12);
    search_result.time_limited = true;

    let widget = EvaluationWidget::new(&search_result);
    let content = widget.content();

    // Should indicate when search was time-limited
    assert!(content.contains("(time limited)"));

    // Test non-time-limited case
    let mut search_result2 = create_mock_search_result(100, 12);
    search_result2.time_limited = false;

    let widget2 = EvaluationWidget::new(&search_result2);
    let content2 = widget2.content();

    assert!(!content2.contains("(time limited)"));
}

fn create_mock_search_result(evaluation: i32, depth: u8) -> SearchResult {
    // Create a simple move (e2e4) for testing
    let from = Square::from_index(12).unwrap(); // e2
    let to = Square::from_index(28).unwrap(); // e4
    let best_move = Move::new(from, to, MoveType::Quiet);

    SearchResult {
        best_move,
        evaluation,
        depth,
        completed_depth: depth,
        nodes_searched: 10000,
        nodes_pruned: 5000,
        time_ms: 100,
        time_limited: false,
        iterations_completed: depth,
        tt_hit_rate: 0.85,
        tt_hits: 8500,
        tt_stores: 1500,
        aspiration_fails: 2,
        aspiration_researches: 1,
        aspiration_window_size: 50,
        principal_variation: vec![best_move],
        dtm_result: None,
        mate_sequence: None,
        used_dtm_ordering: false,
        dtm_analysis_status: "not_attempted".to_string(),
    }
}

fn create_mock_search_result_with_metrics(
    evaluation: i32,
    depth: u8,
    nodes: u64,
    time_ms: u64,
) -> SearchResult {
    let from = Square::from_index(12).unwrap(); // e2
    let to = Square::from_index(28).unwrap(); // e4
    let best_move = Move::new(from, to, MoveType::Quiet);

    SearchResult {
        best_move,
        evaluation,
        depth,
        completed_depth: depth,
        nodes_searched: nodes,
        nodes_pruned: nodes / 2,
        time_ms,
        time_limited: false,
        iterations_completed: depth,
        tt_hit_rate: 0.85,
        tt_hits: (nodes as f64 * 0.85) as u64,
        tt_stores: nodes / 10,
        aspiration_fails: 2,
        aspiration_researches: 1,
        aspiration_window_size: 50,
        principal_variation: vec![best_move],
        dtm_result: None,
        mate_sequence: None,
        used_dtm_ordering: false,
        dtm_analysis_status: "not_attempted".to_string(),
    }
}
