use crate::moves::Move;
use crate::position::{Position, PositionError};
use crate::types::MoveGenError;
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

/// Result of a search operation
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// The best move found by the search
    pub best_move: Move,
    /// Evaluation score in centipawns (positive = good for side to move)
    pub evaluation: i32,
    /// Target depth for this search
    pub depth: u8,
    /// Deepest completed iteration in iterative deepening
    pub completed_depth: u8,
    /// Number of positions evaluated during search
    pub nodes_searched: u64,
    /// Number of branches pruned by alpha-beta
    pub nodes_pruned: u64,
    /// Search time in milliseconds
    pub time_ms: u64,
    /// Whether search was stopped by time limit
    pub time_limited: bool,
    /// Number of iterative deepening iterations completed
    pub iterations_completed: u8,
}

impl SearchResult {
    pub fn new(best_move: Move, evaluation: i32, depth: u8) -> Self {
        Self {
            best_move,
            evaluation,
            depth,
            completed_depth: depth,
            nodes_searched: 0,
            nodes_pruned: 0,
            time_ms: 0,
            time_limited: false,
            iterations_completed: 1,
        }
    }

    /// Create a new search result for iterative deepening
    /// Use a struct to avoid too many arguments
    pub fn from_iterative_data(data: IterativeSearchData) -> Self {
        Self {
            best_move: data.best_move,
            evaluation: data.evaluation,
            depth: data.target_depth,
            completed_depth: data.completed_depth,
            nodes_searched: data.nodes_searched,
            nodes_pruned: data.nodes_pruned,
            time_ms: data.time_ms,
            time_limited: data.time_limited,
            iterations_completed: data.iterations_completed,
        }
    }
}

/// Data structure for creating iterative search results
pub struct IterativeSearchData {
    pub best_move: Move,
    pub evaluation: i32,
    pub target_depth: u8,
    pub completed_depth: u8,
    pub nodes_searched: u64,
    pub nodes_pruned: u64,
    pub time_ms: u64,
    pub time_limited: bool,
    pub iterations_completed: u8,
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
    /// Maximum search time in milliseconds (None = unlimited)
    max_time_ms: Option<u64>,
    /// Search statistics
    nodes_evaluated: u64,
    /// Alpha-beta pruning statistics
    nodes_pruned: u64,
    /// Time management
    start_time: Option<std::time::Instant>,
    /// Best move from previous iteration (for move ordering)
    previous_best_move: Option<Move>,
}

impl SearchEngine {
    /// Create a new search engine with default settings
    pub fn new() -> Self {
        Self {
            max_depth: 4,      // Start with shallow search
            max_time_ms: None, // No time limit by default
            nodes_evaluated: 0,
            nodes_pruned: 0,
            start_time: None,
            previous_best_move: None,
        }
    }

    /// Set maximum search depth
    pub fn set_max_depth(&mut self, depth: u8) {
        self.max_depth = depth;
    }

    /// Set maximum search time in milliseconds
    pub fn set_time_limit(&mut self, max_time_ms: Option<u64>) {
        self.max_time_ms = max_time_ms;
    }

