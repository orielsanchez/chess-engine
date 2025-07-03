use crate::types::*;
use crate::position::Position;
use crate::moves::*;

impl Position {
    /// Generate all pseudo-legal moves for the current position
    pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        
        for (square, piece) in self.pieces_of_color(self.side_to_move) {
            match piece.piece_type {
                PieceType::Pawn => self.generate_pawn_moves(square, &mut moves),
                PieceType::Knight => self.generate_knight_moves(square, &mut moves),
                PieceType::Bishop => self.generate_bishop_moves(square, &mut moves),
                PieceType::Rook => self.generate_rook_moves(square, &mut moves),
                PieceType::Queen => self.generate_queen_moves(square, &mut moves),
                PieceType::King => self.generate_king_moves(square, &mut moves),
            }
        }
        
        moves
    }
    
    /// Generate all legal moves for the current position
    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let pseudo_legal = self.generate_pseudo_legal_moves();
        let mut legal_moves = Vec::new();
        
        for mv in pseudo_legal {
            if self.is_legal_move(mv) {
                legal_moves.push(mv);
            }
        }
        
        legal_moves
    }
    
    /// Check if a move is legal (doesn't leave king in check)
    pub fn is_legal_move(&self, mv: Move) -> bool {
        let mut position_copy = self.clone();
        position_copy.make_move_unchecked(mv);
        !position_copy.is_check(self.side_to_move)
    }
    
    /// Make a move without validation (for internal use)
    fn make_move_unchecked(&mut self, mv: Move) {
        let moving_piece = self.piece_at(mv.from);
        
        // Handle special moves first
        match mv.move_type {
            MoveType::EnPassant => {
                // Remove the captured pawn
                let captured_pawn_rank = match self.side_to_move {
                    Color::White => mv.to.rank() - 1,
                    Color::Black => mv.to.rank() + 1,
                };
                let captured_square = Square::new(captured_pawn_rank, mv.to.file()).unwrap();
                self.set_piece(captured_square, None);
                
                // Move the pawn
                self.set_piece(mv.from, None);
                self.set_piece(mv.to, moving_piece);
            }
            
            MoveType::CastleKingside => {
                let back_rank = match self.side_to_move {
                    Color::White => 0,
                    Color::Black => 7,
                };
                
                // Move king
                self.set_piece(mv.from, None);
                self.set_piece(mv.to, moving_piece);
                
                // Move rook
                let rook_from = Square::new(back_rank, 7).unwrap();
                let rook_to = Square::new(back_rank, 5).unwrap();
                let rook = self.piece_at(rook_from);
                self.set_piece(rook_from, None);
                self.set_piece(rook_to, rook);
            }
            
            MoveType::CastleQueenside => {
                let back_rank = match self.side_to_move {
                    Color::White => 0,
                    Color::Black => 7,
                };
                
                // Move king
                self.set_piece(mv.from, None);
                self.set_piece(mv.to, moving_piece);
                
                // Move rook
                let rook_from = Square::new(back_rank, 0).unwrap();
                let rook_to = Square::new(back_rank, 3).unwrap();
                let rook = self.piece_at(rook_from);
                self.set_piece(rook_from, None);
                self.set_piece(rook_to, rook);
            }
            
            _ if mv.move_type.is_promotion() => {
                // Handle promotions
                self.set_piece(mv.from, None);
                if let Some(promotion_piece) = mv.move_type.promotion_piece() {
                    let promoted = Piece::new(self.side_to_move, promotion_piece);
                    self.set_piece(mv.to, Some(promoted));
                }
            }
            
            _ => {
                // Normal move
                self.set_piece(mv.from, None);
                self.set_piece(mv.to, moving_piece);
            }
        }
        
        // Update game state
        self.update_castling_rights(&mv);
        self.update_en_passant(&mv);
        self.update_clocks(&mv);
        self.switch_side();
    }
    
    fn update_castling_rights(&mut self, mv: &Move) {
        let piece = self.piece_at(mv.to);
        
        if let Some(p) = piece {
            match p.piece_type {
                PieceType::King => {
                    self.castling_rights.remove_all(p.color);
                }
                PieceType::Rook => {
                    match (p.color, mv.from.file()) {
                        (Color::White, 0) => self.castling_rights.remove_queenside(Color::White),
                        (Color::White, 7) => self.castling_rights.remove_kingside(Color::White),
                        (Color::Black, 0) => self.castling_rights.remove_queenside(Color::Black),
                        (Color::Black, 7) => self.castling_rights.remove_kingside(Color::Black),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Check if rook was captured
        match (mv.to.rank(), mv.to.file()) {
            (0, 0) => self.castling_rights.remove_queenside(Color::White),
            (0, 7) => self.castling_rights.remove_kingside(Color::White),
            (7, 0) => self.castling_rights.remove_queenside(Color::Black),
            (7, 7) => self.castling_rights.remove_kingside(Color::Black),
            _ => {}
        }
    }
    
    fn update_en_passant(&mut self, mv: &Move) {
        self.en_passant = None;
        
        // Check for pawn double push
        if let Some(piece) = self.piece_at(mv.to) {
            if piece.piece_type == PieceType::Pawn {
                let rank_diff = (mv.to.rank() as i8 - mv.from.rank() as i8).abs();
                if rank_diff == 2 {
                    let ep_rank = (mv.from.rank() + mv.to.rank()) / 2;
                    self.en_passant = Square::new(ep_rank, mv.from.file()).ok();
                }
            }
        }
    }
    
    fn update_clocks(&mut self, mv: &Move) {
        let is_capture = mv.move_type.is_capture();
        let is_pawn_move = self.piece_at(mv.to)
            .map(|p| p.piece_type == PieceType::Pawn)
            .unwrap_or(false);
        
        if is_capture || is_pawn_move {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
    }
    
    /// Generate pawn moves from a given square
    fn generate_pawn_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        let direction = match color {
            Color::White => 1i8,
            Color::Black => -1i8,
        };
        
        let start_rank = match color {
            Color::White => 1,
            Color::Black => 6,
        };
        
        let promotion_rank = match color {
            Color::White => 7,
            Color::Black => 0,
        };
        
        let from_rank = from.rank() as i8;
        let from_file = from.file();
        
        // Forward moves
        let one_forward_rank = from_rank + direction;
        if (0..8).contains(&one_forward_rank) {
            let one_forward = Square::new(one_forward_rank as u8, from_file).unwrap();
            
            if self.is_empty(one_forward) {
                if one_forward_rank as u8 == promotion_rank {
                    // Promotions
                    moves.push(Move::promotion(from, one_forward, PieceType::Queen, false));
                    moves.push(Move::promotion(from, one_forward, PieceType::Rook, false));
                    moves.push(Move::promotion(from, one_forward, PieceType::Bishop, false));
                    moves.push(Move::promotion(from, one_forward, PieceType::Knight, false));
                } else {
                    moves.push(Move::quiet(from, one_forward));
                }
                
                // Double push from starting position
                if from.rank() == start_rank {
                    let two_forward_rank = from_rank + 2 * direction;
                    if (0..8).contains(&two_forward_rank) {
                        let two_forward = Square::new(two_forward_rank as u8, from_file).unwrap();
                        if self.is_empty(two_forward) {
                            moves.push(Move::quiet(from, two_forward));
                        }
                    }
                }
            }
        }
        
        // Captures
        for &file_offset in &[-1i8, 1i8] {
            let capture_file = from_file as i8 + file_offset;
            let capture_rank = from_rank + direction;
            
            if (0..8).contains(&capture_file) && (0..8).contains(&capture_rank) {
                let capture_square = Square::new(capture_rank as u8, capture_file as u8).unwrap();
                
                if self.is_occupied_by(capture_square, color.opposite()) {
                    if capture_rank as u8 == promotion_rank {
                        // Promotion captures
                        moves.push(Move::promotion(from, capture_square, PieceType::Queen, true));
                        moves.push(Move::promotion(from, capture_square, PieceType::Rook, true));
                        moves.push(Move::promotion(from, capture_square, PieceType::Bishop, true));
                        moves.push(Move::promotion(from, capture_square, PieceType::Knight, true));
                    } else {
                        moves.push(Move::capture(from, capture_square));
                    }
                }
                
                // En passant
                if let Some(ep_square) = self.en_passant {
                    if capture_square == ep_square {
                        moves.push(Move::en_passant(from, capture_square));
                    }
                }
            }
        }
    }
    
    /// Generate knight moves from a given square
    fn generate_knight_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        
        let knight_offsets = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1)
        ];
        
        for (rank_offset, file_offset) in knight_offsets.iter() {
            let to_rank = from_rank + rank_offset;
            let to_file = from_file + file_offset;
            
            if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                let to_square = Square::new(to_rank as u8, to_file as u8).unwrap();
                
                if self.is_empty(to_square) {
                    moves.push(Move::quiet(from, to_square));
                } else if self.is_occupied_by(to_square, self.side_to_move.opposite()) {
                    moves.push(Move::capture(from, to_square));
                }
            }
        }
    }
    
    /// Generate bishop moves from a given square
    fn generate_bishop_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        self.generate_sliding_moves(from, &directions, moves);
    }
    
    /// Generate rook moves from a given square
    fn generate_rook_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        self.generate_sliding_moves(from, &directions, moves);
    }
    
    /// Generate queen moves from a given square
    fn generate_queen_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        self.generate_sliding_moves(from, &directions, moves);
    }
    
    /// Generate sliding piece moves (bishop, rook, queen)
    fn generate_sliding_moves(&self, from: Square, directions: &[(i8, i8)], moves: &mut Vec<Move>) {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        
        for (rank_dir, file_dir) in directions.iter() {
            let mut rank = from_rank;
            let mut file = from_file;
            
            loop {
                rank += rank_dir;
                file += file_dir;
                
                if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                    break;
                }
                
                let to_square = Square::new(rank as u8, file as u8).unwrap();
                
                if self.is_empty(to_square) {
                    moves.push(Move::quiet(from, to_square));
                } else if self.is_occupied_by(to_square, self.side_to_move.opposite()) {
                    moves.push(Move::capture(from, to_square));
                    break; // Can't continue past capture
                } else {
                    break; // Own piece blocks further movement
                }
            }
        }
    }
    
    /// Generate king moves from a given square
    fn generate_king_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        
        let king_offsets = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        
        // Normal king moves
        for (rank_offset, file_offset) in king_offsets.iter() {
            let to_rank = from_rank + rank_offset;
            let to_file = from_file + file_offset;
            
            if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                let to_square = Square::new(to_rank as u8, to_file as u8).unwrap();
                
                if self.is_empty(to_square) {
                    moves.push(Move::quiet(from, to_square));
                } else if self.is_occupied_by(to_square, self.side_to_move.opposite()) {
                    moves.push(Move::capture(from, to_square));
                }
            }
        }
        
        // Castling
        self.generate_castling_moves(from, moves);
    }
    
    /// Generate castling moves
    fn generate_castling_moves(&self, from: Square, moves: &mut Vec<Move>) {
        let color = self.side_to_move;
        
        // Can't castle if in check
        if self.is_check(color) {
            return;
        }
        
        let back_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };
        
        // Kingside castling
        if self.castling_rights.can_castle_kingside(color) {
            let king_start = Square::new(back_rank, 4).unwrap();
            let king_end = Square::new(back_rank, 6).unwrap();
            let rook_square = Square::new(back_rank, 7).unwrap();
            
            if from == king_start &&
               self.is_empty(Square::new(back_rank, 5).unwrap()) &&
               self.is_empty(king_end) &&
               self.is_occupied_by(rook_square, color) &&
               !self.is_square_attacked(Square::new(back_rank, 5).unwrap(), color.opposite()) &&
               !self.is_square_attacked(king_end, color.opposite()) {
                moves.push(Move::castle_kingside(from, king_end));
            }
        }
        
        // Queenside castling
        if self.castling_rights.can_castle_queenside(color) {
            let king_start = Square::new(back_rank, 4).unwrap();
            let king_end = Square::new(back_rank, 2).unwrap();
            let rook_square = Square::new(back_rank, 0).unwrap();
            
            if from == king_start &&
               self.is_empty(Square::new(back_rank, 1).unwrap()) &&
               self.is_empty(king_end) &&
               self.is_empty(Square::new(back_rank, 3).unwrap()) &&
               self.is_occupied_by(rook_square, color) &&
               !self.is_square_attacked(Square::new(back_rank, 3).unwrap(), color.opposite()) &&
               !self.is_square_attacked(king_end, color.opposite()) {
                moves.push(Move::castle_queenside(from, king_end));
            }
        }
    }
}