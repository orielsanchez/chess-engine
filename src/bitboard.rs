/// Bitboard representation for optimized move generation
///
/// A bitboard is a 64-bit integer where each bit represents a square on the chess board.
/// This allows for extremely fast operations using bitwise operators.
///
/// Square mapping (little-endian rank-file mapping):
/// a1=0, b1=1, c1=2, ..., h1=7
/// a2=8, b2=9, c2=10, ..., h2=15
/// ...
/// a8=56, b8=57, c8=58, ..., h8=63
use crate::types::{Color, PieceType, Square};

/// A bitboard represented as a 64-bit integer
pub type Bitboard = u64;

/// Collection of bitboards representing the complete board state
#[derive(Debug, Clone)]
pub struct BitboardSet {
    /// Bitboard for each piece type and color
    pub white_pawns: Bitboard,
    pub white_knights: Bitboard,
    pub white_bishops: Bitboard,
    pub white_rooks: Bitboard,
    pub white_queens: Bitboard,
    pub white_king: Bitboard,

    pub black_pawns: Bitboard,
    pub black_knights: Bitboard,
    pub black_bishops: Bitboard,
    pub black_rooks: Bitboard,
    pub black_queens: Bitboard,
    pub black_king: Bitboard,

    /// Combined bitboards for optimization
    pub white_pieces: Bitboard,
    pub black_pieces: Bitboard,
    pub all_pieces: Bitboard,
}

impl BitboardSet {
    /// Create empty bitboard set
    pub fn new() -> Self {
        Self {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,

            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,

            white_pieces: 0,
            black_pieces: 0,
            all_pieces: 0,
        }
    }

    /// Update combined bitboards after modifying individual piece bitboards
    pub fn update_combined(&mut self) {
        self.white_pieces = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king;

        self.black_pieces = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king;

        self.all_pieces = self.white_pieces | self.black_pieces;
    }

    /// Get piece bitboard by color and type
    pub fn get_piece_bitboard(&self, color: Color, piece_type: PieceType) -> Bitboard {
        match (color, piece_type) {
            (Color::White, PieceType::Pawn) => self.white_pawns,
            (Color::White, PieceType::Knight) => self.white_knights,
            (Color::White, PieceType::Bishop) => self.white_bishops,
            (Color::White, PieceType::Rook) => self.white_rooks,
            (Color::White, PieceType::Queen) => self.white_queens,
            (Color::White, PieceType::King) => self.white_king,

            (Color::Black, PieceType::Pawn) => self.black_pawns,
            (Color::Black, PieceType::Knight) => self.black_knights,
            (Color::Black, PieceType::Bishop) => self.black_bishops,
            (Color::Black, PieceType::Rook) => self.black_rooks,
            (Color::Black, PieceType::Queen) => self.black_queens,
            (Color::Black, PieceType::King) => self.black_king,
        }
    }

    /// Get mutable reference to piece bitboard
    pub fn get_piece_bitboard_mut(&mut self, color: Color, piece_type: PieceType) -> &mut Bitboard {
        match (color, piece_type) {
            (Color::White, PieceType::Pawn) => &mut self.white_pawns,
            (Color::White, PieceType::Knight) => &mut self.white_knights,
            (Color::White, PieceType::Bishop) => &mut self.white_bishops,
            (Color::White, PieceType::Rook) => &mut self.white_rooks,
            (Color::White, PieceType::Queen) => &mut self.white_queens,
            (Color::White, PieceType::King) => &mut self.white_king,

            (Color::Black, PieceType::Pawn) => &mut self.black_pawns,
            (Color::Black, PieceType::Knight) => &mut self.black_knights,
            (Color::Black, PieceType::Bishop) => &mut self.black_bishops,
            (Color::Black, PieceType::Rook) => &mut self.black_rooks,
            (Color::Black, PieceType::Queen) => &mut self.black_queens,
            (Color::Black, PieceType::King) => &mut self.black_king,
        }
    }

    /// Set a piece on the bitboard
    pub fn set_piece(&mut self, square: Square, color: Color, piece_type: PieceType) {
        let bit = 1u64 << square.index();
        *self.get_piece_bitboard_mut(color, piece_type) |= bit;
        self.update_combined();
    }

    /// Remove a piece from the bitboard
    pub fn clear_piece(&mut self, square: Square, color: Color, piece_type: PieceType) {
        let bit = !(1u64 << square.index());
        *self.get_piece_bitboard_mut(color, piece_type) &= bit;
        self.update_combined();
    }
}

