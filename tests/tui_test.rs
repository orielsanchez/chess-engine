use chess_engine::Move;
use chess_engine::position::Position;
use chess_engine::search::SearchResult;
use chess_engine::tui::{LayoutMode, TuiApp, TuiState};
use ratatui::layout::Rect;

#[test]
fn test_tui_app_creation() {
    let app = TuiApp::new().unwrap();

    // Should start with starting position
    let expected_position = Position::starting_position().unwrap();
    assert_eq!(app.position(), &expected_position);

    // Should start in command mode
    assert!(matches!(app.state(), TuiState::Command));

    // Should have empty command buffer
    assert_eq!(app.command_buffer(), "");
}

#[test]
fn test_tui_state_transitions() {
    let mut app = TuiApp::new().unwrap();

    // Start in command mode
    assert!(matches!(app.state(), TuiState::Command));

    // Switch to board mode
    app.set_state(TuiState::Board);
    assert!(matches!(app.state(), TuiState::Board));

    // Switch back to command mode
    app.set_state(TuiState::Command);
    assert!(matches!(app.state(), TuiState::Command));
}

#[test]
fn test_command_buffer_management() {
    let mut app = TuiApp::new().unwrap();

    // Initially empty
    assert_eq!(app.command_buffer(), "");

    // Add characters
    app.add_char('a');
    assert_eq!(app.command_buffer(), "a");

    app.add_char('n');
    app.add_char('a');
    app.add_char('l');
    app.add_char('y');
    app.add_char('z');
    app.add_char('e');
    assert_eq!(app.command_buffer(), "analyze");

    // Remove character
    app.remove_char();
    assert_eq!(app.command_buffer(), "analyz");

    // Clear buffer
    app.clear_command_buffer();
    assert_eq!(app.command_buffer(), "");
}

#[test]
fn test_layout_constraints() {
    let app = TuiApp::new().unwrap();

    // Test terminal size constraints
    let terminal_rect = Rect::new(0, 0, 100, 30);
    let layout = app.create_layout(terminal_rect);

    // Should have two areas: board and command
    assert_eq!(layout.len(), 2);

    // Board area should be larger (left side)
    let board_area = layout[0];
    let command_area = layout[1];

    // Board and command areas should have equal width (50/50 split)
    assert_eq!(board_area.width, command_area.width);

    // Both should have full height
    assert_eq!(board_area.height, terminal_rect.height);
    assert_eq!(command_area.height, terminal_rect.height);
}

#[test]
fn test_board_widget_creation() {
    let app = TuiApp::new().unwrap();
    let position = Position::starting_position().unwrap();

    let widget = app.create_board_widget(&position);

    // Should have title
    assert!(widget.title().is_some());

    // Should have borders
    assert!(widget.has_borders());
}

#[test]
fn test_command_widget_creation() {
    let mut app = TuiApp::new().unwrap();

    app.add_char('h');
    app.add_char('e');
    app.add_char('l');
    app.add_char('p');

    let widget = app.create_command_widget();

    // Should have title
    assert!(widget.title().is_some());

    // Should have borders
    assert!(widget.has_borders());

    // Should contain command buffer content
    let content = widget.content();
    assert!(content.contains("help"));
}

#[test]
fn test_position_updates() {
    let mut app = TuiApp::new().unwrap();

    // Initially starting position
    let starting_position = Position::starting_position().unwrap();
    assert_eq!(app.position(), &starting_position);

    // Update to new position
    let new_position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
    app.update_position(new_position.clone());

    assert_eq!(app.position(), &new_position);
}

#[test]
fn test_command_execution() {
    let mut app = TuiApp::new().unwrap();

    // Set up analyze command
    app.add_char('a');
    app.add_char('n');
    app.add_char('a');
    app.add_char('l');
    app.add_char('y');
    app.add_char('z');
    app.add_char('e');

    // Execute command
    let result = app.execute_command();

    // Should succeed
    assert!(result.is_ok());

    // Command buffer should be cleared after execution
    assert_eq!(app.command_buffer(), "");
}

