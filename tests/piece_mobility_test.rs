use chess_engine::position::Position;

#[cfg(test)]
mod piece_mobility_tests {
    use super::*;

    #[test]
    fn test_knight_mobility_center_vs_corner() {
        // Knight on e4 (center) should have higher mobility than knight on a1 (corner)
        let center_position = Position::from_fen("8/8/8/8/4N3/8/8/8 w - - 0 1").unwrap();
        let corner_position = Position::from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();

        let center_mobility = center_position.evaluate_piece_mobility();
        let corner_mobility = corner_position.evaluate_piece_mobility();

        // Center knight should have higher mobility score
        assert!(center_mobility.mg > corner_mobility.mg);
        assert!(center_mobility.eg > corner_mobility.eg);
    }

    #[test]
    fn test_bishop_mobility_open_vs_blocked() {
        // Bishop with high mobility vs bishop blocked by friendly pieces
        let open_diagonal = Position::from_fen("8/8/8/8/3B4/8/8/4K3 w - - 0 1").unwrap();
        let blocked_diagonal =
            Position::from_fen("4k3/8/8/2PPP3/2PBP3/2PPP3/8/4K3 w - - 0 1").unwrap();

        let open_mobility = open_diagonal.evaluate_piece_mobility();
        let blocked_mobility = blocked_diagonal.evaluate_piece_mobility();

        // Bishop on open diagonal should have more moves than on blocked diagonal

        // Open diagonal should have higher mobility
        assert!(open_mobility.mg > blocked_mobility.mg);
        assert!(open_mobility.eg > blocked_mobility.eg);
    }

    #[test]
    fn test_rook_mobility_open_vs_blocked() {
        // Rook on open file vs blocked file
        let open_file = Position::from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
        let blocked_file = Position::from_fen("8/3p4/8/8/3R4/8/3p4/8 w - - 0 1").unwrap();

        let open_mobility = open_file.evaluate_piece_mobility();
        let blocked_mobility = blocked_file.evaluate_piece_mobility();

        // Open file should have higher mobility
        assert!(open_mobility.mg > blocked_mobility.mg);
        assert!(open_mobility.eg > blocked_mobility.eg);
    }

    #[test]
    fn test_queen_mobility_center_vs_corner() {
        // Queen in center vs corner
        let center_position = Position::from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
        let corner_position = Position::from_fen("Q7/8/8/8/8/8/8/8 w - - 0 1").unwrap();

        let center_mobility = center_position.evaluate_piece_mobility();
        let corner_mobility = corner_position.evaluate_piece_mobility();

        // Center queen should have higher mobility
        assert!(center_mobility.mg > corner_mobility.mg);
        assert!(center_mobility.eg > corner_mobility.eg);
    }

    #[test]
    fn test_pawn_mobility_forward_vs_blocked() {
        // Pawn with forward moves vs no pawn at all (mobility difference test)
        let with_pawn = Position::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let without_pawn = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let with_mobility = with_pawn.evaluate_piece_mobility();
        let without_mobility = without_pawn.evaluate_piece_mobility();

        // Position with pawn should have higher mobility than position without pawn
        assert!(with_mobility.mg > without_mobility.mg);
        assert!(with_mobility.eg > without_mobility.eg);
    }

    #[test]
    fn test_mobility_game_phase_sensitivity() {
        // Same position should give higher mobility bonus in middlegame vs endgame
        let middlegame_position = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 0 1",
        )
        .unwrap();
        let endgame_position = Position::from_fen("8/8/8/8/4N3/8/8/4K3 w - - 0 1").unwrap();

        let _middlegame_mobility = middlegame_position.evaluate_piece_mobility();
        let _endgame_mobility = endgame_position.evaluate_piece_mobility();

        // Middlegame should emphasize mobility more (higher mg relative to eg)
        let middlegame_phase = middlegame_position.get_game_phase_factor();
        let endgame_phase = endgame_position.get_game_phase_factor();

        assert!(middlegame_phase > endgame_phase);

        // For same piece count, middlegame should weight mobility higher
        let knight_middlegame = Position::from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 0 1",
        )
        .unwrap();
        let knight_endgame = Position::from_fen("8/8/8/8/4N3/8/8/4K3 w - - 0 1").unwrap();

        let mg_interpolated = knight_middlegame
            .evaluate_piece_mobility()
            .interpolate(middlegame_phase);
        let eg_interpolated = knight_endgame
            .evaluate_piece_mobility()
            .interpolate(endgame_phase);

        // The actual interpolated values should reflect game phase appropriately
        assert!(mg_interpolated != eg_interpolated);
    }

    #[test]
    fn test_mobility_middlegame_vs_endgame_bonus() {
        // Mobility should be weighted higher in middlegame than endgame
        let position = Position::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mobility_score = position.evaluate_piece_mobility();

        // Middlegame mobility score should be higher than endgame
        assert!(mobility_score.mg > mobility_score.eg);
    }

    #[test]
    fn test_mobility_phase_interpolation() {
        // Test that mobility properly interpolates between game phases
        let position = Position::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let mobility_score = position.evaluate_piece_mobility();

        // Test interpolation at different phase values
        let opening_phase = 1.0;
        let endgame_phase = 0.0;
        let middlegame_phase = 0.5;

        let opening_value = mobility_score.interpolate(opening_phase);
        let endgame_value = mobility_score.interpolate(endgame_phase);
        let middlegame_value = mobility_score.interpolate(middlegame_phase);

        // Should interpolate smoothly between phases
        assert!(opening_value >= middlegame_value);
        assert!(middlegame_value >= endgame_value);
    }

    #[test]
    fn test_mobility_integration_with_total_evaluation() {
        // Test that mobility is properly integrated with overall position evaluation
        let high_mobility = Position::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let low_mobility = Position::from_fen("N7/8/8/8/8/8/8/8 w - - 0 1").unwrap();

        let high_eval = high_mobility.evaluate();
        let low_eval = low_mobility.evaluate();

        // Higher mobility position should have higher evaluation
        assert!(high_eval > low_eval);
    }

    #[test]
    fn test_mobility_both_sides_evaluated() {
        // Test that both white and black mobility are evaluated
        let white_advantage = Position::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
        let black_advantage = Position::from_fen("8/8/8/8/3n4/8/8/8 w - - 0 1").unwrap();

        let white_mobility = white_advantage.evaluate_piece_mobility();
        let black_mobility = black_advantage.evaluate_piece_mobility();

        // White advantage should be positive, black advantage should be negative
        assert!(white_mobility.mg > 0);
        assert!(black_mobility.mg < 0);

        // Absolute values should be similar for similar positions
        assert!((white_mobility.mg.abs() - black_mobility.mg.abs()).abs() < 50);
    }
}
