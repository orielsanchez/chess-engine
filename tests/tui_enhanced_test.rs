use chess_engine::tui::{CommandCompletion, CommandHistory, TuiApp};

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

        let completions = completion.complete_command("m");
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
        assert_eq!(completion.expand_alias("m e4"), "move e4");
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
}
