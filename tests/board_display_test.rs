use chess_engine::position::Position;

#[test]
fn test_ascii_board_starting_position() {
    let position = Position::starting_position().unwrap();
    let display = position.to_ascii_board();

    // Should show starting chess position with Unicode pieces
    let expected = concat!(
        "  a b c d e f g h\n",
        "8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 8\n",
        "7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 7\n",
        "6 · · · · · · · · 6\n",
        "5 · · · · · · · · 5\n",
        "4 · · · · · · · · 4\n",
        "3 · · · · · · · · 3\n",
        "2 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 2\n",
        "1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ 1\n",
        "  a b c d e f g h"
    );

    assert_eq!(display, expected);
}

#[test]
fn test_ascii_board_empty_position() {
    // Create position with only kings (white king on e1, black king on h8)
    let position = Position::from_fen("7k/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let display = position.to_ascii_board();

    let expected = concat!(
        "  a b c d e f g h\n",
        "8 · · · · · · · ♚ 8\n",
        "7 · · · · · · · · 7\n",
        "6 · · · · · · · · 6\n",
        "5 · · · · · · · · 5\n",
        "4 · · · · · · · · 4\n",
        "3 · · · · · · · · 3\n",
        "2 · · · · · · · · 2\n",
        "1 · · · · ♔ · · · 1\n",
        "  a b c d e f g h"
    );

    assert_eq!(display, expected);
}

#[test]
fn test_ascii_board_piece_symbols() {
    // Test each piece type individually
    let test_cases = vec![
        ("8/8/8/8/8/8/8/K7 w - - 0 1", "♔"), // White King
        ("8/8/8/8/8/8/8/Q7 w - - 0 1", "♕"), // White Queen
        ("8/8/8/8/8/8/8/R7 w - - 0 1", "♖"), // White Rook
        ("8/8/8/8/8/8/8/B7 w - - 0 1", "♗"), // White Bishop
        ("8/8/8/8/8/8/8/N7 w - - 0 1", "♘"), // White Knight
        ("8/8/8/8/8/8/8/P7 w - - 0 1", "♙"), // White Pawn
        ("k7/8/8/8/8/8/8/8 w - - 0 1", "♚"), // Black King
        ("q7/8/8/8/8/8/8/8 w - - 0 1", "♛"), // Black Queen
        ("r7/8/8/8/8/8/8/8 w - - 0 1", "♜"), // Black Rook
        ("b7/8/8/8/8/8/8/8 w - - 0 1", "♝"), // Black Bishop
        ("n7/8/8/8/8/8/8/8 w - - 0 1", "♞"), // Black Knight
        ("p7/8/8/8/8/8/8/8 w - - 0 1", "♟"), // Black Pawn
    ];

    for (fen, expected_symbol) in test_cases {
        let position = Position::from_fen(fen).unwrap();
        let display = position.to_ascii_board();
        assert!(
            display.contains(expected_symbol),
            "FEN {fen} should contain symbol {expected_symbol}"
        );
    }
}

#[test]
fn test_ascii_board_coordinates() {
    let position = Position::starting_position().unwrap();
    let display = position.to_ascii_board();

    // Should have file letters at top and bottom
    assert!(display.contains("  a b c d e f g h\n"));
    assert!(display.ends_with("  a b c d e f g h"));

    // Should have rank numbers on both sides
    for rank in 1..=8 {
        let rank_str = rank.to_string();
        assert!(display.contains(&format!("{rank_str} ")));
        assert!(display.contains(&format!(" {rank_str}")));
    }
}

#[test]
fn test_ascii_board_layout_structure() {
    let position = Position::starting_position().unwrap();
    let display = position.to_ascii_board();

    let lines: Vec<&str> = display.lines().collect();

    // Should have 10 lines total (header + 8 ranks + footer)
    assert_eq!(lines.len(), 10);

    // First line should be file coordinates
    assert_eq!(lines[0], "  a b c d e f g h");

    // Ranks 8-1 (lines 1-8)
    for (i, line) in lines.iter().enumerate().take(9).skip(1) {
        let expected_rank = 9 - i; // Line 1 = rank 8, line 8 = rank 1
        assert!(line.starts_with(&format!("{expected_rank} ")));
        assert!(line.ends_with(&format!(" {expected_rank}")));
    }

    // Last line should be file coordinates again
    assert_eq!(lines[9], "  a b c d e f g h");
}
