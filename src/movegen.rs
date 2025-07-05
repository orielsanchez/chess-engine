use crate::moves::*;
use crate::position::Position;
use crate::types::*;

impl Position {
    /// Generate all pseudo-legal moves for the current position
    pub fn generate_pseudo_legal_moves(&self) -> Result<Vec<Move>, MoveGenError> {
        let mut moves = Vec::new();

        for (square, piece) in self.pieces_of_color(self.side_to_move) {
            match piece.piece_type {
                PieceType::Pawn => self.generate_pawn_moves(square, &mut moves)?,
                PieceType::Knight => self.generate_knight_moves(square, &mut moves)?,
                PieceType::Bishop => self.generate_bishop_moves(square, &mut moves)?,
                PieceType::Rook => self.generate_rook_moves(square, &mut moves)?,
                PieceType::Queen => self.generate_queen_moves(square, &mut moves)?,
                PieceType::King => self.generate_king_moves(square, &mut moves)?,
            }
        }

        Ok(moves)
    }

    /// Generate pin-aware pseudo-legal moves (optimization for better efficiency)
    pub fn generate_pin_aware_moves(&self) -> Result<Vec<Move>, MoveGenError> {
        let mut moves = Vec::new();
        let pinned_pieces = self.get_pinned_pieces(self.side_to_move)?;

        for (square, piece) in self.pieces_of_color(self.side_to_move) {
            // Check if this piece is pinned
            let pin_direction = pinned_pieces.get(&square);

            match piece.piece_type {
                PieceType::Pawn => {
                    self.generate_pin_aware_pawn_moves(square, pin_direction, &mut moves)?
                }
                PieceType::Knight => {
                    // Knights can't move when pinned (except capturing the pinner)
                    if pin_direction.is_none() {
                        self.generate_knight_moves(square, &mut moves)?;
                    }
                }
                PieceType::Bishop => {
                    self.generate_pin_aware_bishop_moves(square, pin_direction, &mut moves)?
                }
                PieceType::Rook => {
                    self.generate_pin_aware_rook_moves(square, pin_direction, &mut moves)?
                }
                PieceType::Queen => {
                    self.generate_pin_aware_queen_moves(square, pin_direction, &mut moves)?
                }
                PieceType::King => self.generate_king_moves(square, &mut moves)?, // King moves are always checked separately
            }
        }

        Ok(moves)
    }

    /// Generate all legal moves for the current position
    pub fn generate_legal_moves(&self) -> Result<Vec<Move>, MoveGenError> {
        // Use optimized legal move generation for better efficiency
        self.generate_legal_moves_optimized()
    }

    /// Generate legal moves with maximum optimization
    fn generate_legal_moves_optimized(&self) -> Result<Vec<Move>, MoveGenError> {
        let mut legal_moves = Vec::new();

        // Check if we're in check - this affects move generation strategy
        let in_check = self.is_check(self.side_to_move);

        if in_check {
            // In check: only generate moves that get out of check
            self.generate_check_evasion_moves(&mut legal_moves)?;
        } else {
            // Not in check: use pin-aware generation with optimized validation
            let pin_aware_moves = self.generate_pin_aware_moves()?;

            for mv in pin_aware_moves {
                // For pin-aware moves, we can use faster validation
                if self.is_legal_move_fast(mv)? {
                    legal_moves.push(mv);
                }
            }
        }

        Ok(legal_moves)
    }

