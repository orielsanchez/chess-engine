// Send me btc:
// bc1q5tz7795uy42g66agth0eaqpe8zc9e304vwrl74

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
    println!("Bitboard: {:x}\n", bitboard);
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

fn south_one(bitboard: u64) -> u64 {
    bitboard >> 8
}

fn north_one(bitboard: u64) -> u64 {
    bitboard << 8
}

const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
const EMPTY_SQUARES: u64 = 0;
const FULL_SQUARES: u64 = 0xffffffffffffffff;
const WHITE_PAWNS: u64 = 0x00ff000000000000;
const BLACK_PAWNS: u64 = 0x000000000000ff00;
const WHITE_KNIGHTS: u64 = 0x4200000000000000;
const BLACK_KNIGHTS: u64 = 0x0000000000000042;
const WHITE_BISHOPS: u64 = 0x2400000000000000;
const BLACK_BISHOPS: u64 = 0x0000000000000024;
const WHITE_ROOKS: u64 = 0x8100000000000000;
const BLACK_ROOKS: u64 = 0x0000000000000081;
const WHITE_QUEENS: u64 = 0x0800000000000000;
const BLACK_QUEENS: u64 = 0x0000000000000008;
const WHITE_KING: u64 = 0x1000000000000000;
const BLACK_KING: u64 = 0x0000000000000010;
const WHITE_PIECES: u64 =
    WHITE_PAWNS | WHITE_KNIGHTS | WHITE_BISHOPS | WHITE_ROOKS | WHITE_QUEENS | WHITE_KING;
const BLACK_PIECES: u64 =
    BLACK_PAWNS | BLACK_KNIGHTS | BLACK_BISHOPS | BLACK_ROOKS | BLACK_QUEENS | BLACK_KING;
const ALL_PIECES: u64 = WHITE_PIECES | BLACK_PIECES;
const EMPTY: u64 = 0;
const RANK_1: u64 = 0x00000000000000ff;
const RANK_2: u64 = 0x000000000000ff00;
const RANK_3: u64 = 0x0000000000ff0000;
const RANK_4: u64 = 0x00000000ff000000;
const RANK_5: u64 = 0x000000ff00000000;
const RANK_6: u64 = 0x0000ff0000000000;
const RANK_7: u64 = 0x00ff000000000000;
const RANK_8: u64 = 0xff00000000000000;
const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_C: u64 = 0x0404040404040404;
const FILE_D: u64 = 0x0808080808080808;
const FILE_E: u64 = 0x1010101010101010;
const FILE_F: u64 = 0x2020202020202020;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

fn east_one(bitboard: u64) -> u64 {
    (bitboard << 1) & NOT_A_FILE
}

fn north_east_one(bitboard: u64) -> u64 {
    (bitboard << 9) & NOT_A_FILE
}

fn south_east_one(bitboard: u64) -> u64 {
    (bitboard >> 7) & NOT_A_FILE
}

fn west_one(bitboard: u64) -> u64 {
    (bitboard >> 1) & NOT_H_FILE
}

fn north_west_one(bitboard: u64) -> u64 {
    (bitboard << 7) & NOT_H_FILE
}

fn south_west_one(bitboard: u64) -> u64 {
    (bitboard >> 9) & NOT_H_FILE
}

// Main
fn main() {
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
