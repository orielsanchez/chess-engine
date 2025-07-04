use chess_engine::moves::{Move, MoveType};
use chess_engine::search::SearchResult;
use chess_engine::tui::{PrincipalVariationWidget, TuiApp};
use chess_engine::types::Square;

#[test]
fn test_principal_variation_widget_creation() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5"], 12);

    let widget = PrincipalVariationWidget::new(&search_result);

    // Should have title
    assert!(widget.title().is_some());
    assert_eq!(widget.title().unwrap(), "Principal Variation");

    // Should have borders
    assert!(widget.has_borders());
}

#[test]
fn test_principal_variation_widget_single_move_display() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4"], 8);

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should display depth
    assert!(content.contains("Depth: 8"));

    // Should display single move
    assert!(content.contains("1. e4"));
}

#[test]
fn test_principal_variation_widget_multiple_moves_display() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5", "g1f3", "b8c6"], 15);

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should display depth
    assert!(content.contains("Depth: 15"));

    // Should display moves in proper format
    assert!(content.contains("1. e4    e5"));
    assert!(content.contains("2. Nf3    Nc6"));
}

#[test]
fn test_principal_variation_widget_long_line_display() {
    let search_result = create_mock_search_result_with_pv(
        vec![
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6",
        ],
        20,
    );

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should display first several moves
    assert!(content.contains("1. e4    e5"));
    assert!(content.contains("2. Nf3    Nc6"));
    assert!(content.contains("3. Bb5    a6"));
    assert!(content.contains("4. b5a4    Nf6"));
}

#[test]
fn test_principal_variation_widget_odd_number_moves() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5", "g1f3"], 12);

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should handle odd number of moves correctly
    assert!(content.contains("1. e4    e5"));
    assert!(content.contains("2. Nf3"));
}

#[test]
fn test_principal_variation_widget_empty_pv() {
    let search_result = create_mock_search_result_with_pv(vec![], 5);

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should handle empty PV gracefully
    assert!(content.contains("Depth: 5"));
    assert!(content.contains("No variation"));
}

#[test]
fn test_principal_variation_widget_formatting_layout() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5"], 10);

    let widget = PrincipalVariationWidget::new(&search_result);
    let content = widget.content();

    // Should have proper formatting structure
    assert!(content.contains("Depth:"));
    assert!(content.lines().count() >= 3); // Header + moves + spacing
}

#[test]
fn test_principal_variation_widget_integration_with_tui_app() {
    let app = TuiApp::new().unwrap();
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5"], 12);

    // Test that widget can be created from TUI app context
    let widget = app.create_principal_variation_widget(&search_result);

    // Should have proper styling
    assert!(widget.title().is_some());
    assert!(widget.has_borders());

    let content = widget.content();
    assert!(content.contains("Depth:"));
    assert!(content.contains("1. e4"));
}

#[test]
fn test_principal_variation_widget_content_structure() {
    let search_result = create_mock_search_result_with_pv(vec!["e2e4", "e7e5", "g1f3"], 15);
    let widget = PrincipalVariationWidget::new(&search_result);

    let content = widget.content();

    // Should contain expected sections
    assert!(content.contains("Depth: 15"));
    assert!(content.contains("1. e4    e5"));
    assert!(content.contains("2. Nf3"));

    // Should be well-formatted
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 3); // Depth line + at least 2 move lines
}

#[test]
fn test_principal_variation_widget_dynamic_content_updates() {
    // Test with different PV lengths
    let test_cases = vec![
        (vec!["e2e4"], "1. e4"),
        (vec!["e2e4", "e7e5"], "1. e4    e5"),
        (vec!["d2d4", "d7d5", "c2c4"], "1. d4    d5\n2. c4"),
    ];

    for (moves, expected) in test_cases {
        let search_result = create_mock_search_result_with_pv(moves, 12);
        let widget = PrincipalVariationWidget::new(&search_result);
        let content = widget.content();

        // Content should contain expected move sequence
        assert!(
            content.contains(expected),
            "Failed for moves: {:?}",
            search_result.principal_variation
        );
    }
}

fn create_mock_search_result_with_pv(move_notations: Vec<&str>, depth: u8) -> SearchResult {
    let mut principal_variation = Vec::new();

    // Convert string notations to Move objects
    for notation in move_notations {
        let move_obj = parse_coordinate_notation(notation);
        principal_variation.push(move_obj);
    }

    // Default best move (first move in PV or e2e4)
    let best_move = if principal_variation.is_empty() {
        let from = Square::from_index(12).unwrap(); // e2
        let to = Square::from_index(28).unwrap(); // e4
        Move::new(from, to, MoveType::Quiet)
    } else {
        principal_variation[0]
    };

    SearchResult {
        best_move,
        evaluation: 50, // +0.50
        depth,
        completed_depth: depth,
        nodes_searched: 15000,
        nodes_pruned: 7500,
        time_ms: 150,
        time_limited: false,
        iterations_completed: depth,
        tt_hit_rate: 0.80,
        tt_hits: 12000,
        tt_stores: 3000,
        aspiration_fails: 1,
        aspiration_researches: 0,
        aspiration_window_size: 50,
        principal_variation,
    }
}

fn parse_coordinate_notation(notation: &str) -> Move {
    // Parse coordinate notation like "e2e4" into Move
    let from_str = &notation[0..2];
    let to_str = &notation[2..4];

    let from = Square::from_algebraic(from_str).unwrap();
    let to = Square::from_algebraic(to_str).unwrap();

    Move::new(from, to, MoveType::Quiet)
}
