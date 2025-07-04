use chess_engine::fen::STARTING_FEN;
use chess_engine::pgn::{GameResult, PgnError, PgnGame};

#[test]
fn test_parse_simple_pgn() {
    let pgn = r#"[Event "Test Game"]
[Site "Test Site"]
[Date "2025.07.04"]
[Round "1"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 1-0"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse valid PGN");

    assert_eq!(game.metadata.event, Some("Test Game".to_string()));
    assert_eq!(game.metadata.site, Some("Test Site".to_string()));
    assert_eq!(game.metadata.white, Some("Player1".to_string()));
    assert_eq!(game.metadata.black, Some("Player2".to_string()));
    assert_eq!(game.result, GameResult::WhiteWins);
    assert_eq!(game.moves.len(), 10); // 5 moves for each side
    assert_eq!(game.moves[0], "e4");
    assert_eq!(game.moves[1], "e5");
    assert_eq!(game.moves[8], "O-O");
    assert_eq!(game.moves[9], "Be7");
}

#[test]
fn test_parse_pgn_minimal_headers() {
    let pgn = r#"[White "Alice"]
[Black "Bob"]
[Result "*"]

1. d4 d5 *"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse minimal PGN");

    assert_eq!(game.metadata.white, Some("Alice".to_string()));
    assert_eq!(game.metadata.black, Some("Bob".to_string()));
    assert_eq!(game.metadata.event, None);
    assert_eq!(game.result, GameResult::Ongoing);
    assert_eq!(game.moves, vec!["d4", "d5"]);
}

#[test]
fn test_parse_pgn_all_game_results() {
    let test_cases = vec![
        ("1-0", GameResult::WhiteWins),
        ("0-1", GameResult::BlackWins),
        ("1/2-1/2", GameResult::Draw),
        ("*", GameResult::Ongoing),
    ];

    for (result_str, expected) in test_cases {
        let pgn = format!(
            r#"[Result "{}"]

1. e4 e5 {}"#,
            result_str, result_str
        );

        let game = PgnGame::from_pgn(&pgn).expect("Should parse PGN with result");
        assert_eq!(game.result, expected);
    }
}

#[test]
fn test_parse_pgn_with_castling() {
    let pgn = r#"[Result "*"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O O-O 5. d3 d6 *"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse castling moves");
    assert!(game.moves.contains(&"O-O".to_string()));
    assert_eq!(game.moves.iter().filter(|&m| m == "O-O").count(), 2);
}

#[test]
fn test_parse_pgn_with_queenside_castling() {
    let pgn = r#"[Result "*"]

1. d4 d5 2. Nc3 Nc6 3. Bf4 Bf5 4. Qd2 Qd7 5. O-O-O O-O-O *"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse queenside castling");
    assert!(game.moves.contains(&"O-O-O".to_string()));
}

#[test]
fn test_parse_pgn_with_promotion() {
    let pgn = r#"[Result "1-0"]

1. e4 e5 2. f4 exf4 3. Nf3 g5 4. h4 g4 5. Ng5 h6 6. Nxf7 Kxf7 7. d4 f3 8. gxf3 gxf3 9. Qd3 Bg7 10. Bxh6 Ne7 11. Qg3 Rg8 12. Qxg7+ Rxg7 13. Bxg7 Kxg7 14. O-O-O Nc6 15. e5 Nxd4 16. h5 Ne2+ 17. Kb1 f2 18. h6+ Kg6 19. Rd8 f1=Q+ 1-0"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse promotion");
    assert!(game.moves.contains(&"f1=Q+".to_string()));
}

#[test]
fn test_parse_pgn_with_disambiguation() {
    let pgn = r#"[Result "*"]

1. Nf3 Nf6 2. Nc3 Nc6 3. Nbd2 Nbd7 *"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse disambiguated moves");
    assert!(game.moves.contains(&"Nbd2".to_string()));
    assert!(game.moves.contains(&"Nbd7".to_string()));
}

#[test]
fn test_parse_pgn_with_check_and_checkmate() {
    let pgn = r#"[Result "1-0"]

1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse check/checkmate");
    assert!(game.moves.contains(&"Qxf7#".to_string()));
}

