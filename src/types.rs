use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    #[must_use]
    pub const fn material_value(self) -> i32 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            PieceType::Pawn => "P",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

impl Piece {
    #[must_use]
    pub const fn new(color: Color, piece_type: PieceType) -> Self {
        Self { color, piece_type }
    }

    pub fn material_value(self) -> i32 {
        self.piece_type.material_value()
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.color {
            Color::White => self.piece_type.to_string(),
            Color::Black => self.piece_type.to_string().to_lowercase(),
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub const fn new(rank: u8, file: u8) -> Result<Self, &'static str> {
        if rank > 7 || file > 7 {
            return Err("Invalid rank or file");
        }
        Ok(Square(rank * 8 + file))
    }

    pub const fn from_index(index: u8) -> Result<Self, &'static str> {
        if index > 63 {
            return Err("Invalid square index");
        }
        Ok(Square(index))
    }

    pub fn from_algebraic(notation: &str) -> Result<Self, &'static str> {
        if notation.len() != 2 {
            return Err("Invalid algebraic notation");
        }

        let mut chars = notation.chars();
        let file_char = chars.next().ok_or("Missing file")?;
        let rank_char = chars.next().ok_or("Missing rank")?;

        let file = match file_char {
            'a'..='h' => file_char as u8 - b'a',
            _ => return Err("Invalid file"),
        };

        let rank = match rank_char {
            '1'..='8' => rank_char as u8 - b'1',
            _ => return Err("Invalid rank"),
        };

        Ok(Square(rank * 8 + file))
    }

    #[must_use]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    #[must_use]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    pub fn to_algebraic(self) -> String {
        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        format!("{}{}", file, rank)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    #[must_use]
    pub const fn none() -> Self {
        Self {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    #[must_use]
    pub const fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_kingside,
            Color::Black => self.black_kingside,
        }
    }

    #[must_use]
    pub const fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_queenside,
            Color::Black => self.black_queenside,
        }
    }

    pub const fn remove_kingside(&mut self, color: Color) {
        match color {
            Color::White => self.white_kingside = false,
            Color::Black => self.black_kingside = false,
        }
    }

    pub const fn remove_queenside(&mut self, color: Color) {
        match color {
            Color::White => self.white_queenside = false,
            Color::Black => self.black_queenside = false,
        }
    }

    pub const fn remove_all(&mut self, color: Color) {
        match color {
            Color::White => {
                self.white_kingside = false;
                self.white_queenside = false;
            }
            Color::Black => {
                self.black_kingside = false;
                self.black_queenside = false;
            }
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveGenError {
    InvalidSquare(&'static str),
    InvalidMove(String),
}

impl fmt::Display for MoveGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveGenError::InvalidSquare(msg) => write!(f, "Invalid square: {}", msg),
            MoveGenError::InvalidMove(msg) => write!(f, "Invalid move: {}", msg),
        }
    }
}

impl std::error::Error for MoveGenError {}
