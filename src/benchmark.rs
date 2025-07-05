use crate::position::Position;
use std::time::{Duration, Instant};

/// Structure to hold benchmark results for move generation
#[derive(Debug, Clone)]
pub struct MoveGenBenchmarkResult {
    pub position_name: String,
    pub position_fen: String,
    pub pseudo_legal_moves_per_second: f64,
    pub legal_moves_per_second: f64,
    pub pseudo_legal_move_count: usize,
    pub legal_move_count: usize,
    pub pseudo_legal_time_ns: u64,
    pub legal_time_ns: u64,
}

/// Structure to hold comprehensive benchmark results across multiple positions
#[derive(Debug, Clone)]
pub struct ComprehensiveBenchmarkResults {
    pub results: Vec<MoveGenBenchmarkResult>,
    pub total_pseudo_legal_moves: usize,
    pub total_legal_moves: usize,
    pub average_pseudo_legal_moves_per_second: f64,
    pub average_legal_moves_per_second: f64,
    pub benchmark_duration: Duration,
}

impl MoveGenBenchmarkResult {
    pub fn efficiency_ratio(&self) -> f64 {
        if self.pseudo_legal_moves_per_second == 0.0 {
            0.0
        } else {
            self.legal_moves_per_second / self.pseudo_legal_moves_per_second
        }
    }

    pub fn format_detailed(&self) -> String {
        format!(
            "Position: {}\n\
             FEN: {}\n\
             Legal moves: {} ({:.0} moves/sec)\n\
             Pseudo-legal moves: {} ({:.0} moves/sec)\n\
             Efficiency ratio: {:.2}\n\
             Timing: pseudo-legal={}μs, legal={}μs",
            self.position_name,
            self.position_fen,
            self.legal_move_count,
            self.legal_moves_per_second,
            self.pseudo_legal_move_count,
            self.pseudo_legal_moves_per_second,
            self.efficiency_ratio(),
            self.pseudo_legal_time_ns / 1000,
            self.legal_time_ns / 1000
        )
    }
}

impl ComprehensiveBenchmarkResults {
    pub fn format_summary(&self) -> String {
        format!(
            "Move Generation Benchmark Summary:\n\
             - Positions tested: {}\n\
             - Total pseudo-legal moves: {}\n\
             - Total legal moves: {}\n\
             - Average pseudo-legal moves/sec: {:.0}\n\
             - Average legal moves/sec: {:.0}\n\
             - Total benchmark time: {:.2}s",
            self.results.len(),
            self.total_pseudo_legal_moves,
            self.total_legal_moves,
            self.average_pseudo_legal_moves_per_second,
            self.average_legal_moves_per_second,
            self.benchmark_duration.as_secs_f64()
        )
    }

    pub fn format_detailed_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&self.format_summary());
        report.push_str("\n\nDetailed Results:\n");
        report.push_str(&"=".repeat(80));
        report.push('\n');

        for (i, result) in self.results.iter().enumerate() {
            report.push_str(&format!("\n{}. {}\n", i + 1, result.format_detailed()));
            report.push_str(&"-".repeat(60));
            report.push('\n');
        }

        report
    }
}

