use chess_engine::tui::{
    ClockWidget, CommandCompletion, CommandHistory, MenuWidget, TuiApp, TuiState,
};
use chess_engine::types::Color;

#[cfg(test)]
mod tui_enhanced_tests {
    use super::*;

    #[test]
    fn test_command_completion_basic() {
        let _app = TuiApp::new().unwrap();
        let completion = CommandCompletion::new();

        // Test basic command completion
        let completions = completion.complete_command("a");
        assert!(completions.contains(&"analyze".to_string()));

        let completions = completion.complete_command("l");
        assert!(completions.contains(&"legal".to_string()));

        let completions = completion.complete_command("mov");
        assert!(completions.contains(&"move".to_string()));

        let completions = completion.complete_command("h");
        assert!(completions.contains(&"help".to_string()));
    }

    #[test]
    fn test_command_completion_exact_match() {
        let completion = CommandCompletion::new();

        // Test exact matches
        let completions = completion.complete_command("analyze");
        assert_eq!(completions, vec!["analyze"]);

        let completions = completion.complete_command("legal");
        assert_eq!(completions, vec!["legal"]);
    }

    #[test]
    fn test_command_completion_aliases() {
        let completion = CommandCompletion::new();

        // Test aliases
        let completions = completion.complete_command("a");
        assert!(completions.contains(&"analyze".to_string()));

        let completions = completion.complete_command("l");
        assert!(completions.contains(&"legal".to_string()));
    }

    #[test]
    fn test_move_completion() {
        let app = TuiApp::new().unwrap();
        let completion = CommandCompletion::new();

        // Test move completion for starting position (using coordinate notation)
        let completions = completion.complete_move(&app, "e");
        assert!(completions.contains(&"e2e4".to_string()));
        assert!(completions.contains(&"e2e3".to_string()));

        let completions = completion.complete_move(&app, "g1");
        assert!(completions.contains(&"g1f3".to_string()));
        assert!(completions.contains(&"g1h3".to_string()));
    }

    #[test]
    fn test_command_history_basic() {
        let mut history = CommandHistory::new();

        // Add commands to history
        history.add_command("analyze".to_string());
        history.add_command("legal".to_string());
        history.add_command("move e4".to_string());

        // Test history navigation
        assert_eq!(history.get_previous(), Some("move e4".to_string()));
        assert_eq!(history.get_previous(), Some("legal".to_string()));
        assert_eq!(history.get_previous(), Some("analyze".to_string()));
        assert_eq!(history.get_previous(), None); // No more history

        // Test forward navigation
        assert_eq!(history.get_next(), Some("legal".to_string()));
        assert_eq!(history.get_next(), Some("move e4".to_string()));
        assert_eq!(history.get_next(), None); // At end
    }

    #[test]
    fn test_command_history_limit() {
        let mut history = CommandHistory::new();

        // Add more than 50 commands
        for i in 0..60 {
            history.add_command(format!("command_{}", i));
        }

        // Should only keep last 50
        assert_eq!(history.len(), 50);
        assert_eq!(history.get_previous(), Some("command_59".to_string()));
    }

    #[test]
    fn test_smart_alias_expansion() {
        let completion = CommandCompletion::new();

        // Test alias expansion
        assert_eq!(completion.expand_alias("a"), "analyze");
        assert_eq!(completion.expand_alias("l"), "legal");
        assert_eq!(completion.expand_alias("p fen"), "position fen");
        assert_eq!(completion.expand_alias("analyze"), "analyze"); // No change for full command
    }

