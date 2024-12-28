use chess_core::{Board, Position, piece::{PieceType, Color}};

// Piece values
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

// Evaluation constants
const DOUBLED_PAWN_PENALTY: i32 = -20;
const CONNECTED_PAWN_BONUS: i32 = 15;
const KING_PROTECTION_BONUS: i32 = 25;

// Piece-square tables for middlegame
const PAWN_TABLE: [[i32; 8]; 8] = [
    [  0,  0,  0,  0,  0,  0,  0,  0],
    [ 50, 50, 50, 50, 50, 50, 50, 50],
    [ 10, 10, 20, 30, 30, 20, 10, 10],
    [  5,  5, 10, 25, 25, 10,  5,  5],
    [  0,  0,  0, 20, 20,  0,  0,  0],
    [  5, -5,-10,  0,  0,-10, -5,  5],
    [  5, 10, 10,-20,-20, 10, 10,  5],
    [  0,  0,  0,  0,  0,  0,  0,  0]
];

const KNIGHT_TABLE: [[i32; 8]; 8] = [
    [-50,-40,-30,-30,-30,-30,-40,-50],
    [-40,-20,  0,  0,  0,  0,-20,-40],
    [-30,  0, 10, 15, 15, 10,  0,-30],
    [-30,  5, 15, 20, 20, 15,  5,-30],
    [-30,  0, 15, 20, 20, 15,  0,-30],
    [-30,  5, 10, 15, 15, 10,  5,-30],
    [-40,-20,  0,  5,  5,  0,-20,-40],
    [-50,-40,-30,-30,-30,-30,-40,-50]
];

const BISHOP_TABLE: [[i32; 8]; 8] = [
    [-20,-10,-10,-10,-10,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5, 10, 10,  5,  0,-10],
    [-10,  5,  5, 10, 10,  5,  5,-10],
    [-10,  0, 10, 10, 10, 10,  0,-10],
    [-10, 10, 10, 10, 10, 10, 10,-10],
    [-10,  5,  0,  0,  0,  0,  5,-10],
    [-20,-10,-10,-10,-10,-10,-10,-20]
];

const ROOK_TABLE: [[i32; 8]; 8] = [
    [  0,  0,  0,  0,  0,  0,  0,  0],
    [  5, 10, 10, 10, 10, 10, 10,  5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [ -5,  0,  0,  0,  0,  0,  0, -5],
    [  0,  0,  0,  5,  5,  0,  0,  0]
];

const QUEEN_TABLE: [[i32; 8]; 8] = [
    [-20,-10,-10, -5, -5,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5,  5,  5,  5,  0,-10],
    [ -5,  0,  5,  5,  5,  5,  0, -5],
    [  0,  0,  5,  5,  5,  5,  0, -5],
    [-10,  5,  5,  5,  5,  5,  0,-10],
    [-10,  0,  5,  0,  0,  0,  0,-10],
    [-20,-10,-10, -5, -5,-10,-10,-20]
];

const KING_MIDDLEGAME_TABLE: [[i32; 8]; 8] = [
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-20,-30,-30,-40,-40,-30,-30,-20],
    [-10,-20,-20,-20,-20,-20,-20,-10],
    [ 20, 20,  0,  0,  0,  0, 20, 20],
    [ 20, 30, 10,  0,  0, 10, 30, 20]
];

const KING_ENDGAME_TABLE: [[i32; 8]; 8] = [
    [-50,-40,-30,-20,-20,-30,-40,-50],
    [-30,-20,-10,  0,  0,-10,-20,-30],
    [-30,-10, 20, 30, 30, 20,-10,-30],
    [-30,-10, 30, 40, 40, 30,-10,-30],
    [-30,-10, 30, 40, 40, 30,-10,-30],
    [-30,-10, 20, 30, 30, 20,-10,-30],
    [-30,-30,  0,  0,  0,  0,-30,-30],
    [-50,-30,-30,-30,-30,-30,-30,-50]
];

pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;
    let is_endgame = is_endgame_phase(board);

    // Material and positional evaluation
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = get_piece_value(piece.piece_type);
                let position_bonus = get_position_bonus(piece.piece_type, pos, piece.color == Color::White, is_endgame);
                let multiplier = if piece.color == Color::White { 1 } else { -1 };
                
                score += multiplier * (piece_value + position_bonus);

                // Add mobility bonus
                let valid_moves = board.get_valid_moves(pos);
                score += multiplier * (valid_moves.len() as i32 * get_mobility_bonus(piece.piece_type));

                // Evaluate pawn structure
                if piece.piece_type == PieceType::Pawn {
                    score += multiplier * evaluate_pawn_structure(board, pos, piece.color == Color::White);
                }

                // Evaluate king safety (more important in middlegame)
                if piece.piece_type == PieceType::King && !is_endgame {
                    score += multiplier * evaluate_king_safety(board, pos, piece.color == Color::White);
                }
            }
        }
    }

    // Adjust score based on game phase
    if is_endgame {
        score = adjust_endgame_score(board, score);
    }

    score
}

fn get_mobility_bonus(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 2,
        PieceType::Knight => 4,
        PieceType::Bishop => 5,
        PieceType::Rook => 3,
        PieceType::Queen => 2,
        PieceType::King => 0,
    }
}

fn is_endgame_phase(board: &Board) -> bool {
    let mut queens = 0;
    let mut minor_pieces = 0;

    for rank in 1..=8 {
        for file in 1..=8 {
            if let Some(piece) = board.get_piece(Position { rank, file }) {
                match piece.piece_type {
                    PieceType::Queen => queens += 1,
                    PieceType::Rook | PieceType::Bishop | PieceType::Knight => minor_pieces += 1,
                    _ => {}
                }
            }
        }
    }

    queens == 0 || (queens == 2 && minor_pieces <= 2)
}

fn adjust_endgame_score(board: &Board, mut score: i32) -> i32 {
    // In endgame, encourage pushing opponent king to the corner
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::King {
                    let multiplier = if piece.color == Color::White { -1 } else { 1 };
                    // Distance from center penalty
                    let center_dist = ((4.5 - rank as f32).abs() + (4.5 - file as f32).abs()) as i32;
                    score += multiplier * center_dist * 10;
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
        PieceType::King => KING_VALUE,
    }
}

fn get_position_bonus(piece_type: PieceType, pos: Position, is_white: bool, is_endgame: bool) -> i32 {
    let rank_idx = if is_white { 8 - pos.rank } else { pos.rank - 1 } as usize;
    let file_idx = (pos.file - 1) as usize;
    
    match piece_type {
        PieceType::Pawn => PAWN_TABLE[rank_idx][file_idx],
        PieceType::Knight => KNIGHT_TABLE[rank_idx][file_idx],
        PieceType::Bishop => BISHOP_TABLE[rank_idx][file_idx],
        PieceType::Rook => ROOK_TABLE[rank_idx][file_idx],
        PieceType::Queen => QUEEN_TABLE[rank_idx][file_idx],
        PieceType::King => {
            if is_endgame {
                KING_ENDGAME_TABLE[rank_idx][file_idx]
            } else {
                KING_MIDDLEGAME_TABLE[rank_idx][file_idx]
            }
        }
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