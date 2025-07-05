use crate::board::{Board, BoardError};
use crate::tablebase::Tablebase;
use crate::transposition::ZOBRIST_HASHER;
use crate::types::*;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

// Global flag for enabling/disabling tablebase lookup (for testing)
static TABLEBASE_ENABLED: AtomicBool = AtomicBool::new(true);

/// Convert a piece to its Unicode chess symbol
fn piece_to_unicode_symbol(piece: Piece) -> &'static str {
    match (piece.color, piece.piece_type) {
        (Color::White, PieceType::King) => "♔",
        (Color::White, PieceType::Queen) => "♕",
        (Color::White, PieceType::Rook) => "♖",
        (Color::White, PieceType::Bishop) => "♗",
        (Color::White, PieceType::Knight) => "♘",
        (Color::White, PieceType::Pawn) => "♙",
        (Color::Black, PieceType::King) => "♚",
        (Color::Black, PieceType::Queen) => "♛",
        (Color::Black, PieceType::Rook) => "♜",
        (Color::Black, PieceType::Bishop) => "♝",
        (Color::Black, PieceType::Knight) => "♞",
        (Color::Black, PieceType::Pawn) => "♟",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionError {
    BoardError(BoardError),
}

impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionError::BoardError(e) => write!(f, "Position error: {}", e),
        }
    }
}

impl std::error::Error for PositionError {}

impl From<BoardError> for PositionError {
    fn from(error: BoardError) -> Self {
        PositionError::BoardError(error)
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub board: Board,
    pub side_to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_hash: u64,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.side_to_move == other.side_to_move
            && self.castling_rights == other.castling_rights
            && self.en_passant == other.en_passant
            && self.halfmove_clock == other.halfmove_clock
            && self.fullmove_number == other.fullmove_number
        // Explicitly exclude zobrist_hash - it's derived from other fields
    }
}

