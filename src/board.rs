use crate::types::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum BoardError {
    InvalidSquare(&'static str),
    SetupError(String),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardError::InvalidSquare(msg) => write!(f, "Invalid square: {}", msg),
            BoardError::SetupError(msg) => write!(f, "Board setup error: {}", msg),
        }
    }
}

impl std::error::Error for BoardError {}

impl From<&'static str> for BoardError {
    fn from(msg: &'static str) -> Self {
        BoardError::InvalidSquare(msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    squares: [Option<Piece>; 64],
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [None; 64],
        }
    }

    pub fn starting_position() -> Result<Self, BoardError> {
        let mut board = Self::new();

        // White pieces
        board.set_piece(
            Square::from_algebraic("a1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Rook)),
        );
        board.set_piece(
            Square::from_algebraic("b1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Knight)),
        );
        board.set_piece(
            Square::from_algebraic("c1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Bishop)),
        );
        board.set_piece(
            Square::from_algebraic("d1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Queen)),
        );
        board.set_piece(
            Square::from_algebraic("e1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::King)),
        );
        board.set_piece(
            Square::from_algebraic("f1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Bishop)),
        );
        board.set_piece(
            Square::from_algebraic("g1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Knight)),
        );
        board.set_piece(
            Square::from_algebraic("h1").map_err(BoardError::from)?,
            Some(Piece::new(Color::White, PieceType::Rook)),
        );

        for file in 0..8 {
            let square = Square::new(1, file).map_err(BoardError::from)?;
            board.set_piece(square, Some(Piece::new(Color::White, PieceType::Pawn)));
        }

        // Black pieces
        board.set_piece(
            Square::from_algebraic("a8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Rook)),
        );
        board.set_piece(
            Square::from_algebraic("b8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Knight)),
        );
        board.set_piece(
            Square::from_algebraic("c8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Bishop)),
        );
        board.set_piece(
            Square::from_algebraic("d8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Queen)),
        );
        board.set_piece(
            Square::from_algebraic("e8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::King)),
        );
        board.set_piece(
            Square::from_algebraic("f8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Bishop)),
        );
        board.set_piece(
            Square::from_algebraic("g8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Knight)),
        );
        board.set_piece(
            Square::from_algebraic("h8").map_err(BoardError::from)?,
            Some(Piece::new(Color::Black, PieceType::Rook)),
        );

        for file in 0..8 {
            let square = Square::new(6, file).map_err(BoardError::from)?;
            board.set_piece(square, Some(Piece::new(Color::Black, PieceType::Pawn)));
        }

        Ok(board)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.squares[square.index() as usize]
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.index() as usize] = piece;
    }

    pub fn is_empty(&self, square: Square) -> bool {
        self.piece_at(square).is_none()
    }

    pub fn is_occupied(&self, square: Square) -> bool {
        self.piece_at(square).is_some()
    }

    pub fn is_occupied_by(&self, square: Square, color: Color) -> bool {
        match self.piece_at(square) {
            Some(piece) => piece.color == color,
            None => false,
        }
    }

    pub fn find_king(&self, color: Color) -> Option<Square> {
        for index in 0..64 {
            if let Ok(square) = Square::from_index(index) {
                if let Some(piece) = self.piece_at(square) {
                    if piece.color == color && piece.piece_type == PieceType::King {
                        return Some(square);
                    }
                }
            }
        }
        None
    }

    pub fn pieces_of_color(&self, color: Color) -> Vec<(Square, Piece)> {
        let mut pieces = Vec::new();
        for index in 0..64 {
            if let Ok(square) = Square::from_index(index) {
                if let Some(piece) = self.piece_at(square) {
                    if piece.color == color {
                        pieces.push((square, piece));
                    }
                }
            }
        }
        pieces
    }

    pub fn pieces_of_type(&self, color: Color, piece_type: PieceType) -> Vec<Square> {
        let mut squares = Vec::new();
        for index in 0..64 {
            if let Ok(square) = Square::from_index(index) {
                if let Some(piece) = self.piece_at(square) {
                    if piece.color == color && piece.piece_type == piece_type {
                        squares.push(square);
                    }
                }
            }
        }
        squares
    }

    pub fn material_count(&self, color: Color) -> i32 {
        let mut total = 0;
        for index in 0..64 {
            if let Ok(square) = Square::from_index(index) {
                if let Some(piece) = self.piece_at(square) {
                    if piece.color == color {
                        total += piece.material_value();
                    }
                }
            }
        }
        total
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                if let Ok(square) = Square::new(rank, file) {
                    let piece_str = match self.piece_at(square) {
                        Some(piece) => piece.to_string(),
                        None => ".".to_string(),
                    };
                    write!(f, "{} ", piece_str)?;
                } else {
                    write!(f, "? ")?; // This should never happen with valid rank/file
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
        Ok(())
    }
}
