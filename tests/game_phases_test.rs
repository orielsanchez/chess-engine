use chess_engine::position::Position;

/// Test comprehensive game phase detection and phase-specific evaluation
///
/// Game phases should be:
/// - Opening: First 10-15 moves, development focus, full material
/// - Early Middlegame: Material ≥ 75% of maximum, most pieces developed
/// - Late Middlegame: Material 25-75% of maximum, tactical complexity
/// - Endgame: Material < 25% of maximum, king activity important
/// - Pawn Endgame: Only pawns + kings, precise calculation needed

#[cfg(test)]
mod tests {
    use super::*;

    // GAME PHASE DETECTION TESTS

    #[test]
    fn test_opening_phase_detection() {
        // Starting position should be detected as opening phase
        let position = Position::starting_position().expect("Valid starting position");

        let phase = position.get_game_phase();

        assert_eq!(
            phase,
            chess_engine::eval::GamePhase::Opening,
            "Starting position should be detected as opening phase"
        );
    }

    #[test]
    fn test_early_middlegame_phase_detection() {
        // Position with a few pieces exchanged (knights and a bishop gone)
        let fen = "r1bqk2r/ppp2ppp/5n2/3p4/1bPP4/5N2/PP3PPP/RNBQKB1R w KQkq - 0 8";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let phase = position.get_game_phase();

        assert_eq!(
            phase,
            chess_engine::eval::GamePhase::EarlyMiddlegame,
            "Position with some exchanges should be early middlegame"
        );
    }

    #[test]
    fn test_late_middlegame_phase_detection() {
        // Position with many pieces exchanged (only rooks and queen left)
        let fen = "2r2rk1/1pp2ppp/p7/8/8/2P5/1P3PPP/2R1R1K1 w - - 0 20";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let phase = position.get_game_phase();

        assert_eq!(
            phase,
            chess_engine::eval::GamePhase::LateMiddlegame,
            "Position with fewer pieces should be late middlegame"
        );
    }

    #[test]
    fn test_endgame_phase_detection() {
        // Endgame with few pieces remaining
        let fen = "8/8/3k4/8/8/3K4/4R3/8 w - - 0 40";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let phase = position.get_game_phase();

        assert_eq!(
            phase,
            chess_engine::eval::GamePhase::Endgame,
            "Position with minimal material should be endgame"
        );
    }

    #[test]
    fn test_pawn_endgame_phase_detection() {
        // Pure pawn endgame
        let fen = "8/3p4/8/2P5/8/8/5PPP/6K1 w - - 0 45";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let phase = position.get_game_phase();

        assert_eq!(
            phase,
            chess_engine::eval::GamePhase::PawnEndgame,
            "Position with only pawns and kings should be pawn endgame"
        );
    }

    // OPENING-SPECIFIC EVALUATION TESTS

    #[test]
    fn test_opening_development_bonus() {
        // Position where white has developed pieces, black hasn't
        let developed = "rnbqkbnr/pppppppp/8/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 1 2";
        let undeveloped = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";

        let developed_pos = Position::from_fen(developed).expect("Valid FEN");
        let undeveloped_pos = Position::from_fen(undeveloped).expect("Valid FEN");

        let developed_eval = developed_pos.evaluate_opening_phase();
        let undeveloped_eval = undeveloped_pos.evaluate_opening_phase();

        assert!(
            developed_eval.mg > undeveloped_eval.mg,
            "Developed pieces should get opening bonus, got developed: {}, undeveloped: {}",
            developed_eval.mg,
            undeveloped_eval.mg
        );
    }

    #[test]
    fn test_opening_center_control_bonus() {
        // Position with central pawns vs edge pawns
        let center_control = "rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2";
        let no_center = "rnbqkbnr/pppppppp/8/8/7P/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1";

        let center_pos = Position::from_fen(center_control).expect("Valid FEN");
        let no_center_pos = Position::from_fen(no_center).expect("Valid FEN");

        let center_eval = center_pos.evaluate_opening_phase();
        let no_center_eval = no_center_pos.evaluate_opening_phase();

        assert!(
            center_eval.mg > no_center_eval.mg,
            "Central pawn control should get opening bonus"
        );
    }

    #[test]
    fn test_opening_castling_bonus() {
        // Castled vs uncastled king in opening
        let castled = "rnbq1rk1/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 w - - 6 4";
        let uncastled = "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 5 4";

        let castled_pos = Position::from_fen(castled).expect("Valid FEN");
        let uncastled_pos = Position::from_fen(uncastled).expect("Valid FEN");

        let castled_eval = castled_pos.evaluate_opening_phase();
        let uncastled_eval = uncastled_pos.evaluate_opening_phase();

        assert!(
            castled_eval.mg > uncastled_eval.mg,
            "Castled king should get opening safety bonus"
        );
    }

    // MIDDLEGAME-SPECIFIC EVALUATION TESTS

