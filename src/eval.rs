use crate::position::Position;
use crate::types::{Color, PieceType, Square};

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

        score
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
}
