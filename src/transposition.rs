use crate::moves::Move;
use crate::position::Position;
use crate::types::{CastlingRights, Color, Piece, PieceType, Square};
use std::fmt;
use std::sync::LazyLock;

/// Global Zobrist hasher instance for consistent hashing
pub static ZOBRIST_HASHER: LazyLock<ZobristHasher> = LazyLock::new(ZobristHasher::new);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZobristError {
    InvalidSquare(&'static str),
    InvalidPiece(String),
    ComputationError(String),
}

impl fmt::Display for ZobristError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZobristError::InvalidSquare(msg) => write!(f, "Invalid square: {}", msg),
            ZobristError::InvalidPiece(msg) => write!(f, "Invalid piece: {}", msg),
            ZobristError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
        }
    }
}

impl std::error::Error for ZobristError {}

/// Zobrist hashing system for chess positions
/// Uses deterministic pseudo-random keys for consistent hashing
pub struct ZobristHasher {
    /// Keys for pieces: [color][piece_type][square]
    pieces: [[[u64; 64]; 6]; 2],
    /// Keys for castling rights (16 combinations)
    castling: [u64; 16],
    /// Keys for en passant files (a-h)
    en_passant: [u64; 8],
    /// Key for side to move
    side_to_move: u64,
}

impl ZobristHasher {
    /// Create a new Zobrist hasher with deterministic keys
    pub fn new() -> Self {
        let mut hasher = Self {
            pieces: [[[0; 64]; 6]; 2],
            castling: [0; 16],
            en_passant: [0; 8],
            side_to_move: 0,
        };

        hasher.initialize_keys();
        hasher
    }

    /// Initialize all Zobrist keys using a simple deterministic PRNG
    fn initialize_keys(&mut self) {
        let mut rng = SimpleRng::new(0x517c_c1b7_2722_0a95); // Fixed seed for deterministic keys

        // Initialize piece keys
        for color in 0..2 {
            for piece_type in 0..6 {
                for square in 0..64 {
                    self.pieces[color][piece_type][square] = rng.next();
                }
            }
        }

        // Initialize castling keys
        for i in 0..16 {
            self.castling[i] = rng.next();
        }

        // Initialize en passant keys
        for i in 0..8 {
            self.en_passant[i] = rng.next();
        }

        // Initialize side to move key
        self.side_to_move = rng.next();
    }

    /// Compute the full Zobrist hash for a position
    pub fn compute_hash(&self, position: &Position) -> Result<u64, ZobristError> {
        let mut hash = 0u64;

        // Hash all pieces on the board
        for square_index in 0..64 {
            let square = Square::from_index(square_index).map_err(ZobristError::InvalidSquare)?;

            if let Some(piece) = position.piece_at(square) {
                hash ^= self.hash_piece(piece.color, piece.piece_type, square)?;
            }
        }

        // Hash castling rights
        hash ^= self.hash_castling(&position.castling_rights)?;

        // Hash en passant square
        hash ^= self.hash_en_passant(position.en_passant)?;

        // Hash side to move
        hash ^= self.hash_side_to_move(position.side_to_move)?;

        Ok(hash)
    }

    /// Get the Zobrist key for a piece on a specific square
    pub fn hash_piece(
        &self,
        color: Color,
        piece_type: PieceType,
        square: Square,
    ) -> Result<u64, ZobristError> {
        let color_index = match color {
            Color::White => 0,
            Color::Black => 1,
        };

        let piece_index = match piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5,
        };

        let square_index = square.index() as usize;
        if square_index >= 64 {
            return Err(ZobristError::InvalidSquare("Square index out of bounds"));
        }