    #[test]
    fn test_middlegame_piece_coordination() {
        // Position with well-coordinated pieces vs scattered pieces
        let coordinated = "r2q1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQR1K1 w - - 0 8";
        let scattered = "r2q1rk1/ppp2ppp/2np4/2b1p3/4P1n1/3P1N2/PPP2PPP/RNBQR1K1 w - - 0 8";

        let coordinated_pos = Position::from_fen(coordinated).expect("Valid FEN");
        let scattered_pos = Position::from_fen(scattered).expect("Valid FEN");

        let coordinated_eval = coordinated_pos.evaluate_middlegame_phase();
        let scattered_eval = scattered_pos.evaluate_middlegame_phase();

        assert!(
            coordinated_eval.mg > scattered_eval.mg,
            "Coordinated pieces should get middlegame bonus"
        );
    }

    #[test]
    fn test_middlegame_tactical_opportunities() {
        // Position with tactical threats vs quiet position
        let tactical = "r1bq1rk1/ppp2ppp/2n5/3pp3/1bBP4/2N2N2/PPP2PPP/R1BQK2R w KQ - 0 8";
        let quiet = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4";

        let tactical_pos = Position::from_fen(tactical).expect("Valid FEN");
        let quiet_pos = Position::from_fen(quiet).expect("Valid FEN");

        let tactical_eval = tactical_pos.evaluate_middlegame_phase();
        let quiet_eval = quiet_pos.evaluate_middlegame_phase();

        // Tactical position should have different evaluation characteristics
        assert!(
            tactical_eval.mg != quiet_eval.mg,
            "Tactical positions should be evaluated differently in middlegame"
        );
    }

    // ENDGAME-SPECIFIC EVALUATION TESTS

    #[test]
    fn test_endgame_king_activity() {
        // Active king vs passive king in endgame
        let active_king = "8/8/3k4/8/3K4/8/8/8 w - - 0 50";
        let passive_king = "8/8/8/8/8/3k4/8/7K w - - 0 50";

        let active_pos = Position::from_fen(active_king).expect("Valid FEN");
        let passive_pos = Position::from_fen(passive_king).expect("Valid FEN");

        let active_eval = active_pos.evaluate_endgame_phase();
        let passive_eval = passive_pos.evaluate_endgame_phase();

        assert!(
            active_eval.eg > passive_eval.eg,
            "Active king should get endgame bonus, got active: {}, passive: {}",
            active_eval.eg,
            passive_eval.eg
        );
    }

    #[test]
    fn test_endgame_pawn_promotion_race() {
        // Pawn promotion race scenarios
        let white_wins_race = "8/8/8/8/8/8/P7/7k w - - 0 50";
        let black_wins_race = "7K/8/8/8/8/8/8/7p w - - 0 50";

        let white_race_pos = Position::from_fen(white_wins_race).expect("Valid FEN");
        let black_race_pos = Position::from_fen(black_wins_race).expect("Valid FEN");

        let white_race_eval = white_race_pos.evaluate_endgame_phase();
        let black_race_eval = black_race_pos.evaluate_endgame_phase();

        assert!(
            white_race_eval.eg > black_race_eval.eg,
            "Faster promotion should be favored in endgame"
        );
    }

    #[test]
    fn test_endgame_opposition() {
        // King opposition in pawn endgames
        let with_opposition = "8/8/8/3k4/8/3K4/8/8 w - - 0 50";
        let without_opposition = "8/8/8/2k5/8/3K4/8/8 w - - 0 50";

        let opposition_pos = Position::from_fen(with_opposition).expect("Valid FEN");
        let no_opposition_pos = Position::from_fen(without_opposition).expect("Valid FEN");

        let opposition_eval = opposition_pos.evaluate_endgame_phase();
        let no_opposition_eval = no_opposition_pos.evaluate_endgame_phase();

        // Opposition should matter in king endgames
        assert!(
            opposition_eval.eg != no_opposition_eval.eg,
            "Opposition should affect endgame evaluation"
        );
    }

    // INTEGRATION TESTS

    #[test]
    fn test_phase_specific_evaluation_integration() {
        // Same material, different phases should evaluate differently
        let opening = Position::starting_position().expect("Valid starting position");

        // Create endgame with same material count but different piece distribution
        let endgame = "4k3/8/8/8/8/8/8/Q3K2R w K - 0 40";
        let endgame_pos = Position::from_fen(endgame).expect("Valid FEN");

        let opening_eval = opening.evaluate();
        let endgame_eval = endgame_pos.evaluate();

        // Different phases should produce meaningfully different evaluations
        assert!(
            (opening_eval - endgame_eval).abs() > 50,
            "Different game phases should evaluate positions differently"
        );
    }

    #[test]
    fn test_smooth_phase_transitions() {
        // Gradual material loss should create smooth phase transitions
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Opening
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4", // Early mid
            "r2q1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQR1K1 w - - 0 8", // Late mid
            "8/8/3k4/8/8/3K4/4R3/8 w - - 0 40",                         // Endgame
        ];

        let mut prev_factor = 2.0; // Start higher than any possible value

        for fen in positions {
            let pos = Position::from_fen(fen).expect("Valid FEN");
            let factor = pos.get_game_phase_factor();

            // Game phase factor should decrease monotonically as material decreases
            assert!(
                factor < prev_factor,
                "Game phase factor should decrease as material decreases, got {} after {}",
                factor,
                prev_factor
            );
            prev_factor = factor;
        }
    }
}