#[test]
fn test_split_pane_layout_proportions() {
    let app = TuiApp::new().unwrap();

    // Test various terminal sizes
    let test_sizes = vec![
        (80, 24),  // Standard terminal
        (120, 30), // Wide terminal
        (100, 40), // Tall terminal
        (160, 50), // Large terminal
    ];

    for (width, height) in test_sizes {
        let rect = Rect::new(0, 0, width, height);
        let layout = app.create_layout(rect);

        let board_area = layout[0];
        let command_area = layout[1];

        // Board should get approximately 50% of width (equal split)
        let board_ratio = board_area.width as f32 / width as f32;
        assert!(
            (0.49..=0.51).contains(&board_ratio),
            "Board ratio {} not in range 0.49-0.51 for size {}x{}",
            board_ratio,
            width,
            height
        );

        // Areas should not overlap
        assert!(board_area.x + board_area.width <= command_area.x);

        // Areas should cover full height
        assert_eq!(board_area.height, height);
        assert_eq!(command_area.height, height);
    }
}

#[test]
fn test_three_panel_layout() {
    let app = TuiApp::new().unwrap();
    let terminal_rect = Rect::new(0, 0, 120, 30);

    // Test three-panel layout mode
    let layout = app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, terminal_rect);

    // Should have three areas: board, command, and analysis
    assert_eq!(layout.len(), 3);

    let board_area = layout[0];
    let command_area = layout[1];
    let analysis_area = layout[2];

    // Board should take 50% of width (left side)
    let board_ratio = board_area.width as f32 / terminal_rect.width as f32;
    assert!(
        (0.48..=0.52).contains(&board_ratio),
        "Board ratio {} not in range 0.48-0.52",
        board_ratio
    );

    // Command and analysis should each take 50% of right side height
    let right_side_height = terminal_rect.height;
    let command_ratio = command_area.height as f32 / right_side_height as f32;
    assert!(
        (0.48..=0.52).contains(&command_ratio),
        "Command ratio {} not in range 0.48-0.52",
        command_ratio
    );

    // Analysis should take 50% of right side height
    let analysis_ratio = analysis_area.height as f32 / right_side_height as f32;
    assert!(
        (0.48..=0.52).contains(&analysis_ratio),
        "Analysis ratio {} not in range 0.48-0.52",
        analysis_ratio
    );
}

#[test]
fn test_layout_mode_enum() {
    let app = TuiApp::new().unwrap();
    let terminal_rect = Rect::new(0, 0, 100, 30);

    // Test two-panel classic mode (existing behavior)
    let classic_layout = app.create_layout_with_mode(LayoutMode::TwoPanelClassic, terminal_rect);
    assert_eq!(classic_layout.len(), 2);

    // Should match existing create_layout behavior
    let existing_layout = app.create_layout(terminal_rect);
    assert_eq!(classic_layout, existing_layout);

    // Test three-panel analysis mode
    let analysis_layout =
        app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, terminal_rect);
    assert_eq!(analysis_layout.len(), 3);

    // Layouts should be different
    assert_ne!(classic_layout.len(), analysis_layout.len());
}

#[test]
fn test_minimum_terminal_size_three_panel() {
    let app = TuiApp::new().unwrap();
    let small_rect = Rect::new(0, 0, 80, 24); // Minimum terminal size

    let layout = app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, small_rect);

    // Should still create 3 panels even on small terminal
    assert_eq!(layout.len(), 3);

    let board_area = layout[0];
    let command_area = layout[1];
    let analysis_area = layout[2];

    // All areas should have positive dimensions
    assert!(board_area.width > 0);
    assert!(board_area.height > 0);
    assert!(command_area.width > 0);
    assert!(command_area.height > 0);
    assert!(analysis_area.width > 0);
    assert!(analysis_area.height > 0);

    // Board should still get reasonable space
    assert!(board_area.width >= 35); // At least 35 columns for board
}

#[test]
fn test_responsive_three_panel_layout() {
    let app = TuiApp::new().unwrap();

    let test_sizes = vec![
        (80, 24),  // Minimum
        (100, 30), // Standard
        (120, 40), // Large
        (160, 50), // Extra large
    ];

    for (width, height) in test_sizes {
        let rect = Rect::new(0, 0, width, height);
        let layout = app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, rect);

        assert_eq!(layout.len(), 3);

        let board_area = layout[0];
        let command_area = layout[1];
        let analysis_area = layout[2];

        // Board proportions should be consistent across sizes
        let board_ratio = board_area.width as f32 / width as f32;
        assert!(
            (0.48..=0.52).contains(&board_ratio),
            "Board ratio {} inconsistent for size {}x{}",
            board_ratio,
            width,
            height
        );

        // Right side panels should not overlap
        assert!(command_area.y + command_area.height <= analysis_area.y);

        // All panels should fit within terminal
        assert!(board_area.x + board_area.width <= width);
        assert!(command_area.x + command_area.width <= width);
        assert!(analysis_area.x + analysis_area.width <= width);
        assert!(analysis_area.y + analysis_area.height <= height);
    }
}

