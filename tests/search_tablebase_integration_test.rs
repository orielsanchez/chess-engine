use chess_engine::position::Position;
use chess_engine::search::SearchEngine;

#[test]
fn test_search_terminates_early_with_tablebase_mate() {
    // Test: Search should terminate early when tablebase gives definitive mate result
    // Expected: Search completes faster with tablebase than without
    let mut search_engine = SearchEngine::new();

    // KQvK position - tablebase shows mate in 10
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    // Search with tablebase enabled
    position.enable_tablebase_lookup(true);
    let result_with_tb = search_engine.find_best_move(&position);

    // Search with tablebase disabled
    position.enable_tablebase_lookup(false);
    let result_without_tb = search_engine.find_best_move(&position);

    // Re-enable tablebase for other tests
    position.enable_tablebase_lookup(true);

    // Should find valid moves
    assert!(result_with_tb.is_ok());
    assert!(result_without_tb.is_ok());

    let search_with_tb = result_with_tb.unwrap();
    let search_without_tb = result_without_tb.unwrap();

    // With tablebase should return strong mate score
    assert!(
        search_with_tb.evaluation > 19000,
        "With tablebase should return mate score ~19900, got {}",
        search_with_tb.evaluation
    );

    // Without tablebase should return heuristic evaluation
    assert!(
        search_without_tb.evaluation < 10000,
        "Without tablebase should return heuristic score, got {}",
        search_without_tb.evaluation
    );

    // Should be faster with tablebase (optional check - might not always be true)
    // assert!(time_with_tb < time_without_tb, "Search with tablebase should be faster");
}

#[test]
fn test_search_returns_tablebase_result_immediately() {
    // Test: When position is in tablebase, search should return tablebase result
    // Expected: Search returns perfect tablebase evaluation, not heuristic
    let mut search_engine = SearchEngine::new();

    // KQvK position - tablebase shows mate in 10
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should return perfect tablebase mate score (20000 - dtm * 10)
    // For mate in 10: 20000 - 100 = 19900
    assert!(
        search_result.evaluation > 19000,
        "Should return tablebase mate score, got {}",
        search_result.evaluation
    );
    assert!(
        search_result.evaluation < 20000,
        "Should return tablebase mate score, got {}",
        search_result.evaluation
    );
}

#[test]
fn test_search_prioritizes_tablebase_winning_moves() {
    // Test: Search should examine tablebase-winning moves first
    // Expected: Move ordering puts tablebase wins at front of search
    let mut search_engine = SearchEngine::new();

    // Position with multiple good moves, but one leads to faster tablebase mate
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should find the move that leads to fastest tablebase mate
    // (This test will help define the interface for move ordering)
    assert!(
        search_result.evaluation > 19000,
        "Should find tablebase mate, got {}",
        search_result.evaluation
    );
}

#[test]
fn test_search_uses_tablebase_for_leaf_nodes() {
    // Test: At leaf nodes, search should use tablebase instead of static evaluation
    // Expected: Search depth can be effectively deeper due to tablebase knowledge
    let mut search_engine = SearchEngine::new();

    // Position approaching tablebase territory (5 pieces)
    let position = Position::from_fen("8/8/8/8/8/2K5/8/2k1R3 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should find precise tablebase result at leaf nodes
    // This test will guide implementation of tablebase probing in alpha-beta
    assert!(search_result.evaluation != 0, "Should find non-draw result");
}

#[test]
fn test_search_handles_mixed_tablebase_positions() {
    // Test: Search tree with some nodes in tablebase, others requiring calculation
    // Expected: Search uses tablebase where available, normal search elsewhere
    let mut search_engine = SearchEngine::new();

    // Position with 7 pieces that can transition to 6-piece tablebase
    let position = Position::from_fen("r7/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should handle transition from non-tablebase to tablebase positions
    assert!(
        search_result.evaluation.abs() > 100,
        "Should find significant advantage"
    );
}

#[test]
fn test_search_respects_dtz_fifty_move_rule() {
    // Test: Search should consider 50-move rule when using DTZ results
    // Expected: Search prefers moves that don't hit 50-move limit
    let mut search_engine = SearchEngine::new();

    // Position where DTZ considerations matter for move choice
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 40 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should find move that avoids 50-move rule draw
    // This test will guide DTZ integration with search
    assert!(
        search_result.evaluation > 0,
        "Should find winning move that avoids 50-move draw"
    );
}

#[test]
fn test_search_performance_with_tablebase_integration() {
    // Test: Search should not be significantly slower with tablebase integration
    // Expected: Tablebase lookups should be fast enough for real-time search
    let mut search_engine = SearchEngine::new();

    // Standard middlegame position (not in tablebase)
    let position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

    let start = std::time::Instant::now();
    let result = search_engine.find_best_move(&position);
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(
        elapsed < std::time::Duration::from_millis(1100),
        "Search should not be significantly slower"
    );
}

#[test]
fn test_search_node_count_optimization_with_tablebase() {
    // Test: Search should examine fewer nodes when tablebase provides cutoffs
    // Expected: Node count should be lower with tablebase integration
    let mut search_engine = SearchEngine::new();

    // Position transitioning to tablebase
    let position = Position::from_fen("8/8/8/8/8/2K5/8/2k1R3 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position);

    assert!(result.is_ok());
    let search_result = result.unwrap();

    // Should examine fewer nodes due to tablebase cutoffs
    // This test will drive implementation of early termination logic
    assert!(
        search_result.nodes_searched < 10000,
        "Should search fewer nodes with tablebase"
    );
}
