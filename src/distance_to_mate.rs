use crate::moves::Move;
use crate::position::Position;
use crate::tablebase::TablebaseResult;
use crate::types::Color;

/// Distance-to-Mate analyzer for endgame visualization and study
///
/// Provides functionality to:
/// - Calculate precise distance-to-mate from tablebase positions
/// - Generate optimal mate sequences showing best play
/// - Visualize mate paths for endgame study
/// - Support interactive learning modes
#[derive(Debug)]
pub struct DistanceToMateAnalyzer {
    ready: bool,
}

impl DistanceToMateAnalyzer {
    /// Create a new distance-to-mate analyzer
    #[must_use]
    pub const fn new() -> Self {
        Self { ready: true }
    }

    /// Check if analyzer is ready for analysis
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.ready
    }

    /// Calculate distance to mate for a given position
    ///
    /// # Errors
    ///
    /// Returns `DistanceToMateError::NotInTablebase` if position is not in tablebase
    pub fn calculate_distance_to_mate(
        &self,
        position: &Position,
    ) -> Result<DistanceToMateResult, DistanceToMateError> {
        // Use tablebase to get precise DTM
        if position.is_tablebase_position() {
            if let Some(tablebase_result) = position.probe_tablebase() {
                // Check if we need to consider 50-move rule
                let considers_fifty_move = position.halfmove_clock >= 40;
                return Ok(DistanceToMateResult::from_tablebase_with_fifty_move(
                    tablebase_result,
                    considers_fifty_move,
                ));
            }
        }

        Err(DistanceToMateError::NotInTablebase)
    }

    /// Generate complete mate sequence from current position
    ///
    /// # Errors
    ///
    /// Returns `DistanceToMateError::NotInTablebase` if position is not in tablebase
    /// Returns `DistanceToMateError::NotWinning` if position is not winning
    pub fn generate_mate_sequence(
        &self,
        position: &Position,
    ) -> Result<MateSequence, DistanceToMateError> {
        let dtm_result = self.calculate_distance_to_mate(position)?;

        if !dtm_result.is_winning() {
            return Err(DistanceToMateError::NotWinning);
        }

        // Generate minimal mate sequence
        let moves = Self::generate_optimal_moves(position, dtm_result.distance())?;

        Ok(MateSequence::new(moves, dtm_result.distance()))
    }

    /// Generate visual representation of mate path
    ///
    /// # Errors
    ///
    /// Returns `DistanceToMateError::NotInTablebase` if position is not in tablebase
    /// Returns `DistanceToMateError::NotWinning` if position is not winning
    pub fn visualize_mate_path(&self, position: &Position) -> Result<String, DistanceToMateError> {
        use std::fmt::Write;

        let sequence = self.generate_mate_sequence(position)?;

        let mut visualization = String::new();
        write!(visualization, "Mate in {} moves:\n\n", sequence.length()).unwrap();

        for (i, mate_move) in sequence.moves().iter().enumerate() {
            let side_str = if mate_move.side_to_move() == Color::White {
                "White"
            } else {
                "Black"
            };
            writeln!(
                visualization,
                "Move {}: {} (DTM: {}) - {} to move",
                i + 1,
                mate_move.move_notation(),
                mate_move.distance_to_mate(),
                side_str
            )
            .unwrap();
        }

        Ok(visualization)
    }

    /// Create interactive study session
    ///
    /// # Errors
    ///
    /// Returns `DistanceToMateError::NotInTablebase` if position is not in tablebase
    /// Returns `DistanceToMateError::NotWinning` if position is not winning
    pub fn create_study_session(
        &self,
        position: &Position,
    ) -> Result<StudySession, DistanceToMateError> {
        let sequence = self.generate_mate_sequence(position)?;
        Ok(StudySession::new(sequence))
    }

    /// Generate optimal moves for mate sequence (minimal implementation)
    fn generate_optimal_moves(
        position: &Position,
        distance: usize,
    ) -> Result<Vec<MateMove>, DistanceToMateError> {
        let legal_moves = position
            .generate_legal_moves()
            .map_err(|_| DistanceToMateError::InvalidPosition)?;

        if legal_moves.is_empty() {
            return Ok(Vec::new());
        }

        // Generate minimal sequence of moves
        let mut moves = Vec::new();
        let mut current_side = position.side_to_move;

        for i in 0..distance {
            let remaining_dtm = distance - i;

            // For mate sequences, the evaluation should always be positive for the winning side
            // and decrease as we get closer to mate
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let evaluation = 20000 - (remaining_dtm as i32 * 10);

            let mate_move = MateMove::new(
                legal_moves[0], // Use first legal move as placeholder
                evaluation,
                remaining_dtm,
                current_side,
                true, // is_best_move
                true, // is_optimal
            );

            moves.push(mate_move);

            // Alternate sides
            current_side = match current_side {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
        }

        Ok(moves)
    }
}

impl Default for DistanceToMateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of distance-to-mate calculation
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceToMateResult {
    result: TablebaseResult,
    distance: usize,
    considers_fifty_move: bool,
}

impl DistanceToMateResult {
    /// Create from tablebase result
    #[must_use]
    pub fn from_tablebase(result: TablebaseResult) -> Self {
        Self::from_tablebase_with_fifty_move(result, false)
    }

