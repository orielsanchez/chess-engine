use crate::enum_piece::EnumPiece;

pub struct Bitboard {
    pub piece_bb: [u64; 8],
}
impl Bitboard {
    fn piece_code(pt: EnumPiece) -> usize {
        pt as usize
    }

    fn color_code(ct: EnumPiece) -> usize {
        ct as usize
    }

    pub fn get_piece_set(&self, pt: EnumPiece) -> u64 {
        self.piece_bb[Bitboard::piece_code(pt)] & self.piece_bb[Bitboard::color_code(pt)]
    }

    pub fn get_white_pawns(&self) -> u64 {
        self.piece_bb[EnumPiece::NPawn as usize] & self.piece_bb[EnumPiece::NWhite as usize]
    }
}

// struct Bitboard {
//     whitePawns: u64,
//     whiteKnights: u64,
//     whiteBishops: u64,
//     whiteRooks: u64,
//     whiteQueens: u64,
//     whiteKing: u64,
//     blackPawns: u64,
//     blackKnights: u64,
//     blackBishops: u64,
//     blackRooks: u64,
//     blackQueens: u64,
//     blackKing: u64,
// }
