use chess_engine::*;

fn main() {
    println!("Chess Engine v0.1.0");
    println!("===================");

    // Test starting position
    let position = match Position::starting_position() {
        Ok(pos) => pos,
        Err(e) => {
            eprintln!("Error creating starting position: {}", e);
            return;
        }
    };
    println!("Starting position:");
    println!("{}", position.board);

    // Test FEN generation
    let fen = position.to_fen();
    println!("Starting FEN: {}", fen);
    println!("Expected FEN:  {}", STARTING_FEN);
    println!("FEN matches: {}", fen == STARTING_FEN);

    // Test FEN parsing
    println!("\n--- Testing FEN Parsing ---");
    match Position::from_fen(STARTING_FEN) {
        Ok(parsed_position) => {
            println!("✓ Successfully parsed starting FEN");
            println!("Positions match: {}", position == parsed_position);
        }
        Err(e) => println!("✗ Failed to parse FEN: {}", e),
    }

    // Test custom position
    println!("\n--- Testing Custom Position ---");
    let test_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    match Position::from_fen(test_fen) {
        Ok(test_position) => {
            println!("✓ Successfully parsed custom FEN");
            println!("Side to move: {}", test_position.side_to_move);
            println!("En passant: {:?}", test_position.en_passant);
            println!("Board after 1.e4:");
            println!("{}", test_position.board);
        }
        Err(e) => println!("✗ Failed to parse custom FEN: {}", e),
    }

    // Test move generation
    println!("\n--- Move Generation Testing ---");
    match position.generate_legal_moves() {
        Ok(legal_moves) => {
            println!("Legal moves from starting position: {}", legal_moves.len());

            // Show first few moves
            println!("First 10 moves:");
            for (i, mv) in legal_moves.iter().take(10).enumerate() {
                println!("  {}: {}", i + 1, mv);
            }
        }
        Err(e) => println!("✗ Failed to generate legal moves: {}", e),
    }

    // Test position after 1.e4
    if let Ok(e4_position) =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
    {
        match e4_position.generate_legal_moves() {
            Ok(e4_moves) => {
                println!("\nLegal moves after 1.e4: {}", e4_moves.len());

                // Show pawn moves for Black
                println!("Black pawn moves:");
                for mv in e4_moves
                    .iter()
                    .filter(|m| {
                        if let Some(piece) = e4_position.piece_at(m.from) {
                            piece.piece_type == PieceType::Pawn
                        } else {
                            false
                        }
                    })
                    .take(5)
                {
                    println!("  {}", mv);
                }
            }
            Err(e) => println!("✗ Failed to generate legal moves after 1.e4: {}", e),
        }
    }

    // Test basic functionality
    println!("\n--- Basic Functionality ---");
    println!("Side to move: {}", position.side_to_move);
    println!("White material: {}", position.material_count(Color::White));
    println!("Black material: {}", position.material_count(Color::Black));

    let white_king = position.find_king(Color::White);
    let black_king = position.find_king(Color::Black);

    println!("White king at: {:?}", white_king);
    println!("Black king at: {:?}", black_king);

    if let Some(_king_square) = white_king {
        println!("White king in check: {}", position.is_check(Color::White));
    }

    if let Some(_king_square) = black_king {
        println!("Black king in check: {}", position.is_check(Color::Black));
    }

    // Test Search Engine
    println!("\n--- Testing Search Engine ---");
    let mut search_engine = chess_engine::search::SearchEngine::new();
    search_engine.set_max_depth(3); // Shallow search for demo speed

    match search_engine.find_best_move(&position) {
        Ok(result) => {
            println!("✓ Search completed successfully!");
            println!("Best move: {}", result.best_move);
            println!("Evaluation: {} centipawns", result.evaluation);
            println!("Depth: {}", result.depth);
            println!("Nodes searched: {}", result.nodes_searched);
            println!("Time: {}ms", result.time_ms);

            // Demonstrate position evaluation
            let eval = position.evaluate();
            println!("Starting position evaluation: {} centipawns", eval);
        }
        Err(e) => {
            println!("✗ Search failed: {}", e);
        }
    }
}