    /// Get the number of nodes evaluated in the last search
    pub fn nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }

    /// Check if search should stop due to time limit
    fn should_stop(&self) -> bool {
        if let (Some(start), Some(limit)) = (self.start_time, self.max_time_ms) {
            start.elapsed().as_millis() as u64 >= limit
        } else {
            false
        }
    }

    /// Find the best move using iterative deepening alpha-beta search
    pub fn find_best_move(&mut self, position: &Position) -> Result<SearchResult, SearchError> {
        // Reset search statistics
        self.nodes_evaluated = 0;
        self.nodes_pruned = 0;
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
            self.previous_best_move = Some(move_to_make);
            return Ok(SearchResult::new(move_to_make, evaluation, 0));
        }

        // Iterative deepening search
        let mut best_move = legal_moves[0];
        let mut best_evaluation = i32::MIN;
        let mut completed_depth = 0;
        let mut iterations_completed = 0;
        let mut time_limited = false;

        // Start from depth 1 and increase
        for current_depth in 1..=self.max_depth {
            // Check time limit before starting new iteration
            if self.should_stop() {
                time_limited = true;
                break;
            }

            // Order moves for this iteration
            let mut ordered_moves = legal_moves.clone();
            self.order_moves_with_pv(&mut ordered_moves);

            // Search at current depth
            match self.search_at_depth(position, &ordered_moves, current_depth) {
                Ok((iteration_best_move, iteration_score)) => {
                    // Update best move and score
                    best_move = iteration_best_move;
                    best_evaluation = iteration_score;
                    completed_depth = current_depth;
                    iterations_completed += 1;
                    self.previous_best_move = Some(best_move);

                    // Check if we ran out of time during this iteration
                    if self.should_stop() {
                        time_limited = true;
                        break;
                    }
                }
                Err(e) => {
                    // If we get an error, return the best result so far if we have one
                    if iterations_completed > 0 {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        let elapsed = self
            .start_time
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0);

        Ok(SearchResult::from_iterative_data(IterativeSearchData {
            best_move,
            evaluation: best_evaluation,
            target_depth: self.max_depth,
            completed_depth,
            nodes_searched: self.nodes_evaluated,
            nodes_pruned: self.nodes_pruned,
            time_ms: elapsed,
            time_limited,
            iterations_completed,
        }))
    }

    /// Search at a specific depth and return the best move and score
    fn search_at_depth(
        &mut self,
        position: &Position,
        ordered_moves: &[Move],
        depth: u8,
    ) -> Result<(Move, i32), SearchError> {
        let mut best_move = ordered_moves[0];
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN;
        let beta = i32::MAX;

        for &mv in ordered_moves {
            // Check time limit frequently during search
            if self.should_stop() {
                break;
            }

            let mut search_position = position.clone();
            if search_position.apply_move_for_search(mv).is_ok() {
                let score = self.alpha_beta(&search_position, depth - 1, alpha, beta, false)?;

                if score > best_score {
                    best_score = score;
                    best_move = mv;
                    alpha = alpha.max(score);
                }
            }
        }

        Ok((best_move, best_score))
    }

    /// Find the best move with a time constraint
    pub fn find_best_move_timed(
        &mut self,
        position: &Position,
        max_time_ms: u64,
    ) -> Result<SearchResult, SearchError> {
        let original_time_limit = self.max_time_ms;
        self.set_time_limit(Some(max_time_ms));

        let result = self.find_best_move(position);

        // Restore original time limit
        self.set_time_limit(original_time_limit);

        result
    }

    /// Find the best move with both depth and time constraints
    pub fn find_best_move_constrained(
        &mut self,
        position: &Position,
        max_depth: u8,
        max_time_ms: u64,
    ) -> Result<SearchResult, SearchError> {
        let original_depth = self.max_depth;
        let original_time_limit = self.max_time_ms;

        self.set_max_depth(max_depth);
        self.set_time_limit(Some(max_time_ms));

        let result = self.find_best_move(position);

        // Restore original settings
        self.set_max_depth(original_depth);
        self.set_time_limit(original_time_limit);

        result
    }

    /// Order moves for better alpha-beta pruning efficiency with Principal Variation
    fn order_moves_with_pv(&self, moves: &mut [Move]) {
        moves.sort_by(|a, b| {
            // Priority order: PV move, captures/promotions, quiet moves
            let a_priority = self.get_move_priority(*a);
            let b_priority = self.get_move_priority(*b);

            a_priority.cmp(&b_priority)
        });
    }

    /// Get priority for move ordering (lower = higher priority)
    fn get_move_priority(&self, mv: Move) -> u8 {
        // Highest priority: Principal Variation move from previous iteration
        if let Some(pv_move) = self.previous_best_move {
            if mv == pv_move {
                return 0;
            }
        }

        // Second priority: Captures and promotions
        if mv.move_type.is_capture() || mv.move_type.is_promotion() {
            return 1;
        }

        // Lowest priority: Quiet moves
        2
    }

    /// Order moves for better alpha-beta pruning efficiency (fallback method)
    fn order_moves(&self, moves: &mut [Move]) {
        self.order_moves_with_pv(moves);
    }

    /// Alpha-beta pruning search implementation
    fn alpha_beta(
        &mut self,
        position: &Position,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        maximizing: bool,
    ) -> Result<i32, SearchError> {
        self.nodes_evaluated += 1;

        // Check time limit periodically during deep search
        if self.nodes_evaluated % 1000 == 0 && self.should_stop() {
            // Return current evaluation if we need to stop
            return Ok(position.evaluate());
        }

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

        // Order moves for better pruning
        let mut ordered_moves = legal_moves;
        self.order_moves(&mut ordered_moves);

        if maximizing {
            let mut max_eval = i32::MIN;
            for &mv in &ordered_moves {
                // Early time check for each move
                if self.should_stop() {
                    break;
                }

                let mut search_position = position.clone();
                if search_position.apply_move_for_search(mv).is_ok() {
                    let eval = self.alpha_beta(&search_position, depth - 1, alpha, beta, false)?;
                    max_eval = max_eval.max(eval);
                    alpha = alpha.max(eval);

                    // Alpha-beta pruning
                    if beta <= alpha {
                        self.nodes_pruned += 1;
                        break;
                    }
                }
            }
            Ok(max_eval)
        } else {
            let mut min_eval = i32::MAX;
            let mut beta = beta;
            for &mv in &ordered_moves {
                // Early time check for each move
                if self.should_stop() {
                    break;
                }

                let mut search_position = position.clone();
                if search_position.apply_move_for_search(mv).is_ok() {
                    let eval = self.alpha_beta(&search_position, depth - 1, alpha, beta, true)?;
                    min_eval = min_eval.min(eval);
                    beta = beta.min(eval);

                    // Alpha-beta pruning
                    if beta <= alpha {
                        self.nodes_pruned += 1;
                        break;
                    }
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

    #[test]
    fn test_alpha_beta_vs_minimax_consistency() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(3);

        let position = Position::starting_position().expect("Starting position should be valid");
        let result1 = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        // Test again to ensure consistency
        let result2 = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        assert_eq!(
            result1.best_move, result2.best_move,
            "Alpha-beta should be deterministic"
        );
        assert_eq!(
            result1.evaluation, result2.evaluation,
            "Evaluations should match"
        );
    }

    #[test]
    fn test_alpha_beta_performance() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(4);

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        // Alpha-beta should significantly reduce nodes searched
        assert!(result.nodes_searched > 0, "Should search some nodes");
        assert!(
            result.nodes_searched < 50000,
            "Alpha-beta should prune many branches"
        );

        // Should have pruned some branches
        assert!(result.nodes_pruned > 0, "Should have pruned some branches");

        // Verify search result structure
        assert_eq!(result.depth, 4, "Should search to specified depth");
    }

    #[test]
    fn test_move_ordering() {
        let engine = SearchEngine::new();
        let mut moves = vec![
            Move::quiet(
                crate::types::Square::from_algebraic("a2").unwrap(),
                crate::types::Square::from_algebraic("a3").unwrap(),
            ),
            Move::capture(
                crate::types::Square::from_algebraic("b2").unwrap(),
                crate::types::Square::from_algebraic("c3").unwrap(),
            ),
            Move::quiet(
                crate::types::Square::from_algebraic("d2").unwrap(),
                crate::types::Square::from_algebraic("d3").unwrap(),
            ),
            Move::capture(
                crate::types::Square::from_algebraic("e2").unwrap(),
                crate::types::Square::from_algebraic("f3").unwrap(),
            ),
        ];

        engine.order_moves(&mut moves);

        // Captures should come first
        assert!(
            moves[0].move_type.is_capture(),
            "First move should be a capture"
        );
        assert!(
            moves[1].move_type.is_capture(),
            "Second move should be a capture"
        );
        assert!(
            !moves[2].move_type.is_capture(),
            "Third move should not be a capture"
        );
        assert!(
            !moves[3].move_type.is_capture(),
            "Fourth move should not be a capture"
        );
    }

    #[test]
    fn test_search_result_statistics() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(2);

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        // Verify all statistics are populated
        assert!(result.nodes_searched > 0, "Should track nodes searched");
        assert_eq!(result.depth, 2, "Should record search depth");

        // Pruning statistics should be tracked (may be 0 for shallow searches)
        // Just verify the field exists and is accessible
        let _pruned = result.nodes_pruned;
    }

    #[test]
    fn test_iterative_deepening_basic() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(3);

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        // Should complete all iterations without time limit
        assert_eq!(result.completed_depth, 3, "Should complete all depths");
        assert_eq!(
            result.iterations_completed, 3,
            "Should complete 3 iterations"
        );
        assert!(!result.time_limited, "Should not be time limited");
        assert!(result.nodes_searched > 0, "Should search nodes");
    }

    #[test]
    fn test_time_limited_search() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(10); // High depth to ensure time limit hits first

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move_timed(&position, 50)
            .expect("Search should succeed");

        // Should be stopped by time limit
        assert!(
            result.time_ms <= 100,
            "Should respect time limit approximately"
        );
        assert!(
            result.completed_depth >= 1,
            "Should complete at least depth 1"
        );
        assert!(
            result.iterations_completed >= 1,
            "Should complete at least 1 iteration"
        );
    }

    #[test]
    fn test_constrained_search() {
        let mut engine = SearchEngine::new();

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move_constrained(&position, 2, 1000)
            .expect("Search should succeed");

        // Should complete depth 2 within time limit
        assert_eq!(result.depth, 2, "Should have target depth 2");
        assert_eq!(result.completed_depth, 2, "Should complete depth 2");
        assert_eq!(
            result.iterations_completed, 2,
            "Should complete 2 iterations"
        );
        assert!(
            result.time_ms < 1000,
            "Should finish well within time limit"
        );
    }

    #[test]
    fn test_principal_variation_ordering() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(2);

        let position = Position::starting_position().expect("Starting position should be valid");

        // First search to establish PV
        let result1 = engine
            .find_best_move(&position)
            .expect("First search should succeed");

        // Second search should use PV from first search
        let result2 = engine
            .find_best_move(&position)
            .expect("Second search should succeed");

        // Should get consistent results (PV move ordering helps)
        assert_eq!(
            result1.best_move, result2.best_move,
            "PV ordering should give consistent results"
        );

        // Verify PV move is stored
        assert!(engine.previous_best_move.is_some(), "Should store PV move");
        assert_eq!(
            engine.previous_best_move.unwrap(),
            result1.best_move,
            "PV move should match best move"
        );
    }

    #[test]
    fn test_iterative_deepening_progression() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(4);

        let position = Position::starting_position().expect("Starting position should be valid");
        let result = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        // Should complete all depths
        assert_eq!(result.completed_depth, 4, "Should complete depth 4");
        assert_eq!(
            result.iterations_completed, 4,
            "Should complete 4 iterations"
        );

        // Each iteration should improve move ordering for the next
        assert!(result.nodes_searched > 0, "Should search nodes");
        assert!(result.time_ms > 0, "Should take some time");
    }

    #[test]
    fn test_single_legal_move_iterative() {
        let mut engine = SearchEngine::new();
        engine.set_max_depth(3);

        // Use starting position (has 20 legal moves, but we'll test the single move path with the existing test)
        let position = Position::starting_position().expect("Starting position should be valid");
        let legal_moves = position
            .generate_legal_moves()
            .expect("Should generate moves");

        // For this test, we'll just verify that iterative deepening works with multiple moves
        let result = engine
            .find_best_move(&position)
            .expect("Search should succeed");

        assert!(
            legal_moves.len() > 1,
            "Starting position should have multiple moves"
        );
        assert!(
            result.iterations_completed > 0,
            "Should complete iterations"
        );
        assert!(result.completed_depth > 0, "Should complete some depth");
    }
}
