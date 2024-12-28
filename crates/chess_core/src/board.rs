use std::collections::HashMap;
use crate::{Piece, Position, piece::{PieceType, Color}, Move};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pieces: HashMap<Position, Piece>,
    current_turn: Color,
    castling_rights: CastlingRights,
    last_move: Option<Move>,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            pieces: HashMap::new(),
            current_turn: Color::White,
            castling_rights: CastlingRights::default(),
            last_move: None,
        };
        board.setup_initial_position();
        board
    }

    pub fn setup_initial_position(&mut self) {
        // Setup pawns
        for file in 1..=8 {
            self.pieces.insert(Position { file, rank: 2 }, Piece::new(PieceType::Pawn, Color::White));
            self.pieces.insert(Position { file, rank: 7 }, Piece::new(PieceType::Pawn, Color::Black));
        }

        // Setup other pieces
        let piece_order = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];

        for (file, &piece_type) in (1..=8).zip(piece_order.iter()) {
            // White pieces on rank 1
            self.pieces.insert(Position { file, rank: 1 }, Piece::new(piece_type, Color::White));
            // Black pieces on rank 8
            self.pieces.insert(Position { file, rank: 8 }, Piece::new(piece_type, Color::Black));
        }
    }

    pub fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    pub fn current_turn(&self) -> Color {
        self.current_turn
    }

    pub fn make_move(&mut self, chess_move: Move) -> Result<(), &'static str> {
        // Clone the piece early to avoid borrow checker issues
        let piece = *self.pieces.get(&chess_move.from).ok_or("No piece at starting position")?;

        if piece.color != self.current_turn {
            return Err("Not your turn");
        }

        // Check if this is a castling move
        if piece.piece_type == PieceType::King {
            let file_diff = (chess_move.to.file as i8 - chess_move.from.file as i8).abs();
            if file_diff == 2 {
                self.handle_castling(chess_move)?;
                return Ok(());
            }
        }

        // Validate the move
        if !chess_move.is_valid(self) {
            return Err("Invalid move for this piece");
        }

        // Make a clone of the board and try the move
        let mut temp_board = self.clone();
        temp_board.make_move_without_validation(chess_move)?;

        // Check if the move puts/leaves the king in check
        if temp_board.is_in_check(piece.color) {
            return Err("Move would leave king in check");
        }

        // Update castling rights
        self.update_castling_rights(&piece, chess_move);

        // Actually make the move
        self.make_move_without_validation(chess_move)?;
        self.last_move = Some(chess_move);

        Ok(())
    }

    fn make_move_without_validation(&mut self, chess_move: Move) -> Result<(), &'static str> {
        let piece = self.pieces.remove(&chess_move.from).unwrap();

        // Handle en passant capture
        if piece.piece_type == PieceType::Pawn {
            let file_diff = (chess_move.to.file as i8 - chess_move.from.file as i8).abs();
            let is_diagonal = file_diff == 1;

            if is_diagonal && !self.pieces.contains_key(&chess_move.to) {
                // This might be an en passant capture
                if let Some(last_move) = self.last_move {
                    if last_move.from.file == chess_move.to.file {
                        if let Some(last_piece) = self.pieces.get(&last_move.to) {
                            if last_piece.piece_type == PieceType::Pawn {
                                let last_rank_diff = (last_move.to.rank as i8 - last_move.from.rank as i8).abs();
                                if last_rank_diff == 2 {
                                    // Remove the captured pawn
                                    self.pieces.remove(&last_move.to);
                                }
                            }
                        }
                    }
                }
            }
        }

        let final_piece = if let Some(promotion_type) = chess_move.promotion {
            if piece.piece_type != PieceType::Pawn {
                return Err("Only pawns can be promoted");
            }
            if (piece.color == Color::White && chess_move.to.rank != 8) ||
               (piece.color == Color::Black && chess_move.to.rank != 1) {
                return Err("Pawns can only be promoted on the last rank");
            }
            Piece::new(promotion_type, piece.color)
        } else {
            piece
        };

        self.pieces.insert(chess_move.to, final_piece);
        self.current_turn = match self.current_turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }

    fn handle_castling(&mut self, chess_move: Move) -> Result<(), &'static str> {
        // Clone the king early to avoid borrow checker issues
        let king = *self.pieces.get(&chess_move.from).ok_or("No king at starting position")?;
        let rank = if king.color == Color::White { 1 } else { 8 };
        
        // Check if castling is allowed
        let is_kingside = chess_move.to.file == 7;
        let can_castle = if king.color == Color::White {
            if is_kingside { self.castling_rights.white_kingside } else { self.castling_rights.white_queenside }
        } else {
            if is_kingside { self.castling_rights.black_kingside } else { self.castling_rights.black_queenside }
        };

        if !can_castle {
            return Err("Castling is not allowed");
        }

        // Check if path is clear and not under attack
        let path = if is_kingside { 
            vec![Position::new(5, rank).unwrap(), Position::new(6, rank).unwrap(), Position::new(7, rank).unwrap()]
        } else {
            vec![Position::new(3, rank).unwrap(), Position::new(4, rank).unwrap()]
        };

        for pos in &path {
            if self.pieces.contains_key(pos) {
                return Err("Path is not clear for castling");
            }
            if self.is_position_under_attack(*pos, king.color) {
                return Err("Cannot castle through check");
            }
        }

        // Move the king
        self.pieces.remove(&chess_move.from);
        self.pieces.insert(chess_move.to, king);

        // Move the rook
        let rook_from = Position::new(if is_kingside { 8 } else { 1 }, rank).unwrap();
        let rook_to = Position::new(if is_kingside { 6 } else { 4 }, rank).unwrap();
        
        // Get and remove the rook
        let rook = self.pieces.remove(&rook_from).ok_or("No rook found for castling")?;
        self.pieces.insert(rook_to, rook);

        // Update castling rights
        if king.color == Color::White {
            self.castling_rights.white_kingside = false;
            self.castling_rights.white_queenside = false;
        } else {
            self.castling_rights.black_kingside = false;
            self.castling_rights.black_queenside = false;
        }

        self.current_turn = match self.current_turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }

    fn update_castling_rights(&mut self, piece: &Piece, chess_move: Move) {
        match piece.piece_type {
            PieceType::King => {
                if piece.color == Color::White {
                    self.castling_rights.white_kingside = false;
                    self.castling_rights.white_queenside = false;
                } else {
                    self.castling_rights.black_kingside = false;
                    self.castling_rights.black_queenside = false;
                }
            }
            PieceType::Rook => {
                let (rank, file) = (chess_move.from.rank, chess_move.from.file);
                if piece.color == Color::White && rank == 1 {
                    if file == 1 {
                        self.castling_rights.white_queenside = false;
                    } else if file == 8 {
                        self.castling_rights.white_kingside = false;
                    }
                } else if piece.color == Color::Black && rank == 8 {
                    if file == 1 {
                        self.castling_rights.black_queenside = false;
                    } else if file == 8 {
                        self.castling_rights.black_kingside = false;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        // Find the king
        let king_pos = self.pieces.iter()
            .find(|(_, piece)| piece.piece_type == PieceType::King && piece.color == color)
            .map(|(pos, _)| *pos)
            .unwrap();

        self.is_position_under_attack(king_pos, color)
    }

    pub fn is_position_under_attack(&self, pos: Position, defending_color: Color) -> bool {
        // Check for attacks from each enemy piece
        for (&attacker_pos, attacker) in self.pieces.iter() {
            if attacker.color == defending_color {
                continue;
            }

            let attack_move = Move::new(attacker_pos, pos);
            if attack_move.is_valid(self) {
                return true;
            }
        }
        false
    }

    pub fn is_checkmate(&self) -> bool {
        if !self.is_in_check(self.current_turn) {
            return false;
        }

        // Check if any move can get out of check
        for (&from, piece) in self.pieces.iter() {
            if piece.color != self.current_turn {
                continue;
            }

            for rank in 1..=8 {
                for file in 1..=8 {
                    let to = Position::new(file, rank).unwrap();
                    let chess_move = Move::new(from, to);
                    
                    // Try the move on a cloned board
                    let mut temp_board = self.clone();
                    if chess_move.is_valid(&temp_board) {
                        if temp_board.make_move_without_validation(chess_move).is_ok() {
                            if !temp_board.is_in_check(self.current_turn) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    pub fn is_position_valid(&self, pos: Position) -> bool {
        pos.file >= 1 && pos.file <= 8 && pos.rank >= 1 && pos.rank <= 8
    }

    pub fn get_all_pieces(&self) -> &HashMap<Position, Piece> {
        &self.pieces
    }

    pub fn get_valid_moves(&self, pos: Position) -> Vec<Move> {
        let mut valid_moves = Vec::new();
        
        if let Some(piece) = self.get_piece(pos) {
            if piece.color != self.current_turn {
                return valid_moves;
            }

            // Generate all possible positions
            for rank in 1..=8 {
                for file in 1..=8 {
                    let target_pos = Position { file, rank };
                    let chess_move = Move::new(pos, target_pos);
                    if chess_move.is_valid(self) {
                        valid_moves.push(chess_move);
                    }

                    // Check for pawn promotion
                    if piece.piece_type == PieceType::Pawn {
                        if (piece.color == Color::White && rank == 8) ||
                           (piece.color == Color::Black && rank == 1) {
                            for promotion_type in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                let promotion_move = Move::with_promotion(pos, target_pos, promotion_type);
                                if promotion_move.is_valid(self) {
                                    valid_moves.push(promotion_move);
                                }
                            }
                        }
                    }
                }
            }
        }

        valid_moves
    }

    pub fn last_move(&self) -> Option<Move> {
        self.last_move
    }

    pub fn is_stalemate(&self) -> bool {
        if self.is_in_check(self.current_turn) {
            return false;
        }

        // Check if any legal move exists
        for (&from, piece) in self.pieces.iter() {
            if piece.color != self.current_turn {
                continue;
            }

            for rank in 1..=8 {
                for file in 1..=8 {
                    let to = Position::new(file, rank).unwrap();
                    let chess_move = Move::new(from, to);
                    
                    // Try the move on a cloned board
                    let mut temp_board = self.clone();
                    if chess_move.is_valid(&temp_board) {
                        if temp_board.make_move_without_validation(chess_move).is_ok() {
                            if !temp_board.is_in_check(self.current_turn) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    pub fn has_insufficient_material(&self) -> bool {
        let mut white_pieces = Vec::new();
        let mut black_pieces = Vec::new();

        for piece in self.pieces.values() {
            match piece.color {
                Color::White => white_pieces.push(piece),
                Color::Black => black_pieces.push(piece),
            }
        }

        // King vs King
        if white_pieces.len() == 1 && black_pieces.len() == 1 {
            return true;
        }

        // King and Bishop/Knight vs King
        if (white_pieces.len() == 2 && black_pieces.len() == 1) ||
           (white_pieces.len() == 1 && black_pieces.len() == 2) {
            let larger_side = if white_pieces.len() > black_pieces.len() { &white_pieces } else { &black_pieces };
            let piece = larger_side.iter()
                .find(|p| p.piece_type != PieceType::King)
                .unwrap();
            
            return matches!(piece.piece_type, PieceType::Bishop | PieceType::Knight);
        }

        // King and Bishop vs King and Bishop (same color bishops)
        if white_pieces.len() == 2 && black_pieces.len() == 2 {
            let white_bishop = white_pieces.iter()
                .find(|p| p.piece_type == PieceType::Bishop);
            let black_bishop = black_pieces.iter()
                .find(|p| p.piece_type == PieceType::Bishop);
            
            if let (Some(wb), Some(bb)) = (white_bishop, black_bishop) {
                // Check if bishops are on same colored squares
                let white_bishop_pos = self.pieces.iter()
                    .find(|(_, p)| p.piece_type == PieceType::Bishop && p.color == Color::White)
                    .map(|(pos, _)| pos)
                    .unwrap();
                let black_bishop_pos = self.pieces.iter()
                    .find(|(_, p)| p.piece_type == PieceType::Bishop && p.color == Color::Black)
                    .map(|(pos, _)| pos)
                    .unwrap();
                
                return (white_bishop_pos.file + white_bishop_pos.rank) % 2 ==
                       (black_bishop_pos.file + black_bishop_pos.rank) % 2;
            }
        }

        false
    }
} 