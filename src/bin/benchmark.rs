use chess_engine::benchmark::*;
use std::time::Duration;

fn main() {
    println!("Chess Engine Move Generation Benchmark");
    println!("======================================\n");

    // Check for command line arguments
    let args: Vec<String> = std::env::args().collect();
    let (per_position_duration, is_quick) = if args.len() > 1 {
        let arg = &args[1];
        if arg == "--quick" || arg == "-q" {
            (Duration::from_millis(20), true)
        } else {
            let ms: u64 = arg.parse().unwrap_or(200);
            (Duration::from_millis(ms), false)
        }
    } else {
        (Duration::from_millis(200), false)
    };

    if is_quick {
        println!("Running quick benchmark (20ms per position) for CI/testing...\n");
    } else {
        println!(
            "Running benchmark with {}ms per position...\n",
            per_position_duration.as_millis()
        );
        println!("Usage: cargo run --bin benchmark [duration_ms | --quick]\n");
    }

    // Run standard benchmark suite
    let positions = get_standard_benchmark_suite();
    let results = benchmark_multiple_positions(positions, per_position_duration);

    // Print summary table
    println!("Results Summary:");
    println!("================");
    println!(
        "{:<20} {:<8} {:<12} {:<12} {:<8}",
        "Position", "Legal", "Legal/sec", "Pseudo/sec", "Ratio"
    );
    println!("{}", "-".repeat(70));

    for result in &results.results {
        println!(
            "{:<20} {:<8} {:<12.0} {:<12.0} {:<8.2}",
            result.position_name.chars().take(18).collect::<String>(),
            result.legal_move_count,
            result.legal_moves_per_second,
            result.pseudo_legal_moves_per_second,
            result.efficiency_ratio()
        );
    }
    println!("{}", "-".repeat(70));
    println!(
        "{:<20} {:<8} {:<12.0} {:<12.0} {:<8.2}",
        "AVERAGE",
        results.total_legal_moves / results.results.len(),
        results.average_legal_moves_per_second,
        results.average_pseudo_legal_moves_per_second,
        results.average_legal_moves_per_second / results.average_pseudo_legal_moves_per_second
    );

    // Print detailed results only if not in quick mode
    if !is_quick {
        println!("\n{}", results.format_detailed_report());
    }

    // Performance analysis
    println!("\nPerformance Analysis:");
    println!("====================");

    let fastest_legal = results
        .results
        .iter()
        .max_by(|a, b| {
            a.legal_moves_per_second
                .partial_cmp(&b.legal_moves_per_second)
                .unwrap()
        })
        .unwrap();
    let slowest_legal = results
        .results
        .iter()
        .min_by(|a, b| {
            a.legal_moves_per_second
                .partial_cmp(&b.legal_moves_per_second)
                .unwrap()
        })
        .unwrap();

    println!(
        "Fastest position: {} ({:.0} legal moves/sec)",
        fastest_legal.position_name, fastest_legal.legal_moves_per_second
    );
    println!(
        "Slowest position: {} ({:.0} legal moves/sec)",
        slowest_legal.position_name, slowest_legal.legal_moves_per_second
    );

    let speed_ratio = fastest_legal.legal_moves_per_second / slowest_legal.legal_moves_per_second;
    println!("Speed variation: {:.1}x", speed_ratio);

    // Overall rating
    let avg_speed = results.average_legal_moves_per_second;
    let rating = if avg_speed > 50000.0 {
        "Excellent"
    } else if avg_speed > 20000.0 {
        "Good"
    } else if avg_speed > 10000.0 {
        "Fair"
    } else {
        "Needs optimization"
    };

    println!(
        "\nOverall Performance Rating: {} ({:.0} avg legal moves/sec)",
        rating, avg_speed
    );
}
