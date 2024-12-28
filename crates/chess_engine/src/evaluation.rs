use chess_core::{Board, Position, piece::{PieceType, Color}};

// Piece values
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Bonus for controlling center squares
const CENTER_CONTROL_BONUS: i32 = 10;

// Bonus for piece mobility
const MOBILITY_BONUS: i32 = 5;

// Penalty for doubled pawns
const DOUBLED_PAWN_PENALTY: i32 = -20;

// Bonus for connected pawns
const CONNECTED_PAWN_BONUS: i32 = 15;

// Bonus for pieces protecting the king
const KING_PROTECTION_BONUS: i32 = 25;

pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;

    // Material and positional evaluation
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = get_piece_value(piece.piece_type);
                let position_bonus = get_position_bonus(piece.piece_type, pos, piece.color == Color::White);
                let multiplier = if piece.color == Color::White { 1 } else { -1 };
                
                score += multiplier * (piece_value + position_bonus);

                // Add mobility bonus
                let valid_moves = board.get_valid_moves(pos);
                score += multiplier * (valid_moves.len() as i32 * MOBILITY_BONUS);

                // Evaluate pawn structure
                if piece.piece_type == PieceType::Pawn {
                    score += multiplier * evaluate_pawn_structure(board, pos, piece.color == Color::White);
                }

                // Evaluate king safety
                if piece.piece_type == PieceType::King {
                    score += multiplier * evaluate_king_safety(board, pos, piece.color == Color::White);
                }
            }
        }
    }

    score
}

fn get_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => PAWN_VALUE,
        PieceType::Knight => KNIGHT_VALUE,
        PieceType::Bishop => BISHOP_VALUE,
        PieceType::Rook => ROOK_VALUE,
        PieceType::Queen => QUEEN_VALUE,
        PieceType::King => 0, // King's value is not counted in material evaluation
    }
}

fn get_position_bonus(piece_type: PieceType, pos: Position, is_white: bool) -> i32 {
    let rank = if is_white { i32::from(pos.rank) } else { 9 - i32::from(pos.rank) };
    let file = i32::from(pos.file);
    
    match piece_type {
        PieceType::Pawn => {
            // Pawns are more valuable as they advance
            (rank - 2) * 10
        }
        PieceType::Knight => {
            // Knights are more valuable in the center
            if (3..=6).contains(&file) && (3..=6).contains(&rank) {
                CENTER_CONTROL_BONUS
            } else {
                0
            }
        }
        PieceType::Bishop => {
            // Bishops are more valuable in open diagonals
            if (3..=6).contains(&file) && (3..=6).contains(&rank) {
                CENTER_CONTROL_BONUS / 2
            } else {
                0
            }
        }
        PieceType::Rook => {
            // Rooks are more valuable on open files
            if file == 1 || file == 8 {
                20
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn evaluate_pawn_structure(board: &Board, pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    let file = pos.file;

    // Check for doubled pawns
    let mut doubled = false;
    for rank in 1..=8 {
        if rank != pos.rank {
            let check_pos = Position { rank, file };
            if let Some(piece) = board.get_piece(check_pos) {
                let is_same_color = piece.color == Color::White;
                if piece.piece_type == PieceType::Pawn && is_same_color == is_white {
                    doubled = true;
                    break;
                }
            }
        }
    }
    if doubled {
        score += DOUBLED_PAWN_PENALTY;
    }

    // Check for connected pawns
    for adj_file in (file - 1)..=(file + 1) {
        if adj_file == file || adj_file < 1 || adj_file > 8 {
            continue;
        }
        let check_pos = Position { rank: pos.rank, file: adj_file };
        if let Some(piece) = board.get_piece(check_pos) {
            let is_same_color = piece.color == Color::White;
            if piece.piece_type == PieceType::Pawn && is_same_color == is_white {
                score += CONNECTED_PAWN_BONUS;
            }
        }
    }

    score
}

fn evaluate_king_safety(board: &Board, king_pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    let rank = king_pos.rank;
    let file = king_pos.file;

    // Check pieces protecting the king
    for r in (rank - 1)..=(rank + 1) {
        for f in (file - 1)..=(file + 1) {
            if r < 1 || r > 8 || f < 1 || f > 8 {
                continue;
            }
            let pos = Position { rank: r, file: f };
            if let Some(piece) = board.get_piece(pos) {
                let is_same_color = piece.color == Color::White;
                if is_same_color == is_white && piece.piece_type != PieceType::King {
                    score += KING_PROTECTION_BONUS;
                }
            }
        }
    }

    score
} 