// TDD Tests for PrincipalVariationWidget Integration

#[test]
fn test_three_panel_analysis_rendering_with_search_result() {
    let mut app = TuiApp::new().unwrap();

    // Create mock SearchResult with principal variation
    let mock_move = Move::from_algebraic("e2e4").unwrap();
    let mut search_result = SearchResult::new(mock_move, 50, 15);
    search_result.principal_variation = vec![mock_move];
    search_result.nodes_searched = 1000000;
    search_result.time_ms = 2000;

    // Store search result in app (this should fail initially)
    app.set_search_result(Some(search_result.clone()));

    // Create principal variation widget
    let pv_widget = app.create_principal_variation_widget(&search_result);

    // Should have title and borders
    assert!(pv_widget.title().is_some());
    assert!(pv_widget.has_borders());

    // Should have title and content
    assert_eq!(pv_widget.title().unwrap(), "Principal Variation");
    let content = pv_widget.content();
    assert!(content.contains("Depth: 15"));
    assert!(content.contains("1. e4"));
}

#[test]
fn test_three_panel_render_with_analysis_widgets() {
    let mut app = TuiApp::new().unwrap();

    // Create mock SearchResult
    let mock_move = Move::from_algebraic("e2e4").unwrap();
    let mut search_result = SearchResult::new(mock_move, 75, 12);
    search_result.principal_variation = vec![mock_move];
    search_result.nodes_searched = 500000;
    search_result.time_ms = 1500;

    // Set search result and switch to three-panel mode
    app.set_search_result(Some(search_result));
    app.set_layout_mode(LayoutMode::ThreePanelAnalysis);

    // Test that render method handles three-panel layout
    // This should fail initially since render() doesn't handle analysis panel
    let terminal_rect = Rect::new(0, 0, 120, 30);
    let layout = app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, terminal_rect);

    // Should have 3 panels when in analysis mode with search results
    assert_eq!(layout.len(), 3);

    // Analysis panel should be available for rendering
    let analysis_area = layout[2];
    assert!(analysis_area.width > 0);
    assert!(analysis_area.height > 0);
}

#[test]
fn test_fallback_when_no_search_results() {
    let app = TuiApp::new().unwrap();

    // No search results set initially
    // App should handle case when no search results available

    // Should fall back to two-panel mode or show empty analysis
    let terminal_rect = Rect::new(0, 0, 120, 30);
    let layout = app.create_layout(terminal_rect);

    // Behavior when no search results available
    // Could be 2 panels (fallback) or 3 panels with empty analysis
    assert!(layout.len() >= 2);
}

#[test]
fn test_principal_variation_widget_integration() {
    let mut app = TuiApp::new().unwrap();

    // Create SearchResult with multi-move principal variation
    let move1 = Move::from_algebraic("e2e4").unwrap();
    let move2 = Move::from_algebraic("e7e5").unwrap();
    let mut search_result = SearchResult::new(move1, 25, 10);
    search_result.principal_variation = vec![move1, move2];
    search_result.nodes_searched = 250000;
    search_result.time_ms = 1000;

    app.set_search_result(Some(search_result.clone()));

    // Create PV widget and verify content
    let pv_widget = app.create_principal_variation_widget(&search_result);
    let content = pv_widget.content();

    // Should show formatted principal variation
    assert_eq!(pv_widget.title().unwrap(), "Principal Variation");
    assert!(content.contains("Depth: 10"));
    assert!(content.contains("1. e4    e5")); // Formatted move pair
}

#[test]
fn test_layout_mode_switching_with_analysis() {
    let app = TuiApp::new().unwrap();

    // Test different layout modes using create_layout_with_mode
    let terminal_rect = Rect::new(0, 0, 120, 30);

    // Two-panel mode
    let classic_layout = app.create_layout_with_mode(LayoutMode::TwoPanelClassic, terminal_rect);
    assert_eq!(classic_layout.len(), 2);

    // Three-panel analysis mode
    let analysis_layout =
        app.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, terminal_rect);
    assert_eq!(analysis_layout.len(), 3);

    // Should be different
    assert_ne!(classic_layout.len(), analysis_layout.len());
}
