use crate::position::Position;
use crate::types::{Color, PieceType, Square};

/// Evaluation score with separate middlegame and endgame values
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Score {
    pub mg: i32, // Middlegame score
    pub eg: i32, // Endgame score
}

impl Score {
    pub fn new(mg: i32, eg: i32) -> Self {
        Self { mg, eg }
    }

    pub fn interpolate(&self, phase_factor: f32) -> i32 {
        (self.mg as f32 * phase_factor + self.eg as f32 * (1.0 - phase_factor)) as i32
    }
}

// Piece-square tables (from white's perspective)
// Values in centipawns (100 = 1 pawn)

const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_MIDDLE_GAME_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

impl Position {
    /// Evaluate the position from the perspective of the side to move
    /// Returns a score in centipawns (100 = 1 pawn advantage)
    /// Positive scores favor the side to move
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;

        // Material and positional evaluation
        score += self.evaluate_material_and_position();

        // Flip score for black to move
        if self.side_to_move == Color::Black {
            -score
        } else {
            score
        }
    }

    fn evaluate_material_and_position(&self) -> i32 {
        let mut score = 0;

        // Iterate through all squares and evaluate pieces
        for square in 0..64 {
            if let Ok(square_obj) = Square::from_index(square as u8) {
                if let Some(piece) = self.board.piece_at(square_obj) {
                    let piece_value = self.get_piece_value(piece.piece_type);
                    let positional_value =
                        self.get_positional_value(piece.piece_type, piece.color, square);

                    let total_value = piece_value + positional_value;

                    match piece.color {
                        Color::White => score += total_value,
                        Color::Black => score -= total_value,
                    }
                }
            }
        }

        // Add pawn structure evaluation
        let pawn_structure_score = self.evaluate_pawn_structure();
        let phase_factor = self.get_game_phase_factor();
        score += pawn_structure_score.interpolate(phase_factor);

        score
    }

    /// Evaluate pawn structure features like isolated, doubled, passed pawns
    fn evaluate_pawn_structure(&self) -> Score {
        let mut score = Score::default();

        // Evaluate isolated pawns
        score.mg += self.evaluate_isolated_pawns().mg;
        score.eg += self.evaluate_isolated_pawns().eg;

        score
    }

    /// Evaluate isolated pawns for both sides
    fn evaluate_isolated_pawns(&self) -> Score {
        let mut score = Score::default();

        // Penalties for isolated pawns
        const ISOLATED_PAWN_PENALTY: Score = Score { mg: -12, eg: -18 };

        // Check all squares for pawns
        for square in 0..64 {
            if let Ok(square_obj) = Square::from_index(square as u8) {
                if let Some(piece) = self.board.piece_at(square_obj) {
                    if piece.piece_type == PieceType::Pawn && self.is_isolated_pawn(square) {
                        match piece.color {
                            Color::White => {
                                score.mg += ISOLATED_PAWN_PENALTY.mg;
                                score.eg += ISOLATED_PAWN_PENALTY.eg;
                            }
                            Color::Black => {
                                score.mg -= ISOLATED_PAWN_PENALTY.mg;
                                score.eg -= ISOLATED_PAWN_PENALTY.eg;
                            }
                        }
                    }
                }
            }
        }

        score
    }

    /// Check if a pawn at the given square is isolated
    fn is_isolated_pawn(&self, square: usize) -> bool {
        let file = square % 8;

        // Get the color of the pawn at this square
        if let Ok(square_obj) = Square::from_index(square as u8) {
            if let Some(pawn) = self.board.piece_at(square_obj) {
                if pawn.piece_type != PieceType::Pawn {
                    return false;
                }

                let pawn_color = pawn.color;

                // Check adjacent files for friendly pawns
                let adjacent_files = if file == 0 {
                    vec![1] // a-file only has b-file adjacent
                } else if file == 7 {
                    vec![6] // h-file only has g-file adjacent  
                } else {
                    vec![file - 1, file + 1] // both adjacent files
                };

                // Check all ranks in adjacent files for friendly pawns
                for adj_file in adjacent_files {
                    for adj_rank in 0..8 {
                        let adj_square = adj_rank * 8 + adj_file;
                        if let Ok(adj_square_obj) = Square::from_index(adj_square as u8) {
                            if let Some(adj_piece) = self.board.piece_at(adj_square_obj) {
                                if adj_piece.piece_type == PieceType::Pawn
                                    && adj_piece.color == pawn_color
                                {
                                    return false; // Found a friendly pawn on adjacent file
                                }
                            }
                        }
                    }
                }

                return true; // No friendly pawns found on adjacent files - this pawn is isolated
            }
        }

        false // Not a pawn or invalid square
    }

    /// Calculate game phase factor (1.0 = opening, 0.0 = endgame)
    fn get_game_phase_factor(&self) -> f32 {
        const MAX_PHASE_MATERIAL: i32 = 2 * (4 * 320 + 2 * 500 + 900); // Knights, Bishops, Rooks, Queen per side
        let current_material = self.get_non_pawn_material();
        (current_material as f32 / MAX_PHASE_MATERIAL as f32).min(1.0)
    }

    /// Get total non-pawn material on the board
    fn get_non_pawn_material(&self) -> i32 {
        let mut material = 0;

        for square in 0..64 {
            if let Ok(square_obj) = Square::from_index(square as u8) {
                if let Some(piece) = self.board.piece_at(square_obj) {
                    if piece.piece_type != PieceType::Pawn && piece.piece_type != PieceType::King {
                        material += self.get_piece_value(piece.piece_type);
                    }
                }
            }
        }

        material
    }

    fn get_piece_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000, // King is invaluable
        }
    }

    fn get_positional_value(&self, piece_type: PieceType, color: Color, square: usize) -> i32 {
        let table = match piece_type {
            PieceType::Pawn => &PAWN_TABLE,
            PieceType::Knight => &KNIGHT_TABLE,
            PieceType::Bishop => &BISHOP_TABLE,
            PieceType::Rook => &ROOK_TABLE,
            PieceType::Queen => &QUEEN_TABLE,
            PieceType::King => &KING_MIDDLE_GAME_TABLE,
        };

        let index = match color {
            Color::White => square,
            Color::Black => 63 - square, // Flip the table for black
        };

        table[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_evaluation() {
        let position = Position::starting_position().expect("Starting position should be valid");
        let eval = position.evaluate();

        // Starting position should be roughly equal (small positional differences allowed)
        assert!(
            eval.abs() < 100,
            "Starting position evaluation should be near zero, got {}",
            eval
        );
    }

    #[test]
    fn test_material_advantage() {
        // Position with white having an extra queen (remove a black piece)
        let fen = "rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let position = Position::from_fen(fen).expect("Valid FEN");
        let eval = position.evaluate();

        // White should have about a rook's worth of advantage
        assert!(
            eval > 400,
            "White should have significant material advantage, got {}",
            eval
        );
    }

    #[test]
    fn test_black_material_advantage() {
        // Position with black having an extra piece (remove a white piece)
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN1 b KQkq - 0 1";
        let position = Position::from_fen(fen).expect("Valid FEN");
        let eval = position.evaluate();

        // Black to move should see significant advantage
        assert!(
            eval > 400,
            "Black should have significant material advantage, got {}",
            eval
        );
    }

    #[test]
    fn test_positional_evaluation() {
        // Test that pieces in center are valued higher
        let center_knight = "rnbqkbnr/pppppppp/8/8/3N4/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
        let corner_knight = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        let center_pos = Position::from_fen(center_knight).expect("Valid FEN");
        let corner_pos = Position::from_fen(corner_knight).expect("Valid FEN");

        // Knight in center should be valued higher than starting position
        assert!(
            center_pos.evaluate() > corner_pos.evaluate(),
            "Knight in center should be valued higher"
        );
    }

    // PAWN STRUCTURE EVALUATION TESTS

    #[test]
    fn test_single_isolated_d_pawn() {
        // Simple case: only a d4 pawn for white, no c or e pawns
        let fen = "4k3/8/8/8/3P4/8/8/4K3 w - - 0 1";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let pawn_score = position.evaluate_isolated_pawns();

        // White should have penalty for isolated d4 pawn
        let expected_score = Score::new(-12, -18);
        assert_eq!(
            pawn_score, expected_score,
            "White's isolated d4 pawn should be penalized"
        );
    }

    #[test]
    fn test_no_isolated_pawns() {
        // Standard opening position - no isolated pawns
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let position = Position::from_fen(fen).expect("Valid FEN");
        let pawn_score = position.evaluate_isolated_pawns();

        // No isolated pawns = no penalty
        let expected_score = Score::new(0, 0);
        assert_eq!(
            pawn_score, expected_score,
            "No isolated pawns should mean no penalty"
        );
    }

    #[test]
    fn test_multiple_isolated_pawns() {
        // Position with several isolated pawns for white: a4, d4, g4 with no adjacent pawns
        let fen = "4k3/8/8/8/P2P2P1/8/8/4K3 w - - 0 1";
        let position = Position::from_fen(fen).expect("Valid FEN");
        let pawn_score = position.evaluate_isolated_pawns();

        // Multiple isolated pawns should stack penalties (3 isolated pawns)
        let expected_score = Score::new(-36, -54); // 3x single penalty
        assert_eq!(
            pawn_score, expected_score,
            "Multiple isolated pawns should accumulate penalties"
        );
    }

    #[test]
    fn test_rook_pawn_isolated() {
        // Edge case: isolated rook pawn
        let fen = "4k3/8/8/8/P7/8/8/4K3 w - - 0 1";
        let position = Position::from_fen(fen).expect("Valid FEN");
        let pawn_score = position.evaluate_isolated_pawns();

        // Isolated rook pawn should still be penalized
        let expected_score = Score::new(-12, -18);
        assert_eq!(
            pawn_score, expected_score,
            "Isolated rook pawn should be penalized"
        );
    }
}
