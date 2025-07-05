use chess_engine::position::Position;
use chess_engine::tablebase::{
    DtzResult, Tablebase, TablebaseError, TablebaseKey, TablebaseResult,
};
use chess_engine::types::{Color, PieceType};

/// Test suite for endgame tablebase integration
///
/// This test suite defines the expected behavior for tablebase integration:
/// 1. Canonical key generation for endgame positions
/// 2. Tablebase lookup with known results  
/// 3. Integration with evaluation system
/// 4. Search optimization with tablebase knowledge

#[cfg(test)]
mod tablebase_tests {
    use super::*;

    #[test]
    fn test_tablebase_key_generation_kqvk() {
        // Test canonical key generation for King + Queen vs King endgame
        let position = Position::from_fen("4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1").unwrap();

        let key = TablebaseKey::from_position(&position).unwrap();

        // Should generate consistent key for KQvK material
        assert_eq!(key.material_signature(), "KQvK");
        assert_eq!(key.side_to_move(), Color::White);

        // Key should be deterministic - same position generates same key
        let key2 = TablebaseKey::from_position(&position).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn test_tablebase_key_generation_krvk() {
        // Test canonical key generation for King + Rook vs King endgame
        let position = Position::from_fen("4k3/8/8/8/8/8/4R3/4K3 b - - 0 1").unwrap();

        let key = TablebaseKey::from_position(&position).unwrap();

        assert_eq!(key.material_signature(), "KRvK");
        assert_eq!(key.side_to_move(), Color::Black);
    }

    #[test]
    fn test_tablebase_key_ignores_irrelevant_state() {
        // Tablebase keys should ignore castling rights and en passant in endgames
        let pos1 = Position::from_fen("4k3/8/8/8/8/8/4Q3/4K3 w KQkq - 0 1").unwrap();
        let pos2 = Position::from_fen("4k3/8/8/8/8/8/4Q3/4K3 w - - 5 25").unwrap();

        let key1 = TablebaseKey::from_position(&pos1).unwrap();
        let key2 = TablebaseKey::from_position(&pos2).unwrap();

        // Keys should be identical despite different castling rights and move counters
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_tablebase_result_kqvk_mate_in_10() {
        // Test known KQvK position: White to move, mate in 10
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        let tablebase = MockTablebase::new();
        let result = tablebase.probe(&position).unwrap();

        match result {
            TablebaseResult::Win(dtm) => {
                assert_eq!(dtm, 10); // Distance to mate should be 10

                // Score should be large positive value adjusted for distance
                let score = result.to_search_score();
                assert!(score > 10000); // Strong winning score
                assert!(score < 32000); // But not immediate mate
            }
            _ => panic!("Expected winning result for KQvK position"),
        }
    }

    #[test]
    fn test_tablebase_result_krvk_draw() {
        // Test known KRvK drawn position
        let position = Position::from_fen("8/8/8/8/8/8/k7/1K1R4 b - - 0 1").unwrap();

        let tablebase = MockTablebase::new();
        let result = tablebase.probe(&position).unwrap();

        match result {
            TablebaseResult::Draw => {
                let score = result.to_search_score();
                assert_eq!(score, 0); // Draw should score exactly 0
            }
            _ => panic!("Expected draw for this KRvK position"),
        }
    }

    #[test]
    fn test_tablebase_result_side_to_move_perspective() {
        // Test that tablebase results are adjusted for side to move
        let white_to_move = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();
        let black_to_move = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 b - - 0 1").unwrap();

        let tablebase = MockTablebase::new();

        let white_result = tablebase.probe(&white_to_move).unwrap();
        let black_result = tablebase.probe(&black_to_move).unwrap();

        // Both should be wins for white, but scores should be opposite perspective
        let white_score = white_result.to_search_score();
        let black_score = black_result.to_search_score();

        assert!(white_score > 10000); // Strong positive for white to move
        assert!(black_score < -10000); // Strong negative for black to move (black is losing)
    }

    #[test]
    fn test_position_evaluate_with_tablebase() {
        // Test that Position::evaluate() uses tablebase when available
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Enable tablebase for this position
        position.enable_tablebase_lookup(true);

        let score = position.evaluate();

        // Should return tablebase score, not regular evaluation
        assert!(score > 10000); // Should be strong winning score from tablebase

        // Verify it's different from regular evaluation
        position.enable_tablebase_lookup(false);
        let regular_score = position.evaluate();
        assert_ne!(score, regular_score);
    }

    #[test]
    fn test_position_evaluate_fallback_when_no_tablebase() {
        // Test that evaluation falls back to regular eval when no tablebase data
        let position =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        position.enable_tablebase_lookup(true);
        let score_with_tb = position.evaluate();

        position.enable_tablebase_lookup(false);
        let score_without_tb = position.evaluate();

        // Should be identical since opening position has no tablebase data
        assert_eq!(score_with_tb, score_without_tb);
    }

    #[test]
    fn test_is_tablebase_position_piece_count() {
        // Test detection of positions suitable for tablebase lookup

        // KQvK - should be tablebase position (3 pieces)
        let kqvk = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();
        assert!(kqvk.is_tablebase_position());

        // Opening position - should not be tablebase position (32 pieces)
        let opening =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert!(!opening.is_tablebase_position());

        // Complex endgame - 5 pieces, should be tablebase position with 6-piece limit
        let complex = Position::from_fen("8/8/2kp4/8/2KP4/8/1R6/8 w - - 0 1").unwrap();
        // 5 pieces (k, p, K, P, R) should be within our 6-piece tablebase limit
        assert!(complex.is_tablebase_position());
    }

    #[test]
    fn test_search_finds_tablebase_best_move() {
        // Test that search returns optimal move when tablebase data available
        let position = Position::from_fen("8/8/8/8/8/2k5/1Q6/2K5 w - - 0 1").unwrap();

        let search_result = position.find_best_move_with_tablebase(1000); // 1 second

        // Should find a move that leads to mate
        assert!(search_result.best_move.is_some());

        // Score should indicate forced mate
        assert!(search_result.score > 10000);

        // Should find relatively quickly due to tablebase knowledge
        assert!(search_result.depth >= 10); // Can search deeper with tablebase
    }

    #[test]
    fn test_search_early_termination_with_tablebase() {
        // Test that search can terminate early when tablebase gives perfect information
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        let start_time = std::time::Instant::now();
        let result = position.find_best_move_with_tablebase(5000); // 5 seconds allowed
        let elapsed = start_time.elapsed();

        // Should terminate much faster than 5 seconds due to tablebase
        assert!(elapsed.as_millis() < 1000); // Should complete in under 1 second

        // But should still find the optimal move
        assert!(result.best_move.is_some());
        assert!(result.score > 10000);
    }
}

/// Mock tablebase implementation for testing
#[derive(Debug)]
struct MockTablebase {
    // Known tablebase results for testing
}

impl MockTablebase {
    fn new() -> Self {
        Self {}
    }
}

impl Tablebase for MockTablebase {
    fn probe(&self, position: &Position) -> Result<TablebaseResult, TablebaseError> {
        let key = TablebaseKey::from_position(position)?;

        match key.material_signature() {
            "KQvK" => {
                // Determine who has the advantage
                let white_has_queen = position.has_piece(Color::White, PieceType::Queen);

                // Return result from the perspective of the side to move
                if white_has_queen {
                    // White is stronger
                    if position.side_to_move == Color::White {
                        Ok(TablebaseResult::Win(10)) // White to move, White wins
                    } else {
                        Ok(TablebaseResult::Loss(10)) // Black to move, Black loses
                    }
                } else {
                    // Black is stronger (though this shouldn't happen in our test)
                    if position.side_to_move == Color::Black {
                        Ok(TablebaseResult::Win(10)) // Black to move, Black wins
                    } else {
                        Ok(TablebaseResult::Loss(10)) // White to move, White loses
                    }
                }
            }
            "KRvK" => {
                // Simplified: some KRvK positions are drawn
                Ok(TablebaseResult::Draw)
            }
            _ => Err(TablebaseError::NotFound),
        }
    }

    fn probe_dtz_specific(&self, _position: &Position) -> Result<DtzResult, TablebaseError> {
        // Mock implementation for DTZ testing - just return a placeholder
        Ok(DtzResult::Win { dtz: 8 })
    }

    fn is_available(&self, material_signature: &str) -> bool {
        matches!(material_signature, "KQvK" | "KRvK" | "KPvK")
    }
}
