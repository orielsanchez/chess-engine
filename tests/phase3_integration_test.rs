use chess_engine::moves::{Move, MoveType};
use chess_engine::search::SearchResult;
use chess_engine::tui::{EvaluationWidget, LayoutMode, TuiApp};
use chess_engine::types::Square;

#[test]
fn test_phase3_enhanced_evaluation_integration() {
    let mut app = TuiApp::new().unwrap();

    // Create a search result with comprehensive metrics
    let search_result = create_comprehensive_search_result();

    // Set search result in TUI app
    app.set_search_result(Some(search_result.clone()));

    // Manually set to three-panel mode (as would happen with user interaction)
    app.set_layout_mode(LayoutMode::ThreePanelAnalysis);

    // Verify the app is in three-panel mode
    assert_eq!(app.layout_mode(), &LayoutMode::ThreePanelAnalysis);

    // Create evaluation widget and verify Phase 3 features
    let eval_widget = app.create_evaluation_widget(&search_result);
    let content = eval_widget.content();

    // Phase 3 Feature 1: Enhanced evaluation with advantage indicators
    assert!(content.contains("Score: +2.50 (advantage)"));

    // Phase 3 Feature 2: Search performance metrics
    assert!(content.contains("Nodes: 50000"));
    assert!(content.contains("Time: 1000ms"));
    assert!(content.contains("NPS: 50000"));

    // Phase 3 Feature 3: Transposition table statistics
    assert!(content.contains("TT Hit: 80%"));
    assert!(content.contains("TT Hits: 40000"));
    assert!(content.contains("TT Stores: 5000"));

    // Phase 3 Feature 4: Aspiration window statistics
    assert!(content.contains("Asp Fails: 3"));
    assert!(content.contains("Asp Research: 2"));
    assert!(content.contains("Asp Window: 75"));

    // Phase 3 Feature 5: Iterative deepening progress
    assert!(content.contains("Iterations: 14"));
    assert!(content.contains("Target Depth: 15"));
    assert!(content.contains("Completed: 14"));

    // Phase 3 Feature 6: Time management indicators
    assert!(content.contains("(time limited)"));

    // Phase 3 Feature 7: Detailed evaluation breakdown (no longer placeholders)
    assert!(!content.contains("Material: +0.00"));
    assert!(!content.contains("Position: +0.00"));
    assert!(!content.contains("Pawns: +0.00"));
    assert!(content.contains("Material: +1.50"));
    assert!(content.contains("Position: +0.75"));
    assert!(content.contains("Pawns: +0.25"));
}

#[test]
fn test_phase3_three_panel_layout_with_enhanced_analysis() {
    let mut app = TuiApp::new().unwrap();
    let search_result = create_comprehensive_search_result();

    // Test layout without search result (should be two-panel)
    let layout_area = ratatui::layout::Rect::new(0, 0, 100, 50);
    let layout_no_search = app.create_layout(layout_area);
    assert_eq!(layout_no_search.len(), 2); // Two-panel mode

    // Add search result and set three-panel mode
    app.set_search_result(Some(search_result));
    app.set_layout_mode(LayoutMode::ThreePanelAnalysis);
    let layout_with_search = app.create_layout(layout_area);
    assert_eq!(layout_with_search.len(), 3); // Three-panel mode

    // Verify three-panel proportions
    let board_area = layout_with_search[0];
    let command_area = layout_with_search[1];
    let analysis_area = layout_with_search[2];

    // Board should be 50% width
    assert_eq!(board_area.width, 50);

    // Command and analysis should split the remaining 50% vertically
    assert_eq!(command_area.width, 50);
    assert_eq!(analysis_area.width, 50);
    assert_eq!(command_area.height, analysis_area.height);
}

#[test]
fn test_phase3_advantage_indicator_accuracy() {
    let test_cases = vec![
        (25, "slight advantage"),     // +0.25
        (75, "slight advantage"),     // +0.75
        (100, "advantage"),           // +1.00
        (250, "advantage"),           // +2.50
        (300, "winning"),             // +3.00
        (500, "winning"),             // +5.00
        (-50, "slight disadvantage"), // -0.50
        (-150, "disadvantage"),       // -1.50
        (-300, "losing"),             // -3.00
        (0, "equal"),                 // 0.00
    ];

    for (eval_cp, expected_indicator) in test_cases {
        let mut search_result = create_comprehensive_search_result();
        search_result.evaluation = eval_cp;

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
fn test_phase3_nodes_per_second_calculation_accuracy() {
    let test_cases = vec![
        (10000, 1000, 10000),  // 10k nodes / 1s = 10k nps
        (50000, 500, 100000),  // 50k nodes / 0.5s = 100k nps
        (100000, 2000, 50000), // 100k nodes / 2s = 50k nps
        (1000, 100, 10000),    // 1k nodes / 0.1s = 10k nps
    ];

    for (nodes, time_ms, expected_nps) in test_cases {
        let mut search_result = create_comprehensive_search_result();
        search_result.nodes_searched = nodes;
        search_result.time_ms = time_ms;

        let widget = EvaluationWidget::new(&search_result);
        let content = widget.content();

        assert!(
            content.contains(&format!("NPS: {}", expected_nps)),
            "Expected NPS {} for {} nodes in {}ms, but content was: {}",
            expected_nps,
            nodes,
            time_ms,
            content
        );
    }
}

fn create_comprehensive_search_result() -> SearchResult {
    let from = Square::from_index(12).unwrap(); // e2
    let to = Square::from_index(28).unwrap(); // e4
    let best_move = Move::new(from, to, MoveType::Quiet);

    SearchResult {
        best_move,
        evaluation: 250, // +2.50
        depth: 15,
        completed_depth: 14,
        nodes_searched: 50000,
        nodes_pruned: 25000,
        time_ms: 1000,
        time_limited: true,
        iterations_completed: 14,
        tt_hit_rate: 0.80,
        tt_hits: 40000,
        tt_stores: 5000,
        aspiration_fails: 3,
        aspiration_researches: 2,
        aspiration_window_size: 75,
        principal_variation: vec![best_move],
    }
}