    #[test]
    fn test_natural_move_input() {
        let app = TuiApp::new().unwrap();

        // Test coordinate notation parsing (what Move::from_algebraic actually expects)
        let result = app.parse_natural_move("e2e4");
        assert!(result.is_ok());

        let result = app.parse_natural_move("g1f3");
        assert!(result.is_ok());

        let result = app.parse_natural_move("e1g1"); // Castling in coordinate notation
        assert!(result.is_ok());

        // Test invalid move
        let result = app.parse_natural_move("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_tab_completion_integration() {
        let mut app = TuiApp::new().unwrap();

        // Test tab completion integration
        app.set_command_buffer("a".to_string());
        let completed = app.handle_tab_completion();
        assert!(completed);
        assert_eq!(app.command_buffer(), "analyze");

        // Test partial completion
        app.set_command_buffer("m ".to_string());
        let completed = app.handle_tab_completion();
        // Should show legal moves for completion
        assert!(completed);
    }

    #[test]
    fn test_cursor_movement() {
        let mut app = TuiApp::new().unwrap();
        app.set_command_buffer("analyze".to_string());

        // Test cursor movement
        app.move_cursor_left();
        app.move_cursor_left();
        assert_eq!(app.cursor_position(), 5);

        app.move_cursor_right();
        assert_eq!(app.cursor_position(), 6);

        // Test insertion at cursor position
        app.insert_char_at_cursor('d');
        assert_eq!(app.command_buffer(), "analyzde");
    }

    #[test]
    fn test_clock_widget_no_game() {
        let widget = ClockWidget::new(None, Color::White, None);
        let content = widget.content();
        assert_eq!(content, "No active game");
    }

    #[test]
    fn test_clock_widget_with_game() {
        // 5 minutes = 300,000 ms for both players
        let clock_data = Some((300000, 300000));
        let widget = ClockWidget::new(clock_data, Color::White, Some(std::time::Instant::now()));
        let content = widget.content();
        assert_eq!(content, "W: 5:00 | B: 5:00");
    }

    #[test]
    fn test_clock_widget_different_times() {
        // White: 4:23 = 263,000 ms, Black: 5:17 = 317,000 ms
        let clock_data = Some((263000, 317000));
        let widget = ClockWidget::new(clock_data, Color::White, Some(std::time::Instant::now()));
        let content = widget.content();
        assert_eq!(content, "W: 4:23 | B: 5:17");
    }

    #[test]
    fn test_clock_widget_under_minute() {
        // White: 0:45 = 45,000 ms, Black: 0:03 = 3,000 ms
        let clock_data = Some((45000, 3000));
        let widget = ClockWidget::new(clock_data, Color::White, Some(std::time::Instant::now()));
        let content = widget.content();
        assert_eq!(content, "W: 0:45 | B: 0:03");
    }

    #[test]
    fn test_clock_widget_zero_time() {
        // Both players at 0:00
        let clock_data = Some((0, 0));
        let widget = ClockWidget::new(clock_data, Color::White, Some(std::time::Instant::now()));
        let content = widget.content();
        assert_eq!(content, "W: 0:00 | B: 0:00");
    }

    #[test]
    fn test_clock_widget_has_title() {
        let widget = ClockWidget::new(None, Color::White, None);
        assert_eq!(widget.title(), None); // Clock widget doesn't need a title
    }

    #[test]
    fn test_clock_widget_no_borders() {
        let widget = ClockWidget::new(None, Color::White, None);
        assert!(!widget.has_borders()); // Clock widget should not have borders
    }

    #[test]
    fn test_clock_integration_with_game_start() {
        let mut app = TuiApp::new().unwrap();

        // Initially no clock should be active
        let clock_widget = app.create_clock_widget();
        assert_eq!(clock_widget.content(), "No active game");

        // Start an engine game
        app.start_engine_game(chess_engine::types::Color::White, 5);

        // Clock should now be active with 5:00 for both players
        let clock_widget = app.create_clock_widget();
        assert_eq!(clock_widget.content(), "W: 5:00 | B: 5:00");

        // Verify game clock is set in game state
        assert!(app.get_game_clock().is_some());
        let (white_time, black_time) = app.get_game_clock().unwrap();
        assert_eq!(white_time, 300000); // 5 minutes in milliseconds
        assert_eq!(black_time, 300000);
    }

    #[test]
    fn test_menu_widget_creation() {
        let widget = MenuWidget::new();
        assert!(widget.title().is_some());
        assert!(widget.has_borders());
    }

    #[test]
    fn test_menu_widget_content() {
        let widget = MenuWidget::new();
        let content = widget.content();

        // Menu should contain key options
        assert!(content.contains("[1]"));
        assert!(content.contains("Quick Game"));
        assert!(content.contains("[2]"));
        assert!(content.contains("Puzzle"));
        assert!(content.contains("[3]"));
        assert!(content.contains("Analysis"));
        assert!(content.contains("ESC"));
    }

    #[test]
    fn test_tui_state_menu_transitions() {
        let mut app = TuiApp::new().unwrap();

        // Should start in Command state
        assert_eq!(app.get_state(), TuiState::Command);

        // Can transition to Menu state
        app.set_tui_state(TuiState::Menu);
        assert_eq!(app.get_state(), TuiState::Menu);

        // Can transition back to Command state
        app.set_tui_state(TuiState::Command);
        assert_eq!(app.get_state(), TuiState::Command);
    }

    #[test]
    fn test_menu_quick_game_action() {
        let mut app = TuiApp::new().unwrap();

        // Initially no game active
        assert!(app.get_game_clock().is_none());

        // Simulate menu quick game action
        app.handle_menu_quick_game();

        // Should start engine game and transition to GamePlay state
        assert!(app.get_game_clock().is_some());
        assert_eq!(app.get_state(), TuiState::GamePlay);

        // Should be playing as white against engine
        match app.get_game_mode() {
            chess_engine::tui::GameMode::PlayVsEngine {
                player_color,
                difficulty,
            } => {
                assert_eq!(player_color, chess_engine::types::Color::White);
                assert_eq!(difficulty, 5); // Default difficulty
            }
            _ => panic!("Expected PlayVsEngine mode"),
        }
    }

    #[test]
    fn test_full_user_workflow_simulation() {
        let mut app = TuiApp::new().unwrap();

        // 1. Initial state - should be Command mode, no clock
        assert_eq!(app.state(), &TuiState::Command);
        assert!(app.get_game_clock().is_none());
        let clock_widget = app.create_clock_widget();
        assert_eq!(clock_widget.content(), "No active game");

        // 2. Open menu with 'ESC' key simulation
        app.set_state(TuiState::Menu);
        assert_eq!(app.state(), &TuiState::Menu);

        // 3. Menu should be visible and have correct content
        let menu_widget = app.create_menu_widget();
        let menu_content = menu_widget.content();
        assert!(menu_content.contains("[1] Quick Game"));
        assert!(menu_content.contains("Press number key or ESC"));

        // 4. Select option 1 (Quick Game)
        app.handle_menu_quick_game();

        // 5. Should now be in GamePlay state with active clock
        assert_eq!(app.state(), &TuiState::GamePlay);
        assert!(app.get_game_clock().is_some());

        // 6. Clock should now show game time
        let clock_widget = app.create_clock_widget();
        assert_eq!(clock_widget.content(), "W: 5:00 | B: 5:00");

        // 7. Verify complete game setup
        match app.get_game_mode() {
            chess_engine::tui::GameMode::PlayVsEngine {
                player_color,
                difficulty,
            } => {
                assert_eq!(player_color, chess_engine::types::Color::White);
                assert_eq!(difficulty, 5);
            }
            _ => panic!("Expected PlayVsEngine mode after quick game"),
        }
    }

    #[test]
    fn test_direct_move_input_detection() {
        let app = TuiApp::new().unwrap();

        // Coordinate notation should be detected as moves
        assert!(app.is_move_input("e2e4"));
        assert!(app.is_move_input("g1f3"));
        assert!(app.is_move_input("a1h8"));

        // Algebraic notation should be detected as moves
        assert!(app.is_move_input("e4"));
        assert!(app.is_move_input("Nf3"));
        assert!(app.is_move_input("Qh5"));
        assert!(app.is_move_input("Bb5"));

        // Castling should be detected as moves
        assert!(app.is_move_input("O-O"));
        assert!(app.is_move_input("O-O-O"));
        assert!(app.is_move_input("0-0"));
        assert!(app.is_move_input("0-0-0"));

        // Commands should NOT be detected as moves
        assert!(!app.is_move_input("help"));
        assert!(!app.is_move_input("analyze"));
        assert!(!app.is_move_input("legal"));
        assert!(!app.is_move_input("position"));
        assert!(!app.is_move_input("move e4")); // Full command

        // Empty or invalid should not be moves
        assert!(!app.is_move_input(""));
        assert!(!app.is_move_input("   "));
        assert!(!app.is_move_input("xyz"));
        assert!(!app.is_move_input("123"));
    }

    #[test]
    fn test_menu_quit_option() {
        let widget = MenuWidget::new();
        let content = widget.content();

        // Menu should contain quit option
        assert!(content.contains("[5]"));
        assert!(content.contains("Quit"));
    }

    #[test]
    fn test_menu_quit_handler() {
        let mut app = TuiApp::new().unwrap();

        // Quit handler should return true to signal quit
        assert!(app.handle_menu_quit());
    }
}
