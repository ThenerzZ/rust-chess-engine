use chess_core::{Board, Position, piece::{PieceType, Color}};

// Piece values (adjusted to standard chess values * 100)
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

// Evaluation constants (strengthened)
const DOUBLED_PAWN_PENALTY: i32 = -30;
const CONNECTED_PAWN_BONUS: i32 = 25;
const KING_PROTECTION_BONUS: i32 = 40;

// Additional evaluation constants (strengthened)
const BISHOP_PAIR_BONUS: i32 = 50;
const ROOK_ON_OPEN_FILE_BONUS: i32 = 35;
const ROOK_ON_SEMI_OPEN_FILE_BONUS: i32 = 25;
const KNIGHT_OUTPOST_BONUS: i32 = 45;
const PAWN_SHIELD_BONUS: i32 = 30;
const PAWN_STORM_BONUS: i32 = 20;
const BACKWARD_PAWN_PENALTY: i32 = -25;
const ISOLATED_PAWN_PENALTY: i32 = -25;
// Increased passed pawn bonuses significantly
const PASSED_PAWN_BONUS: [i32; 8] = [0, 20, 40, 60, 100, 150, 200, 250]; // Indexed by rank

// Mobility bonuses (new)
const MOBILITY_BONUS: [i32; 6] = [
    0,   // King
    4,   // Pawn
    8,   // Knight
    7,   // Bishop
    5,   // Rook
    3,   // Queen
];

// Center control bonuses (new)
const CENTER_CONTROL_BONUS: i32 = 20;
const EXTENDED_CENTER_BONUS: i32 = 10;

