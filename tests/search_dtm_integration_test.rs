use chess_engine::position::Position;
use chess_engine::search::SearchEngine;

// 🔴 RED PHASE: Distance-to-Mate Integration Tests
// These tests should FAIL initially - they define the behavior we want to implement

#[test]
fn test_search_engine_integrates_with_dtm_analyzer() {
    // Test: SearchEngine should have a DistanceToMateAnalyzer instance
    // Expected: SearchEngine exposes DTM analysis functionality
    let search_engine = SearchEngine::new();

    // This should fail - method doesn't exist yet
    let dtm_analyzer = search_engine.distance_to_mate_analyzer();
    assert!(
        dtm_analyzer.is_some(),
        "SearchEngine should have DTM analyzer"
    );

    let analyzer = dtm_analyzer.unwrap();
    assert!(analyzer.is_ready(), "DTM analyzer should be ready");
}

#[test]
fn test_search_result_includes_dtm_information() {
    // Test: SearchResult should include DTM analysis when available
    // Expected: SearchResult has fields for DTM data and mate sequences
    let mut search_engine = SearchEngine::new();

    // KQvK position - mate in 10
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // These fields should fail - they don't exist yet in SearchResult
    assert!(
        result.distance_to_mate().is_some(),
        "Should include DTM information"
    );
    assert!(
        result.mate_sequence().is_some(),
        "Should include mate sequence"
    );
    assert_eq!(
        result.distance_to_mate().unwrap(),
        10,
        "Should have correct DTM"
    );

    let mate_sequence = result.mate_sequence().unwrap();
    assert!(mate_sequence.is_forced_mate(), "Should be forced mate");
    assert_eq!(mate_sequence.length(), 10, "Should have 10-move sequence");
}

#[test]
fn test_search_uses_dtm_optimal_move_ordering() {
    // Test: Search should order moves using DTM analysis for optimal play
    // Expected: DTM-optimal moves are examined first in search
    let mut search_engine = SearchEngine::new();

    // Position with multiple moves, but one leads to fastest mate
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // This should fail - SearchResult doesn't have DTM move ordering info yet
    assert!(
        result.used_dtm_ordering(),
        "Should use DTM for move ordering"
    );
    assert!(
        result.nodes_searched < 100,
        "DTM ordering should reduce search nodes"
    );

    // Should find the DTM-optimal move
    let best_move = result.best_move;
    assert!(
        result.is_dtm_optimal_move(best_move),
        "Should find DTM-optimal move"
    );
}

#[test]
fn test_search_provides_mate_visualization_data() {
    // Test: Search should provide data for mate path visualization
    // Expected: SearchResult includes visualization-ready mate path
    let mut search_engine = SearchEngine::new();

    // KQvK endgame
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // These methods should fail - don't exist yet
    let visualization = result.generate_mate_visualization().unwrap();
    assert!(
        visualization.contains("Mate in"),
        "Should contain mate description"
    );
    assert!(visualization.contains("moves:"), "Should list moves");

    let study_session = result.create_study_session().unwrap();
    assert!(
        study_session.has_next_move(),
        "Should provide study session"
    );
}

#[test]
fn test_search_handles_dtm_analysis_errors_gracefully() {
    // Test: Search should gracefully handle DTM analysis failures
    // Expected: Falls back to normal search when DTM analysis fails
    let mut search_engine = SearchEngine::new();

    // Position not in tablebase (too many pieces)
    let position =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // Should succeed even when DTM analysis fails
    assert!(result.best_move.is_valid(), "Should find valid move");
    assert!(
        result.distance_to_mate().is_none(),
        "No DTM for non-tablebase position"
    );
    assert!(
        result.mate_sequence().is_none(),
        "No mate sequence for non-tablebase"
    );

    // Should indicate DTM analysis was attempted but failed
    assert_eq!(result.dtm_analysis_status(), "not_in_tablebase");
}

#[test]
fn test_search_with_dtm_analyzer_performance() {
    // Test: DTM integration should not significantly impact search performance
    // Expected: Search with DTM analysis completes within reasonable time
    let mut search_engine = SearchEngine::new();
    search_engine.set_max_depth(4);

    // Tablebase position
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let start = std::time::Instant::now();
    let result = search_engine.find_best_move(&position).unwrap();
    let elapsed = start.elapsed();

    // Should complete quickly with DTM analysis
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "DTM analysis should complete quickly"
    );
    assert!(result.distance_to_mate().is_some(), "Should provide DTM");
    assert!(result.time_ms < 500, "Should be fast with DTM integration");
}

#[test]
fn test_search_dtm_integration_with_time_constraints() {
    // Test: DTM analysis should work within search time limits
    // Expected: DTM analysis respects search time constraints
    let mut search_engine = SearchEngine::new();

    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    // Short time limit
    let result = search_engine.find_best_move_timed(&position, 100).unwrap();

    assert!(result.time_ms <= 150, "Should respect time limit with DTM");
    assert!(
        result.distance_to_mate().is_some(),
        "Should still provide DTM in time limit"
    );
}

#[test]
fn test_search_dtm_integration_with_fifty_move_rule() {
    // Test: DTM analysis should consider 50-move rule implications
    // Expected: Search considers DTZ data for 50-move rule compliance
    let mut search_engine = SearchEngine::new();

    // Position with high halfmove clock
    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 45 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // Should consider 50-move rule in DTM analysis
    assert!(
        result.considers_fifty_move_rule(),
        "Should consider 50-move rule"
    );
    assert!(
        result.distance_to_mate().is_some(),
        "Should provide DTM with 50-move consideration"
    );

    let dtm_result = result.dtm_result().unwrap();
    assert!(
        dtm_result.considers_fifty_move_rule(),
        "DTM result should consider 50-move rule"
    );
}

#[test]
fn test_search_dtm_analyzer_caching() {
    // Test: DTM analysis results should be cached for performance
    // Expected: Repeated analysis of same position uses cached results
    let mut search_engine = SearchEngine::new();

    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    // First search
    let start1 = std::time::Instant::now();
    let result1 = search_engine.find_best_move(&position).unwrap();
    let time1 = start1.elapsed();

    // Second search (should use cache)
    let start2 = std::time::Instant::now();
    let result2 = search_engine.find_best_move(&position).unwrap();
    let time2 = start2.elapsed();

    // Results should be identical
    assert_eq!(result1.distance_to_mate(), result2.distance_to_mate());
    assert_eq!(result1.best_move, result2.best_move);

    // Second search should be faster due to caching
    assert!(time2 <= time1, "Cached DTM analysis should be faster");
}

#[test]
fn test_search_dtm_integration_with_study_mode() {
    // Test: Search should provide study-friendly DTM analysis
    // Expected: Search result enables interactive study of mate sequences
    let mut search_engine = SearchEngine::new();

    let position = Position::from_fen("8/8/8/8/8/8/1K6/1Q1k4 w - - 0 1").unwrap();

    let result = search_engine.find_best_move(&position).unwrap();

    // Should enable study mode features
    let study_session = result.create_study_session().unwrap();
    assert!(study_session.has_next_move(), "Should have moves to study");

    let visualization = result.generate_mate_visualization().unwrap();
    assert!(
        visualization.len() > 50,
        "Should provide detailed visualization"
    );

    // Should allow step-by-step analysis
    assert!(
        result.supports_interactive_analysis(),
        "Should support interactive analysis"
    );
}