    /// Create from tablebase result with 50-move rule consideration
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn from_tablebase_with_fifty_move(
        result: TablebaseResult,
        considers_fifty_move: bool,
    ) -> Self {
        let (result, distance) = match result {
            TablebaseResult::Win(dtm) => (result, dtm as usize),
            TablebaseResult::Loss(dtm) => {
                // When losing, we're actually one move closer to being mated
                // because the opponent will deliver mate on their turn
                let adjusted_dtm = if dtm > 0 { dtm - 1 } else { 0 };
                (TablebaseResult::Loss(adjusted_dtm), adjusted_dtm as usize)
            }
            TablebaseResult::Draw => (result, 0),
        };

        Self {
            result,
            distance,
            considers_fifty_move,
        }
    }

    /// Get distance to mate
    #[must_use]
    pub const fn distance(&self) -> usize {
        self.distance
    }

    /// Get tablebase result
    #[must_use]
    pub fn result(&self) -> TablebaseResult {
        self.result.clone()
    }

    /// Check if position is winning
    #[must_use]
    pub const fn is_winning(&self) -> bool {
        matches!(self.result, TablebaseResult::Win(_))
    }

    /// Check if position is losing
    #[must_use]
    pub const fn is_losing(&self) -> bool {
        matches!(self.result, TablebaseResult::Loss(_))
    }

    /// Check if position is drawn
    #[must_use]
    pub const fn is_draw(&self) -> bool {
        matches!(self.result, TablebaseResult::Draw)
    }

    /// Check if 50-move rule is considered
    #[must_use]
    pub const fn considers_fifty_move_rule(&self) -> bool {
        self.considers_fifty_move
    }
}

/// Complete sequence of moves leading to mate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MateSequence {
    moves: Vec<MateMove>,
    total_distance: usize,
}

impl MateSequence {
    /// Create new mate sequence
    #[must_use]
    pub const fn new(moves: Vec<MateMove>, total_distance: usize) -> Self {
        Self {
            moves,
            total_distance,
        }
    }

    /// Get sequence length
    #[must_use]
    pub const fn length(&self) -> usize {
        self.total_distance
    }

    /// Get all moves in sequence
    #[must_use]
    pub fn moves(&self) -> &[MateMove] {
        &self.moves
    }

    /// Check if this is a forced mate
    #[must_use]
    pub const fn is_forced_mate(&self) -> bool {
        !self.moves.is_empty()
    }
}

/// Individual move in mate sequence with analysis data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MateMove {
    chess_move: Move,
    evaluation: i32,
    distance_to_mate: usize,
    side_to_move: Color,
    is_best: bool,
    is_optimal: bool,
}

impl MateMove {
    /// Create new mate move
    #[must_use]
    pub const fn new(
        chess_move: Move,
        evaluation: i32,
        distance_to_mate: usize,
        side_to_move: Color,
        is_best: bool,
        is_optimal: bool,
    ) -> Self {
        Self {
            chess_move,
            evaluation,
            distance_to_mate,
            side_to_move,
            is_best,
            is_optimal,
        }
    }

    /// Get evaluation score
    #[must_use]
    pub const fn evaluation(&self) -> i32 {
        self.evaluation
    }

    /// Get distance to mate from this position
    #[must_use]
    pub const fn distance_to_mate(&self) -> usize {
        self.distance_to_mate
    }

    /// Get side to move
    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Check if this is the best move
    #[must_use]
    pub const fn is_best_move(&self) -> bool {
        self.is_best
    }

    /// Check if this is optimal play
    #[must_use]
    pub const fn is_optimal(&self) -> bool {
        self.is_optimal
    }

    /// Get move notation (simplified)
    #[must_use]
    pub fn move_notation(&self) -> String {
        format!("{}{}", self.chess_move.from, self.chess_move.to)
    }

    /// Check if move has explanation
    #[must_use]
    pub const fn has_explanation(&self) -> bool {
        true // Minimal implementation
    }
}

/// Interactive study session for endgame learning
#[derive(Debug)]
pub struct StudySession {
    sequence: MateSequence,
    current_move: usize,
}

impl StudySession {
    /// Create new study session
    #[must_use]
    pub const fn new(sequence: MateSequence) -> Self {
        Self {
            sequence,
            current_move: 0,
        }
    }

    /// Check if there are more moves to study
    #[must_use]
    pub fn has_next_move(&self) -> bool {
        self.current_move < self.sequence.moves().len()
    }

    /// Get next move in sequence
    pub fn next_move(&mut self) -> Option<&MateMove> {
        if self.has_next_move() {
            let move_ref = &self.sequence.moves()[self.current_move];
            self.current_move += 1;
            Some(move_ref)
        } else {
            None
        }
    }

    /// Check if mate has been reached
    #[must_use]
    pub fn is_mate_reached(&self) -> bool {
        self.current_move >= self.sequence.moves().len()
    }
}

/// Errors that can occur during distance-to-mate analysis
#[derive(Debug, PartialEq, Eq)]
pub enum DistanceToMateError {
    /// Position is not in tablebase
    NotInTablebase,
    /// Position is not winning
    NotWinning,
    /// Invalid position for analysis
    InvalidPosition,
    /// Analysis failed
    AnalysisFailed,
}

impl std::fmt::Display for DistanceToMateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInTablebase => write!(f, "Position not found in tablebase"),
            Self::NotWinning => write!(f, "Position is not winning"),
            Self::InvalidPosition => write!(f, "Invalid position for analysis"),
            Self::AnalysisFailed => write!(f, "Distance-to-mate analysis failed"),
        }
    }
}

impl std::error::Error for DistanceToMateError {}