/// Enhanced benchmark function with better accuracy and configuration
pub fn benchmark_position(
    name: &str,
    position: &Position,
    duration: Duration,
) -> MoveGenBenchmarkResult {
    let position_fen = position.to_fen();

    // Ensure minimum iterations for meaningful results
    let min_iterations = 100;

    // Benchmark pseudo-legal move generation
    let start_time = Instant::now();
    let mut pseudo_legal_count = 0;
    let mut iterations = 0;

    // Run for specified duration or minimum iterations, whichever is longer
    while start_time.elapsed() < duration || iterations < min_iterations {
        let moves = position.generate_pseudo_legal_moves().unwrap_or_default();
        pseudo_legal_count = moves.len(); // Store the actual count from last iteration
        iterations += 1;

        // Break if we've exceeded a reasonable time limit (avoid infinite loops)
        if start_time.elapsed() > Duration::from_secs(1) {
            break;
        }
    }
    let pseudo_legal_time = start_time.elapsed();

    // Benchmark legal move generation
    let start_time = Instant::now();
    let mut legal_count = 0;
    let mut legal_iterations = 0;

    // Run for specified duration or minimum iterations, whichever is longer
    while start_time.elapsed() < duration || legal_iterations < min_iterations {
        let moves = position.generate_legal_moves().unwrap_or_default();
        legal_count = moves.len(); // Store the actual count from last iteration
        legal_iterations += 1;

        // Break if we've exceeded a reasonable time limit (avoid infinite loops)
        if start_time.elapsed() > Duration::from_secs(1) {
            break;
        }
    }
    let legal_time = start_time.elapsed();

    // Calculate moves per second
    let pseudo_legal_moves_per_second = if pseudo_legal_time.as_secs_f64() > 0.0 {
        (iterations as f64 * pseudo_legal_count as f64) / pseudo_legal_time.as_secs_f64()
    } else {
        0.0
    };

    let legal_moves_per_second = if legal_time.as_secs_f64() > 0.0 {
        (legal_iterations as f64 * legal_count as f64) / legal_time.as_secs_f64()
    } else {
        0.0
    };

    MoveGenBenchmarkResult {
        position_name: name.to_string(),
        position_fen,
        pseudo_legal_moves_per_second,
        legal_moves_per_second,
        pseudo_legal_move_count: pseudo_legal_count,
        legal_move_count: legal_count,
        pseudo_legal_time_ns: pseudo_legal_time.as_nanos() as u64,
        legal_time_ns: legal_time.as_nanos() as u64,
    }
}

pub fn benchmark_multiple_positions(
    positions: Vec<(&str, &str)>,
    per_position_duration: Duration,
) -> ComprehensiveBenchmarkResults {
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut total_pseudo_legal_moves = 0;
    let mut total_legal_moves = 0;
    let mut total_pseudo_legal_moves_per_second = 0.0;
    let mut total_legal_moves_per_second = 0.0;

    for (name, fen) in positions {
        let position = Position::from_fen(fen).expect("Valid FEN for benchmark");
        let result = benchmark_position(name, &position, per_position_duration);

        total_pseudo_legal_moves += result.pseudo_legal_move_count;
        total_legal_moves += result.legal_move_count;
        total_pseudo_legal_moves_per_second += result.pseudo_legal_moves_per_second;
        total_legal_moves_per_second += result.legal_moves_per_second;

        results.push(result);
    }

    let average_pseudo_legal_moves_per_second = if !results.is_empty() {
        total_pseudo_legal_moves_per_second / results.len() as f64
    } else {
        0.0
    };

    let average_legal_moves_per_second = if !results.is_empty() {
        total_legal_moves_per_second / results.len() as f64
    } else {
        0.0
    };

    ComprehensiveBenchmarkResults {
        results,
        total_pseudo_legal_moves,
        total_legal_moves,
        average_pseudo_legal_moves_per_second,
        average_legal_moves_per_second,
        benchmark_duration: start_time.elapsed(),
    }
}

/// Standard benchmark suite with representative positions
pub fn get_standard_benchmark_suite() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Starting Position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "Open Game",
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 4 5",
        ),
        (
            "Complex Middlegame",
            "r2q1rk1/ppp2ppp/2n1bn2/2bpp3/3PP3/2P2N2/PP1N1PPP/R1BQKB1R w KQ - 0 8",
        ),
        (
            "Tactical Position",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
        ("King and Queen Endgame", "8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1"),
        ("Rook Endgame", "8/8/8/8/8/8/k7/1K1R4 w - - 0 1"),
        ("Pawn Endgame", "8/3p4/8/2P5/8/8/5PPP/6K1 w - - 0 1"),
        ("Promotion Race", "8/1P6/8/8/8/8/7p/7k w - - 0 1"),
    ]
}