// Development bonus (new)
const DEVELOPMENT_BONUS: i32 = 15;

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
    let mut center_control = [0, 0]; // White, Black control of center
    let mut developed_pieces = [0, 0]; // White, Black developed pieces

    let mut white_bishops = 0;
    let mut black_bishops = 0;
    let mut pawn_files = [[false; 8]; 2]; // Track pawn files for both colors

    // Material and positional evaluation
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = get_piece_value(piece.piece_type);
                let position_bonus = get_position_bonus(piece.piece_type, pos, piece.color == Color::White, is_endgame);
                let multiplier = if piece.color == Color::White { 1 } else { -1 };
                
                score += multiplier * (piece_value + position_bonus);

                // Count bishops for bishop pair bonus
                if piece.piece_type == PieceType::Bishop {
                    if piece.color == Color::White {
                        white_bishops += 1;
                    } else {
                        black_bishops += 1;
                    }
                }

                // Track pawn files and evaluate center control
                match piece.piece_type {
                    PieceType::Pawn => {
                        pawn_files[if piece.color == Color::White { 0 } else { 1 }][file as usize - 1] = true;
                        // Evaluate center control for pawns
                        if (3..=6).contains(&rank) && (3..=6).contains(&file) {
                            if piece.color == Color::White {
                                center_control[0] += 1;
                            } else {
                                center_control[1] += 1;
                            }
                        }
                    }
                    PieceType::Knight | PieceType::Bishop => {
                        // Count developed pieces
                        if (piece.color == Color::White && rank > 2) || 
                           (piece.color == Color::Black && rank < 7) {
                            if piece.color == Color::White {
                                developed_pieces[0] += 1;
                            } else {
                                developed_pieces[1] += 1;
                            }
                        }
                        // Evaluate center control
                        let moves = board.get_valid_moves(pos);
                        for mv in moves {
                            if (3..=6).contains(&mv.to.rank) && (3..=6).contains(&mv.to.file) {
                                if piece.color == Color::White {
                                    center_control[0] += 1;
                                } else {
                                    center_control[1] += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Evaluate piece-specific features
                match piece.piece_type {
                    PieceType::Pawn => {
                        score += multiplier * evaluate_pawn_structure(board, pos, piece.color == Color::White);
                        score += multiplier * evaluate_passed_pawn(board, pos, piece.color == Color::White);
                    }
                    PieceType::Knight => {
                        score += multiplier * evaluate_knight(board, pos, piece.color == Color::White);
                    }
                    PieceType::Bishop => {
                        score += multiplier * evaluate_bishop(board, pos, piece.color == Color::White);
                    }
                    PieceType::Rook => {
                        score += multiplier * evaluate_rook(board, pos, piece.color == Color::White);
                    }
                    PieceType::King => {
                        if !is_endgame {
                            score += multiplier * evaluate_king_safety(board, pos, piece.color == Color::White);
                            score += multiplier * evaluate_pawn_shield(board, pos, piece.color == Color::White);
                        }
                    }
                    _ => {}
                }

                // Add mobility bonus
                let moves = board.get_valid_moves(pos);
                score += multiplier * (moves.len() as i32 * MOBILITY_BONUS[piece.piece_type as usize]);
            }
        }
    }

    // Bishop pair bonus
    if white_bishops >= 2 {
        score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        score -= BISHOP_PAIR_BONUS;
    }

    // Add center control bonus
    score += (center_control[0] - center_control[1]) * CENTER_CONTROL_BONUS;

    // Add development bonus in opening/middlegame
    if !is_endgame {
        score += (developed_pieces[0] - developed_pieces[1]) * DEVELOPMENT_BONUS;
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

fn evaluate_passed_pawn(board: &Board, pos: Position, is_white: bool) -> i32 {
    let file = pos.file;
    let rank = if is_white { pos.rank } else { 9 - pos.rank };
    let mut is_passed = true;
    let color = if is_white { Color::White } else { Color::Black };

    // Check if there are any enemy pawns that can block or capture
    let enemy_pawn_ranks = if is_white { (pos.rank + 1)..=7 } else { 2..=pos.rank };
    for check_file in (file - 1).max(1)..=(file + 1).min(8) {
        for check_rank in enemy_pawn_ranks.clone() {
            let check_pos = Position { rank: check_rank, file: check_file };
            if let Some(piece) = board.get_piece(check_pos) {
                if piece.piece_type == PieceType::Pawn && piece.color != color {
                    is_passed = false;
                    break;
                }
            }
        }
        if !is_passed {
            break;
        }
    }

    if is_passed {
        PASSED_PAWN_BONUS[rank as usize - 1]
    } else {
        0
    }
}

fn evaluate_knight(board: &Board, pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    let rank = if is_white { pos.rank } else { 9 - pos.rank };
    let file = pos.file;

    // Check if knight is on outpost (protected by pawn, can't be attacked by enemy pawns)
    let is_outpost = is_white && rank >= 4 || !is_white && rank <= 5;
    if is_outpost {
        let protected_by_pawn = if is_white {
            has_friendly_pawn_protection(board, pos, Color::White)
        } else {
            has_friendly_pawn_protection(board, pos, Color::Black)
        };
        
        let safe_from_enemy_pawns = if is_white {
            !can_be_attacked_by_enemy_pawns(board, pos, Color::White)
        } else {
            !can_be_attacked_by_enemy_pawns(board, pos, Color::Black)
        };

        if protected_by_pawn && safe_from_enemy_pawns {
            score += KNIGHT_OUTPOST_BONUS;
        }
    }

    score
}

fn evaluate_bishop(board: &Board, pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    
    // Evaluate bishop mobility (number of squares it can move to)
    let moves = board.get_valid_moves(pos);
    score += moves.len() as i32 * 4;

    score
}

fn evaluate_rook(board: &Board, pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    let file = pos.file;

    // Check if rook is on open or semi-open file
    let mut has_friendly_pawn = false;
    let mut has_enemy_pawn = false;

    for rank in 1..=8 {
        let check_pos = Position { rank, file };
        if let Some(piece) = board.get_piece(check_pos) {
            if piece.piece_type == PieceType::Pawn {
                if piece.color == if is_white { Color::White } else { Color::Black } {
                    has_friendly_pawn = true;
                } else {
                    has_enemy_pawn = true;
                }
            }
        }
    }

    if !has_friendly_pawn && !has_enemy_pawn {
        score += ROOK_ON_OPEN_FILE_BONUS;
    } else if !has_friendly_pawn && has_enemy_pawn {
        score += ROOK_ON_SEMI_OPEN_FILE_BONUS;
    }

    score
}

fn evaluate_pawn_shield(board: &Board, king_pos: Position, is_white: bool) -> i32 {
    let mut score = 0;
    let base_rank = if is_white { 1 } else { 8 };
    let pawn_rank = if is_white { 2 } else { 7 };
    let advance_rank = if is_white { 3 } else { 6 };
    let color = if is_white { Color::White } else { Color::Black };

    // Only evaluate pawn shield for kings on the kingside or queenside
    if king_pos.rank == base_rank && (king_pos.file <= 3 || king_pos.file >= 6) {
        let shield_files = if king_pos.file <= 3 {
            1..=3 // Queenside
        } else {
            6..=8 // Kingside
        };

        // Check pawns in front of king
        for file in shield_files {
            let shield_pos = Position { rank: pawn_rank, file };
            let advance_pos = Position { rank: advance_rank, file };
            
            if let Some(piece) = board.get_piece(shield_pos) {
                if piece.piece_type == PieceType::Pawn && piece.color == color {
                    score += PAWN_SHIELD_BONUS;
                }
            }
            
            // Bonus for advanced shield pawns
            if let Some(piece) = board.get_piece(advance_pos) {
                if piece.piece_type == PieceType::Pawn && piece.color == color {
                    score += PAWN_STORM_BONUS;
                }
            }
        }
    }

    score
}

fn has_friendly_pawn_protection(board: &Board, pos: Position, color: Color) -> bool {
    let protect_rank = if color == Color::White { pos.rank - 1 } else { pos.rank + 1 };
    if protect_rank < 1 || protect_rank > 8 {
        return false;
    }

    for file_offset in [-1, 1].iter() {
        let protect_file = pos.file as i32 + file_offset;
        if protect_file >= 1 && protect_file <= 8 {
            let protect_pos = Position { rank: protect_rank, file: protect_file as u8 };
            if let Some(piece) = board.get_piece(protect_pos) {
                if piece.piece_type == PieceType::Pawn && piece.color == color {
                    return true;
                }
            }
        }
    }
    false
}

fn can_be_attacked_by_enemy_pawns(board: &Board, pos: Position, friendly_color: Color) -> bool {
    let enemy_color = if friendly_color == Color::White { Color::Black } else { Color::White };
    let attack_rank = if friendly_color == Color::White { pos.rank + 1 } else { pos.rank - 1 };
    if attack_rank < 1 || attack_rank > 8 {
        return false;
    }

    for file_offset in [-1, 1].iter() {
        let attack_file = pos.file as i32 + file_offset;
        if attack_file >= 1 && attack_file <= 8 {
            let attack_pos = Position { rank: attack_rank, file: attack_file as u8 };
            if let Some(piece) = board.get_piece(attack_pos) {
                if piece.piece_type == PieceType::Pawn && piece.color == enemy_color {
                    return true;
                }
            }
        }
    }
    false
} 