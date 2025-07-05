use crate::position::Position;
use crate::types::Color;
use std::collections::HashMap;

/// Maximum number of pieces for tablebase lookup
pub const MAX_TABLEBASE_PIECES: usize = 6;

/// Canonical key for tablebase position lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TablebaseKey {
    material_sig: String,
    side_to_move: Color,
    position_hash: u64,
}

impl TablebaseKey {
    /// Create a tablebase key from a chess position
    pub fn from_position(position: &Position) -> Result<Self, TablebaseError> {
        let material_sig = Self::generate_material_signature(position);
        let position_hash = Self::generate_position_hash(position);

        Ok(Self {
            material_sig,
            side_to_move: position.side_to_move,
            position_hash,
        })
    }

    /// Get the material signature (e.g. "KQvK")
    pub fn material_signature(&self) -> &str {
        &self.material_sig
    }

    /// Get the side to move
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    fn generate_material_signature(position: &Position) -> String {
        let mut white_pieces = Vec::new();
        let mut black_pieces = Vec::new();

        // Count pieces for each side
        for (square, piece) in position.board.pieces() {
            let piece_char = match piece.piece_type {
                crate::types::PieceType::King => 'K',
                crate::types::PieceType::Queen => 'Q',
                crate::types::PieceType::Rook => 'R',
                crate::types::PieceType::Bishop => 'B',
                crate::types::PieceType::Knight => 'N',
                crate::types::PieceType::Pawn => 'P',
            };

            match piece.color {
                Color::White => white_pieces.push(piece_char),
                Color::Black => black_pieces.push(piece_char),
            }
        }

        // Sort for canonical representation
        white_pieces.sort();
        black_pieces.sort();

        format!(
            "{}v{}",
            white_pieces.iter().collect::<String>(),
            black_pieces.iter().collect::<String>()
        )
    }

    fn generate_position_hash(position: &Position) -> u64 {
        // Simplified position hash - just use piece placement
        let mut hash = 0u64;

        for (square, piece) in position.board.pieces() {
            let piece_value = match piece.piece_type {
                crate::types::PieceType::Pawn => 1,
                crate::types::PieceType::Knight => 2,
                crate::types::PieceType::Bishop => 3,
                crate::types::PieceType::Rook => 4,
                crate::types::PieceType::Queen => 5,
                crate::types::PieceType::King => 6,
            };

            let color_multiplier = match piece.color {
                Color::White => 1,
                Color::Black => 10,
            };

            hash = hash.wrapping_add((square.index() as u64) * piece_value * color_multiplier);
        }

        hash
    }
}

/// Result of a tablebase lookup
#[derive(Debug, Clone, PartialEq)]
pub enum TablebaseResult {
    /// Position is winning in the given number of moves
    Win(u8),
    /// Position is losing in the given number of moves  
    Loss(u8),
    /// Position is drawn
    Draw,
}

impl TablebaseResult {
    /// Convert tablebase result to search score (centipawns)
    pub fn to_search_score(&self) -> i32 {
        match self {
            Self::Win(dtm) => {
                // Winning score, adjusted for distance to mate
                // Use scores > 10000 to indicate tablebase wins
                20000 - (*dtm as i32 * 10)
            }
            Self::Loss(dtm) => {
                // Losing score, adjusted for distance to mate
                -20000 + (*dtm as i32 * 10)
            }
            Self::Draw => 0,
        }
    }
}

/// Trait for tablebase implementations
pub trait Tablebase {
    /// Probe the tablebase for a position result
    fn probe(&self, position: &Position) -> Result<TablebaseResult, TablebaseError>;

    /// Check if tablebase data is available for this material configuration
    fn is_available(&self, material_signature: &str) -> bool;
}

/// Errors that can occur during tablebase operations
#[derive(Debug, PartialEq)]
pub enum TablebaseError {
    /// Position not found in tablebase
    NotFound,
    /// Invalid position for tablebase lookup
    InvalidPosition,
    /// File I/O error
    FileError,
}

/// Simple in-memory tablebase for development and testing
#[derive(Debug)]
pub struct MockTablebase {
    data: HashMap<String, TablebaseResult>,
}

impl MockTablebase {
    pub fn new() -> Self {
        let mut data = HashMap::new();

        // Add some basic known results
        data.insert("KQvK".to_string(), TablebaseResult::Win(10));
        data.insert("KRvK".to_string(), TablebaseResult::Draw);
        data.insert("KPvK".to_string(), TablebaseResult::Win(15));

        Self { data }
    }
}

impl Tablebase for MockTablebase {
    fn probe(&self, position: &Position) -> Result<TablebaseResult, TablebaseError> {
        let key = TablebaseKey::from_position(position)?;
        let material_sig = key.material_signature();

        if let Some(result) = self.data.get(material_sig) {
            // Determine who has the advantage in this material configuration
            let white_stronger = match material_sig {
                "KQvK" => position.has_piece(Color::White, crate::types::PieceType::Queen),
                "KRvK" => false, // Draw
                "KPvK" => position.has_piece(Color::White, crate::types::PieceType::Pawn),
                _ => false,
            };

            // Adjust result based on side to move and who has the advantage
            let adjusted_result = match result {
                TablebaseResult::Win(dtm) => {
                    // If the side to move has the advantage, it's a win; otherwise it's a loss
                    let is_winning = (position.side_to_move == Color::White && white_stronger)
                        || (position.side_to_move == Color::Black && !white_stronger);

                    if is_winning {
                        TablebaseResult::Win(*dtm)
                    } else {
                        TablebaseResult::Loss(*dtm)
                    }
                }
                other => other.clone(),
            };

            Ok(adjusted_result)
        } else {
            Err(TablebaseError::NotFound)
        }
    }

    fn is_available(&self, material_signature: &str) -> bool {
        self.data.contains_key(material_signature)
    }
}
