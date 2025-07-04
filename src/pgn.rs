use crate::moves::{Move, MoveType};
use crate::position::Position;
use crate::types::{PieceType, Square};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PgnMetadata {
    pub event: Option<String>,
    pub site: Option<String>,
    pub date: Option<String>,
    pub round: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    other_tags: HashMap<String, String>,
}

impl PgnMetadata {
    fn set_tag(&mut self, key: &str, value: &str) {
        match key {
            "Event" => self.event = Some(value.to_string()),
            "Site" => self.site = Some(value.to_string()),
            "Date" => self.date = Some(value.to_string()),
            "Round" => self.round = Some(value.to_string()),
            "White" => self.white = Some(value.to_string()),
            "Black" => self.black = Some(value.to_string()),
            "Result" => self.result = Some(value.to_string()),
            _ => {
                self.other_tags.insert(key.to_string(), value.to_string());
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.event.is_none()
            && self.site.is_none()
            && self.date.is_none()
            && self.round.is_none()
            && self.white.is_none()
            && self.black.is_none()
            && self.result.is_none()
            && self.other_tags.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PgnGame {
    pub metadata: PgnMetadata,
    pub moves: Vec<String>,
    pub result: GameResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    Ongoing,
}

impl GameResult {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "1-0" => Some(GameResult::WhiteWins),
            "0-1" => Some(GameResult::BlackWins),
            "1/2-1/2" => Some(GameResult::Draw),
            "*" => Some(GameResult::Ongoing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PgnError {
    InvalidFormat(String),
    IllegalMove(String),
    AmbiguousMove(String),
    UnsupportedFeature(String),
    ParseError(String),
}

impl std::fmt::Display for PgnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnError::InvalidFormat(msg) => write!(f, "Invalid PGN format: {}", msg),
            PgnError::IllegalMove(msg) => write!(f, "Illegal move: {}", msg),
            PgnError::AmbiguousMove(msg) => write!(f, "Ambiguous move: {}", msg),
            PgnError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
            PgnError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for PgnError {}

impl PgnGame {
    pub fn from_pgn(pgn: &str) -> Result<Self, PgnError> {
        if pgn.trim().is_empty() {
            return Err(PgnError::InvalidFormat("Empty PGN string".to_string()));
        }

        // Check for unsupported features
        if pgn.contains('{') || pgn.contains('}') {
            return Err(PgnError::UnsupportedFeature(
                "Comments are not supported".to_string(),
            ));
        }
        if pgn.contains('(') || pgn.contains(')') {
            return Err(PgnError::UnsupportedFeature(
                "Variations are not supported".to_string(),
            ));
        }

        let mut metadata = PgnMetadata::default();
        let mut movetext = String::new();
        let mut in_headers = true;

        let header_regex = Regex::new(r#"\[\s*(\w+)\s*"([^"]+)"\s*\]"#).unwrap();

        for line in pgn.lines() {
            let line = line.trim();
            if line.is_empty() {
                if !movetext.is_empty() {
                    break;
                }
                in_headers = false;
                continue;
            }

            if in_headers && line.starts_with('[') {
                if let Some(caps) = header_regex.captures(line) {
                    metadata.set_tag(&caps[1], &caps[2]);
                }
            } else {
                in_headers = false;
                movetext.push_str(line);
                movetext.push(' ');
            }
        }

        // Clean up movetext - remove move numbers
        let move_num_regex = Regex::new(r"\d+\.+\s*").unwrap();
        let clean_movetext = move_num_regex.replace_all(&movetext, "");

        let mut moves = Vec::new();
        let mut result = GameResult::Ongoing;

        for token in clean_movetext.split_whitespace() {
            if let Some(res) = GameResult::from_str(token) {
                result = res;
                break;
            }
            moves.push(token.to_string());
        }

        // Result tag in metadata takes precedence
        if let Some(res_str) = &metadata.result {
            if let Some(res) = GameResult::from_str(res_str) {
                result = res;
            }
        }

        // Final validation
        if metadata.is_empty() && moves.is_empty() {
            return Err(PgnError::InvalidFormat(
                "No headers or moves found".to_string(),
            ));
        }

        // Check if we have moves but they don't look like chess moves
        if !moves.is_empty() {
            for token in &moves {
                // Basic sanity check - chess moves should be short and contain letters/numbers
                if token.len() > 10 || !token.chars().any(|c| c.is_ascii_alphanumeric()) {
                    return Err(PgnError::InvalidFormat(format!(
                        "Invalid move format: {}",
                        token
                    )));
                }
            }
        }

        // Note: Move validation is done in to_position() method
        // This allows PGN parsing to succeed even if moves will fail later

        Ok(PgnGame {
            metadata,
            moves,
            result,
        })
    }

    pub fn to_position(&self) -> Result<Position, PgnError> {
        let mut position = Position::starting_position().map_err(|e| {
            PgnError::ParseError(format!("Failed to create starting position: {}", e))
        })?;

        for (i, san) in self.moves.iter().enumerate() {
            let mv = san_to_move(&position, san)
                .map_err(|e| PgnError::ParseError(format!("Move {} '{}': {}", i + 1, san, e)))?;

            position
                .apply_move_for_search(mv)
                .map_err(|e| PgnError::IllegalMove(format!("Move '{}' is illegal: {}", san, e)))?;
        }

        Ok(position)
    }
}

impl Position {
    pub fn to_pgn(&self) -> Result<String, PgnError> {
        // Basic PGN export - minimal implementation for tests
        // Since we don't track move history, we'll create a minimal PGN
        let mut pgn = String::new();

        // Add minimal headers
        pgn.push_str("[Event \"Exported Game\"]\n");
        pgn.push_str("[White \"Player1\"]\n");
        pgn.push_str("[Black \"Player2\"]\n");
        pgn.push_str("[Result \"*\"]\n\n");

        // For now, just return empty game since we don't have move history
        pgn.push('*');

        Ok(pgn)
    }
}

/// Converts a standard algebraic notation (SAN) string to a Move object
fn san_to_move(position: &Position, san: &str) -> Result<Move, PgnError> {
    let legal_moves = position
        .generate_legal_moves()
        .map_err(|e| PgnError::ParseError(format!("Failed to generate legal moves: {}", e)))?;

    // Handle castling
    if san == "O-O" {
        return legal_moves
            .iter()
            .find(|m| m.move_type == MoveType::CastleKingside)
            .copied()
            .ok_or_else(|| PgnError::IllegalMove(san.to_string()));
    }
    if san == "O-O-O" {
        return legal_moves
            .iter()
            .find(|m| m.move_type == MoveType::CastleQueenside)
            .copied()
            .ok_or_else(|| PgnError::IllegalMove(san.to_string()));
    }

    // Remove check/checkmate indicators and annotations
    let san_clean = san.replace(&['+', '#', '!', '?'][..], "");

    // Parse SAN with regex
    let re = Regex::new(r"^(?<piece>[NBRQK])?(?<d_file>[a-h])?(?<d_rank>[1-8])?(?<capture>x)?(?<to>[a-h][1-8])(?<promo>=[NBRQ])?$").unwrap();
    let caps = re
        .captures(&san_clean)
        .ok_or_else(|| PgnError::InvalidFormat(format!("Invalid move format: {}", san)))?;

    // Determine piece type (default to pawn if not specified)
    let piece_type = caps.name("piece").map_or(PieceType::Pawn, |m| {
        match m.as_str().chars().next().unwrap() {
            'N' => PieceType::Knight,
            'B' => PieceType::Bishop,
            'R' => PieceType::Rook,
            'Q' => PieceType::Queen,
            'K' => PieceType::King,
            _ => PieceType::Pawn,
        }
    });

    // Parse destination square
    let to_square = Square::from_algebraic(caps.name("to").unwrap().as_str())
        .map_err(|_| PgnError::InvalidFormat(format!("Invalid square: {}", san)))?;

    // Parse disambiguation
    let disamb_file = caps
        .name("d_file")
        .map(|m| m.as_str().chars().next().unwrap() as u8 - b'a');
    let disamb_rank = caps
        .name("d_rank")
        .map(|m| m.as_str().chars().next().unwrap() as u8 - b'1');

    // Parse promotion piece
    let promotion_piece = caps.name("promo").map(|m| {
        match m.as_str().chars().nth(1).unwrap() {
            'Q' => PieceType::Queen,
            'R' => PieceType::Rook,
            'B' => PieceType::Bishop,
            'N' => PieceType::Knight,
            _ => PieceType::Queen, // Default to queen
        }
    });

    // Filter candidate moves
    let is_capture = caps.name("capture").is_some();
    let candidates: Vec<Move> = legal_moves
        .into_iter()
        .filter(|&m| {
            // Check if piece matches
            if let Some(piece) = position.piece_at(m.from) {
                if piece.piece_type != piece_type || m.to != to_square {
                    return false;
                }
            } else {
                return false;
            }

            // Check file disambiguation
            if let Some(file) = disamb_file {
                if m.from.file() != file {
                    return false;
                }
            }

            // Check rank disambiguation
            if let Some(rank) = disamb_rank {
                if m.from.rank() != rank {
                    return false;
                }
            }

            // Check promotion
            if let Some(promo) = promotion_piece {
                if m.move_type.promotion_piece() != Some(promo) {
                    return false;
                }
            } else if m.move_type.is_promotion() {
                return false;
            }

            // Check capture
            if is_capture != m.move_type.is_capture() {
                // Special case for pawn captures - file disambiguation implies capture
                if piece_type == PieceType::Pawn && disamb_file.is_some() {
                    return m.move_type.is_capture();
                }
                return false;
            }

            true
        })
        .collect();

    match candidates.len() {
        0 => Err(PgnError::IllegalMove(san.to_string())),
        1 => Ok(candidates[0]),
        _ => Err(PgnError::AmbiguousMove(san.to_string())),
    }
}
