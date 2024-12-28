use crate::{Position, Piece, piece::{PieceType, Color}, Board};

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub move_type: MoveType,
    pub promotion: Option<PieceType>,
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && 
        self.to == other.to && 
        self.move_type == other.move_type && 
        self.promotion == other.promotion
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Normal,
    Capture,
    EnPassant,
    Castle,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
            promotion: None,
            move_type: MoveType::Normal,
        }
    }

    pub fn with_promotion(from: Position, to: Position, promotion: PieceType) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
            move_type: MoveType::Normal,
        }
    }

    pub fn castle(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
            promotion: None,
            move_type: MoveType::Castle,
        }
    }

    pub fn is_valid(&self, board: &Board) -> bool {
        let piece = match board.get_piece(self.from) {
            Some(p) => p,
            None => return false,
        };

        // Basic position validation
        if !board.is_position_valid(self.from) || !board.is_position_valid(self.to) {
            return false;
        }

        // Check if destination has a piece of the same color
        if let Some(dest_piece) = board.get_piece(self.to) {
            if dest_piece.color == piece.color {
                return false;
            }
        }

        // Check if the move follows the piece's movement pattern
        self.is_valid_piece_movement(piece, board)
    }

    fn is_valid_piece_movement(&self, piece: &Piece, board: &Board) -> bool {
        match piece.piece_type {
            PieceType::Pawn => self.is_valid_pawn_move(piece.color, board),
            PieceType::Knight => self.is_valid_knight_move(),
            PieceType::Bishop => self.is_valid_diagonal_move(board),
            PieceType::Rook => self.is_valid_straight_move(board),
            PieceType::Queen => self.is_valid_diagonal_move(board) || self.is_valid_straight_move(board),
            PieceType::King => self.is_valid_king_move(),
        }
    }

    fn is_valid_pawn_move(&self, color: Color, board: &Board) -> bool {
        let direction = match color {
            Color::White => 1,
            Color::Black => -1,
        };

        let rank_diff = (self.to.rank as i8) - (self.from.rank as i8);
        let file_diff = (self.to.file as i8) - (self.from.file as i8);

        // Basic forward movement
        if file_diff == 0 {
            if rank_diff == direction {
                return board.get_piece(self.to).is_none();
            }
            // Initial two-square move
            if (color == Color::White && self.from.rank == 2) || 
               (color == Color::Black && self.from.rank == 7) {
                if rank_diff == 2 * direction {
                    let intermediate = Position::new(self.from.file, (self.from.rank as i8 + direction) as u8).unwrap();
                    return board.get_piece(intermediate).is_none() && board.get_piece(self.to).is_none();
                }
            }
        }

        // Regular capture movement
        if file_diff.abs() == 1 && rank_diff == direction {
            if let Some(captured_piece) = board.get_piece(self.to) {
                return captured_piece.color != color;
            }

            // En passant capture
            if let Some(last_move) = board.last_move() {
                if last_move.from.file == self.to.file {
                    if let Some(last_piece) = board.get_piece(last_move.to) {
                        if last_piece.piece_type == PieceType::Pawn {
                            let last_rank_diff = (last_move.to.rank as i8 - last_move.from.rank as i8).abs();
                            if last_rank_diff == 2 {
                                let expected_rank = if color == Color::White { 5 } else { 4 };
                                if self.from.rank == expected_rank {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn is_valid_knight_move(&self) -> bool {
        let rank_diff = (self.to.rank as i8 - self.from.rank as i8).abs();
        let file_diff = (self.to.file as i8 - self.from.file as i8).abs();
        
        (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
    }

    fn is_valid_diagonal_move(&self, board: &Board) -> bool {
        let rank_diff = (self.to.rank as i8 - self.from.rank as i8).abs();
        let file_diff = (self.to.file as i8 - self.from.file as i8).abs();

        if rank_diff != file_diff {
            return false;
        }

        self.is_path_clear(board)
    }

    fn is_valid_straight_move(&self, board: &Board) -> bool {
        let rank_diff = self.to.rank as i8 - self.from.rank as i8;
        let file_diff = self.to.file as i8 - self.from.file as i8;

        if rank_diff != 0 && file_diff != 0 {
            return false;
        }

        self.is_path_clear(board)
    }

    fn is_valid_king_move(&self) -> bool {
        let rank_diff = (self.to.rank as i8 - self.from.rank as i8).abs();
        let file_diff = (self.to.file as i8 - self.from.file as i8).abs();

        rank_diff <= 1 && file_diff <= 1
    }

    fn is_path_clear(&self, board: &Board) -> bool {
        let rank_step = (self.to.rank as i8 - self.from.rank as i8).signum();
        let file_step = (self.to.file as i8 - self.from.file as i8).signum();

        let mut current_rank = self.from.rank as i8 + rank_step;
        let mut current_file = self.from.file as i8 + file_step;
        let target_rank = self.to.rank as i8;
        let target_file = self.to.file as i8;

        while (current_rank != target_rank || current_file != target_file) &&
              current_rank >= 1 && current_rank <= 8 &&
              current_file >= 1 && current_file <= 8 {
            let pos = Position::new(current_file as u8, current_rank as u8).unwrap();
            if board.get_piece(pos).is_some() {
                return false;
            }
            current_rank += rank_step;
            current_file += file_step;
        }

        true
    }
} 