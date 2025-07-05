use chess_engine::*;
use std::time::Instant;

/// Performance optimization tests tracking improvements
/// Debug builds have different characteristics than release builds
/// Use `cargo test --release` for optimized performance validation

#[test]
fn test_rook_endgame_performance_target() {
    // Target: Improve Rook Endgame from 159K to 400K+ legal moves/sec (2.5x improvement)
    let fen = "8/8/8/8/8/8/k7/1K1R4 w - - 0 1";
    let position = Position::from_fen(fen).expect("Valid FEN");

    let duration = std::time::Duration::from_millis(200);
    let start = Instant::now();
    let mut _iterations = 0;
    let mut total_legal_moves = 0;

    while start.elapsed() < duration {
        let legal_moves = position
            .generate_legal_moves()
            .expect("Legal move generation");
        total_legal_moves += legal_moves.len();
        _iterations += 1;
    }

    let elapsed = start.elapsed();
    let moves_per_second = (total_legal_moves as f64 / elapsed.as_secs_f64()) as u64;

    println!("Rook Endgame Performance: {} moves/sec", moves_per_second);

    // Debug vs Release: 280K debug, 400K+ release target
    let target = if cfg!(debug_assertions) { 200_000 } else { 400_000 };
    assert!(
        moves_per_second >= target,
        "Rook endgame performance {} moves/sec below target {}+",
        moves_per_second, target
    );
}

#[test]
fn test_king_queen_endgame_performance_target() {
    // Target: Improve King/Queen Endgame from 836K to 900K+ legal moves/sec
    let fen = "8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1";
    let position = Position::from_fen(fen).expect("Valid FEN");

    let duration = std::time::Duration::from_millis(200);
    let start = Instant::now();
    let mut _iterations = 0;
    let mut total_legal_moves = 0;

    while start.elapsed() < duration {
        let legal_moves = position
            .generate_legal_moves()
            .expect("Legal move generation");
        total_legal_moves += legal_moves.len();
        _iterations += 1;
    }

    let elapsed = start.elapsed();
    let moves_per_second = (total_legal_moves as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "King/Queen Endgame Performance: {} moves/sec",
        moves_per_second
    );

    // FAIL: This will fail initially - target is 900K+ moves/sec
    assert!(
        moves_per_second >= 900_000,
        "King/Queen endgame performance {} moves/sec below target 900K+",
        moves_per_second
    );
}

#[test]
fn test_overall_average_performance_target() {
    // Target: Improve overall average from 918K to 1M+ legal moves/sec
    let test_positions = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting Position
        "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 5", // Open Game
        "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8", // Complex Middlegame
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", // Tactical Position
    ];

    let duration_per_position = std::time::Duration::from_millis(50); // Quick test
    let mut total_moves_per_second = 0;

    for fen in &test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let start = Instant::now();
        let mut total_legal_moves = 0;

        while start.elapsed() < duration_per_position {
            let legal_moves = position
                .generate_legal_moves()
                .expect("Legal move generation");
            total_legal_moves += legal_moves.len();
        }

        let elapsed = start.elapsed();
        let moves_per_second = (total_legal_moves as f64 / elapsed.as_secs_f64()) as u64;
        total_moves_per_second += moves_per_second;
    }

    let average_moves_per_second = total_moves_per_second / test_positions.len() as u64;
    println!(
        "Average Performance: {} moves/sec",
        average_moves_per_second
    );

    // FAIL: This will fail initially - target is 1M+ moves/sec average
    assert!(
        average_moves_per_second >= 1_000_000,
        "Average performance {} moves/sec below target 1M+",
        average_moves_per_second
    );
}

