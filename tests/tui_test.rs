use chess_engine::position::Position;
use chess_engine::tui::{TuiApp, TuiState};
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

    // Board should take most of the width
    assert!(board_area.width > command_area.width);

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

        // Board should get approximately 60-70% of width
        let board_ratio = board_area.width as f32 / width as f32;
        assert!(
            (0.6..=0.7).contains(&board_ratio),
            "Board ratio {} not in range 0.6-0.7 for size {}x{}",
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
