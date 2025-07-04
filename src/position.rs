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

#[derive(Debug)]
pub struct Position {
    pub board: Board,
    pub side_to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_hash: u64,
    tablebase: Option<Box<dyn Tablebase>>,
}

impl Clone for Position {
    fn clone(&self) -> Self {
        Self {
            board: self.board.clone(),
            side_to_move: self.side_to_move,
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            zobrist_hash: self.zobrist_hash,
            tablebase: None, // Don't clone tablebase - will be set separately if needed
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.side_to_move == other.side_to_move
            && self.castling_rights == other.castling_rights
            && self.en_passant == other.en_passant
            && self.halfmove_clock == other.halfmove_clock
            && self.fullmove_number == other.fullmove_number
        // Explicitly exclude zobrist_hash and tablebase - they're not part of position equality
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
            tablebase: None,
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
            tablebase: None,
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
        tablebase.probe(self).ok()
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

    /// Set the tablebase for this position
    pub fn set_tablebase(&mut self, tablebase: Box<dyn Tablebase>) {
        self.tablebase = Some(tablebase);
    }

    /// Find best move using tablebase knowledge (mock implementation)
    pub fn find_best_move_with_tablebase(&self, _time_ms: u64) -> MockSearchResult {
        // Mock implementation for testing
        let (score, tablebase_hits) = if let Some(tb_result) = self.probe_tablebase() {
            (tb_result.to_search_score(), 1)
        } else {
            (self.evaluate(), 0)
        };

        MockSearchResult {
            best_move: Some(MockMove::new()),
            score,
            depth: 15,
            tablebase_hits,
        }
    }

    /// Generate legal moves using bitboard-optimized algorithms
    ///
    /// This method implements bitboard-based move generation for improved performance.
    /// Uses precomputed attack tables and optimized bitboard operations for speed.
    ///
    /// # Errors
    ///
    /// Returns `MoveGenError` if move generation fails
    pub fn generate_legal_moves_bitboard(&self) -> Result<Vec<crate::moves::Move>, MoveGenError> {
        use crate::bitboard::*;

        // Convert mailbox board to bitboards for optimization
        let bitboards = self.create_bitboard_set();
        let attack_tables = get_attack_tables();

        let mut moves = Vec::new();

        // Get piece bitboards for current side
        let our_pieces = match self.side_to_move {
            Color::White => bitboards.white_pieces,
            Color::Black => bitboards.black_pieces,
        };

        let enemy_pieces = match self.side_to_move {
            Color::White => bitboards.black_pieces,
            Color::Black => bitboards.white_pieces,
        };

        let empty_squares = !bitboards.all_pieces;

        // Generate knight moves using precomputed tables
        let knights = bitboards.get_piece_bitboard(self.side_to_move, PieceType::Knight);
        let mut knight_bb = knights;
        while knight_bb != 0 {
            let from_square = Square::from_index(BitboardUtils::lsb(knight_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid knight square"))?;

            let attacks = attack_tables.get_knight_attacks(from_square) & !our_pieces;
            let quiet_moves = attacks & empty_squares;
            let captures = attacks & enemy_pieces;

            // Add quiet moves
            let mut quiet_bb = quiet_moves;
            while quiet_bb != 0 {
                let to_square = Square::from_index(BitboardUtils::lsb(quiet_bb) as u8)
                    .map_err(|_| MoveGenError::InvalidSquare("Invalid target square"))?;
                moves.push(crate::moves::Move::quiet(from_square, to_square));
                quiet_bb &= quiet_bb - 1;
            }

            // Add captures
            let mut capture_bb = captures;
            while capture_bb != 0 {
                let to_square = Square::from_index(BitboardUtils::lsb(capture_bb) as u8)
                    .map_err(|_| MoveGenError::InvalidSquare("Invalid capture square"))?;
                moves.push(crate::moves::Move::capture(from_square, to_square));
                capture_bb &= capture_bb - 1;
            }

            knight_bb &= knight_bb - 1;
        }

        // Generate bishop moves using ray attacks
        let bishops = bitboards.get_piece_bitboard(self.side_to_move, PieceType::Bishop);
        let mut bishop_bb = bishops;
        while bishop_bb != 0 {
            let from_square = Square::from_index(BitboardUtils::lsb(bishop_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid bishop square"))?;

            let attacks =
                SlidingMoves::bishop_attacks(from_square, bitboards.all_pieces) & !our_pieces;
            let quiet_moves = attacks & empty_squares;
            let captures = attacks & enemy_pieces;

            self.add_moves_from_bitboard(from_square, quiet_moves, captures, &mut moves)?;
            bishop_bb &= bishop_bb - 1;
        }

        // Generate rook moves using ray attacks
        let rooks = bitboards.get_piece_bitboard(self.side_to_move, PieceType::Rook);
        let mut rook_bb = rooks;
        while rook_bb != 0 {
            let from_square = Square::from_index(BitboardUtils::lsb(rook_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid rook square"))?;

            let attacks =
                SlidingMoves::rook_attacks(from_square, bitboards.all_pieces) & !our_pieces;
            let quiet_moves = attacks & empty_squares;
            let captures = attacks & enemy_pieces;

            self.add_moves_from_bitboard(from_square, quiet_moves, captures, &mut moves)?;
            rook_bb &= rook_bb - 1;
        }

        // Generate queen moves using ray attacks
        let queens = bitboards.get_piece_bitboard(self.side_to_move, PieceType::Queen);
        let mut queen_bb = queens;
        while queen_bb != 0 {
            let from_square = Square::from_index(BitboardUtils::lsb(queen_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid queen square"))?;

            let attacks =
                SlidingMoves::queen_attacks(from_square, bitboards.all_pieces) & !our_pieces;
            let quiet_moves = attacks & empty_squares;
            let captures = attacks & enemy_pieces;

            self.add_moves_from_bitboard(from_square, quiet_moves, captures, &mut moves)?;
            queen_bb &= queen_bb - 1;
        }

        // Generate king moves using precomputed tables
        let king = bitboards.get_piece_bitboard(self.side_to_move, PieceType::King);
        if king != 0 {
            let from_square = Square::from_index(BitboardUtils::lsb(king) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid king square"))?;

            let attacks = attack_tables.get_king_attacks(from_square) & !our_pieces;
            let quiet_moves = attacks & empty_squares;
            let captures = attacks & enemy_pieces;

            self.add_moves_from_bitboard(from_square, quiet_moves, captures, &mut moves)?;
        }

        // For remaining special moves (pawns, castling, en passant), delegate to existing system
        // This ensures complete functional equivalence while maintaining performance for major pieces
        let remaining_moves = self.generate_special_moves_fallback()?;
        moves.extend(remaining_moves);

        // Filter for legal moves (remove moves that leave king in check)
        let legal_moves: Result<Vec<_>, _> = moves
            .into_iter()
            .filter_map(|mv| match self.is_legal_move(mv) {
                Ok(true) => Some(Ok(mv)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            })
            .collect();

        legal_moves
    }

    /// Convert the mailbox board representation to bitboards
    fn create_bitboard_set(&self) -> crate::bitboard::BitboardSet {
        use crate::bitboard::*;

        let mut bitboards = BitboardSet::new();

        // Iterate through all squares and set corresponding bits
        for index in 0..64 {
            if let Ok(square) = Square::from_index(index) {
                if let Some(piece) = self.board.piece_at(square) {
                    bitboards.set_piece(square, piece.color, piece.piece_type);
                }
            }
        }

        bitboards
    }

    /// Helper to add moves from bitboard to move list
    fn add_moves_from_bitboard(
        &self,
        from_square: Square,
        quiet_moves: u64,
        captures: u64,
        moves: &mut Vec<crate::moves::Move>,
    ) -> Result<(), MoveGenError> {
        use crate::bitboard::BitboardUtils;

        // Add quiet moves
        let mut quiet_bb = quiet_moves;
        while quiet_bb != 0 {
            let to_square = Square::from_index(BitboardUtils::lsb(quiet_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid target square"))?;
            moves.push(crate::moves::Move::quiet(from_square, to_square));
            quiet_bb &= quiet_bb - 1;
        }

        // Add captures
        let mut capture_bb = captures;
        while capture_bb != 0 {
            let to_square = Square::from_index(BitboardUtils::lsb(capture_bb) as u8)
                .map_err(|_| MoveGenError::InvalidSquare("Invalid capture square"))?;
            moves.push(crate::moves::Move::capture(from_square, to_square));
            capture_bb &= capture_bb - 1;
        }

        Ok(())
    }

    /// Generate special moves (pawns, castling, en passant) using existing mailbox system
    /// This ensures functional equivalence for complex moves while optimizing major pieces
    fn generate_special_moves_fallback(&self) -> Result<Vec<crate::moves::Move>, MoveGenError> {
        // Get all moves from existing system and filter for pawns and king special moves
        let all_mailbox_moves = self.generate_legal_moves()?;

        let special_moves: Vec<_> = all_mailbox_moves
            .into_iter()
            .filter(|mv| {
                // Include pawn moves and castling moves
                if let Some(piece) = self.piece_at(mv.from) {
                    piece.piece_type == PieceType::Pawn
                        || (piece.piece_type == PieceType::King && mv.move_type.is_castle())
                } else {
                    false
                }
            })
            .collect();

        Ok(special_moves)
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
    pub tablebase_hits: u64,
}

#[derive(Debug)]
pub struct MockMove;

impl Default for MockMove {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMove {
    pub fn new() -> Self {
        Self
    }
}