    /// Fast legal move validation for pin-aware moves
    fn is_legal_move_fast(&self, mv: Move) -> Result<bool, MoveGenError> {
        // For most pin-aware moves, we can skip the expensive clone+check
        // Only need full validation for king moves and special cases

        let moving_piece = self.piece_at(mv.from);
        if let Some(piece) = moving_piece {
            match piece.piece_type {
                PieceType::King => {
                    // King moves always need full validation
                    self.is_legal_move(mv)
                }
                _ => {
                    // For non-king moves from pin-aware generation,
                    // check if the destination square is attacked
                    if mv.move_type == MoveType::EnPassant {
                        // En passant needs special handling
                        self.is_legal_move(mv)
                    } else {
                        // Most pin-aware moves are legal
                        Ok(true)
                    }
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Generate moves that get out of check
    fn generate_check_evasion_moves(&self, moves: &mut Vec<Move>) -> Result<(), MoveGenError> {
        // When in check, only three types of moves are legal:
        // 1. King moves to safe squares
        // 2. Block the check (if single check)
        // 3. Capture the checking piece (if single check)

        let king_square = self
            .find_king(self.side_to_move)
            .ok_or(MoveGenError::InvalidSquare("King not found"))?;

        // Generate all king moves and filter for safe squares
        let mut king_moves = Vec::new();
        self.generate_king_moves(king_square, &mut king_moves)?;

        for king_move in king_moves {
            if self.is_legal_move(king_move)? {
                moves.push(king_move);
            }
        }

        // For single check, also try blocking/capturing
        let attackers = self.get_attackers_of_square(king_square, self.side_to_move.opposite());
        if attackers.len() == 1 {
            // Single check - can block or capture
            let attacker_square = attackers[0];

            // Try to capture the checking piece
            let defenders = self.get_attackers_of_square(attacker_square, self.side_to_move);
            for defender_square in defenders {
                if defender_square != king_square {
                    // Non-king pieces can capture the checker
                    let capture_move = Move::capture(defender_square, attacker_square);
                    if self.is_legal_move(capture_move)? {
                        moves.push(capture_move);
                    }
                }
            }

            // Try to block the check (only for sliding pieces)
            if let Some(attacker_piece) = self.piece_at(attacker_square) {
                if matches!(
                    attacker_piece.piece_type,
                    PieceType::Queen | PieceType::Rook | PieceType::Bishop
                ) {
                    let blocking_squares =
                        self.get_squares_between(attacker_square, king_square)?;
                    for blocking_square in blocking_squares {
                        let blockers =
                            self.get_attackers_of_square(blocking_square, self.side_to_move);
                        for blocker_square in blockers {
                            if blocker_square != king_square {
                                let block_move = Move::quiet(blocker_square, blocking_square);
                                if self.is_legal_move(block_move)? {
                                    moves.push(block_move);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all pieces of the given color that attack a square
    fn get_attackers_of_square(&self, square: Square, by_color: Color) -> Vec<Square> {
        let mut attackers = Vec::new();

        for (piece_square, piece) in self.pieces_of_color(by_color) {
            if self.piece_attacks_square(piece_square, piece, square) {
                attackers.push(piece_square);
            }
        }

        attackers
    }

    /// Check if a piece at a given square attacks the target square
    fn piece_attacks_square(&self, from: Square, piece: Piece, target: Square) -> bool {
        match piece.piece_type {
            PieceType::Pawn => self.pawn_attacks_square(from, piece.color, target),
            PieceType::Knight => self.knight_attacks_square(from, target),
            PieceType::Bishop => self.bishop_attacks_square(from, target),
            PieceType::Rook => self.rook_attacks_square(from, target),
            PieceType::Queen => {
                self.bishop_attacks_square(from, target) || self.rook_attacks_square(from, target)
            }
            PieceType::King => self.king_attacks_square(from, target),
        }
    }

    /// Check if pawn attacks target square
    fn pawn_attacks_square(&self, from: Square, color: Color, target: Square) -> bool {
        let direction = match color {
            Color::White => 1i8,
            Color::Black => -1i8,
        };

        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let target_rank = target.rank() as i8;
        let target_file = target.file() as i8;

        target_rank == from_rank + direction
            && (target_file == from_file - 1 || target_file == from_file + 1)
    }

    /// Check if knight attacks target square
    fn knight_attacks_square(&self, from: Square, target: Square) -> bool {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let target_rank = target.rank() as i8;
        let target_file = target.file() as i8;

        let rank_diff = (target_rank - from_rank).abs();
        let file_diff = (target_file - from_file).abs();

        (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
    }

    /// Check if bishop attacks target square
    fn bishop_attacks_square(&self, from: Square, target: Square) -> bool {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let target_rank = target.rank() as i8;
        let target_file = target.file() as i8;

        let rank_diff = (target_rank - from_rank).abs();
        let file_diff = (target_file - from_file).abs();

        if rank_diff != file_diff {
            return false; // Not on diagonal
        }

        // Check if path is clear
        let rank_step = if target_rank > from_rank { 1 } else { -1 };
        let file_step = if target_file > from_file { 1 } else { -1 };

        let mut current_rank = from_rank + rank_step;
        let mut current_file = from_file + file_step;

        while current_rank != target_rank {
            if let Ok(square) = Square::new(current_rank as u8, current_file as u8) {
                if !self.is_empty(square) {
                    return false; // Path blocked
                }
            }
            current_rank += rank_step;
            current_file += file_step;
        }

        true
    }

    /// Check if rook attacks target square
    fn rook_attacks_square(&self, from: Square, target: Square) -> bool {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let target_rank = target.rank() as i8;
        let target_file = target.file() as i8;

        if from_rank != target_rank && from_file != target_file {
            return false; // Not on same rank or file
        }

        // Check if path is clear
        let rank_step = if target_rank > from_rank {
            1
        } else if target_rank < from_rank {
            -1
        } else {
            0
        };
        let file_step = if target_file > from_file {
            1
        } else if target_file < from_file {
            -1
        } else {
            0
        };

        let mut current_rank = from_rank + rank_step;
        let mut current_file = from_file + file_step;

        while current_rank != target_rank || current_file != target_file {
            if let Ok(square) = Square::new(current_rank as u8, current_file as u8) {
                if !self.is_empty(square) {
                    return false; // Path blocked
                }
            }
            current_rank += rank_step;
            current_file += file_step;
        }

        true
    }

    /// Check if king attacks target square
    fn king_attacks_square(&self, from: Square, target: Square) -> bool {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let target_rank = target.rank() as i8;
        let target_file = target.file() as i8;

        let rank_diff = (target_rank - from_rank).abs();
        let file_diff = (target_file - from_file).abs();

        rank_diff <= 1 && file_diff <= 1 && (rank_diff + file_diff > 0)
    }

    /// Get squares between two squares (for blocking checks)
    fn get_squares_between(&self, from: Square, to: Square) -> Result<Vec<Square>, MoveGenError> {
        let mut squares = Vec::new();

        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;
        let to_rank = to.rank() as i8;
        let to_file = to.file() as i8;

        let rank_diff = to_rank - from_rank;
        let file_diff = to_file - from_file;

        // Must be on same rank, file, or diagonal
        if rank_diff != 0 && file_diff != 0 && rank_diff.abs() != file_diff.abs() {
            return Ok(squares); // Not a valid line
        }

        let rank_step = if rank_diff > 0 {
            1
        } else if rank_diff < 0 {
            -1
        } else {
            0
        };
        let file_step = if file_diff > 0 {
            1
        } else if file_diff < 0 {
            -1
        } else {
            0
        };

        let mut current_rank = from_rank + rank_step;
        let mut current_file = from_file + file_step;

        while current_rank != to_rank || current_file != to_file {
            squares.push(
                Square::new(current_rank as u8, current_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?,
            );
            current_rank += rank_step;
            current_file += file_step;
        }

        Ok(squares)
    }

    /// Generate all legal moves using original method (for comparison/fallback)
    pub fn generate_legal_moves_original(&self) -> Result<Vec<Move>, MoveGenError> {
        let pseudo_legal = self.generate_pseudo_legal_moves()?;
        let mut legal_moves = Vec::new();

        for mv in pseudo_legal {
            if self.is_legal_move(mv)? {
                legal_moves.push(mv);
            }
        }

        Ok(legal_moves)
    }

    /// Check if a move is legal (doesn't leave king in check)
    pub fn is_legal_move(&self, mv: Move) -> Result<bool, MoveGenError> {
        let mut position_copy = self.clone();
        position_copy.make_move_unchecked(mv)?;
        Ok(!position_copy.is_check(self.side_to_move))
    }

    /// Make a move without validation (for internal use)
    fn make_move_unchecked(&mut self, mv: Move) -> Result<(), MoveGenError> {
        let moving_piece = self.piece_at(mv.from);

        // Handle special moves first
        match mv.move_type {
            MoveType::EnPassant => {
                // Remove the captured pawn
                let captured_pawn_rank = match self.side_to_move {
                    Color::White => mv.to.rank() - 1,
                    Color::Black => mv.to.rank() + 1,
                };
                let captured_square = Square::new(captured_pawn_rank, mv.to.file())
                    .map_err(MoveGenError::InvalidSquare)?;
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
                let rook_from = Square::new(back_rank, 7).map_err(MoveGenError::InvalidSquare)?;
                let rook_to = Square::new(back_rank, 5).map_err(MoveGenError::InvalidSquare)?;
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
                let rook_from = Square::new(back_rank, 0).map_err(MoveGenError::InvalidSquare)?;
                let rook_to = Square::new(back_rank, 3).map_err(MoveGenError::InvalidSquare)?;
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
        self.update_en_passant(&mv)?;
        self.update_clocks(&mv);
        self.switch_side();
        Ok(())
    }

    /// Apply a move for search purposes (public wrapper around make_move_unchecked)
    pub fn apply_move_for_search(&mut self, mv: Move) -> Result<(), MoveGenError> {
        self.make_move_unchecked(mv)
    }

    fn update_castling_rights(&mut self, mv: &Move) {
        let piece = self.piece_at(mv.to);

        if let Some(p) = piece {
            match p.piece_type {
                PieceType::King => {
                    self.castling_rights.remove_all(p.color);
                }
                PieceType::Rook => match (p.color, mv.from.file()) {
                    (Color::White, 0) => self.castling_rights.remove_queenside(Color::White),
                    (Color::White, 7) => self.castling_rights.remove_kingside(Color::White),
                    (Color::Black, 0) => self.castling_rights.remove_queenside(Color::Black),
                    (Color::Black, 7) => self.castling_rights.remove_kingside(Color::Black),
                    _ => {}
                },
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

    fn update_en_passant(&mut self, mv: &Move) -> Result<(), MoveGenError> {
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
        Ok(())
    }

    fn update_clocks(&mut self, mv: &Move) {
        let is_capture = mv.move_type.is_capture();
        let is_pawn_move = self
            .piece_at(mv.to)
            .map(|p| p.piece_type == PieceType::Pawn)
            .unwrap_or(false);

        if is_capture || is_pawn_move {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
    }

    /// Generate pawn moves from a given square
    fn generate_pawn_moves(&self, from: Square, moves: &mut Vec<Move>) -> Result<(), MoveGenError> {
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
            let one_forward = Square::new(one_forward_rank as u8, from_file)
                .map_err(MoveGenError::InvalidSquare)?;

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
                        let two_forward = Square::new(two_forward_rank as u8, from_file)
                            .map_err(MoveGenError::InvalidSquare)?;
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
                let capture_square = Square::new(capture_rank as u8, capture_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?;

                if self.is_occupied_by(capture_square, color.opposite()) {
                    if capture_rank as u8 == promotion_rank {
                        // Promotion captures
                        moves.push(Move::promotion(
                            from,
                            capture_square,
                            PieceType::Queen,
                            true,
                        ));
                        moves.push(Move::promotion(from, capture_square, PieceType::Rook, true));
                        moves.push(Move::promotion(
                            from,
                            capture_square,
                            PieceType::Bishop,
                            true,
                        ));
                        moves.push(Move::promotion(
                            from,
                            capture_square,
                            PieceType::Knight,
                            true,
                        ));
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
        Ok(())
    }

    /// Generate knight moves from a given square
    fn generate_knight_moves(
        &self,
        from: Square,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;

        let knight_offsets = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        for (rank_offset, file_offset) in knight_offsets.iter() {
            let to_rank = from_rank + rank_offset;
            let to_file = from_file + file_offset;

            if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                let to_square = Square::new(to_rank as u8, to_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?;

                if self.is_empty(to_square) {
                    moves.push(Move::quiet(from, to_square));
                } else if self.is_occupied_by(to_square, self.side_to_move.opposite()) {
                    moves.push(Move::capture(from, to_square));
                }
            }
        }
        Ok(())
    }

    /// Generate bishop moves from a given square
    fn generate_bishop_moves(
        &self,
        from: Square,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        self.generate_sliding_moves(from, &directions, moves)
    }

    /// Generate rook moves from a given square
    fn generate_rook_moves(&self, from: Square, moves: &mut Vec<Move>) -> Result<(), MoveGenError> {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        self.generate_sliding_moves(from, &directions, moves)
    }

    /// Generate queen moves from a given square
    fn generate_queen_moves(
        &self,
        from: Square,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        self.generate_sliding_moves(from, &directions, moves)
    }

    /// Generate sliding piece moves (bishop, rook, queen)
    fn generate_sliding_moves(
        &self,
        from: Square,
        directions: &[(i8, i8)],
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
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

                let to_square =
                    Square::new(rank as u8, file as u8).map_err(MoveGenError::InvalidSquare)?;

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
        Ok(())
    }

    /// Generate king moves from a given square
    fn generate_king_moves(&self, from: Square, moves: &mut Vec<Move>) -> Result<(), MoveGenError> {
        let from_rank = from.rank() as i8;
        let from_file = from.file() as i8;

        let king_offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        // Normal king moves
        for (rank_offset, file_offset) in king_offsets.iter() {
            let to_rank = from_rank + rank_offset;
            let to_file = from_file + file_offset;

            if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                let to_square = Square::new(to_rank as u8, to_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?;

                if self.is_empty(to_square) {
                    moves.push(Move::quiet(from, to_square));
                } else if self.is_occupied_by(to_square, self.side_to_move.opposite()) {
                    moves.push(Move::capture(from, to_square));
                }
            }
        }

        // Castling
        self.generate_castling_moves(from, moves)?;
        Ok(())
    }

    /// Generate castling moves
    fn generate_castling_moves(
        &self,
        from: Square,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let color = self.side_to_move;

        // Can't castle if in check
        if self.is_check(color) {
            return Ok(());
        }

        let back_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };

        // Kingside castling
        if self.castling_rights.can_castle_kingside(color) {
            let king_start = Square::new(back_rank, 4).map_err(MoveGenError::InvalidSquare)?;
            let king_end = Square::new(back_rank, 6).map_err(MoveGenError::InvalidSquare)?;
            let rook_square = Square::new(back_rank, 7).map_err(MoveGenError::InvalidSquare)?;
            let king_through = Square::new(back_rank, 5).map_err(MoveGenError::InvalidSquare)?;

            if from == king_start
                && self.is_empty(king_through)
                && self.is_empty(king_end)
                && self.is_occupied_by(rook_square, color)
                && !self.is_square_attacked(king_through, color.opposite())
                && !self.is_square_attacked(king_end, color.opposite())
            {
                moves.push(Move::castle_kingside(from, king_end));
            }
        }

        // Queenside castling
        if self.castling_rights.can_castle_queenside(color) {
            let king_start = Square::new(back_rank, 4).map_err(MoveGenError::InvalidSquare)?;
            let king_end = Square::new(back_rank, 2).map_err(MoveGenError::InvalidSquare)?;
            let rook_square = Square::new(back_rank, 0).map_err(MoveGenError::InvalidSquare)?;
            let bishop_square = Square::new(back_rank, 1).map_err(MoveGenError::InvalidSquare)?;
            let king_through = Square::new(back_rank, 3).map_err(MoveGenError::InvalidSquare)?;

            if from == king_start
                && self.is_empty(bishop_square)
                && self.is_empty(king_end)
                && self.is_empty(king_through)
                && self.is_occupied_by(rook_square, color)
                && !self.is_square_attacked(king_through, color.opposite())
                && !self.is_square_attacked(king_end, color.opposite())
            {
                moves.push(Move::castle_queenside(from, king_end));
            }
        }
        Ok(())
    }

    /// Detect pinned pieces for the given color
    /// Returns a map of square -> pin direction (rank_delta, file_delta)
    fn get_pinned_pieces(
        &self,
        color: Color,
    ) -> Result<std::collections::HashMap<Square, (i8, i8)>, MoveGenError> {
        use std::collections::HashMap;

        let mut pinned_pieces = HashMap::new();

        // Find our king
        let king_square = match self.find_king(color) {
            Some(square) => square,
            None => return Ok(pinned_pieces), // No king, no pins
        };

        let king_rank = king_square.rank() as i8;
        let king_file = king_square.file() as i8;

        // Check all sliding directions for potential pins
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1), // Up-left, up, up-right
            (0, -1),
            (0, 1), // Left, right
            (1, -1),
            (1, 0),
            (1, 1), // Down-left, down, down-right
        ];

        for &(rank_delta, file_delta) in &directions {
            let mut current_rank = king_rank + rank_delta;
            let mut current_file = king_file + file_delta;
            let mut found_our_piece: Option<Square> = None;

            // Walk in this direction from the king
            while (0..8).contains(&current_rank) && (0..8).contains(&current_file) {
                let current_square = Square::new(current_rank as u8, current_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?;

                if let Some(piece) = self.piece_at(current_square) {
                    if piece.color == color {
                        // Found one of our pieces
                        if found_our_piece.is_some() {
                            // Second piece blocks the pin
                            break;
                        }
                        found_our_piece = Some(current_square);
                    } else {
                        // Found opponent piece
                        if let Some(our_piece_square) = found_our_piece {
                            // Check if this opponent piece can pin along this direction
                            if self
                                .can_piece_attack_along_direction(piece, (rank_delta, file_delta))
                            {
                                pinned_pieces.insert(our_piece_square, (rank_delta, file_delta));
                            }
                        }
                        break; // Stop searching in this direction
                    }
                }

                current_rank += rank_delta;
                current_file += file_delta;
            }
        }

        Ok(pinned_pieces)
    }

    /// Check if a piece can attack along a given direction
    fn can_piece_attack_along_direction(&self, piece: Piece, direction: (i8, i8)) -> bool {
        match piece.piece_type {
            PieceType::Rook => direction.0 == 0 || direction.1 == 0, // Horizontal or vertical
            PieceType::Bishop => direction.0.abs() == direction.1.abs(), // Diagonal
            PieceType::Queen => true,                                // Can attack in any direction
            _ => false, // Other pieces don't create pins
        }
    }

    /// Generate pin-aware pawn moves
    fn generate_pin_aware_pawn_moves(
        &self,
        from: Square,
        pin_direction: Option<&(i8, i8)>,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        if let Some(&(rank_delta, file_delta)) = pin_direction {
            // Pawn is pinned - can only move along the pin direction
            if rank_delta == 0 {
                // Pinned horizontally - pawn can't move
                return Ok(());
            }
            // Pinned vertically or diagonally - generate limited moves
            self.generate_pawn_moves_along_pin(from, (rank_delta, file_delta), moves)
        } else {
            // Not pinned - generate all pawn moves
            self.generate_pawn_moves(from, moves)
        }
    }

    /// Generate pin-aware bishop moves
    fn generate_pin_aware_bishop_moves(
        &self,
        from: Square,
        pin_direction: Option<&(i8, i8)>,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        if let Some(&(rank_delta, file_delta)) = pin_direction {
            // Bishop is pinned - can only move along the pin direction
            if rank_delta.abs() == file_delta.abs() {
                // Pin is diagonal - bishop can move along it
                self.generate_sliding_moves_along_pin(from, (rank_delta, file_delta), moves)
            } else {
                // Pin is not diagonal - bishop can't move
                Ok(())
            }
        } else {
            // Not pinned - generate all bishop moves
            self.generate_bishop_moves(from, moves)
        }
    }

    /// Generate pin-aware rook moves
    fn generate_pin_aware_rook_moves(
        &self,
        from: Square,
        pin_direction: Option<&(i8, i8)>,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        if let Some(&(rank_delta, file_delta)) = pin_direction {
            // Rook is pinned - can only move along the pin direction
            if rank_delta == 0 || file_delta == 0 {
                // Pin is horizontal or vertical - rook can move along it
                self.generate_sliding_moves_along_pin(from, (rank_delta, file_delta), moves)
            } else {
                // Pin is diagonal - rook can't move
                Ok(())
            }
        } else {
            // Not pinned - generate all rook moves
            self.generate_rook_moves(from, moves)
        }
    }

    /// Generate pin-aware queen moves
    fn generate_pin_aware_queen_moves(
        &self,
        from: Square,
        pin_direction: Option<&(i8, i8)>,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        if let Some(&(rank_delta, file_delta)) = pin_direction {
            // Queen is pinned - can only move along the pin direction
            self.generate_sliding_moves_along_pin(from, (rank_delta, file_delta), moves)
        } else {
            // Not pinned - generate all queen moves
            self.generate_queen_moves(from, moves)
        }
    }

    /// Generate pawn moves along a specific pin direction
    fn generate_pawn_moves_along_pin(
        &self,
        from: Square,
        pin_direction: (i8, i8),
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let color = self.piece_at(from).unwrap().color;
        let direction = match color {
            Color::White => 1i8,
            Color::Black => -1i8,
        };

        let (rank_delta, file_delta) = pin_direction;

        // If pinned vertically, pawn can move forward
        if file_delta == 0 && rank_delta * direction > 0 {
            // Generate normal forward moves (but not captures)
            self.generate_pawn_forward_moves_only(from, moves)?;
        }

        // If pinned diagonally, pawn can only capture along the pin
        if rank_delta * direction > 0 && file_delta.abs() == 1 {
            self.generate_pawn_capture_along_pin(from, pin_direction, moves)?;
        }

        Ok(())
    }

    /// Generate sliding moves along a specific pin direction
    fn generate_sliding_moves_along_pin(
        &self,
        from: Square,
        pin_direction: (i8, i8),
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let (rank_delta, file_delta) = pin_direction;

        // Generate moves in both directions along the pin
        for &direction_multiplier in &[1, -1] {
            let dr = rank_delta * direction_multiplier;
            let df = file_delta * direction_multiplier;

            let mut current_rank = from.rank() as i8 + dr;
            let mut current_file = from.file() as i8 + df;

            while (0..8).contains(&current_rank) && (0..8).contains(&current_file) {
                let target_square = Square::new(current_rank as u8, current_file as u8)
                    .map_err(MoveGenError::InvalidSquare)?;

                if self.is_empty(target_square) {
                    // Empty square - can move here
                    moves.push(Move::quiet(from, target_square));
                } else if self.is_occupied_by(target_square, self.side_to_move.opposite()) {
                    // Opponent piece - can capture
                    moves.push(Move::capture(from, target_square));
                    break; // Can't move further
                } else {
                    // Our own piece - can't move here
                    break;
                }

                current_rank += dr;
                current_file += df;
            }
        }

        Ok(())
    }

    /// Generate only forward pawn moves (no captures)
    fn generate_pawn_forward_moves_only(
        &self,
        from: Square,
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let color = self.piece_at(from).unwrap().color;
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

        // Forward moves only
        let one_forward_rank = from_rank + direction;
        if (0..8).contains(&one_forward_rank) {
            let one_forward = Square::new(one_forward_rank as u8, from_file)
                .map_err(MoveGenError::InvalidSquare)?;

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
                        let two_forward = Square::new(two_forward_rank as u8, from_file)
                            .map_err(MoveGenError::InvalidSquare)?;
                        if self.is_empty(two_forward) {
                            moves.push(Move::quiet(from, two_forward));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate pawn capture along a specific pin direction
    fn generate_pawn_capture_along_pin(
        &self,
        from: Square,
        pin_direction: (i8, i8),
        moves: &mut Vec<Move>,
    ) -> Result<(), MoveGenError> {
        let color = self.piece_at(from).unwrap().color;
        let direction = match color {
            Color::White => 1i8,
            Color::Black => -1i8,
        };

        let promotion_rank = match color {
            Color::White => 7,
            Color::Black => 0,
        };

        let (_rank_delta, file_delta) = pin_direction;
        let capture_rank = from.rank() as i8 + direction;
        let capture_file = from.file() as i8 + file_delta;

        if (0..8).contains(&capture_file) && (0..8).contains(&capture_rank) {
            let capture_square = Square::new(capture_rank as u8, capture_file as u8)
                .map_err(MoveGenError::InvalidSquare)?;

            if self.is_occupied_by(capture_square, self.side_to_move.opposite()) {
                if capture_rank as u8 == promotion_rank {
                    // Promotion captures
                    moves.push(Move::promotion(
                        from,
                        capture_square,
                        PieceType::Queen,
                        true,
                    ));
                    moves.push(Move::promotion(from, capture_square, PieceType::Rook, true));
                    moves.push(Move::promotion(
                        from,
                        capture_square,
                        PieceType::Bishop,
                        true,
                    ));
                    moves.push(Move::promotion(
                        from,
                        capture_square,
                        PieceType::Knight,
                        true,
                    ));
                } else {
                    moves.push(Move::capture(from, capture_square));
                }
            }
        }

        // En passant along pin (if applicable)
        if let Some(en_passant_square) = self.en_passant {
            let ep_rank = en_passant_square.rank() as i8;
            let ep_file = en_passant_square.file() as i8;

            if capture_rank == ep_rank && capture_file == ep_file {
                moves.push(Move::en_passant(from, en_passant_square));
            }
        }

        Ok(())
    }
}
