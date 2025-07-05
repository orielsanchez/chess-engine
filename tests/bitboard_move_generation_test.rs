use chess_engine::*;
use std::time::Instant;

/// Comprehensive test suite for bitboard-based move generation optimization
/// These tests define the expected behavior of the new bitboard system
///
/// Testing Strategy:
/// 1. Functional equivalence: Bitboard must produce identical moves to mailbox
/// 2. Performance improvement: Should be faster than current mailbox system
/// 3. Edge case coverage: All special moves and complex positions
/// 4. Integration compatibility: Must work with existing search/evaluation

#[test]
fn test_bitboard_knight_moves_equivalence() {
    // Test that bitboard knight move generation produces identical results to mailbox
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
        "8/8/8/3N4/8/8/8/8 w - - 0 1",                              // Knight in center
        "8/8/8/8/8/8/8/N7 w - - 0 1",                               // Knight in corner
        "8/8/8/8/8/8/8/7N w - - 0 1",                               // Knight in opposite corner
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 5", // Complex position
    ];

    for fen in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        // Get moves from current mailbox system
        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");

        // Get moves from new bitboard system
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        // Extract only knight moves for comparison
        let mailbox_knight_moves: Vec<_> = mailbox_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::Knight
                } else {
                    false
                }
            })
            .collect();

        let bitboard_knight_moves: Vec<_> = bitboard_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::Knight
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(
            mailbox_knight_moves.len(),
            bitboard_knight_moves.len(),
            "Knight move count mismatch for position: {}",
            fen
        );

        // Verify all moves are identical (order independent)
        for mailbox_move in &mailbox_knight_moves {
            assert!(
                bitboard_knight_moves.contains(mailbox_move),
                "Bitboard missing knight move: {} in position: {}",
                mailbox_move,
                fen
            );
        }
    }
}

#[test]
fn test_bitboard_sliding_pieces_equivalence() {
    // Test bishops, rooks, and queens produce identical moves
    let test_positions = vec![
        "8/8/8/3Q4/8/8/8/8 w - - 0 1", // Queen in center
        "8/8/8/8/3R4/8/8/8 w - - 0 1", // Rook in center
        "8/8/8/8/8/3B4/8/8 w - - 0 1", // Bishop in center
        "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8", // Complex middlegame
    ];

    for fen in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        // Test each sliding piece type
        for piece_type in [PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
            let mailbox_sliding: Vec<_> = mailbox_moves
                .iter()
                .filter(|mv| {
                    if let Some(piece) = position.piece_at(mv.from) {
                        piece.piece_type == piece_type
                    } else {
                        false
                    }
                })
                .collect();

            let bitboard_sliding: Vec<_> = bitboard_moves
                .iter()
                .filter(|mv| {
                    if let Some(piece) = position.piece_at(mv.from) {
                        piece.piece_type == piece_type
                    } else {
                        false
                    }
                })
                .collect();

            assert_eq!(
                mailbox_sliding.len(),
                bitboard_sliding.len(),
                "{:?} move count mismatch for position: {}",
                piece_type,
                fen
            );

            for mailbox_move in &mailbox_sliding {
                assert!(
                    bitboard_sliding.contains(mailbox_move),
                    "Bitboard missing {:?} move: {} in position: {}",
                    piece_type,
                    mailbox_move,
                    fen
                );
            }
        }
    }
}

#[test]
fn test_bitboard_pawn_moves_equivalence() {
    // Test all pawn move types: normal, double push, captures, en passant, promotions
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
        "8/2P5/8/8/8/8/2p5/8 w - - 0 1",                            // Promotion scenario
        "8/8/8/3pP3/8/8/8/8 w - d6 0 1",                            // En passant available
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 5", // Pawn structure
    ];

    for fen in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        let mailbox_pawn_moves: Vec<_> = mailbox_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::Pawn
                } else {
                    false
                }
            })
            .collect();

        let bitboard_pawn_moves: Vec<_> = bitboard_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::Pawn
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(
            mailbox_pawn_moves.len(),
            bitboard_pawn_moves.len(),
            "Pawn move count mismatch for position: {}",
            fen
        );

        for mailbox_move in &mailbox_pawn_moves {
            assert!(
                bitboard_pawn_moves.contains(mailbox_move),
                "Bitboard missing pawn move: {} in position: {}",
                mailbox_move,
                fen
            );
        }
    }
}

