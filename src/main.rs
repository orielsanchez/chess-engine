use enum_piece::BoardSquares;

pub mod bitboard;
pub mod enum_piece;

fn print_bitboard(bitboard: u64) {
    println!("");
    for rank in 0..8 {
        for file in 0..8 {
            let square: u64 = rank * 8 + file;
            if file == 0 {
                print!(" {} ", 8 - rank);
            }
            let x;
            match get_bit(&bitboard, square) {
                true => {
                    x = 1;
                }
                false => {
                    x = 0;
                }
            }
            print!(" {}", x);
        }
        println!("");
    }
    println!("\n    a b c d e f g h\n");
    println!("Bitboard: {}\n", bitboard);
}

fn set_bit(bitboard: &mut u64, square: BoardSquares) {
    *bitboard |= 1 << square as u64;
}

fn get_bit(bitboard: &u64, square: u64) -> bool {
    bitboard & (1 << square) != 0
}
fn toggle_bit(bitboard: &mut u64, square: BoardSquares) {
    *bitboard ^= 1 << square as u64;
}

fn reset_bit(bitboard: &mut u64, square: BoardSquares) {
    *bitboard &= !(1 << square as u64);
}
fn main() {
    let mut bitboard: u64 = 0;
    print_bitboard(bitboard);
    set_bit(&mut bitboard, BoardSquares::E2);
    print_bitboard(bitboard);
    toggle_bit(&mut bitboard, BoardSquares::A1);
    print_bitboard(bitboard);
    reset_bit(&mut bitboard, BoardSquares::E2);
    print_bitboard(bitboard);

    // for rank in (1..9).rev() {
    //     println!(
    //         "A{}, B{}, C{}, D{}, E{}, F{}, G{}, H{},",
    //         rank, rank, rank, rank, rank, rank, rank, rank
    //     );
    // }
    // let board = Bitboard { piece_bb: [0; 8] };
    // let white_pawns = board.get_white_pawns();
    // println!("White pawns bitboard: {}", white_pawns);
}