        Ok(self.pieces[color_index][piece_index][square_index])
    }

    /// Get the Zobrist key for castling rights
    pub const fn hash_castling(&self, rights: &CastlingRights) -> Result<u64, ZobristError> {
        let mut index = 0;

        if rights.white_kingside {
            index |= 1;
        }
        if rights.white_queenside {
            index |= 2;
        }
        if rights.black_kingside {
            index |= 4;
        }
        if rights.black_queenside {
            index |= 8;
        }

        Ok(self.castling[index])
    }

    /// Get the Zobrist key for en passant square
    pub fn hash_en_passant(&self, en_passant: Option<Square>) -> Result<u64, ZobristError> {
        match en_passant {
            Some(square) => {
                let file = square.file() as usize;
                if file >= 8 {
                    return Err(ZobristError::InvalidSquare("En passant file out of bounds"));
                }
                Ok(self.en_passant[file])
            }
            None => Ok(0),
        }
    }

    /// Get the Zobrist key for side to move
    pub const fn hash_side_to_move(&self, color: Color) -> Result<u64, ZobristError> {
        match color {
            Color::White => Ok(0), // White is represented by no additional key
            Color::Black => Ok(self.side_to_move),
        }
    }

    /// Update hash when a piece moves (without capture)
    pub fn update_piece_move(
        &self,
        hash: u64,
        from: Square,
        to: Square,
        piece: Piece,
    ) -> Result<u64, ZobristError> {
        let piece_key = self.hash_piece(piece.color, piece.piece_type, from)?;
        let new_piece_key = self.hash_piece(piece.color, piece.piece_type, to)?;

        // Remove piece from old square and add to new square
        Ok(hash ^ piece_key ^ new_piece_key)
    }

    /// Update hash when a piece is captured
    pub fn update_piece_capture(
        &self,
        hash: u64,
        square: Square,
        captured_piece: Piece,
    ) -> Result<u64, ZobristError> {
        let piece_key = self.hash_piece(captured_piece.color, captured_piece.piece_type, square)?;

        // Remove captured piece
        Ok(hash ^ piece_key)
    }

    /// Update hash when castling rights change
    pub fn update_castling_rights(
        &self,
        hash: u64,
        old_rights: &CastlingRights,
        new_rights: &CastlingRights,
    ) -> Result<u64, ZobristError> {
        let old_key = self.hash_castling(old_rights)?;
        let new_key = self.hash_castling(new_rights)?;

        // Remove old castling key and add new one
        Ok(hash ^ old_key ^ new_key)
    }

    /// Update hash when en passant square changes
    pub fn update_en_passant(
        &self,
        hash: u64,
        old_square: Option<Square>,
        new_square: Option<Square>,
    ) -> Result<u64, ZobristError> {
        let old_key = self.hash_en_passant(old_square)?;
        let new_key = self.hash_en_passant(new_square)?;

        // Remove old en passant key and add new one
        Ok(hash ^ old_key ^ new_key)
    }

    /// Update hash when side to move changes
    pub const fn update_side_to_move(&self, hash: u64) -> Result<u64, ZobristError> {
        // Simply toggle the side to move key
        Ok(hash ^ self.side_to_move)
    }

    /// Apply a complete move to the hash (piece move + side change)
    pub fn apply_move(
        &self,
        hash: u64,
        from: Square,
        to: Square,
        piece: Piece,
        captured_piece: Option<Piece>,
    ) -> Result<u64, ZobristError> {
        let mut new_hash = hash;

        // Handle piece movement
        new_hash = self.update_piece_move(new_hash, from, to, piece)?;

        // Handle capture if any
        if let Some(captured) = captured_piece {
            new_hash = self.update_piece_capture(new_hash, to, captured)?;
        }

        // Switch side to move
        new_hash = self.update_side_to_move(new_hash)?;

        Ok(new_hash)
    }
}

impl Default for ZobristHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of evaluation stored in transposition table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Exact evaluation (search completed normally within window)
    Exact,
    /// Lower bound (alpha cutoff, actual score >= this value)
    LowerBound,
    /// Upper bound (beta cutoff, actual score <= this value)
    UpperBound,
}

/// Entry in the transposition table
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    /// Full Zobrist hash for collision detection
    pub hash: u64,
    /// Evaluation score in centipawns
    pub score: i32,
    /// Search depth that produced this entry
    pub depth: u8,
    /// Best move from this position (for move ordering)
    pub best_move: Option<Move>,
    /// Type of score stored
    pub node_type: NodeType,
    /// Age for replacement strategy (generation counter)
    pub age: u8,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: 0,
            score: 0,
            depth: 0,
            best_move: None,
            node_type: NodeType::Exact,
            age: 0,
        }
    }
}

