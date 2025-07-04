use chess_engine::interactive::{InteractiveCommand, InteractiveEngine, InteractiveResponse};
use chess_engine::position::Position;

#[test]
fn test_interactive_engine_creation() {
    let engine = InteractiveEngine::new().unwrap();

    // Should start with the starting position
    let position = engine.current_position();
    let expected = Position::starting_position().unwrap();
    assert_eq!(*position, expected);
}

#[test]
fn test_parse_analyze_command() {
    let command = InteractiveEngine::parse_command("analyze").unwrap();
    assert!(matches!(command, InteractiveCommand::Analyze));
}

#[test]
fn test_parse_legal_moves_command() {
    let command = InteractiveEngine::parse_command("legal").unwrap();
    assert!(matches!(command, InteractiveCommand::Legal));
}

#[test]
fn test_parse_move_command() {
    let command = InteractiveEngine::parse_command("move e2e4").unwrap();
    assert!(
        matches!(command, InteractiveCommand::Move { algebraic_move } if algebraic_move == "e2e4")
    );
}

#[test]
fn test_parse_position_command() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let command = InteractiveEngine::parse_command(&format!("position {}", fen)).unwrap();
    assert!(matches!(command, InteractiveCommand::Position { fen: f } if f == fen));
}

#[test]
fn test_parse_undo_command() {
    let command = InteractiveEngine::parse_command("undo").unwrap();
    assert!(matches!(command, InteractiveCommand::Undo));
}

#[test]
fn test_parse_help_command() {
    let command = InteractiveEngine::parse_command("help").unwrap();
    assert!(matches!(command, InteractiveCommand::Help));
}

#[test]
fn test_parse_invalid_command() {
    let result = InteractiveEngine::parse_command("invalid_command");
    assert!(result.is_err());
}

#[test]
fn test_analyze_starting_position() {
    let mut engine = InteractiveEngine::new().unwrap();
    let command = InteractiveCommand::Analyze;

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::Analysis {
            evaluation,
            best_move,
            depth,
        } => {
            // Starting position should be roughly equal (within 50 centipawns)
            assert!(
                evaluation.abs() <= 50,
                "Evaluation should be near zero, got {}",
                evaluation
            );
            assert!(!best_move.is_empty(), "Should provide a best move");
            assert!(depth > 0, "Should search to some depth");
        }
        _ => panic!("Expected Analysis response, got {:?}", response),
    }
}

#[test]
fn test_legal_moves_starting_position() {
    let mut engine = InteractiveEngine::new().unwrap();
    let command = InteractiveCommand::Legal;

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::LegalMoves { moves } => {
            // Starting position should have 20 legal moves
            assert_eq!(
                moves.len(),
                20,
                "Starting position should have 20 legal moves"
            );

            // Check for some expected moves
            assert!(moves.contains(&"e2e4".to_string()));
            assert!(moves.contains(&"d2d4".to_string()));
            assert!(moves.contains(&"g1f3".to_string()));
            assert!(moves.contains(&"b1c3".to_string()));
        }
        _ => panic!("Expected LegalMoves response, got {:?}", response),
    }
}

#[test]
fn test_make_move() {
    let mut engine = InteractiveEngine::new().unwrap();
    let command = InteractiveCommand::Move {
        algebraic_move: "e2e4".to_string(),
    };

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::MoveResult {
            success,
            resulting_fen,
        } => {
            assert!(
                success,
                "e2e4 should be a valid move from starting position"
            );
            assert!(!resulting_fen.is_empty(), "Should provide resulting FEN");

            // Verify the position actually changed
            let expected_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
            assert_eq!(resulting_fen, expected_fen);
        }
        _ => panic!("Expected MoveResult response, got {:?}", response),
    }
}

#[test]
fn test_invalid_move() {
    let mut engine = InteractiveEngine::new().unwrap();
    let command = InteractiveCommand::Move {
        algebraic_move: "e2e5".to_string(),
    }; // Invalid move

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::MoveResult {
            success,
            resulting_fen: _,
        } => {
            assert!(!success, "e2e5 should be invalid from starting position");
        }
        _ => panic!("Expected MoveResult response, got {:?}", response),
    }
}

#[test]
fn test_set_position() {
    let mut engine = InteractiveEngine::new().unwrap();
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let command = InteractiveCommand::Position {
        fen: fen.to_string(),
    };

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::PositionSet {
            success,
            fen: returned_fen,
        } => {
            assert!(success, "Should be able to set valid FEN");
            assert_eq!(returned_fen, fen);
        }
        _ => panic!("Expected PositionSet response, got {:?}", response),
    }
}

#[test]
fn test_undo_move() {
    let mut engine = InteractiveEngine::new().unwrap();

    // Make a move first
    let move_command = InteractiveCommand::Move {
        algebraic_move: "e2e4".to_string(),
    };
    engine.handle_command(move_command).unwrap();

    // Now undo it
    let undo_command = InteractiveCommand::Undo;
    let response = engine.handle_command(undo_command).unwrap();

    match response {
        InteractiveResponse::UndoResult {
            success,
            resulting_fen,
        } => {
            assert!(success, "Should be able to undo the move");

            // Should be back to starting position
            let starting_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
            assert_eq!(resulting_fen, starting_fen);
        }
        _ => panic!("Expected UndoResult response, got {:?}", response),
    }
}

#[test]
fn test_undo_without_moves() {
    let mut engine = InteractiveEngine::new().unwrap();

    let command = InteractiveCommand::Undo;
    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::UndoResult {
            success,
            resulting_fen: _,
        } => {
            assert!(!success, "Should not be able to undo when no moves made");
        }
        _ => panic!("Expected UndoResult response, got {:?}", response),
    }
}

#[test]
fn test_help_command() {
    let mut engine = InteractiveEngine::new().unwrap();
    let command = InteractiveCommand::Help;

    let response = engine.handle_command(command).unwrap();

    match response {
        InteractiveResponse::Help { commands } => {
            assert!(commands.contains("analyze"));
            assert!(commands.contains("legal"));
            assert!(commands.contains("move"));
            assert!(commands.contains("position"));
            assert!(commands.contains("undo"));
            assert!(commands.contains("help"));
        }
        _ => panic!("Expected Help response, got {:?}", response),
    }
}

#[test]
fn test_format_analysis_response() {
    let response = InteractiveResponse::Analysis {
        evaluation: 25,
        best_move: "e2e4".to_string(),
        depth: 6,
    };

    let formatted = InteractiveEngine::format_response(&response);

    assert!(formatted.contains("Evaluation: +0.25"));
    assert!(formatted.contains("Best move: e2e4"));
    assert!(formatted.contains("Depth: 6"));
}

#[test]
fn test_format_legal_moves_response() {
    let moves = vec!["e2e4".to_string(), "d2d4".to_string(), "g1f3".to_string()];
    let response = InteractiveResponse::LegalMoves { moves };

    let formatted = InteractiveEngine::format_response(&response);

    assert!(formatted.contains("Legal moves"));
    assert!(formatted.contains("e2e4"));
    assert!(formatted.contains("d2d4"));
    assert!(formatted.contains("g1f3"));
}