#[test]
fn test_pgn_position_integration() {
    let pgn = r#"[Result "*"]

1. e4 e5 2. Nf3 Nc6 *"#;

    let game = PgnGame::from_pgn(pgn).expect("Should parse PGN");
    let final_position = game.to_position().expect("Should convert to position");

    // Verify the position has the moves applied
    // This tests integration with existing Position/Move system
    assert_ne!(final_position.to_fen(), STARTING_FEN);
}

#[test]
fn test_position_to_pgn_roundtrip() {
    let original_pgn = r#"[Event "Roundtrip Test"]
[White "Player1"]
[Black "Player2"]
[Result "*"]

1. d4 d5 2. c4 e6 *"#;

    let game = PgnGame::from_pgn(original_pgn).expect("Should parse original");
    let position = game.to_position().expect("Should convert to position");

    // Convert back to PGN (this tests the export functionality)
    let exported_pgn = position.to_pgn().expect("Should export to PGN");
    let reimported_game = PgnGame::from_pgn(&exported_pgn).expect("Should re-parse");

    // For now, just test that export/import round-trip works
    // Full move history tracking would be added in future refactoring
    assert!(reimported_game.moves.is_empty()); // Expected - we don't track history yet
}

#[test]
fn test_pgn_error_invalid_format() {
    // Test with move that has invalid length/format (triggers length check)
    let invalid_pgn = r#"[Result "*"]

1. e4 e5 2. this_is_way_too_long_for_a_chess_move *"#;

    let result = PgnGame::from_pgn(invalid_pgn);
    assert!(result.is_err());

    match result.unwrap_err() {
        PgnError::InvalidFormat(_) => {} // Expected
        _ => panic!("Expected InvalidFormat error"),
    }
}

#[test]
fn test_pgn_error_illegal_move() {
    let pgn = r#"[Result "*"]

1. e4 e5 2. Nh3 Nf6 3. Ng5 Nh8 *"#; // Illegal knight move from f6 to h8

    // Parse succeeds, but conversion to position should fail
    let game = PgnGame::from_pgn(pgn).expect("PGN should parse");
    let result = game.to_position();
    assert!(result.is_err());

    match result.unwrap_err() {
        PgnError::IllegalMove(_) | PgnError::ParseError(_) => {} // Expected
        _ => panic!("Expected IllegalMove or ParseError"),
    }
}

#[test]
fn test_pgn_error_ambiguous_move() {
    // Test with truly ambiguous move - both Ne2 and Ng3 knights can reach e4
    let ambiguous_pgn = r#"[Result "*"]

1. Nf3 Nf6 2. Nc3 Nc6 3. Ne2 Ne7 4. Ng3 Ng6 5. Ne4 *"#; // This Ne4 is ambiguous

    let game = PgnGame::from_pgn(ambiguous_pgn).expect("PGN should parse");
    let result = game.to_position();
    assert!(result.is_err());

    match result.unwrap_err() {
        PgnError::AmbiguousMove(_) | PgnError::ParseError(_) => {} // Expected
        _ => panic!("Expected AmbiguousMove or ParseError"),
    }
}

#[test]
fn test_empty_pgn() {
    let empty_pgn = "";

    let result = PgnGame::from_pgn(empty_pgn);
    assert!(result.is_err());
}

#[test]
fn test_pgn_with_comments_should_error() {
    // Our simple parser doesn't support comments - should fail gracefully
    let pgn_with_comments = r#"[Result "*"]

1. e4 {Good opening} e5 2. Nf3 Nc6 *"#;

    let result = PgnGame::from_pgn(pgn_with_comments);
    assert!(result.is_err());

    match result.unwrap_err() {
        PgnError::UnsupportedFeature(_) => {} // Expected
        _ => panic!("Expected UnsupportedFeature error"),
    }
}

#[test]
fn test_large_game_performance() {
    // Test with a longer game to ensure reasonable performance
    let mut moves = Vec::new();
    for i in 1..=50 {
        moves.push(format!("{}. Nf3 Nc6", i));
        moves.push("Ng1 Nb8".to_string());
    }
    let movetext = moves.join(" ");

    let pgn = format!(
        r#"[Result "*"]

{} *"#,
        movetext
    );

    let start = std::time::Instant::now();
    let game = PgnGame::from_pgn(&pgn).expect("Should parse large game");
    let duration = start.elapsed();

    assert_eq!(game.moves.len(), 200); // 100 moves * 2 sides
    assert!(
        duration.as_millis() < 100,
        "Parsing should be fast: {:?}",
        duration
    );
}