impl Default for BitboardSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Precomputed attack tables for each piece type
pub struct AttackTables {
    pub knight_attacks: [Bitboard; 64],
    pub king_attacks: [Bitboard; 64],
    pub pawn_attacks: [[Bitboard; 64]; 2], // [color][square]
}

impl Default for AttackTables {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackTables {
    /// Initialize precomputed attack tables
    pub fn new() -> Self {
        let mut tables = Self {
            knight_attacks: [0; 64],
            king_attacks: [0; 64],
            pawn_attacks: [[0; 64]; 2],
        };

        tables.init_knight_attacks();
        tables.init_king_attacks();
        tables.init_pawn_attacks();

        tables
    }

    fn init_knight_attacks(&mut self) {
        for square in 0..64 {
            let mut attacks = 0u64;
            let rank = square / 8;
            let file = square % 8;

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

            for (rank_offset, file_offset) in knight_moves.iter() {
                let new_rank = rank as i8 + rank_offset;
                let new_file = file as i8 + file_offset;

                if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                    let target_square = new_rank as u8 * 8 + new_file as u8;
                    attacks |= 1u64 << target_square;
                }
            }

            self.knight_attacks[square as usize] = attacks;
        }
    }

    fn init_king_attacks(&mut self) {
        for square in 0..64 {
            let mut attacks = 0u64;
            let rank = square / 8;
            let file = square % 8;

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

            for (rank_offset, file_offset) in king_moves.iter() {
                let new_rank = rank as i8 + rank_offset;
                let new_file = file as i8 + file_offset;

                if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                    let target_square = new_rank as u8 * 8 + new_file as u8;
                    attacks |= 1u64 << target_square;
                }
            }

            self.king_attacks[square as usize] = attacks;
        }
    }

    fn init_pawn_attacks(&mut self) {
        for square in 0..64 {
            let rank = square / 8;
            let file = square % 8;

            // White pawn attacks (moving up the board)
            let mut white_attacks = 0u64;
            if rank < 7 {
                // Attack diagonally up-left
                if file > 0 {
                    white_attacks |= 1u64 << ((rank + 1) * 8 + (file - 1));
                }
                // Attack diagonally up-right
                if file < 7 {
                    white_attacks |= 1u64 << ((rank + 1) * 8 + (file + 1));
                }
            }
            self.pawn_attacks[0][square as usize] = white_attacks; // Color::White = 0

            // Black pawn attacks (moving down the board)
            let mut black_attacks = 0u64;
            if rank > 0 {
                // Attack diagonally down-left
                if file > 0 {
                    black_attacks |= 1u64 << ((rank - 1) * 8 + (file - 1));
                }
                // Attack diagonally down-right
                if file < 7 {
                    black_attacks |= 1u64 << ((rank - 1) * 8 + (file + 1));
                }
            }
            self.pawn_attacks[1][square as usize] = black_attacks; // Color::Black = 1
        }
    }

    /// Get knight attack bitboard for a square
    #[must_use]
    pub fn get_knight_attacks(&self, square: Square) -> Bitboard {
        self.knight_attacks[square.index() as usize]
    }

    /// Get king attack bitboard for a square
    #[must_use]
    pub fn get_king_attacks(&self, square: Square) -> Bitboard {
        self.king_attacks[square.index() as usize]
    }

    /// Get pawn attack bitboard for a square
    #[must_use]
    pub fn get_pawn_attacks(&self, square: Square, color: Color) -> Bitboard {
        let color_index = match color {
            Color::White => 0,
            Color::Black => 1,
        };
        self.pawn_attacks[color_index][square.index() as usize]
    }
}

/// Global instance of precomputed attack tables
static ATTACK_TABLES: std::sync::OnceLock<AttackTables> = std::sync::OnceLock::new();

/// Get reference to global attack tables
pub fn get_attack_tables() -> &'static AttackTables {
    ATTACK_TABLES.get_or_init(AttackTables::new)
}

/// Sliding piece move generation using classical approach
pub struct SlidingMoves;

impl SlidingMoves {
    /// Generate bishop moves from a square with given occupancy
    #[must_use]
    pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let mut attacks = 0u64;
        let square_bb = 1u64 << square.index();

        // Four diagonal directions: NE, NW, SE, SW
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        for (rank_dir, file_dir) in directions.iter() {
            attacks |= Self::ray_attacks(square, occupancy, *rank_dir, *file_dir);
        }