impl Position {
    pub fn new() -> Self {
        let mut position = Self {
            board: Board::new(),
            side_to_move: Color::White,
            castling_rights: CastlingRights::new(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0, // Temporary
        };
        position.zobrist_hash = ZOBRIST_HASHER.compute_hash(&position).unwrap_or(0);
        position
    }

    pub fn starting_position() -> Result<Self, PositionError> {
        let mut position = Self {
            board: Board::starting_position()?,
            side_to_move: Color::White,
            castling_rights: CastlingRights::new(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0, // Temporary
        };
        position.zobrist_hash = ZOBRIST_HASHER.compute_hash(&position).unwrap_or(0);
        Ok(position)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board.piece_at(square)
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.board.set_piece(square, piece);
    }

    pub fn is_empty(&self, square: Square) -> bool {
        self.board.is_empty(square)
    }

    pub fn is_occupied(&self, square: Square) -> bool {
        self.board.is_occupied(square)
    }

    pub fn is_occupied_by(&self, square: Square, color: Color) -> bool {
        self.board.is_occupied_by(square, color)
    }

    pub fn find_king(&self, color: Color) -> Option<Square> {
        self.board.find_king(color)
    }

    pub fn pieces_of_color(&self, color: Color) -> Vec<(Square, Piece)> {
        self.board.pieces_of_color(color)
    }

    pub fn pieces_of_type(&self, color: Color, piece_type: PieceType) -> Vec<Square> {
        self.board.pieces_of_type(color, piece_type)
    }

    pub fn material_count(&self, color: Color) -> i32 {
        self.board.material_count(color)
    }

    pub fn is_check(&self, color: Color) -> bool {
        // Find the king
        let king_square = match self.find_king(color) {
            Some(square) => square,
            None => return false,
        };

        // Check if any opponent piece can attack the king
        self.is_square_attacked(king_square, color.opposite())
    }

    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        // Check pawn attacks
        if self.is_square_attacked_by_pawn(square, by_color) {
            return true;
        }

        // Check knight attacks
        if self.is_square_attacked_by_knight(square, by_color) {
            return true;
        }

        // Check bishop/queen diagonal attacks
        if self.is_square_attacked_by_bishop_or_queen(square, by_color) {
            return true;
        }

        // Check rook/queen straight attacks
        if self.is_square_attacked_by_rook_or_queen(square, by_color) {
            return true;
        }

        // Check king attacks
        if self.is_square_attacked_by_king(square, by_color) {
            return true;
        }

        false
    }

    fn is_square_attacked_by_pawn(&self, square: Square, by_color: Color) -> bool {
        let rank = square.rank();
        let file = square.file();

        let attack_rank = match by_color {
            Color::White => {
                if rank == 0 {
                    return false;
                }
                rank - 1
            }
            Color::Black => {
                if rank == 7 {
                    return false;
                }
                rank + 1
            }
        };

        // Check left diagonal
        if file > 0 {
            if let Ok(attack_square) = Square::new(attack_rank, file - 1) {
                if let Some(piece) = self.piece_at(attack_square) {
                    if piece.color == by_color && piece.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }

        // Check right diagonal
        if file < 7 {
            if let Ok(attack_square) = Square::new(attack_rank, file + 1) {
                if let Some(piece) = self.piece_at(attack_square) {
                    if piece.color == by_color && piece.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn is_square_attacked_by_knight(&self, square: Square, by_color: Color) -> bool {
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        let knight_moves = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        for (dr, df) in knight_moves.iter() {
            let new_rank = rank + dr;
            let new_file = file + df;

            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                if let Ok(attack_square) = Square::new(new_rank as u8, new_file as u8) {
                    if let Some(piece) = self.piece_at(attack_square) {
                        if piece.color == by_color && piece.piece_type == PieceType::Knight {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn is_square_attacked_by_bishop_or_queen(&self, square: Square, by_color: Color) -> bool {
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

        for (dr, df) in directions.iter() {
            let mut rank = square.rank() as i8;
            let mut file = square.file() as i8;

            loop {
                rank += dr;
                file += df;

                if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                    break;
                }

                if let Ok(check_square) = Square::new(rank as u8, file as u8) {
                    if let Some(piece) = self.piece_at(check_square) {
                        if piece.color == by_color
                            && (piece.piece_type == PieceType::Bishop
                                || piece.piece_type == PieceType::Queen)
                        {
                            return true;
                        }
                        break; // Piece blocks further movement
                    }
                }
            }
        }

        false
    }

    fn is_square_attacked_by_rook_or_queen(&self, square: Square, by_color: Color) -> bool {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dr, df) in directions.iter() {
            let mut rank = square.rank() as i8;
            let mut file = square.file() as i8;

            loop {
                rank += dr;
                file += df;

                if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                    break;
                }

                if let Ok(check_square) = Square::new(rank as u8, file as u8) {
                    if let Some(piece) = self.piece_at(check_square) {
                        if piece.color == by_color
                            && (piece.piece_type == PieceType::Rook
                                || piece.piece_type == PieceType::Queen)
                        {
                            return true;
                        }
                        break; // Piece blocks further movement
                    }
                }
            }
        }

        false
    }

    fn is_square_attacked_by_king(&self, square: Square, by_color: Color) -> bool {
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        let king_moves = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dr, df) in king_moves.iter() {
            let new_rank = rank + dr;
            let new_file = file + df;

            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                if let Ok(attack_square) = Square::new(new_rank as u8, new_file as u8) {
                    if let Some(piece) = self.piece_at(attack_square) {
                        if piece.color == by_color && piece.piece_type == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.opposite();
        if self.side_to_move == Color::White {
            self.fullmove_number += 1;
        }
        // Update zobrist hash for side change
        self.zobrist_hash = ZOBRIST_HASHER
            .update_side_to_move(self.zobrist_hash)
            .unwrap_or(self.zobrist_hash);
    }

    /// Get the current zobrist hash of this position
    pub fn hash(&self) -> u64 {
        self.zobrist_hash
    }

    /// Recompute zobrist hash from scratch (for verification)
    pub fn recompute_hash(&mut self) {
        self.zobrist_hash = ZOBRIST_HASHER.compute_hash(self).unwrap_or(0);
    }

    /// Generate ASCII representation of the board
    pub fn to_ascii_board(&self) -> String {
        let mut result = String::new();

        // Header with file coordinates
        result.push_str("  a b c d e f g h\n");

        // Ranks 8 to 1 (top to bottom)
        for rank in (0..8).rev() {
            result.push_str(&format!("{} ", rank + 1));

            for file in 0..8 {
                if let Ok(square) = Square::new(rank, file) {
                    let symbol = match self.board.piece_at(square) {
                        Some(piece) => piece_to_unicode_symbol(piece),
                        None => "·",
                    };
                    result.push_str(symbol);
                    if file < 7 {
                        result.push(' ');
                    }
                }
            }

            result.push_str(&format!(" {}\n", rank + 1));
        }

        // Footer with file coordinates
        result.push_str("  a b c d e f g h");

        result
    }

    /// Check if this position is suitable for tablebase lookup
    pub fn is_tablebase_position(&self) -> bool {
        self.board.count_total_pieces() <= crate::tablebase::MAX_TABLEBASE_PIECES
    }

    /// Check if the position has a piece of the given type and color
    pub fn has_piece(&self, color: Color, piece_type: PieceType) -> bool {
        !self.board.pieces_of_type(color, piece_type).is_empty()
    }

    /// Probe tablebase for this position (mock implementation for now)
    pub fn probe_tablebase(&self) -> Option<crate::tablebase::TablebaseResult> {
        if !self.is_tablebase_lookup_enabled() || !self.is_tablebase_position() {
            return None;
        }

        // Use mock tablebase for now
        let tablebase = crate::tablebase::MockTablebase::new();
        if let Ok(result) = tablebase.probe(self) {
            Some(result)
        } else {
            None
        }
    }

    /// Enable or disable tablebase lookup (for testing)
    pub fn enable_tablebase_lookup(&self, enabled: bool) {
        // For testing, modify a global flag
        // In a real implementation, this would be a field on Position or a global setting
        TABLEBASE_ENABLED.store(enabled, Ordering::Relaxed);
    }

    /// Check if tablebase lookup is enabled (for testing)
    pub fn is_tablebase_lookup_enabled(&self) -> bool {
        TABLEBASE_ENABLED.load(Ordering::Relaxed)
    }

    /// Find best move using tablebase knowledge (mock implementation)
    pub fn find_best_move_with_tablebase(&self, _time_ms: u64) -> MockSearchResult {
        // Mock implementation for testing
        let score = if let Some(tb_result) = self.probe_tablebase() {
            tb_result.to_search_score()
        } else {
            self.evaluate()
        };

        MockSearchResult {
            best_move: Some(MockMove::new()),
            score,
            depth: 15,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock types for testing tablebase integration
#[derive(Debug)]
pub struct MockSearchResult {
    pub best_move: Option<MockMove>,
    pub score: i32,
    pub depth: u8,
}

#[derive(Debug)]
pub struct MockMove;

impl MockMove {
    pub fn new() -> Self {
        Self
    }
}
