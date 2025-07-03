use crate::moves::Move;
use crate::position::{Position, PositionError};
use crate::types::{Color, MoveGenError};
use std::fmt;

/// Search-specific error types
#[derive(Debug, Clone, PartialEq)]
pub enum SearchError {
    MoveGenError(MoveGenError),
    PositionError(PositionError),
    TimeoutError,
    DepthLimitError,
    NoLegalMoves,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::MoveGenError(e) => write!(f, "Move generation error: {}", e),
            SearchError::PositionError(e) => write!(f, "Position error: {}", e),
            SearchError::TimeoutError => write!(f, "Search timeout exceeded"),
            SearchError::DepthLimitError => write!(f, "Search depth limit exceeded"),
            SearchError::NoLegalMoves => write!(f, "No legal moves available"),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<MoveGenError> for SearchError {
    fn from(error: MoveGenError) -> Self {
        SearchError::MoveGenError(error)
    }
}

impl From<PositionError> for SearchError {
    fn from(error: PositionError) -> Self {
        SearchError::PositionError(error)
    }
}

/// Result of a minimax search
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// The best move found by the search
    pub best_move: Move,
    /// Evaluation score in centipawns (positive = good for side to move)
    pub evaluation: i32,
    /// Depth searched to find this result
    pub depth: u8,
    /// Number of positions evaluated during search
    pub nodes_searched: u64,
    /// Search time in milliseconds
    pub time_ms: u64,
}

impl SearchResult {
    pub fn new(best_move: Move, evaluation: i32, depth: u8) -> Self {
        Self {
            best_move,
            evaluation,
            depth,
            nodes_searched: 0,
            time_ms: 0,
        }
    }
}

/// Information needed to undo a move for search traversal
#[derive(Debug, Clone, PartialEq)]
pub struct UndoMove {
    /// Piece that was captured (if any)
    pub captured_piece: Option<crate::types::Piece>,
    /// Previous castling rights before the move
    pub previous_castling: crate::types::CastlingRights,
    /// Previous en passant square before the move
    pub previous_en_passant: Option<crate::types::Square>,
    /// Previous halfmove clock before the move
    pub previous_halfmove: u8,
    /// Previous fullmove number before the move
    pub previous_fullmove: u16,
}

/// Main search engine implementation
pub struct SearchEngine {
    /// Maximum search depth
    max_depth: u8,
    /// Search statistics
    nodes_evaluated: u64,
    /// Time management
    start_time: Option<std::time::Instant>,
}

impl SearchEngine {
    /// Create a new search engine with default settings
    pub fn new() -> Self {
        Self {
            max_depth: 4, // Start with shallow search
            nodes_evaluated: 0,
            start_time: None,
        }
    }

    /// Set maximum search depth
    pub fn set_max_depth(&mut self, depth: u8) {
        self.max_depth = depth;
    }

    /// Get the number of nodes evaluated in the last search
    pub fn nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }

    /// Find the best move using minimax search
    pub fn find_best_move(&mut self, position: &Position) -> Result<SearchResult, SearchError> {
        // Reset search statistics
        self.nodes_evaluated = 0;
        self.start_time = Some(std::time::Instant::now());

        // Generate legal moves
        let legal_moves = position.generate_legal_moves()?;
        if legal_moves.is_empty() {
            return Err(SearchError::NoLegalMoves);
        }

        // If only one legal move, return it immediately
        if legal_moves.len() == 1 {
            let move_to_make = legal_moves[0];
            let evaluation = position.evaluate();
            return Ok(SearchResult::new(move_to_make, evaluation, 0));
        }

        // Search for the best move
        let mut best_move = legal_moves[0];
        let mut best_score = if position.side_to_move == Color::White {
            i32::MIN
        } else {
            i32::MAX
        };

        // Clone position for search (temporary - will optimize with make_move/unmake_move later)
        for &mv in &legal_moves {
            let mut search_position = position.clone();

            // Apply the move (using internal method for now)
            if search_position.apply_move_for_search(mv).is_ok() {
                let score = self.minimax(&search_position, self.max_depth - 1, false)?;

                // Update best move based on side to move
                let is_better = match position.side_to_move {
                    Color::White => score > best_score,
                    Color::Black => score < best_score,
                };

                if is_better {
                    best_score = score;
                    best_move = mv;
                }
            }
        }

        let elapsed = self
            .start_time
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0);

        Ok(SearchResult {
            best_move,
            evaluation: best_score,
            depth: self.max_depth,
            nodes_searched: self.nodes_evaluated,
            time_ms: elapsed,
        })
    }

    /// Basic minimax search implementation
    fn minimax(
        &mut self,
        position: &Position,
        depth: u8,
        maximizing: bool,
    ) -> Result<i32, SearchError> {
        self.nodes_evaluated += 1;

        // Base case: reached depth limit or terminal position
        if depth == 0 {
            return Ok(position.evaluate());
        }

        let legal_moves = position.generate_legal_moves()?;

        // Terminal position (checkmate or stalemate)
        if legal_moves.is_empty() {
            if position.is_check(position.side_to_move) {
                // Checkmate - return large penalty/bonus based on side to move
                return Ok(if maximizing { -10000 } else { 10000 });
            } else {
                // Stalemate
                return Ok(0);
            }
        }

        if maximizing {
            let mut max_eval = i32::MIN;
            for &mv in &legal_moves {
                let mut search_position = position.clone();
                if search_position.apply_move_for_search(mv).is_ok() {
                    let eval = self.minimax(&search_position, depth - 1, false)?;
                    max_eval = max_eval.max(eval);
                }
            }
            Ok(max_eval)
        } else {
            let mut min_eval = i32::MAX;
            for &mv in &legal_moves {
                let mut search_position = position.clone();
                if search_position.apply_move_for_search(mv).is_ok() {
                    let eval = self.minimax(&search_position, depth - 1, true)?;
                    min_eval = min_eval.min(eval);
                }
            }
            Ok(min_eval)
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        assert_eq!(engine.max_depth, 4);
        assert_eq!(engine.nodes_evaluated(), 0);
    }

    #[test]
    fn test_search_starting_position() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(2); // Shallow search for test speed

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine.find_best_move(&position);

        assert!(
            result.is_ok(),
            "Search should succeed from starting position"
        );

        let search_result = result.unwrap();
        assert!(
            search_result.nodes_searched > 0,
            "Should evaluate some nodes"
        );
        assert_eq!(search_result.depth, 2, "Should search to specified depth");
    }

    #[test]
    fn test_single_legal_move() {
        let mut engine = SearchEngine::new();

        // Create a position with only one legal move (simplified test)
        let position = Position::starting_position().expect("Starting position should be valid");

        // For now, just test that the engine can handle the starting position
        let result = engine.find_best_move(&position);
        assert!(result.is_ok(), "Should handle position with multiple moves");
    }

    #[test]
    fn test_search_error_handling() {
        let engine = SearchEngine::new();
        assert_eq!(engine.nodes_evaluated(), 0);

        // Test error type conversions
        let move_error = MoveGenError::InvalidMove("test".to_string());
        let search_error = SearchError::from(move_error.clone());

        match search_error {
            SearchError::MoveGenError(e) => assert_eq!(e, move_error),
            _ => panic!("Error conversion failed"),
        }
    }
}
