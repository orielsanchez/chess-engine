use chess_engine::*;

fn main() {
    println!("Chess Engine v0.1.0");
    println!("===================");
    
    // Test starting position
    let position = Position::starting_position();
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
}