/// Transposition table for storing position evaluations
pub struct TranspositionTable {
    /// Table entries (power of 2 size for fast indexing)
    entries: Vec<TTEntry>,
    /// Current generation for age-based replacement
    generation: u8,
    /// Statistics
    pub hits: u64,
    pub misses: u64,
    pub collisions: u64,
    pub stores: u64,
}

impl TranspositionTable {
    /// Create new transposition table with specified size in MB
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;

        // Round down to nearest power of 2 for fast indexing
        let power_of_2_entries = num_entries.next_power_of_two() / 2;

        Self {
            entries: vec![TTEntry::default(); power_of_2_entries],
            generation: 0,
            hits: 0,
            misses: 0,
            collisions: 0,
            stores: 0,
        }
    }

    /// Create default 64MB transposition table
    pub fn default_size() -> Self {
        Self::new(64)
    }

    /// Get table index from hash (fast with power-of-2 size)
    const fn get_index(&self, hash: u64) -> usize {
        (hash as usize) & (self.entries.len() - 1)
    }

    /// Probe the table for a position
    pub fn probe(&mut self, hash: u64, depth: u8) -> Option<TTEntry> {
        let index = self.get_index(hash);
        let entry = self.entries[index];

        // Check if entry matches this position
        if entry.hash == hash {
            // Check if depth is sufficient
            if entry.depth >= depth {
                self.hits += 1;
                return Some(entry);
            }
        } else if entry.hash != 0 {
            // Hash collision
            self.collisions += 1;
        }

        self.misses += 1;
        None
    }

    /// Store an entry in the table
    pub fn store(
        &mut self,
        hash: u64,
        score: i32,
        depth: u8,
        best_move: Option<Move>,
        node_type: NodeType,
    ) {
        let index = self.get_index(hash);
        let current = &self.entries[index];

        // Replacement strategy: always replace if empty or same position
        // For different positions, only replace if deeper search or entry is old
        let should_replace = current.hash == 0
            || current.hash == hash
            || depth >= current.depth
            || current.age < self.generation.saturating_sub(2);

        if should_replace {
            self.entries[index] = TTEntry {
                hash,
                score,
                depth,
                best_move,
                node_type,
                age: self.generation,
            };
            self.stores += 1;
        }
    }

    /// Clear the table and increment generation
    pub const fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = TTEntry::default();
        }
        self.generation = 0;
        self.hits = 0;
        self.misses = 0;
        self.collisions = 0;
        self.stores = 0;
    }

    /// Get table statistics
    #[allow(clippy::cast_precision_loss)]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get memory usage in MB
    #[allow(clippy::cast_precision_loss)]
    pub fn memory_usage_mb(&self) -> f64 {
        let bytes = self.entries.len() * std::mem::size_of::<TTEntry>();
        bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get number of entries
    #[must_use]
    pub const fn size(&self) -> usize {
        self.entries.len()
    }
}