#[test]
fn test_bitboard_king_and_castling_equivalence() {
    // Test king moves and castling rights
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",                     // Castling available
        "r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1",                        // No castling rights
        "8/8/8/3K4/8/8/8/8 w - - 0 1",                              // King in center
    ];

    for fen in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        let mailbox_king_moves: Vec<_> = mailbox_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::King
                } else {
                    false
                }
            })
            .collect();

        let bitboard_king_moves: Vec<_> = bitboard_moves
            .iter()
            .filter(|mv| {
                if let Some(piece) = position.piece_at(mv.from) {
                    piece.piece_type == PieceType::King
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(
            mailbox_king_moves.len(),
            bitboard_king_moves.len(),
            "King move count mismatch for position: {}",
            fen
        );

        for mailbox_move in &mailbox_king_moves {
            assert!(
                bitboard_king_moves.contains(mailbox_move),
                "Bitboard missing king move: {} in position: {}",
                mailbox_move,
                fen
            );
        }
    }
}

#[test]
fn test_bitboard_performance_baseline() {
    // Test that bitboard move generation maintains reasonable performance
    // Focus is on functional correctness, performance optimization comes later

    let test_position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let position = Position::from_fen(test_position).expect("Valid FEN");

    // Simple performance validation - just ensure it runs in reasonable time
    let duration = std::time::Duration::from_millis(50);
    let start = Instant::now();
    let mut total_moves = 0;

    while start.elapsed() < duration {
        let moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");
        total_moves += moves.len();
    }

    let elapsed = start.elapsed();
    let moves_per_sec = (total_moves as f64 / elapsed.as_secs_f64()) as u64;

    println!("Bitboard performance: {} moves/sec", moves_per_sec);

    // Very low bar - just ensure it's not completely broken
    assert!(
        moves_per_sec >= 100_000,
        "Bitboard performance {} moves/sec below minimum threshold",
        moves_per_sec
    );
}

#[test]
fn test_bitboard_complete_position_equivalence() {
    // Test that complete move lists are identical between systems
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 5",
        "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ];

    for fen in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        assert_eq!(
            mailbox_moves.len(),
            bitboard_moves.len(),
            "Total move count mismatch for position: {}",
            fen
        );

        // Verify all moves are identical (order independent)
        for mailbox_move in &mailbox_moves {
            assert!(
                bitboard_moves.contains(mailbox_move),
                "Bitboard missing move: {} in position: {}",
                mailbox_move,
                fen
            );
        }

        for bitboard_move in &bitboard_moves {
            assert!(
                mailbox_moves.contains(bitboard_move),
                "Bitboard has extra move: {} in position: {}",
                bitboard_move,
                fen
            );
        }
    }
}

#[test]
fn test_bitboard_pin_aware_generation() {
    // Test that bitboard system properly handles pinned pieces
    let pinned_positions = vec![
        "8/8/8/8/2k5/8/2R5/2K5 w - - 0 1", // Rook pinned to king
        "8/8/8/1b6/8/1N6/8/1K6 w - - 0 1", // Knight pinned (can't move)
        "8/8/8/1r6/8/1P6/8/1K6 w - - 0 1", // Pawn pinned along file
        "8/8/2q5/8/8/2P5/8/2K5 w - - 0 1", // Pawn pinned diagonally
    ];

    for fen in pinned_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        assert_eq!(
            mailbox_moves.len(),
            bitboard_moves.len(),
            "Pin-aware move count mismatch for position: {}",
            fen
        );

        for mailbox_move in &mailbox_moves {
            assert!(
                bitboard_moves.contains(mailbox_move),
                "Bitboard missing pin-aware move: {} in position: {}",
                mailbox_move,
                fen
            );
        }
    }
}

#[test]
fn test_bitboard_check_evasion() {
    // Test that bitboard system properly generates check evasion moves
    let check_positions = vec![
        "8/8/8/8/8/2k5/1R6/2K5 b - - 0 1",  // Black king in check from rook
        "8/8/8/8/8/2k5/2Q5/2K5 b - - 0 1",  // Black king in check from queen
        "8/8/8/8/8/1nk5/2Q5/2K5 b - - 0 1", // Black in check, can block or capture
        "r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1", // Black in check, test castling rights
    ];

    for fen in check_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let mailbox_moves = position.generate_legal_moves().expect("Mailbox moves");
        let bitboard_moves = position
            .generate_legal_moves_bitboard()
            .expect("Bitboard moves");

        assert_eq!(
            mailbox_moves.len(),
            bitboard_moves.len(),
            "Check evasion move count mismatch for position: {}",
            fen
        );

        for mailbox_move in &mailbox_moves {
            assert!(
                bitboard_moves.contains(mailbox_move),
                "Bitboard missing check evasion move: {} in position: {}",
                mailbox_move,
                fen
            );
        }
    }
}
