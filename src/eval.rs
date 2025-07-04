use crate::position::Position;
use crate::types::{Color, PieceType, Square};
use std::ops::{AddAssign, SubAssign};

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

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        self.mg += other.mg;
        self.eg += other.eg;
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, other: Self) {
        self.mg -= other.mg;
        self.eg -= other.eg;
    }
}

// Piece mobility weights (mg, eg values in centipawns per move)
const MOBILITY_WEIGHTS: [(i32, i32); 6] = [
    (5, 3), // Pawn
    (4, 2), // Knight
    (3, 2), // Bishop
    (2, 1), // Rook
    (1, 1), // Queen
    (0, 0), // King (not evaluated)
];

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

    /// Evaluate piece mobility for all pieces
    ///
    /// Piece mobility rewards pieces that have more legal moves available.
    /// Different piece types have different mobility weights:
    /// - Pawns: 5mg/3eg per move (high value as pawn mobility is rare)
    /// - Knights: 4mg/2eg per move (knights value mobility highly)
    /// - Bishops: 3mg/2eg per move (bishops need open diagonals)
    /// - Rooks: 2mg/1eg per move (rooks are powerful but less mobility-dependent)
    /// - Queens: 1mg/1eg per move (queens always have moves, less critical)
    /// - Kings: Not evaluated for mobility
    ///
    /// Mobility is more important in the middlegame than the endgame.
    pub fn evaluate_piece_mobility(&self) -> Score {
        let mut score = Score::default();

        // Evaluate white mobility (positive)
        score += self.evaluate_mobility_for_color(Color::White);

        // Evaluate black mobility (negative)
        score -= self.evaluate_mobility_for_color(Color::Black);

        score
    }

    /// Evaluate mobility for pieces of a specific color
    fn evaluate_mobility_for_color(&self, color: Color) -> Score {
        let mut score = Score::default();

        for (square, piece) in self.pieces_of_color(color) {
            if piece.color == color {
                let move_count = self.count_piece_moves(square, piece.piece_type);
                let piece_index = piece.piece_type as usize;
                let (mg_weight, eg_weight) = MOBILITY_WEIGHTS[piece_index];

                let mobility_bonus = Score::new(mg_weight * move_count, eg_weight * move_count);
                score += mobility_bonus;
            }
        }

        score
    }

    /// Count the number of pseudo-legal moves for a piece at a given square
    fn count_piece_moves(&self, square: Square, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Knight => self.count_knight_moves(square),
            PieceType::Bishop => self.count_bishop_moves(square),
            PieceType::Rook => self.count_rook_moves(square),
            PieceType::Queen => self.count_queen_moves(square),
            PieceType::Pawn => self.count_pawn_moves(square),
            PieceType::King => 0, // King mobility not evaluated
        }
    }

    /// Count knight moves from a square
    fn count_knight_moves(&self, from: Square) -> i32 {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let knight_offsets = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        let mut count = 0;
        for (rank_offset, file_offset) in knight_offsets.iter() {
            let to_rank = from_rank + rank_offset;
            let to_file = from_file + file_offset;

            if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                if let Ok(to_square) = Square::new(to_rank as u8, to_file as u8) {
                    if let Some(piece_at_square) = self.piece_at(from) {
                        let piece_color = piece_at_square.color;
                        if self.is_empty(to_square)
                            || self.is_occupied_by(to_square, piece_color.opposite())
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    /// Count bishop moves from a square  
    pub fn count_bishop_moves(&self, from: Square) -> i32 {
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        self.count_sliding_moves(from, &directions)
    }

    /// Count rook moves from a square
    fn count_rook_moves(&self, from: Square) -> i32 {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        self.count_sliding_moves(from, &directions)
    }

    /// Count queen moves from a square
    fn count_queen_moves(&self, from: Square) -> i32 {
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        self.count_sliding_moves(from, &directions)
    }

    /// Count pawn moves from a square
    fn count_pawn_moves(&self, from: Square) -> i32 {
        if let Some(piece) = self.piece_at(from) {
            let color = piece.color;
            let direction = match color {
                Color::White => 1i8,
                Color::Black => -1i8,
            };
            let start_rank = match color {
                Color::White => 1,
                Color::Black => 6,
            };

            let from_rank = from.rank() as i8;
            let from_file = from.file();
            let mut count = 0;

            // Forward moves
            let one_forward_rank = from_rank + direction;
            if (0..8).contains(&one_forward_rank) {
                if let Ok(one_forward) = Square::new(one_forward_rank as u8, from_file) {
                    if self.is_empty(one_forward) {
                        count += 1;

                        // Double push from starting position
                        if from.rank() == start_rank {
                            let two_forward_rank = from_rank + 2 * direction;
                            if (0..8).contains(&two_forward_rank) {
                                if let Ok(two_forward) =
                                    Square::new(two_forward_rank as u8, from_file)
                                {
                                    if self.is_empty(two_forward) {
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Captures
            for &file_offset in &[-1i8, 1i8] {
                let capture_file = from_file as i8 + file_offset;
                let capture_rank = from_rank + direction;

                if (0..8).contains(&capture_file) && (0..8).contains(&capture_rank) {
                    if let Ok(capture_square) = Square::new(capture_rank as u8, capture_file as u8)
                    {
                        if self.is_occupied_by(capture_square, color.opposite()) {
                            count += 1;
                        }

                        // En passant
                        if let Some(ep_square) = self.en_passant {
                            if capture_square == ep_square {
                                count += 1;
                            }
                        }
                    }
                }
            }

            count
        } else {
            0
        }
    }

    /// Count sliding piece moves (bishop, rook, queen)
    fn count_sliding_moves(&self, from: Square, directions: &[(i8, i8)]) -> i32 {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let mut count = 0;

        if let Some(piece_at_from) = self.piece_at(from) {
            let piece_color = piece_at_from.color;

            for (rank_dir, file_dir) in directions.iter() {
                let mut rank = from_rank;
                let mut file = from_file;

                loop {
                    rank += rank_dir;
                    file += file_dir;

                    if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                        break;
                    }

                    if let Ok(to_square) = Square::new(rank as u8, file as u8) {
                        if self.is_empty(to_square) {
                            count += 1;
                        } else if self.is_occupied_by(to_square, piece_color.opposite()) {
                            count += 1;
                            break; // Can't continue past capture
                        } else {
                            break; // Own piece blocks further movement
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        count
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

        // Add king safety evaluation
        let king_safety_score = self.evaluate_king_safety();
        score += king_safety_score.interpolate(phase_factor);

        // Add piece mobility evaluation
        let mobility_score = self.evaluate_piece_mobility();
        score += mobility_score.interpolate(phase_factor);

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

    /// Evaluate king safety for both sides
    fn evaluate_king_safety(&self) -> Score {
        let mut score = Score::default();

        // Find king positions
        let white_king_square = self.find_king(Color::White).map(|sq| sq.index() as usize);
        let black_king_square = self.find_king(Color::Black).map(|sq| sq.index() as usize);

        if let (Some(white_king), Some(black_king)) = (white_king_square, black_king_square) {
            // Evaluate white king safety (positive score)
            let white_safety = self.evaluate_single_king_safety(Color::White, white_king);
            score.mg += white_safety.mg;
            score.eg += white_safety.eg;

            // Evaluate black king safety (negative score since it's bad for us)
            let black_safety = self.evaluate_single_king_safety(Color::Black, black_king);
            score.mg -= black_safety.mg;
            score.eg -= black_safety.eg;
        }

        score
    }

    /// Evaluate king safety for a single king
    fn evaluate_single_king_safety(&self, color: Color, king_square: usize) -> Score {
        let mut score = Score::default();

        // King position evaluation
        let rank = king_square / 8;
        let file = king_square % 8;

        // Bonus for castled position (corners are safer in middlegame)
        if self.is_king_castled(color, king_square) {
            score.mg += 40; // Castled king bonus
            score.eg += 10; // Less important in endgame
        } else if (2..=5).contains(&rank) && (2..=5).contains(&file) {
            // King exposed in center - significant penalty
            score.mg -= 80; // Heavy penalty for exposed king
            score.eg -= 20; // Somewhat less dangerous in endgame
        }

        // Pawn shield evaluation
        let pawn_shield_score = self.evaluate_pawn_shield(color, king_square);
        score.mg += pawn_shield_score.mg;
        score.eg += pawn_shield_score.eg;

        // Open file evaluation
        let open_file_score = self.evaluate_open_files_near_king(color, king_square);
        score.mg += open_file_score.mg;
        score.eg += open_file_score.eg;

        score
    }

    /// Check if king is in a castled position
    fn is_king_castled(&self, color: Color, king_square: usize) -> bool {
        let rank = king_square / 8;
        let file = king_square % 8;

        match color {
            Color::White => {
                // White castled: king on g1 (file 6) or c1 (file 2), rank 0
                rank == 0 && (file == 6 || file == 2)
            }
            Color::Black => {
                // Black castled: king on g8 (file 6) or c8 (file 2), rank 7
                rank == 7 && (file == 6 || file == 2)
            }
        }
    }

    /// Evaluate pawn shield around the king
    fn evaluate_pawn_shield(&self, color: Color, king_square: usize) -> Score {
        let mut score = Score::default();
        let file = king_square % 8;

        // Check pawn shield in front of king
        let shield_files = match file {
            0 => vec![0, 1],                     // a-file: check a, b files
            7 => vec![6, 7],                     // h-file: check g, h files
            _ => vec![file - 1, file, file + 1], // middle: check three files
        };

        let target_rank = match color {
            Color::White => 1, // White pawns should be on 2nd rank (index 1)
            Color::Black => 6, // Black pawns should be on 7th rank (index 6)
        };

        for shield_file in shield_files {
            let shield_square = target_rank * 8 + shield_file;
            if let Ok(square_obj) = Square::from_index(shield_square as u8) {
                if let Some(piece) = self.board.piece_at(square_obj) {
                    if piece.piece_type == PieceType::Pawn && piece.color == color {
                        score.mg += 15; // Bonus for each pawn in shield
                        score.eg += 5; // Less important in endgame
                    }
                } else {
                    // Missing pawn in shield
                    score.mg -= 20; // Penalty for missing shield pawn
                    score.eg -= 8; // Less dangerous in endgame
                }
            }
        }

        score
    }

    /// Evaluate open files near the king
    fn evaluate_open_files_near_king(&self, color: Color, king_square: usize) -> Score {
        let mut score = Score::default();
        let king_file = king_square % 8;

        // Check adjacent files for openness
        let check_files = match king_file {
            0 => vec![0, 1],                                    // a-file: check a, b
            7 => vec![6, 7],                                    // h-file: check g, h
            _ => vec![king_file - 1, king_file, king_file + 1], // middle files
        };

        for file in check_files {
            if self.is_file_open_or_semi_open(file, color) {
                score.mg -= 40; // Penalty for open/semi-open file near king
                score.eg -= 15; // Less dangerous in endgame
            }
        }

        score
    }

    /// Check if a file is open or semi-open (no friendly pawns)
    fn is_file_open_or_semi_open(&self, file: usize, color: Color) -> bool {
        for rank in 0..8 {
            let square = rank * 8 + file;
            if let Ok(square_obj) = Square::from_index(square as u8) {
                if let Some(piece) = self.board.piece_at(square_obj) {
                    if piece.piece_type == PieceType::Pawn && piece.color == color {
                        return false; // Found friendly pawn - file is not open
                    }
                }
            }
        }
        true // No friendly pawns found - file is open or semi-open
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
    pub fn get_game_phase_factor(&self) -> f32 {
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

    // KING SAFETY EVALUATION TESTS

    #[test]
    fn test_safe_castled_king() {
        // White king castled kingside with full pawn shield (f2, g2, h2)
        let fen = "r1bqk2r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 w kq - 5 5";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();

        // Safe castled king should get positive bonus
        assert!(
            king_safety_score.mg > 0,
            "Safe castled king should get middlegame bonus, got {}",
            king_safety_score.mg
        );
        assert!(
            king_safety_score.eg >= 0,
            "King safety matters less in endgame, got {}",
            king_safety_score.eg
        );
    }

    #[test]
    fn test_exposed_king_in_center() {
        // White king exposed in center, black king safe
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/4K3/PPPP1PPP/RNBQ3R w kq - 0 5";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();

        // Exposed king should get significant penalty
        assert!(
            king_safety_score.mg < -50,
            "Exposed king in center should get large penalty, got {}",
            king_safety_score.mg
        );
    }

    #[test]
    fn test_broken_pawn_shield() {
        // White king castled but h2 pawn moved (h3), breaking pawn shield
        let fen = "r1bqk2r/pppp1ppp/2n2n2/4p3/2B1P3/5N1P/PPPP1PP1/RNBQ1RK1 w kq - 0 5";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();
        let intact_shield_fen = "r1bqk2r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 w kq - 5 5";
        let intact_position = Position::from_fen(intact_shield_fen).expect("Valid FEN");
        let intact_score = intact_position.evaluate_king_safety();

        // Broken pawn shield should be worse than intact shield
        assert!(
            king_safety_score.mg < intact_score.mg,
            "Broken pawn shield should be penalized more than intact shield"
        );
    }

    #[test]
    fn test_open_file_near_king() {
        // White king with open h-file, black king with intact pawn shield
        let fen = "r1bq1rk1/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PP1/RNBQ1RK1 w - - 0 5";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();

        // White king with open h-file should be worse than black king with full pawn shield
        assert!(
            king_safety_score.mg < 0,
            "Open file near king should be dangerous, got {}",
            king_safety_score.mg
        );
    }

    #[test]
    fn test_king_safety_both_sides() {
        // White king castled and safe, black king exposed in center
        let fen = "r1bq1b1r/pppp1ppp/2n2n2/4k3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b - - 5 5";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();

        // Should evaluate both white and black king safety
        // White castled (safe) vs black exposed in center should show clear difference
        assert!(
            king_safety_score.mg != 0 || king_safety_score.eg != 0,
            "King safety should evaluate both sides and show difference, got mg={}, eg={}",
            king_safety_score.mg,
            king_safety_score.eg
        );
    }

    #[test]
    fn test_endgame_king_safety_reduced() {
        // King safety should matter less in endgame (fewer attacking pieces)
        let fen = "6k1/6pp/8/8/8/8/6PP/6K1 w - - 0 50";
        let position = Position::from_fen(fen).expect("Valid FEN");

        let king_safety_score = position.evaluate_king_safety();

        // In pure king and pawn endgame, king safety bonus should be minimal
        assert!(
            king_safety_score.eg.abs() < king_safety_score.mg.abs()
                || king_safety_score.mg.abs() < 20,
            "King safety should matter less in endgame, mg: {}, eg: {}",
            king_safety_score.mg,
            king_safety_score.eg
        );
    }
}
