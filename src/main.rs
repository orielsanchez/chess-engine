use chess_engine::*;

fn main() {
    println!("Chess Engine v0.1.0");
    println!("===================");

    let position = Position::starting_position();
    println!("Starting position:");
    println!("{}", position.board);

    println!("Side to move: {}", position.side_to_move);
    println!("White material: {}", position.material_count(Color::White));
    println!("Black material: {}", position.material_count(Color::Black));

    // Test basic functionality
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
