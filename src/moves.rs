use crate::types::{PieceType, Square};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveType {
    Quiet,
    Capture,
    EnPassant,
    CastleKingside,
    CastleQueenside,
    PromotionQueen,
    PromotionRook,
    PromotionBishop,
    PromotionKnight,
    PromotionCaptureQueen,
    PromotionCaptureRook,
    PromotionCaptureBishop,
    PromotionCaptureKnight,
}

impl MoveType {
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        matches!(
            self,
            Self::PromotionQueen
                | Self::PromotionRook
                | Self::PromotionBishop
                | Self::PromotionKnight
                | Self::PromotionCaptureQueen
                | Self::PromotionCaptureRook
                | Self::PromotionCaptureBishop
                | Self::PromotionCaptureKnight
        )
    }

    #[must_use]
    pub const fn is_capture(self) -> bool {
        matches!(
            self,
            Self::Capture
                | Self::EnPassant
                | Self::PromotionCaptureQueen
                | Self::PromotionCaptureRook
                | Self::PromotionCaptureBishop
                | Self::PromotionCaptureKnight
        )
    }

    #[must_use]
    pub const fn is_castle(self) -> bool {
        matches!(self, Self::CastleKingside | Self::CastleQueenside)
    }

    #[must_use]
    pub const fn promotion_piece(self) -> Option<PieceType> {
        match self {
            Self::PromotionQueen | Self::PromotionCaptureQueen => Some(PieceType::Queen),
            Self::PromotionRook | Self::PromotionCaptureRook => Some(PieceType::Rook),
            Self::PromotionBishop | Self::PromotionCaptureBishop => Some(PieceType::Bishop),
            Self::PromotionKnight | Self::PromotionCaptureKnight => Some(PieceType::Knight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub move_type: MoveType,
}

impl Move {
    #[must_use]
    pub const fn new(from: Square, to: Square, move_type: MoveType) -> Self {
        Self {
            from,
            to,
            move_type,
        }
    }

    #[must_use]
    pub fn quiet(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveType::Quiet)
    }

    #[must_use]
    pub fn capture(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveType::Capture)
    }

    #[must_use]
    pub fn en_passant(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveType::EnPassant)
    }

    #[must_use]
    pub fn castle_kingside(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveType::CastleKingside)
    }

    #[must_use]
    pub fn castle_queenside(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveType::CastleQueenside)
    }

    #[must_use]
    pub fn promotion(from: Square, to: Square, piece_type: PieceType, is_capture: bool) -> Self {
        let move_type = match (piece_type, is_capture) {
            (PieceType::Queen, false) => MoveType::PromotionQueen,
            (PieceType::Queen, true) => MoveType::PromotionCaptureQueen,
            (PieceType::Rook, false) => MoveType::PromotionRook,
            (PieceType::Rook, true) => MoveType::PromotionCaptureRook,
            (PieceType::Bishop, false) => MoveType::PromotionBishop,
            (PieceType::Bishop, true) => MoveType::PromotionCaptureBishop,
            (PieceType::Knight, false) => MoveType::PromotionKnight,
            (PieceType::Knight, true) => MoveType::PromotionCaptureKnight,
            _ => MoveType::Quiet, // Invalid promotion, fallback
        };
        Self::new(from, to, move_type)
    }

    #[must_use]
    pub fn to_algebraic(&self) -> String {
        let from_str = self.from.to_algebraic();
        let to_str = self.to.to_algebraic();

        let mut result = format!("{from_str}{to_str}");

        if let Some(piece) = self.move_type.promotion_piece() {
            let piece_char = match piece {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => 'q', // Fallback
            };
            result.push(piece_char);
        }

        result
    }

    pub fn from_algebraic(notation: &str) -> Result<Self, &'static str> {
        if notation.len() < 4 {
            return Err("Move notation too short");
        }

        let from_str = &notation[0..2];
        let to_str = &notation[2..4];

        let from = Square::from_algebraic(from_str)?;
        let to = Square::from_algebraic(to_str)?;

        let move_type = if notation.len() == 5 {
            let promotion_char = notation
                .chars()
                .nth(4)
                .ok_or("Invalid promotion notation")?;
            let piece_type = match promotion_char {
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'b' => PieceType::Bishop,
                'n' => PieceType::Knight,
                _ => return Err("Invalid promotion piece"),
            };

            // We can't determine if it's a capture without board context
            // Default to non-capture promotion
            match piece_type {
                PieceType::Queen => MoveType::PromotionQueen,
                PieceType::Rook => MoveType::PromotionRook,
                PieceType::Bishop => MoveType::PromotionBishop,
                PieceType::Knight => MoveType::PromotionKnight,
                _ => MoveType::Quiet,
            }
        } else {
            MoveType::Quiet // Default, will be refined when applied to board
        };

        Ok(Self::new(from, to, move_type))
    }

    /// Check if move has valid squares (minimal implementation)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // Basic validation: squares should be different and valid
        self.from != self.to && self.from.index() < 64 && self.to.index() < 64
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}
