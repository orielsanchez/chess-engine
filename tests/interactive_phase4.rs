use chess_engine::interactive::{InteractiveCommand, InteractiveEngine, InteractiveResponse};

#[test]
fn test_play_command_parsing() {
    let result = InteractiveEngine::parse_command("play white 5");
    assert!(result.is_ok());
    if let Ok(InteractiveCommand::Play {
        player_color,
        difficulty,
    }) = result
    {
        assert_eq!(player_color, "white");
        assert_eq!(difficulty, 5);
    } else {
        panic!("Expected Play command");
    }

    let result = InteractiveEngine::parse_command("play black 10");
    assert!(result.is_ok());
    if let Ok(InteractiveCommand::Play {
        player_color,
        difficulty,
    }) = result
    {
        assert_eq!(player_color, "black");
        assert_eq!(difficulty, 10);
    } else {
        panic!("Expected Play command");
    }
}

#[test]
fn test_play_command_validation() {
    // Invalid color
    let result = InteractiveEngine::parse_command("play red 5");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Color must be"));

    // Invalid difficulty
    let result = InteractiveEngine::parse_command("play white 15");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("between 1 and 10"));

    // Wrong number of arguments
    let result = InteractiveEngine::parse_command("play white");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("requires color and difficulty")
    );
}

#[test]
fn test_puzzle_command_parsing() {
    let result = InteractiveEngine::parse_command("puzzle mate_in_2_001");
    assert!(result.is_ok());
    if let Ok(InteractiveCommand::Puzzle { puzzle_id }) = result {
        assert_eq!(puzzle_id, "mate_in_2_001");
    } else {
        panic!("Expected Puzzle command");
    }

    // Wrong number of arguments
    let result = InteractiveEngine::parse_command("puzzle");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("requires puzzle ID"));
}

#[test]
fn test_simple_phase4_commands() {
    let commands = ["threats", "hint", "clock"];
    let expected = [
        InteractiveCommand::Threats,
        InteractiveCommand::Hint,
        InteractiveCommand::Clock,
    ];

    for (cmd_str, expected_cmd) in commands.iter().zip(expected.iter()) {
        let result = InteractiveEngine::parse_command(cmd_str);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            std::mem::discriminant(&parsed),
            std::mem::discriminant(expected_cmd)
        );
    }
}

#[test]
fn test_phase4_command_execution() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = InteractiveEngine::new()?;

    // Test play command
    let play_cmd = InteractiveCommand::Play {
        player_color: "white".to_string(),
        difficulty: 5,
    };
    let response = engine.handle_command(play_cmd)?;
    if let InteractiveResponse::GameStarted { mode, player_color } = response {
        assert!(mode.contains("Engine"));
        assert_eq!(player_color, "white");
    } else {
        panic!("Expected GameStarted response");
    }

    // Test puzzle command
    let puzzle_cmd = InteractiveCommand::Puzzle {
        puzzle_id: "test_puzzle".to_string(),
    };
    let response = engine.handle_command(puzzle_cmd)?;
    if let InteractiveResponse::PuzzleLoaded {
        objective,
        puzzle_id,
    } = response
    {
        assert!(objective.contains("mate"));
        assert_eq!(puzzle_id, "test_puzzle");
    } else {
        panic!("Expected PuzzleLoaded response");
    }

    // Test simple commands
    let threats_cmd = InteractiveCommand::Threats;
    let response = engine.handle_command(threats_cmd)?;
    if let InteractiveResponse::ThreatsFound {
        threat_count,
        threats: _,
    } = response
    {
        assert!(threat_count > 0);
    } else {
        panic!("Expected ThreatsFound response");
    }

    let hint_cmd = InteractiveCommand::Hint;
    let response = engine.handle_command(hint_cmd)?;
    if let InteractiveResponse::PuzzleHint { hint } = response {
        assert!(!hint.is_empty());
    } else {
        panic!("Expected PuzzleHint response");
    }

    let clock_cmd = InteractiveCommand::Clock;
    let response = engine.handle_command(clock_cmd)?;
    if let InteractiveResponse::GameClock {
        white_time_ms,
        black_time_ms,
    } = response
    {
        assert_eq!(white_time_ms, 300_000);
        assert_eq!(black_time_ms, 300_000);
    } else {
        panic!("Expected GameClock response");
    }

    Ok(())
}

#[test]
fn test_response_formatting() {
    // Test GameStarted formatting
    let response = InteractiveResponse::GameStarted {
        mode: "Playing vs Engine (difficulty 5)".to_string(),
        player_color: "white".to_string(),
    };
    let formatted = chess_engine::interactive::InteractiveEngine::format_response(&response);
    assert!(formatted.contains("Game started"));
    assert!(formatted.contains("white"));

    // Test PuzzleLoaded formatting
    let response = InteractiveResponse::PuzzleLoaded {
        objective: "White to play and mate in 2".to_string(),
        puzzle_id: "mate_in_2_001".to_string(),
    };
    let formatted = chess_engine::interactive::InteractiveEngine::format_response(&response);
    assert!(formatted.contains("Puzzle loaded"));
    assert!(formatted.contains("mate_in_2_001"));

    // Test ThreatsFound formatting
    let response = InteractiveResponse::ThreatsFound {
        threat_count: 2,
        threats: vec!["e4:d5".to_string(), "d1:h5".to_string()],
    };
    let formatted = chess_engine::interactive::InteractiveEngine::format_response(&response);
    assert!(formatted.contains("Threats found (2)"));

    // Test GameClock formatting
    let response = InteractiveResponse::GameClock {
        white_time_ms: 300_000, // 5:00
        black_time_ms: 180_000, // 3:00
    };
    let formatted = chess_engine::interactive::InteractiveEngine::format_response(&response);
    assert!(formatted.contains("White: 5:00"));
    assert!(formatted.contains("Black: 3:00"));
}

#[test]
fn test_help_includes_phase4_commands() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = InteractiveEngine::new()?;
    let help_cmd = InteractiveCommand::Help;
    let response = engine.handle_command(help_cmd)?;

    if let InteractiveResponse::Help { commands } = response {
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