/// Simple deterministic pseudo-random number generator
/// Uses a linear congruential generator for consistent key generation
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    const fn next(&mut self) -> u64 {
        // Linear congruential generator constants (from Numerical Recipes)
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    fn create_test_position() -> Position {
        Position::starting_position().expect("Failed to create starting position")
    }

    #[test]
    fn test_zobrist_hasher_creation() {
        let hasher = ZobristHasher::new();

        // Verify that keys are initialized (not all zero)
        let mut has_non_zero = false;
        for color in 0..2 {
            for piece_type in 0..6 {
                for square in 0..64 {
                    if hasher.pieces[color][piece_type][square] != 0 {
                        has_non_zero = true;
                        break;
                    }
                }
            }
        }
        assert!(has_non_zero, "Zobrist keys should not all be zero");
    }

    #[test]
    fn test_hash_consistency() {
        let hasher = ZobristHasher::new();
        let position = create_test_position();

        let hash1 = hasher
            .compute_hash(&position)
            .expect("Hash computation failed");
        let hash2 = hasher
            .compute_hash(&position)
            .expect("Hash computation failed");

        assert_eq!(
            hash1, hash2,
            "Hash should be consistent for identical positions"
        );
    }

    #[test]
    fn test_hash_piece() {
        let hasher = ZobristHasher::new();
        let square = Square::from_algebraic("e4").expect("Invalid square");

        let white_pawn = hasher
            .hash_piece(Color::White, PieceType::Pawn, square)
            .expect("Hash failed");
        let black_pawn = hasher
            .hash_piece(Color::Black, PieceType::Pawn, square)
            .expect("Hash failed");
        let white_queen = hasher
            .hash_piece(Color::White, PieceType::Queen, square)
            .expect("Hash failed");

        assert_ne!(
            white_pawn, black_pawn,
            "Different colors should have different hashes"
        );
        assert_ne!(
            white_pawn, white_queen,
            "Different piece types should have different hashes"
        );
    }

    #[test]
    fn test_hash_castling() {
        let hasher = ZobristHasher::new();

        let full_rights = CastlingRights::new();
        let no_rights = CastlingRights::none();

        let hash1 = hasher.hash_castling(&full_rights).expect("Hash failed");
        let hash2 = hasher.hash_castling(&no_rights).expect("Hash failed");

        assert_ne!(
            hash1, hash2,
            "Different castling rights should have different hashes"
        );
    }

    #[test]
    fn test_hash_en_passant() {
        let hasher = ZobristHasher::new();

        let square = Square::from_algebraic("e3").expect("Invalid square");
        let hash1 = hasher.hash_en_passant(Some(square)).expect("Hash failed");
        let hash2 = hasher.hash_en_passant(None).expect("Hash failed");

        assert_ne!(hash1, hash2, "En passant presence should affect hash");
    }

    #[test]
    fn test_hash_side_to_move() {
        let hasher = ZobristHasher::new();

        let white_hash = hasher.hash_side_to_move(Color::White).expect("Hash failed");
        let black_hash = hasher.hash_side_to_move(Color::Black).expect("Hash failed");

        assert_ne!(
            white_hash, black_hash,
            "Different sides should have different hashes"
        );
    }

    #[test]
    fn test_update_piece_move() {
        let hasher = ZobristHasher::new();
        let position = create_test_position();

        let original_hash = hasher.compute_hash(&position).expect("Hash failed");

        let from = Square::from_algebraic("e2").expect("Invalid square");
        let to = Square::from_algebraic("e4").expect("Invalid square");
        let piece = Piece::new(Color::White, PieceType::Pawn);

        let updated_hash = hasher
            .update_piece_move(original_hash, from, to, piece)
            .expect("Update failed");

        assert_ne!(original_hash, updated_hash, "Move should change hash");
    }

    #[test]
    fn test_update_side_to_move() {
        let hasher = ZobristHasher::new();
        let position = create_test_position();

        let original_hash = hasher.compute_hash(&position).expect("Hash failed");
        let toggled_hash = hasher
            .update_side_to_move(original_hash)
            .expect("Update failed");
        let double_toggled_hash = hasher
            .update_side_to_move(toggled_hash)
            .expect("Update failed");

        assert_ne!(
            original_hash, toggled_hash,
            "Side change should affect hash"
        );
        assert_eq!(
            original_hash, double_toggled_hash,
            "Double side change should restore original hash"
        );
    }

    #[test]
    fn test_incremental_vs_full_hash() {
        let hasher = ZobristHasher::new();
        let mut position = create_test_position();

        let original_hash = hasher.compute_hash(&position).expect("Hash failed");

        // Simulate a move: e2-e4
        let from = Square::from_algebraic("e2").expect("Invalid square");
        let to = Square::from_algebraic("e4").expect("Invalid square");
        let piece = position.piece_at(from).expect("No piece at e2");

        // Update incrementally
        let mut incremental_hash = original_hash;
        incremental_hash = hasher
            .update_piece_move(incremental_hash, from, to, piece)
            .expect("Update failed");
        incremental_hash = hasher
            .update_side_to_move(incremental_hash)
            .expect("Update failed");

        // Update position and compute full hash
        position.set_piece(from, None);
        position.set_piece(to, Some(piece));
        position.switch_side();

        let full_hash = hasher.compute_hash(&position).expect("Hash failed");

        assert_eq!(
            incremental_hash, full_hash,
            "Incremental and full hash should match"
        );
    }

    #[test]
    fn test_deterministic_keys() {
        let hasher1 = ZobristHasher::new();
        let hasher2 = ZobristHasher::new();

        // Keys should be identical between instances
        assert_eq!(
            hasher1.pieces[0][0][0], hasher2.pieces[0][0][0],
            "Keys should be deterministic"
        );
        assert_eq!(
            hasher1.castling[0], hasher2.castling[0],
            "Castling keys should be deterministic"
        );
        assert_eq!(
            hasher1.en_passant[0], hasher2.en_passant[0],
            "En passant keys should be deterministic"
        );
        assert_eq!(
            hasher1.side_to_move, hasher2.side_to_move,
            "Side to move key should be deterministic"
        );
    }

    #[test]
    fn test_unique_keys() {
        let hasher = ZobristHasher::new();

        // Test that different piece/square combinations have different keys
        let key1 = hasher
            .hash_piece(
                Color::White,
                PieceType::Pawn,
                Square::from_algebraic("a1").unwrap(),
            )
            .unwrap();
        let key2 = hasher
            .hash_piece(
                Color::White,
                PieceType::Pawn,
                Square::from_algebraic("a2").unwrap(),
            )
            .unwrap();
        let key3 = hasher
            .hash_piece(
                Color::White,
                PieceType::Knight,
                Square::from_algebraic("a1").unwrap(),
            )
            .unwrap();
        let key4 = hasher
            .hash_piece(
                Color::Black,
                PieceType::Pawn,
                Square::from_algebraic("a1").unwrap(),
            )
            .unwrap();

        assert_ne!(key1, key2, "Different squares should have different keys");
        assert_ne!(key1, key3, "Different pieces should have different keys");
        assert_ne!(key1, key4, "Different colors should have different keys");
    }

    #[test]
    fn test_castling_combinations() {
        let hasher = ZobristHasher::new();

        let mut rights = CastlingRights::none();
        let hash1 = hasher.hash_castling(&rights).unwrap();

        rights.white_kingside = true;
        let hash2 = hasher.hash_castling(&rights).unwrap();

        rights.white_queenside = true;
        let hash3 = hasher.hash_castling(&rights).unwrap();

        rights.black_kingside = true;
        let hash4 = hasher.hash_castling(&rights).unwrap();

        rights.black_queenside = true;
        let hash5 = hasher.hash_castling(&rights).unwrap();

        // All combinations should be different
        let hash_values = [hash1, hash2, hash3, hash4, hash5];
        for i in 0..hash_values.len() {
            for j in i + 1..hash_values.len() {
                assert_ne!(
                    hash_values[i], hash_values[j],
                    "Castling combinations should have unique hashes"
                );
            }
        }
    }

    // Transposition Table Tests

    #[test]
    fn test_tt_creation() {
        let tt = TranspositionTable::new(1); // 1MB table
        assert!(tt.size() > 0, "Table should have entries");
        assert_eq!(tt.hits, 0, "Should start with 0 hits");
        assert_eq!(tt.misses, 0, "Should start with 0 misses");
        assert!(tt.memory_usage_mb() <= 1.1, "Should be approximately 1MB");
    }

    #[test]
    fn test_tt_store_and_probe_hit() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;
        let score = 150;
        let depth = 5;
        let mv = Move::quiet(
            Square::from_algebraic("e2").unwrap(),
            Square::from_algebraic("e4").unwrap(),
        );

        // Store entry
        tt.store(hash, score, depth, Some(mv), NodeType::Exact);

        // Probe should hit
        let entry = tt.probe(hash, depth);
        assert!(entry.is_some(), "Should find stored entry");

        let entry = entry.unwrap();
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.score, score);
        assert_eq!(entry.depth, depth);
        assert_eq!(entry.best_move, Some(mv));
        assert_eq!(entry.node_type, NodeType::Exact);

        assert_eq!(tt.hits, 1, "Should have 1 hit");
        assert_eq!(tt.stores, 1, "Should have 1 store");
    }

    #[test]
    fn test_tt_probe_miss_on_empty_table() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        let entry = tt.probe(hash, 5);
        assert!(entry.is_none(), "Should miss on empty table");
        assert_eq!(tt.misses, 1, "Should have 1 miss");
    }

    #[test]
    fn test_tt_probe_miss_on_insufficient_depth() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Store with depth 3
        tt.store(hash, 100, 3, None, NodeType::Exact);

        // Probe with depth 5 should miss
        let entry = tt.probe(hash, 5);
        assert!(entry.is_none(), "Should miss when depth is insufficient");
        assert_eq!(tt.misses, 1, "Should have 1 miss");
    }

    #[test]
    fn test_tt_replacement_strategy_deeper_replaces_shallower() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Store shallow entry
        tt.store(hash, 100, 3, None, NodeType::Exact);
        let entry1 = tt.probe(hash, 3).unwrap();
        assert_eq!(entry1.depth, 3);

        // Store deeper entry (should replace)
        tt.store(hash, 200, 5, None, NodeType::LowerBound);
        let entry2 = tt.probe(hash, 3).unwrap();
        assert_eq!(entry2.depth, 5);
        assert_eq!(entry2.score, 200);
        assert_eq!(entry2.node_type, NodeType::LowerBound);
    }

    #[test]
    fn test_tt_replacement_strategy_same_position_deeper_replaces() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Store shallow entry
        tt.store(hash, 100, 3, None, NodeType::Exact);
        let entry1 = tt.probe(hash, 3).unwrap();
        assert_eq!(entry1.depth, 3);

        // Store deeper entry for same position (should replace)
        tt.store(hash, 200, 5, None, NodeType::UpperBound);
        let entry2 = tt.probe(hash, 3).unwrap();
        assert_eq!(
            entry2.depth, 5,
            "Deeper entry should replace shallower for same position"
        );
        assert_eq!(entry2.score, 200);
    }

    #[test]
    fn test_tt_clear() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Store entry
        tt.store(hash, 100, 5, None, NodeType::Exact);
        assert!(tt.probe(hash, 5).is_some());

        // Clear table
        tt.clear();
        assert!(
            tt.probe(hash, 5).is_none(),
            "Entry should be gone after clear"
        );
        assert_eq!(tt.hits, 0, "Stats should be reset");
        assert_eq!(tt.misses, 1, "Miss from probe after clear");
        assert_eq!(tt.stores, 0, "Store count should be reset");
    }

    #[test]
    fn test_tt_collision_detection() {
        let mut tt = TranspositionTable::new(1);

        // Find two hashes that map to the same index
        let hash1 = 0x1000_0000_0000_0000;
        let hash2 = 0x2000_0000_0000_0000;

        // Make sure they map to the same index
        let index1 = (hash1 as usize) & (tt.size() - 1);
        let index2 = (hash2 as usize) & (tt.size() - 1);

        if index1 == index2 {
            // Store first entry
            tt.store(hash1, 100, 5, None, NodeType::Exact);

            // Probe with different hash should miss and count as collision
            let entry = tt.probe(hash2, 5);
            assert!(entry.is_none(), "Should miss on hash collision");
            assert_eq!(tt.collisions, 1, "Should count collision");
        }
    }

    #[test]
    fn test_tt_node_types() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Test all node types
        tt.store(hash, 100, 5, None, NodeType::Exact);
        let entry = tt.probe(hash, 5).unwrap();
        assert_eq!(entry.node_type, NodeType::Exact);

        tt.store(hash, 150, 5, None, NodeType::LowerBound);
        let entry = tt.probe(hash, 5).unwrap();
        assert_eq!(entry.node_type, NodeType::LowerBound);

        tt.store(hash, 50, 5, None, NodeType::UpperBound);
        let entry = tt.probe(hash, 5).unwrap();
        assert_eq!(entry.node_type, NodeType::UpperBound);
    }

    #[test]
    fn test_tt_statistics() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1234_5678_9ABC_DEF0;

        // Initial hit rate should be 0
        assert_eq!(tt.hit_rate(), 0.0);

        // Store and probe
        tt.store(hash, 100, 5, None, NodeType::Exact);
        tt.probe(hash, 5); // Hit
        tt.probe(hash + 1, 5); // Miss

        assert_eq!(tt.hits, 1);
        assert_eq!(tt.misses, 1);
        assert_eq!(tt.hit_rate(), 0.5);
    }
}
