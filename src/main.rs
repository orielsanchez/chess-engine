use bitboard::Bitboard;

pub mod bitboard;
pub mod enum_piece;

fn print_bitboard(bitboard: u64) {
    for i in 0..8 {
        for j in 0..8 {
            let square: u64 = i * 8 + j;
            let x;
            match bitboard & (1 << square) != 0 {
                true => {
                    x = 1;
                }
                false => {
                    x = 0;
                }
            }
            print!("{}", x);
        }
        println!("");
    }
}

fn main() {
    print_bitboard(4);
    // let board = Bitboard { piece_bb: [0; 8] };
    // let white_pawns = board.get_white_pawns();
    // println!("White pawns bitboard: {}", white_pawns);
}
