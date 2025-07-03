use crate::position::Position;
use crate::types::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FenError {
    InvalidFormat,
    InvalidPiece(char),
    InvalidColor(char),
    InvalidCastling(String),
    InvalidEnPassant(String),
    InvalidHalfmove(String),
    InvalidFullmove(String),
    InvalidRankCount,
    InvalidFileCount,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::InvalidFormat => write!(f, "Invalid FEN format"),
            FenError::InvalidPiece(c) => write!(f, "Invalid piece character: {}", c),
            FenError::InvalidColor(c) => write!(f, "Invalid color character: {}", c),
            FenError::InvalidCastling(s) => write!(f, "Invalid castling string: {}", s),
            FenError::InvalidEnPassant(s) => write!(f, "Invalid en passant square: {}", s),
            FenError::InvalidHalfmove(s) => write!(f, "Invalid halfmove clock: {}", s),
            FenError::InvalidFullmove(s) => write!(f, "Invalid fullmove number: {}", s),
            FenError::InvalidRankCount => write!(f, "Invalid number of ranks"),
            FenError::InvalidFileCount => write!(f, "Invalid number of files in rank"),
        }
    }
}

impl std::error::Error for FenError {}

impl Position {
    /// Parse a FEN string into a Position
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenError::InvalidFormat);
        }

        let mut position = Position::new();

        // Parse piece placement
        Self::parse_piece_placement(&mut position, parts[0])?;

        // Parse active color
        position.side_to_move = Self::parse_active_color(parts[1])?;

        // Parse castling availability
        position.castling_rights = Self::parse_castling(parts[2])?;

        // Parse en passant target square
        position.en_passant = Self::parse_en_passant(parts[3])?;

        // Parse halfmove clock
        position.halfmove_clock = Self::parse_halfmove(parts[4])?;

        // Parse fullmove number
        position.fullmove_number = Self::parse_fullmove(parts[5])?;

        Ok(position)
    }

    fn parse_piece_placement(position: &mut Position, placement: &str) -> Result<(), FenError> {
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidRankCount);
        }

        for (rank_index, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_index as u8; // FEN starts from rank 8
            let mut file = 0u8;

            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    let empty_squares = ch.to_digit(10).ok_or(FenError::InvalidPiece(ch))? as u8;
                    file += empty_squares;
                } else {
                    let piece = Self::char_to_piece(ch)?;
                    let square = Square::new(rank, file).map_err(|_| FenError::InvalidFileCount)?;
                    position.set_piece(square, Some(piece));
                    file += 1;
                }

                if file > 8 {
                    return Err(FenError::InvalidFileCount);
                }
            }

            if file != 8 {
                return Err(FenError::InvalidFileCount);
            }
        }

        Ok(())
    }

    fn char_to_piece(ch: char) -> Result<Piece, FenError> {
        let color = if ch.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let piece_type = match ch.to_ascii_lowercase() {
            'p' => PieceType::Pawn,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'r' => PieceType::Rook,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => return Err(FenError::InvalidPiece(ch)),
        };

        Ok(Piece::new(color, piece_type))
    }

    fn parse_active_color(color_str: &str) -> Result<Color, FenError> {
        match color_str {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => {
                let invalid_char = color_str.chars().next().unwrap_or(' ');
                Err(FenError::InvalidColor(invalid_char))
            }
        }
    }

    fn parse_castling(castling_str: &str) -> Result<CastlingRights, FenError> {
        if castling_str == "-" {
            return Ok(CastlingRights::none());
        }

        let mut rights = CastlingRights::none();

        for ch in castling_str.chars() {
            match ch {
                'K' => rights.white_kingside = true,
                'Q' => rights.white_queenside = true,
                'k' => rights.black_kingside = true,
                'q' => rights.black_queenside = true,
                _ => return Err(FenError::InvalidCastling(castling_str.to_string())),
            }
        }

        Ok(rights)
    }

    fn parse_en_passant(ep_str: &str) -> Result<Option<Square>, FenError> {
        if ep_str == "-" {
            return Ok(None);
        }

        Square::from_algebraic(ep_str)
            .map(Some)
            .map_err(|_| FenError::InvalidEnPassant(ep_str.to_string()))
    }

    fn parse_halfmove(halfmove_str: &str) -> Result<u8, FenError> {
        halfmove_str
            .parse::<u8>()
            .map_err(|_| FenError::InvalidHalfmove(halfmove_str.to_string()))
    }

    fn parse_fullmove(fullmove_str: &str) -> Result<u16, FenError> {
        let fullmove = fullmove_str
            .parse::<u16>()
            .map_err(|_| FenError::InvalidFullmove(fullmove_str.to_string()))?;

        if fullmove == 0 {
            return Err(FenError::InvalidFullmove(fullmove_str.to_string()));
        }

        Ok(fullmove)
    }

    /// Generate FEN string from current position
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        fen.push_str(&self.piece_placement_to_fen());
        fen.push(' ');

        // Active color
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });
        fen.push(' ');

        // Castling availability
        fen.push_str(&self.castling_to_fen());
        fen.push(' ');

        // En passant target square
        fen.push_str(&self.en_passant_to_fen());
        fen.push(' ');

        // Halfmove clock
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');

        // Fullmove number
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    fn piece_placement_to_fen(&self) -> String {
        let mut placement = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;

            for file in 0..8 {
                // Since rank and file are guaranteed to be 0-7, create square directly
                let square = Square(rank * 8 + file);

                if let Some(piece) = self.piece_at(square) {
                    if empty_count > 0 {
                        placement.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    placement.push(Self::piece_to_char(piece));
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                placement.push_str(&empty_count.to_string());
            }

            if rank > 0 {
                placement.push('/');
            }
        }

        placement
    }

    fn piece_to_char(piece: Piece) -> char {
        let ch = match piece.piece_type {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };

        match piece.color {
            Color::White => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }

    fn castling_to_fen(&self) -> String {
        let mut castling = String::new();

        if self.castling_rights.white_kingside {
            castling.push('K');
        }
        if self.castling_rights.white_queenside {
            castling.push('Q');
        }
        if self.castling_rights.black_kingside {
            castling.push('k');
        }
        if self.castling_rights.black_queenside {
            castling.push('q');
        }

        if castling.is_empty() {
            castling.push('-');
        }

        castling
    }

    fn en_passant_to_fen(&self) -> String {
        match self.en_passant {
            Some(square) => square.to_algebraic(),
            None => "-".to_string(),
        }
    }
}

// Standard starting position FEN
pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_fen() {
        let position = Position::starting_position().expect("Starting position should be valid");
        let fen = position.to_fen();
        assert_eq!(fen, STARTING_FEN);
    }

    #[test]
    fn test_parse_starting_fen() {
        let position = Position::from_fen(STARTING_FEN).expect("Starting FEN should be valid");
        let expected = Position::starting_position().expect("Starting position should be valid");
        assert_eq!(position, expected);
    }

    #[test]
    fn test_roundtrip_fen() {
        let original_fen = STARTING_FEN;
        let position = Position::from_fen(original_fen).expect("Starting FEN should be valid");
        let generated_fen = position.to_fen();
        assert_eq!(original_fen, generated_fen);
    }
}