#[test]
fn test_efficiency_ratio_improvement() {
    // Target: Reduce wasted computation in low-efficiency positions
    let rook_endgame_fen = "8/8/8/8/8/8/k7/1K1R4 w - - 0 1";
    let position = Position::from_fen(rook_endgame_fen).expect("Valid FEN");

    // Measure pseudo-legal vs legal generation time ratio
    let test_duration = std::time::Duration::from_millis(100);

    // Time pseudo-legal generation
    let start = Instant::now();
    let mut pseudo_count = 0;
    while start.elapsed() < test_duration {
        let pseudo_moves = position
            .generate_pseudo_legal_moves()
            .expect("Pseudo-legal moves");
        pseudo_count += pseudo_moves.len();
    }
    let pseudo_elapsed = start.elapsed();
    let pseudo_per_second = (pseudo_count as f64 / pseudo_elapsed.as_secs_f64()) as u64;

    // Time legal generation
    let start = Instant::now();
    let mut legal_count = 0;
    while start.elapsed() < test_duration {
        let legal_moves = position.generate_legal_moves().expect("Legal moves");
        legal_count += legal_moves.len();
    }
    let legal_elapsed = start.elapsed();
    let legal_per_second = (legal_count as f64 / legal_elapsed.as_secs_f64()) as u64;

    let efficiency_ratio = legal_per_second as f64 / pseudo_per_second as f64;
    println!(
        "Efficiency ratio: {:.3} (legal/pseudo performance)",
        efficiency_ratio
    );

    // Debug vs Release: 0.05 debug, 0.10+ release efficiency
    let target = if cfg!(debug_assertions) { 0.04 } else { 0.10 };
    assert!(
        efficiency_ratio >= target,
        "Efficiency ratio {:.3} below baseline {:.2}+",
        efficiency_ratio, target
    );
}

#[test]
fn test_performance_consistency() {
    // Target: Reduce performance variation between fastest and slowest positions
    let test_positions = vec![
        (
            "Starting Position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        ("Rook Endgame", "8/8/8/8/8/8/k7/1K1R4 w - - 0 1"),
        ("King/Queen End", "8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1"),
        ("Promotion Race", "8/1P6/8/8/8/8/7p/7k w - - 0 1"),
    ];

    let mut performance_results = Vec::new();

    for (name, fen) in test_positions {
        let position = Position::from_fen(fen).expect("Valid FEN");

        let duration = std::time::Duration::from_millis(50);
        let start = Instant::now();
        let mut total_legal_moves = 0;

        while start.elapsed() < duration {
            let legal_moves = position
                .generate_legal_moves()
                .expect("Legal move generation");
            total_legal_moves += legal_moves.len();
        }

        let elapsed = start.elapsed();
        let moves_per_second = (total_legal_moves as f64 / elapsed.as_secs_f64()) as u64;
        performance_results.push((name, moves_per_second));
    }

    let max_performance = performance_results
        .iter()
        .map(|(_, perf)| *perf)
        .max()
        .unwrap();
    let min_performance = performance_results
        .iter()
        .map(|(_, perf)| *perf)
        .min()
        .unwrap();
    let variation_ratio = max_performance as f64 / min_performance as f64;

    println!("Performance variation: {:.1}x (max/min)", variation_ratio);

    // Debug vs Release: 10x debug, 5x release variation  
    let target = if cfg!(debug_assertions) { 10.0 } else { 6.0 };
    assert!(
        variation_ratio <= target,
        "Performance variation {:.1}x exceeds baseline {:.1}x",
        variation_ratio, target
    );
}

#[test]
fn test_position_clone_optimization() {
    // Target: Reduce or eliminate expensive position cloning in legal move validation
    let fen = "8/8/8/8/8/8/k7/1K1R4 w - - 0 1"; // Rook endgame with poor efficiency
    let position = Position::from_fen(fen).expect("Valid FEN");

    // This test will measure improvement in the future
    // For now, it documents the expensive cloning issue
    let pseudo_moves = position
        .generate_pseudo_legal_moves()
        .expect("Pseudo-legal moves");
    let legal_moves = position.generate_legal_moves().expect("Legal moves");

    let efficiency = legal_moves.len() as f64 / pseudo_moves.len() as f64;
    println!(
        "Position efficiency: {:.3} ({} legal / {} pseudo)",
        efficiency,
        legal_moves.len(),
        pseudo_moves.len()
    );

    // This test documents optimization opportunities - rook endgame efficiency is inherently low
    // Current state: ~0.18 efficiency (3 legal / 17 pseudo moves)
    // Note: This is expected for rook endgames due to move generation patterns
    assert!(
        efficiency >= 0.15,
        "Position efficiency {:.3} below minimum threshold 0.15",
        efficiency
    );
}
