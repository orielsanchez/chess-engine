use chess_engine::interactive::{InteractiveCommand, InteractiveEngine, InteractiveResponse};
use chess_engine::tui::TuiApp;

#[test]
fn test_phase4_command_parsing_and_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = TuiApp::new()?;

    // Test that TuiApp can handle Phase 4 commands directly

    // Test play command parsing
    let play_cmd = InteractiveEngine::parse_command("play white 5")?;
    let response = app.handle_command_with_phase4_test(play_cmd)?;

    // Verify response is correct type
    if let InteractiveResponse::GameStarted { mode, player_color } = response {
        assert!(mode.contains("Engine"));
        assert_eq!(player_color, "white");
    } else {
        panic!("Expected GameStarted response");
    }

    // Verify game state changed correctly
    let game_state = app.get_game_state();
    if let chess_engine::tui::GameMode::PlayVsEngine {
        difficulty,
        player_color,
    } = &game_state.mode
    {
        assert_eq!(difficulty, &5);
        assert_eq!(player_color, &chess_engine::types::Color::White);
    } else {
        panic!("Expected PlayVsEngine mode");
    }

    // Test puzzle command
    let puzzle_cmd = InteractiveEngine::parse_command("puzzle test_puzzle")?;
    let response = app.handle_command_with_phase4_test(puzzle_cmd)?;

    if let InteractiveResponse::PuzzleLoaded {
        objective: _,
        puzzle_id,
    } = response
    {
        assert_eq!(puzzle_id, "test_puzzle");
    } else {
        panic!("Expected PuzzleLoaded response");
    }

    // Verify puzzle state changed
    let game_state = app.get_game_state();
    if let chess_engine::tui::GameMode::PuzzleSolving { puzzle_id } = &game_state.mode {
        assert_eq!(puzzle_id, "test_puzzle");
    } else {
        panic!("Expected PuzzleSolving mode");
    }

    // Test simple commands
    let threats_cmd = InteractiveEngine::parse_command("threats")?;
    let response = app.handle_command_with_phase4_test(threats_cmd)?;

    if let InteractiveResponse::ThreatsFound {
        threat_count: _,
        threats: _,
    } = response
    {
        // Good, we got the expected response type
    } else {
        panic!("Expected ThreatsFound response");
    }

    let hint_cmd = InteractiveEngine::parse_command("hint")?;
    let response = app.handle_command_with_phase4_test(hint_cmd)?;

    if let InteractiveResponse::PuzzleHint { hint: _ } = response {
        // Good, we got the expected response type
    } else {
        panic!("Expected PuzzleHint response");
    }

    let clock_cmd = InteractiveEngine::parse_command("clock")?;
    let response = app.handle_command_with_phase4_test(clock_cmd)?;

    if let InteractiveResponse::GameClock {
        white_time_ms: _,
        black_time_ms: _,
    } = response
    {
        // Good, we got the expected response type
    } else {
        panic!("Expected GameClock response");
    }

    Ok(())
}

#[test]
fn test_phase4_command_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test invalid play command
    let result = InteractiveEngine::parse_command("play invalid 5");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Color must be"));

    // Test invalid difficulty
    let result = InteractiveEngine::parse_command("play white 15");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("between 1 and 10"));

    // Test puzzle command without ID
    let result = InteractiveEngine::parse_command("puzzle");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires puzzle ID"));

    Ok(())
}

#[test]
fn test_phase4_help_integration() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = InteractiveEngine::new()?;
    let help_cmd = InteractiveCommand::Help;
    let response = engine.handle_command(help_cmd)?;

    if let InteractiveResponse::Help { commands } = response {
        // Verify all Phase 4 commands are included
        assert!(commands.contains("play"));
        assert!(commands.contains("puzzle"));
        assert!(commands.contains("threats"));
        assert!(commands.contains("hint"));
        assert!(commands.contains("clock"));
    } else {
        panic!("Expected Help response");
    }

    Ok(())
}
