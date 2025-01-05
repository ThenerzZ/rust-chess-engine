use chess_core::{Board, Position, Color, PieceType};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Penalties and bonuses
const DOUBLED_PAWN_PENALTY: i32 = -10;
const ISOLATED_PAWN_PENALTY: i32 = -20;
const PASSED_PAWN_BONUS: i32 = 30;
const BISHOP_PAIR_BONUS: i32 = 30;
const MOBILITY_MULTIPLIER: i32 = 5;

pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;
    
    // Material and basic positional evaluation
    score += evaluate_material(board);
    
    // Pawn structure
    score += evaluate_pawn_structure(board);
    
    // Piece mobility
    score += evaluate_mobility(board);
    
    // Bishop pair bonus
    score += evaluate_bishop_pair(board);
    
    // Return score relative to current player
    if board.current_turn() == Color::White {
        score
    } else {
        -score
    }
}

fn evaluate_material(board: &Board) -> i32 {
    let mut score = 0;
    
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = match piece.piece_type {
                    PieceType::Pawn => PAWN_VALUE,
                    PieceType::Knight => KNIGHT_VALUE,
                    PieceType::Bishop => BISHOP_VALUE,
                    PieceType::Rook => ROOK_VALUE,
                    PieceType::Queen => QUEEN_VALUE,
                    PieceType::King => 0, // King's value not counted in material
                };
                
                if piece.color == Color::White {
                    score += piece_value;
                } else {
                    score -= piece_value;
                }
            }
        }
    }
    
    score
}

fn evaluate_pawn_structure(board: &Board) -> i32 {
    let mut score = 0;
    
    // Evaluate each file
    for file in 1..=8 {
        let mut white_pawns = 0;
        let mut black_pawns = 0;
        let mut white_pawn_ranks = Vec::new();
        let mut black_pawn_ranks = Vec::new();
        
        // Count pawns in this file
        for rank in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::Pawn {
                    if piece.color == Color::White {
                        white_pawns += 1;
                        white_pawn_ranks.push(rank);
                    } else {
                        black_pawns += 1;
                        black_pawn_ranks.push(rank);
                    }
                }
            }
        }
        
        // Doubled pawns
        if white_pawns > 1 {
            score += DOUBLED_PAWN_PENALTY * (white_pawns - 1);
        }
        if black_pawns > 1 {
            score -= DOUBLED_PAWN_PENALTY * (black_pawns - 1);
        }
        
        // Isolated pawns
        let has_neighbor_pawn = |color: Color| {
            for neighbor_file in (file - 1).max(1)..=(file + 1).min(8) {
                if neighbor_file == file {
                    continue;
                }
                for rank in 1..=8 {
                    let pos = Position { rank, file: neighbor_file };
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.piece_type == PieceType::Pawn && piece.color == color {
                            return true;
                        }
                    }
                }
            }
            false
        };
        
        if white_pawns > 0 && !has_neighbor_pawn(Color::White) {
            score += ISOLATED_PAWN_PENALTY;
        }
        if black_pawns > 0 && !has_neighbor_pawn(Color::Black) {
            score -= ISOLATED_PAWN_PENALTY;
        }
        
        // Passed pawns
        let is_passed_pawn = |rank: u8, color: Color| {
            let ranks_to_check = if color == Color::White {
                (rank + 1)..=8
            } else {
                1..=(rank - 1)
            };
            
            for check_file in (file - 1).max(1)..=(file + 1).min(8) {
                for check_rank in ranks_to_check.clone() {
                    let pos = Position { rank: check_rank, file: check_file };
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.piece_type == PieceType::Pawn && piece.color != color {
                            return false;
                        }
                    }
                }
            }
            true
        };
        
        for rank in white_pawn_ranks {
            if is_passed_pawn(rank, Color::White) {
                score += PASSED_PAWN_BONUS;
            }
        }
        for rank in black_pawn_ranks {
            if is_passed_pawn(rank, Color::Black) {
                score -= PASSED_PAWN_BONUS;
            }
        }
    }
    
    score
}

fn evaluate_mobility(board: &Board) -> i32 {
    let mut score = 0;
    
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let moves = board.get_valid_moves(pos);
                let mobility = (moves.len() as i32) * MOBILITY_MULTIPLIER;
                
                if piece.color == Color::White {
                    score += mobility;
                } else {
                    score -= mobility;
                }
            }
        }
    }
    
    score
}

fn evaluate_bishop_pair(board: &Board) -> i32 {
    let mut white_bishops = 0;
    let mut black_bishops = 0;
    
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::Bishop {
                    if piece.color == Color::White {
                        white_bishops += 1;
                    } else {
                        black_bishops += 1;
                    }
                }
            }
        }
    }
    
    let mut score = 0;
    if white_bishops >= 2 {
        score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        score -= BISHOP_PAIR_BONUS;
    }
    
    score
} 