        attacks & !square_bb // Remove origin square
    }

    /// Generate rook moves from a square with given occupancy
    #[must_use]
    pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        let mut attacks = 0u64;
        let square_bb = 1u64 << square.index();

        // Four orthogonal directions: N, S, E, W
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        for (rank_dir, file_dir) in directions.iter() {
            attacks |= Self::ray_attacks(square, occupancy, *rank_dir, *file_dir);
        }

        attacks & !square_bb // Remove origin square
    }

    /// Generate queen moves (combination of bishop and rook)
    #[must_use]
    pub fn queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        Self::bishop_attacks(square, occupancy) | Self::rook_attacks(square, occupancy)
    }

    /// Generate attacks along a ray until hitting a blocker
    fn ray_attacks(square: Square, occupancy: Bitboard, rank_dir: i8, file_dir: i8) -> Bitboard {
        let mut attacks = 0u64;
        let mut rank = square.rank() as i8;
        let mut file = square.file() as i8;

        loop {
            rank += rank_dir;
            file += file_dir;

            // Check bounds
            if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                break;
            }

            let target_square = (rank as u8) * 8 + (file as u8);
            let target_bb = 1u64 << target_square;

            attacks |= target_bb;

            // Stop if we hit an occupied square
            if (occupancy & target_bb) != 0 {
                break;
            }
        }

        attacks
    }
}

/// Pawn move generation
pub struct PawnMoves;

impl PawnMoves {
    /// Generate pawn push moves (one and two squares forward)
    #[must_use]
    pub fn pawn_pushes(pawns: Bitboard, empty: Bitboard, color: Color) -> Bitboard {
        match color {
            Color::White => {
                let single_pushes = (pawns << 8) & empty;
                let double_pushes = ((single_pushes & 0x0000_0000_00FF_0000) << 8) & empty;
                single_pushes | double_pushes
            }
            Color::Black => {
                let single_pushes = (pawns >> 8) & empty;
                let double_pushes = ((single_pushes & 0x0000_FF00_0000_0000) >> 8) & empty;
                single_pushes | double_pushes
            }
        }
    }

    /// Generate pawn capture moves
    #[must_use]
    pub fn pawn_captures(pawns: Bitboard, enemy_pieces: Bitboard, color: Color) -> Bitboard {
        match color {
            Color::White => {
                let left_attacks = ((pawns & 0xFEFE_FEFE_FEFE_FEFE) << 7) & enemy_pieces;
                let right_attacks = ((pawns & 0x7F7F_7F7F_7F7F_7F7F) << 9) & enemy_pieces;
                left_attacks | right_attacks
            }
            Color::Black => {
                let left_attacks = ((pawns & 0xFEFE_FEFE_FEFE_FEFE) >> 9) & enemy_pieces;
                let right_attacks = ((pawns & 0x7F7F_7F7F_7F7F_7F7F) >> 7) & enemy_pieces;
                left_attacks | right_attacks
            }
        }
    }
}

/// Bitboard utility functions
pub struct BitboardUtils;

impl BitboardUtils {
    /// Convert square index to bitboard
    #[must_use]
    pub fn square_to_bitboard(square: Square) -> Bitboard {
        1u64 << square.index()
    }

    /// Check if a square is set in a bitboard
    #[must_use]
    pub fn is_square_set(bitboard: Bitboard, square: Square) -> bool {
        (bitboard & (1u64 << square.index())) != 0
    }

    /// Count number of set bits (population count)
    #[must_use]
    pub fn popcount(bitboard: Bitboard) -> u32 {
        bitboard.count_ones()
    }

    /// Get the least significant bit (trailing zeros)
    #[must_use]
    pub fn lsb(bitboard: Bitboard) -> u32 {
        bitboard.trailing_zeros()
    }

    /// Pop the least significant bit and return its position
    #[must_use]
    pub fn pop_lsb(bitboard: &mut Bitboard) -> Option<u32> {
        if *bitboard == 0 {
            None
        } else {
            let lsb_pos = bitboard.trailing_zeros();
            *bitboard &= *bitboard - 1; // Clear the LSB
            Some(lsb_pos)
        }
    }

    /// Get all set squares from a bitboard
    pub fn get_set_squares(mut bitboard: Bitboard) -> Vec<Square> {
        let mut squares = Vec::new();

        while bitboard != 0 {
            let square_index = Self::lsb(bitboard);
            if let Ok(square) = Square::from_index(square_index as u8) {
                squares.push(square);
            }
            bitboard &= bitboard - 1; // Clear the LSB
        }

        squares
    }

    /// Print bitboard in a human-readable format for debugging
    #[allow(dead_code)]
    pub fn print_bitboard(bitboard: Bitboard) {
        println!("Bitboard: 0x{:016x}", bitboard);
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square_index = rank * 8 + file;
                if (bitboard & (1u64 << square_index)) != 0 {
                    print!("1 ");
                } else {
                    print!("0 ");
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }
}
