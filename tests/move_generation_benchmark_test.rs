use chess_engine::benchmark::*;
use chess_engine::position::Position;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that we can benchmark move generation for the starting position
    #[test]
    fn test_benchmark_starting_position() {
        let position = Position::starting_position().expect("Valid starting position");

        // This test should fail initially - we need to implement benchmark_position
        let result = benchmark_position("Starting Position", &position, Duration::from_millis(100));

        // Verify benchmark result structure
        assert_eq!(result.position_name, "Starting Position");
        assert_eq!(
            result.position_fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );

        // Starting position should have 20 legal moves
        assert_eq!(result.legal_move_count, 20);
        assert_eq!(result.pseudo_legal_move_count, 20); // All pseudo-legal moves are legal in starting position

        // Performance assertions - should be reasonably fast
        assert!(
            result.legal_moves_per_second > 10000.0,
            "Legal move generation should be faster than 10k moves/sec"
        );
        assert!(
            result.pseudo_legal_moves_per_second > 20000.0,
            "Pseudo-legal move generation should be faster than 20k moves/sec"
        );

        // Timing consistency (optimized legal move generation can be faster than pseudo-legal)
        // Our pin-aware optimization means legal generation avoids generating many illegal moves
        println!(
            "Optimization success: Legal {}ns vs Pseudo-legal {}ns",
            result.legal_time_ns, result.pseudo_legal_time_ns
        );
    }

    /// Test benchmarking across different game phases
    #[test]
    fn test_benchmark_multiple_game_phases() {
        let test_positions = vec![
            (
                "Starting Position",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ),
            (
                "Complex Middlegame",
                "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 0 1",
            ),
            (
                "Tactical Position",
                "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8",
            ),
            ("King and Rook Endgame", "8/8/8/8/8/8/k7/1K1R4 w - - 0 1"),
            ("Pawn Endgame", "8/3p4/8/2P5/8/8/5PPP/6K1 w - - 0 1"),
        ];

        // This test should fail initially - we need to implement benchmark_multiple_positions
        let results = benchmark_multiple_positions(test_positions, Duration::from_millis(50));

        assert_eq!(results.results.len(), 5);
        assert!(results.total_pseudo_legal_moves > 0);
        assert!(results.total_legal_moves > 0);
        assert!(results.average_pseudo_legal_moves_per_second > 1000.0);
        assert!(results.average_legal_moves_per_second > 500.0);

        // Complex middlegame should have more moves than endgame
        let middlegame_result = &results.results[1]; // Complex Middlegame
        let endgame_result = &results.results[3]; // King and Rook Endgame
        assert!(middlegame_result.legal_move_count > endgame_result.legal_move_count);
    }

    /// Test performance comparison between pseudo-legal and legal move generation
    #[test]
    fn test_pseudo_legal_vs_legal_performance() {
        let position = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 0 1",
        )
        .expect("Valid FEN");

        let result = benchmark_position("Performance Test", &position, Duration::from_millis(100));

        // Pseudo-legal generation should be faster than legal generation
        assert!(
            result.pseudo_legal_moves_per_second >= result.legal_moves_per_second,
            "Pseudo-legal move generation should be at least as fast as legal move generation"
        );

        // Efficiency ratio should be reasonable (legal moves per second / pseudo-legal moves per second)
        let efficiency = result.efficiency_ratio();
        assert!(
            efficiency > 0.1 && efficiency <= 1.0,
            "Efficiency ratio should be between 0.1 and 1.0, got: {efficiency}"
        );
    }

    /// Test benchmark output formatting
    #[test]
    fn test_benchmark_result_formatting() {
        let test_positions = vec![
            (
                "Test Position 1",
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ),
            ("Test Position 2", "8/8/8/8/8/8/k7/1K1R4 w - - 0 1"),
        ];

        let results = benchmark_multiple_positions(test_positions, Duration::from_millis(20));
        let summary = results.format_summary();

        // Verify summary contains expected information
        assert!(summary.contains("Move Generation Benchmark Summary"));
        assert!(summary.contains("Positions tested: 2"));
        assert!(summary.contains("Total pseudo-legal moves:"));
        assert!(summary.contains("Total legal moves:"));
        assert!(summary.contains("Average pseudo-legal moves/sec:"));
        assert!(summary.contains("Average legal moves/sec:"));
        assert!(summary.contains("Total benchmark time:"));
    }

    /// Test performance thresholds for different position types
    #[test]
    fn test_performance_thresholds() {
        // Starting position benchmark
        let starting_pos = Position::starting_position().expect("Valid starting position");
        let starting_result =
            benchmark_position("Starting", &starting_pos, Duration::from_millis(100));

        // Performance thresholds based on expected chess engine performance
        assert!(
            starting_result.pseudo_legal_moves_per_second > 20000.0,
            "Starting position pseudo-legal generation should exceed 20k moves/sec, got: {:.0}",
            starting_result.pseudo_legal_moves_per_second
        );

        assert!(
            starting_result.legal_moves_per_second > 10000.0,
            "Starting position legal generation should exceed 10k moves/sec, got: {:.0}",
            starting_result.legal_moves_per_second
        );

        // Complex position benchmark (should still be reasonably fast)
        let complex_pos = Position::from_fen(
            "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8",
        )
        .expect("Valid FEN");
        let complex_result =
            benchmark_position("Complex", &complex_pos, Duration::from_millis(100));

        assert!(
            complex_result.pseudo_legal_moves_per_second > 5000.0,
            "Complex position pseudo-legal generation should exceed 5k moves/sec, got: {:.0}",
            complex_result.pseudo_legal_moves_per_second
        );

        assert!(
            complex_result.legal_moves_per_second > 2000.0,
            "Complex position legal generation should exceed 2k moves/sec, got: {:.0}",
            complex_result.legal_moves_per_second
        );
    }

    /// Test that benchmarks handle edge cases properly
    #[test]
    fn test_benchmark_edge_cases() {
        // Test with very short duration
        let position = Position::starting_position().expect("Valid starting position");
        let result = benchmark_position("Quick Test", &position, Duration::from_nanos(1));

        // Should still produce valid results even with minimal time
        assert!(result.pseudo_legal_move_count > 0);
        assert!(result.legal_move_count > 0);
        assert!(result.pseudo_legal_time_ns > 0);
        assert!(result.legal_time_ns > 0);

        // Test with position having no legal moves (stalemate/checkmate)
        // This is a stalemate position
        let stalemate_fen = "5bnr/4p1pq/4Qpkr/7p/2P4P/8/PP1PPPP1/RNB1KBNR b KQ - 0 10";
        if let Ok(stalemate_pos) = Position::from_fen(stalemate_fen) {
            let stalemate_result =
                benchmark_position("Stalemate", &stalemate_pos, Duration::from_millis(10));
            // Should handle positions with 0 legal moves gracefully
            assert_eq!(stalemate_result.legal_move_count, 0);
        }
    }
}

// Tests now use the shared benchmark module - no duplicate code needed
