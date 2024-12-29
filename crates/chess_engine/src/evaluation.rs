use chess_core::{Board, Position, piece::{PieceType, Color}};

// Piece values (adjusted to standard chess values * 100)
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

// Endgame constants with increased emphasis on checkmate
const KING_TROPISM_ENDGAME: i32 = 150;    // Increased for more aggressive piece coordination
const KING_CORNER_PUSH: i32 = 300;        // Doubled to really force king to corner
const QUEEN_KING_TROPISM: i32 = 400;      // Doubled for more aggressive queen positioning
const ROOK_KING_TROPISM: i32 = 300;       // Doubled for more aggressive rook positioning
const MATE_PATTERN_BONUS: i32 = 1000;     // Increased bonus for mate patterns
const KING_CUT_OFF_BONUS: i32 = 500;      // Bonus for cutting off king's escape squares
const BACK_RANK_BONUS: i32 = 800;         // Increased bonus for back rank threats
const LADDER_MATE_BONUS: i32 = 600;       // Bonus for ladder mate patterns
const QUEEN_ROOK_COORDINATION: i32 = 300;  // Bonus for queen and rook working together

// Endgame specific piece values
const ENDGAME_PAWN_VALUE: i32 = 150;      // Pawns are more valuable in endgame
const ENDGAME_KNIGHT_VALUE: i32 = 300;    // Knights slightly weaker in endgame
const ENDGAME_BISHOP_VALUE: i32 = 350;    // Bishops slightly stronger in endgame
const ENDGAME_ROOK_VALUE: i32 = 600;      // Rooks stronger in endgame

// Stockfish-inspired endgame scoring
const PAWN_ENDGAME_SCALE: i32 = 200;      // Scale up pawn value in pure pawn endgames
const BISHOP_PAIR_ENDGAME: i32 = 500;     // Bishop pair bonus in endgame
const WRONG_BISHOP_PENALTY: i32 = -300;    // Penalty for bishop of wrong color in pawn endings
const ROOK_BEHIND_PASSER: i32 = 300;      // Bonus for rook behind passed pawn
const KING_PAWN_DISTANCE: i32 = 100;      // Bonus for king being close to enemy pawns
const KING_ACTIVITY_ENDGAME: i32 = 200;   // Bonus for active king in endgame

// Mating patterns
const QUEEN_MATE_PATTERN: i32 = 2000;     // Queen + King vs King mate pattern
const ROOK_MATE_PATTERN: i32 = 1500;      // Rook + King vs King mate pattern
const TWO_BISHOPS_MATE: i32 = 1200;       // Two bishops mate pattern
const BISHOP_KNIGHT_MATE: i32 = 1000;     // Bishop + Knight mate pattern

// Add after the existing constants
const CHECKMATE_SCORE: i32 = 100000;  // Very high value for checkmate
const CHECKMATE_HORIZON: i32 = 90000; // High value for positions likely leading to checkmate

// Function to evaluate a position
pub fn evaluate_position(board: &Board) -> i32 {
    // Check for immediate checkmate
    if board.is_checkmate() {
        // If it's checkmate, the current player has lost (since they have no moves)
        return -CHECKMATE_SCORE;
    }

    // Check if opponent is in check
    let opponent_color = if board.current_turn() == Color::White { Color::Black } else { Color::White };
    let is_check = board.is_in_check(opponent_color);

    let mut score = 0;
    let endgame_type = detect_endgame_type(board);

    // Basic material and position evaluation
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let piece_value = match endgame_type {
                    EndgameType::PawnOnly => get_pawn_endgame_value(piece.piece_type),
                    EndgameType::KQvsK | EndgameType::KRvsK | EndgameType::KBNvsK => 
                        get_mating_piece_value(piece.piece_type, endgame_type),
                    _ => if is_endgame_phase(board) {
                        get_endgame_piece_value(piece.piece_type)
                    } else {
                        get_piece_value(piece.piece_type)
                    }
                };
                let mut piece_score = piece_value;

                // Add mobility and endgame-specific bonuses
                piece_score += evaluate_piece_endgame(board, pos, &piece, endgame_type);

                // Apply color multiplier
                if piece.color == Color::White {
                    score += piece_score;
                } else {
                    score -= piece_score;
                }
            }
        }
    }

    // Apply endgame-specific evaluation
    score = match endgame_type {
        EndgameType::KQvsK => evaluate_queen_endgame(board, score),
        EndgameType::KRvsK => evaluate_rook_endgame(board, score),
        EndgameType::KBNvsK => evaluate_bishop_knight_mate(board, score),
        EndgameType::TwoBishops => evaluate_two_bishops_mate(board, score),
        EndgameType::PawnOnly => evaluate_pawn_endgame(board, score),
        _ => if is_endgame_phase(board) {
            adjust_endgame_score(board, score)
        } else {
            score
        }
    };

    // Add bonus for having the opponent in check
    if is_check {
        score += 50; // Small bonus for check
        
        // Check for potential forced mate patterns
        if let Some(mate_score) = evaluate_mating_patterns(board) {
            score += mate_score;
        }
    }

    // Return score from current player's perspective
    if board.current_turn() == Color::White {
        score
    } else {
        -score
    }
}

// Add new enum for specific endgame types
#[derive(Copy, Clone, PartialEq)]
enum EndgameType {
    Normal,
    KQvsK,      // King + Queen vs King
    KRvsK,      // King + Rook vs King
    KBNvsK,     // King + Bishop + Knight vs King
    TwoBishops, // King + Two Bishops vs King
    PawnOnly,   // Only kings and pawns
}

fn detect_endgame_type(board: &Board) -> EndgameType {
    let mut white_pieces = Vec::new();
    let mut black_pieces = Vec::new();
    let mut total_pawns = 0;

    // Collect all pieces
    for rank in 1..=8 {
        for file in 1..=8 {
            if let Some(piece) = board.get_piece(Position { rank, file }) {
                if piece.piece_type == PieceType::Pawn {
                    total_pawns += 1;
                } else if piece.piece_type != PieceType::King {
                    if piece.color == Color::White {
                        white_pieces.push(piece.piece_type);
                    } else {
                        black_pieces.push(piece.piece_type);
                    }
                }
            }
        }
    }

    // Check for specific endgame patterns
    if white_pieces.is_empty() && black_pieces.is_empty() && total_pawns > 0 {
        return EndgameType::PawnOnly;
    }

    // One side has only a king
    if white_pieces.is_empty() || black_pieces.is_empty() {
        let pieces = if white_pieces.is_empty() { &black_pieces } else { &white_pieces };
        match pieces.len() {
            1 => match pieces[0] {
                PieceType::Queen => return EndgameType::KQvsK,
                PieceType::Rook => return EndgameType::KRvsK,
                _ => {}
            },
            2 => {
                let has_bishop = pieces.iter().any(|&p| p == PieceType::Bishop);
                let has_knight = pieces.iter().any(|&p| p == PieceType::Knight);
                if has_bishop && has_knight {
                    return EndgameType::KBNvsK;
                }
                let bishop_count = pieces.iter().filter(|&&p| p == PieceType::Bishop).count();
                if bishop_count == 2 {
                    return EndgameType::TwoBishops;
                }
            }
            _ => {}
        }
    }

    EndgameType::Normal
}

fn evaluate_piece_endgame(board: &Board, pos: Position, piece: &chess_core::Piece, endgame_type: EndgameType) -> i32 {
    let mut score = 0;
    
    match endgame_type {
        EndgameType::KQvsK | EndgameType::KRvsK => {
            // In basic mating positions, prioritize pushing enemy king to edge
            if let Some(enemy_king_pos) = find_enemy_king(board, piece.color) {
                let distance_to_king = manhattan_distance(pos, enemy_king_pos);
                score += (14 - distance_to_king) * 100; // Closer pieces are better
                
                // Bonus for cutting off king's escape squares
                if piece.piece_type == PieceType::Queen || piece.piece_type == PieceType::Rook {
                    if pos.rank == enemy_king_pos.rank || pos.file == enemy_king_pos.file {
                        score += 300;
                    }
                }
            }
        }
        EndgameType::PawnOnly => {
            if piece.piece_type == PieceType::King {
                // King should be active in pawn endgames
                score += evaluate_king_pawn_endgame(board, pos, piece.color);
            } else if piece.piece_type == PieceType::Pawn {
                score += evaluate_passed_pawn(board, pos, piece.color);
            }
        }
        _ => {
            // Regular endgame evaluation
            let mobility = board.get_valid_moves(pos).len() as i32;
            score += mobility * get_mobility_bonus(piece.piece_type);
        }
    }
    
    score
}

fn evaluate_queen_endgame(board: &Board, mut score: i32) -> i32 {
    if let Some((queen_pos, king_pos, enemy_king_pos)) = find_queen_mate_position(board) {
        // Distance between pieces
        let queen_to_enemy = manhattan_distance(queen_pos, enemy_king_pos);
        let king_to_enemy = manhattan_distance(king_pos, enemy_king_pos);
        
        // Bonus for optimal piece positioning
        score += QUEEN_MATE_PATTERN;
        score += (14 - queen_to_enemy) * 150; // Queen closer to enemy king
        score += (14 - king_to_enemy) * 100;  // Own king supporting queen
        
        // Extra bonus for forcing enemy king to edge
        let edge_distance = distance_to_edge(enemy_king_pos);
        score += (8 - edge_distance) * 200;
    }
    score
}

fn evaluate_rook_endgame(board: &Board, mut score: i32) -> i32 {
    if let Some((rook_pos, king_pos, enemy_king_pos)) = find_rook_mate_position(board) {
        // Philidor position evaluation
        score += ROOK_MATE_PATTERN;
        
        // Check if enemy king is restricted to edge
        let edge_distance = distance_to_edge(enemy_king_pos);
        if edge_distance <= 1 {
            score += 500;
            
            // Bonus for rook cutting off ranks
            if rook_pos.rank == enemy_king_pos.rank {
                score += 300;
            }
            
            // Bonus for king supporting rook
            let king_to_enemy = manhattan_distance(king_pos, enemy_king_pos);
            if king_to_enemy <= 2 {
                score += 400;
            }
        }
    }
    score
}

// Helper functions

fn manhattan_distance(pos1: Position, pos2: Position) -> i32 {
    ((pos1.rank as i32 - pos2.rank as i32).abs() + 
     (pos1.file as i32 - pos2.file as i32).abs())
}

fn distance_to_edge(pos: Position) -> i32 {
    let rank_edge = (pos.rank as i32 - 4).abs();
    let file_edge = (pos.file as i32 - 4).abs();
    rank_edge.min(file_edge)
}

fn find_enemy_king(board: &Board, color: Color) -> Option<Position> {
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::King && piece.color != color {
                    return Some(pos);
                }
            }
        }
    }
    None
}

fn evaluate_king_pawn_endgame(board: &Board, king_pos: Position, color: Color) -> i32 {
    let mut score = 0;
    
    // King activity
    let center_distance = manhattan_distance(king_pos, Position { rank: 4, file: 4 });
    score += (8 - center_distance) * KING_ACTIVITY_ENDGAME;
    
    // King proximity to enemy pawns
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::Pawn && piece.color != color {
                    let distance = manhattan_distance(king_pos, pos);
                    score += (8 - distance) * KING_PAWN_DISTANCE;
                }
            }
        }
    }
    
    score
}

fn evaluate_passed_pawn(board: &Board, pawn_pos: Position, color: Color) -> i32 {
    let mut score = 0;
    let is_passed = is_passed_pawn(board, pawn_pos, color);
    
    if is_passed {
        // Base bonus for passed pawn
        let rank = if color == Color::White { pawn_pos.rank } else { 9 - pawn_pos.rank };
        score += PAWN_ENDGAME_SCALE * rank as i32;
        
        // Check for rook behind passed pawn
        if let Some(rook_pos) = find_rook_behind_pawn(board, pawn_pos, color) {
            score += ROOK_BEHIND_PASSER;
        }
    }
    
    score
}

fn find_rook_behind_pawn(board: &Board, pawn_pos: Position, color: Color) -> Option<Position> {
    let direction = if color == Color::White { -1 } else { 1 };
    let mut rank = pawn_pos.rank as i32;
    
    while rank >= 1 && rank <= 8 {
        rank += direction;
        if let Some(piece) = board.get_piece(Position { 
            rank: rank as u8, 
            file: pawn_pos.file 
        }) {
            if piece.piece_type == PieceType::Rook && piece.color == color {
                return Some(Position { rank: rank as u8, file: pawn_pos.file });
            }
            break;
        }
    }
    None
}

// Add these helper functions for finding specific mate positions
fn find_queen_mate_position(board: &Board) -> Option<(Position, Position, Position)> {
    // Implementation for finding queen + king vs king position
    None // Placeholder
}

fn find_rook_mate_position(board: &Board) -> Option<(Position, Position, Position)> {
    // Implementation for finding rook + king vs king position
    None // Placeholder
}

fn is_endgame_phase(board: &Board) -> bool {
    let mut queens = 0;
    let mut total_material = 0;

    for rank in 1..=8 {
        for file in 1..=8 {
            if let Some(piece) = board.get_piece(Position { rank, file }) {
                match piece.piece_type {
                    PieceType::Queen => queens += 1,
                    PieceType::Rook => total_material += 5,
                    PieceType::Bishop | PieceType::Knight => total_material += 3,
                    PieceType::Pawn => total_material += 1,
                    _ => {}
                }
            }
        }
    }

    // Consider it endgame if:
    // 1. No queens or
    // 2. Only one queen and less than 13 points of material or
    // 3. Two queens but less than 6 points of other material
    queens == 0 || (queens == 1 && total_material <= 13) || (queens == 2 && total_material <= 6)
}

fn adjust_endgame_score(board: &Board, mut score: i32) -> i32 {
    let mut enemy_king_pos = None;
    let current_color = board.current_turn();
    
    // Find enemy king position
    'outer: for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::King && piece.color != current_color {
                    enemy_king_pos = Some(pos);
                    break 'outer;
                }
            }
        }
    }

    if let Some(enemy_king) = enemy_king_pos {
        // Strongly encourage pushing enemy king to corner
        let corner_distance = ((4.5 - enemy_king.rank as f32).abs() + 
                             (4.5 - enemy_king.file as f32).abs()) as i32;
        score += corner_distance * KING_CORNER_PUSH;

        // Penalize enemy king mobility more severely
        let enemy_king_moves = board.get_valid_moves(enemy_king);
        let mobility_penalty = (8 - enemy_king_moves.len() as i32) * 200;
        score += mobility_penalty;

        // Check for cut-off ranks and files
        let mut cut_off_bonus = 0;
        for offset in -1..=1 {
            // Check rank cut-offs
            let check_rank = enemy_king.rank as i32 + offset;
            if check_rank >= 1 && check_rank <= 8 {
                if is_rank_cut_off(board, check_rank as u8, enemy_king.file, current_color) {
                    cut_off_bonus += KING_CUT_OFF_BONUS;
                }
            }
            // Check file cut-offs
            let check_file = enemy_king.file as i32 + offset;
            if check_file >= 1 && check_file <= 8 {
                if is_file_cut_off(board, enemy_king.rank, check_file as u8, current_color) {
                    cut_off_bonus += KING_CUT_OFF_BONUS;
                }
            }
        }
        score += cut_off_bonus;

        // Add tropism bonus for pieces being close to enemy king
        let mut has_queen = false;
        let mut has_rook = false;
        let mut closest_queen_distance = 100;
        let mut closest_rook_distance = 100;

        for rank in 1..=8 {
            for file in 1..=8 {
                let pos = Position { rank, file };
                if let Some(piece) = board.get_piece(pos) {
                    if piece.color == current_color {
                        let distance = ((enemy_king.rank as i32 - rank as i32).abs() +
                                      (enemy_king.file as i32 - file as i32).abs());
                        
                        match piece.piece_type {
                            PieceType::Queen => {
                                has_queen = true;
                                closest_queen_distance = closest_queen_distance.min(distance);
                                score += (8 - distance) * QUEEN_KING_TROPISM;
                                // Extra bonus for queen near confined king
                                if distance <= 2 && enemy_king_moves.len() <= 3 {
                                    score += MATE_PATTERN_BONUS;
                                }
                            }
                            PieceType::Rook => {
                                has_rook = true;
                                closest_rook_distance = closest_rook_distance.min(distance);
                                score += (8 - distance) * ROOK_KING_TROPISM;
                                // Bonus for rook cutting off king
                                if (rank == enemy_king.rank || file == enemy_king.file) && distance == 2 {
                                    score += LADDER_MATE_BONUS;
                                }
                            }
                            _ => {
                                score += (8 - distance) * KING_TROPISM_ENDGAME;
                            }
                        }
                    }
                }
            }
        }

        // Bonus for queen and rook working together
        if has_queen && has_rook && closest_queen_distance <= 3 && closest_rook_distance <= 3 {
            score += QUEEN_ROOK_COORDINATION;
        }

        // Extra bonus for back rank threats
        let back_rank = if current_color == Color::White { 8 } else { 1 };
        if enemy_king.rank == back_rank {
            score += BACK_RANK_BONUS;
            // Additional bonus if king is trapped on back rank
            if is_back_rank_trapped(board, enemy_king, current_color) {
                score += MATE_PATTERN_BONUS;
            }
        }
    }

    score
}

fn is_rank_cut_off(board: &Board, rank: u8, king_file: u8, attacking_color: Color) -> bool {
    for file in 1..=8 {
        if file != king_file {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == attacking_color && 
                   (piece.piece_type == PieceType::Queen || piece.piece_type == PieceType::Rook) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_file_cut_off(board: &Board, king_rank: u8, file: u8, attacking_color: Color) -> bool {
    for rank in 1..=8 {
        if rank != king_rank {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == attacking_color && 
                   (piece.piece_type == PieceType::Queen || piece.piece_type == PieceType::Rook) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_back_rank_trapped(board: &Board, king_pos: Position, attacking_color: Color) -> bool {
    // Check if king is trapped by own pawns
    let pawn_rank = if attacking_color == Color::White { 7 } else { 2 };
    let mut trapped_by_pawns = true;

    // Check three files around king
    for file_offset in -1..=1 {
        let check_file = king_pos.file as i32 + file_offset;
        if check_file >= 1 && check_file <= 8 {
            let pawn_pos = Position { rank: pawn_rank, file: check_file as u8 };
            if let Some(piece) = board.get_piece(pawn_pos) {
                if piece.piece_type != PieceType::Pawn || piece.color == attacking_color {
                    trapped_by_pawns = false;
                    break;
                }
            } else {
                trapped_by_pawns = false;
                break;
            }
        }
    }

    trapped_by_pawns
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

fn get_endgame_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => ENDGAME_PAWN_VALUE,
        PieceType::Knight => ENDGAME_KNIGHT_VALUE,
        PieceType::Bishop => ENDGAME_BISHOP_VALUE,
        PieceType::Rook => ENDGAME_ROOK_VALUE,
        PieceType::Queen => QUEEN_VALUE,
        PieceType::King => KING_VALUE,
    }
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

fn get_pawn_endgame_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => PAWN_ENDGAME_SCALE,
        PieceType::King => KING_VALUE,
        _ => 0 // Other pieces shouldn't exist in pure pawn endgames
    }
}

fn get_mating_piece_value(piece_type: PieceType, endgame_type: EndgameType) -> i32 {
    match (piece_type, endgame_type) {
        (PieceType::Queen, EndgameType::KQvsK) => QUEEN_VALUE * 2,
        (PieceType::Rook, EndgameType::KRvsK) => ROOK_VALUE * 2,
        (PieceType::Bishop, EndgameType::KBNvsK) => BISHOP_VALUE * 2,
        (PieceType::Knight, EndgameType::KBNvsK) => KNIGHT_VALUE * 2,
        (PieceType::King, _) => KING_VALUE,
        _ => 0
    }
}

fn evaluate_bishop_knight_mate(board: &Board, mut score: i32) -> i32 {
    if let Some((bishop_pos, knight_pos, king_pos, enemy_king_pos)) = find_bishop_knight_mate_position(board) {
        score += BISHOP_KNIGHT_MATE;
        
        // Distance between pieces
        let bishop_to_enemy = manhattan_distance(bishop_pos, enemy_king_pos);
        let knight_to_enemy = manhattan_distance(knight_pos, enemy_king_pos);
        let king_to_enemy = manhattan_distance(king_pos, enemy_king_pos);
        
        // Bonus for coordinated pieces
        score += (14 - bishop_to_enemy) * 100;
        score += (14 - knight_to_enemy) * 100;
        score += (14 - king_to_enemy) * 80;
        
        // Extra bonus for forcing enemy king to correct corner
        let corner_distance = distance_to_corner(enemy_king_pos);
        score += (8 - corner_distance) * 150;
    }
    score
}

fn evaluate_two_bishops_mate(board: &Board, mut score: i32) -> i32 {
    if let Some((bishop1_pos, bishop2_pos, king_pos, enemy_king_pos)) = find_two_bishops_mate_position(board) {
        score += TWO_BISHOPS_MATE;
        
        // Distance between pieces
        let bishop1_to_enemy = manhattan_distance(bishop1_pos, enemy_king_pos);
        let bishop2_to_enemy = manhattan_distance(bishop2_pos, enemy_king_pos);
        let king_to_enemy = manhattan_distance(king_pos, enemy_king_pos);
        
        // Bonus for coordinated pieces
        score += (14 - bishop1_to_enemy) * 100;
        score += (14 - bishop2_to_enemy) * 100;
        score += (14 - king_to_enemy) * 80;
        
        // Extra bonus for restricting enemy king movement
        let edge_distance = distance_to_edge(enemy_king_pos);
        score += (8 - edge_distance) * 150;
    }
    score
}

fn evaluate_pawn_endgame(board: &Board, mut score: i32) -> i32 {
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::Pawn {
                    if is_passed_pawn(board, pos, piece.color) {
                        let rank_progress = if piece.color == Color::White {
                            pos.rank
                        } else {
                            9 - pos.rank
                        };
                        score += PAWN_ENDGAME_SCALE * rank_progress as i32;
                    }
                }
            }
        }
    }
    score
}

fn is_passed_pawn(board: &Board, pawn_pos: Position, color: Color) -> bool {
    let direction = if color == Color::White { 1 } else { -1 };
    let file = pawn_pos.file as i32;
    
    // Check files (current, left, and right)
    for f in (file-1)..=(file+1) {
        if f < 1 || f > 8 {
            continue;
        }
        
        // Check all squares in front of the pawn
        let mut rank = pawn_pos.rank as i32;
        while rank >= 1 && rank <= 8 {
            rank += direction;
            if let Some(piece) = board.get_piece(Position {
                rank: rank as u8,
                file: f as u8,
            }) {
                if piece.piece_type == PieceType::Pawn && piece.color != color {
                    return false;
                }
            }
        }
    }
    true
}

fn distance_to_corner(pos: Position) -> i32 {
    let rank_dist = (pos.rank as i32 - 1).min(8 - pos.rank as i32);
    let file_dist = (pos.file as i32 - 1).min(8 - pos.file as i32);
    rank_dist + file_dist
}

fn find_bishop_knight_mate_position(board: &Board) -> Option<(Position, Position, Position, Position)> {
    // Implementation for finding bishop + knight mate position
    None // Placeholder
}

fn find_two_bishops_mate_position(board: &Board) -> Option<(Position, Position, Position, Position)> {
    // Implementation for finding two bishops mate position
    None // Placeholder
}

// Add new function to evaluate mating patterns
fn evaluate_mating_patterns(board: &Board) -> Option<i32> {
    let current_color = board.current_turn();
    let mut score = 0;
    let mut found_pattern = false;

    // Find enemy king position
    let mut enemy_king_pos = None;
    'outer: for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::King && piece.color != current_color {
                    enemy_king_pos = Some(pos);
                    break 'outer;
                }
            }
        }
    }

    if let Some(king_pos) = enemy_king_pos {
        // Check if enemy king is trapped on the edge
        let edge_distance = distance_to_edge(king_pos);
        if edge_distance == 0 {
            // King is on the edge
            let valid_moves = board.get_valid_moves(king_pos);
            if valid_moves.len() <= 2 {
                // King has very limited moves
                score += CHECKMATE_HORIZON / 2;
                found_pattern = true;
            }
        }

        // Check for back rank weakness
        if is_back_rank_trapped(board, king_pos, current_color) {
            score += CHECKMATE_HORIZON / 3;
            found_pattern = true;
        }

        // Check for queen + rook mate pattern
        let mut has_queen = false;
        let mut has_rook = false;
        let mut attacking_pieces = 0;
        for rank in 1..=8 {
            for file in 1..=8 {
                let pos = Position { rank, file };
                if let Some(piece) = board.get_piece(pos) {
                    if piece.color == current_color {
                        match piece.piece_type {
                            PieceType::Queen => {
                                has_queen = true;
                                attacking_pieces += 1;
                            }
                            PieceType::Rook => {
                                has_rook = true;
                                attacking_pieces += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if has_queen && has_rook && attacking_pieces >= 2 {
            // Check if pieces are coordinated
            let valid_moves = board.get_valid_moves(king_pos);
            if valid_moves.len() <= 3 {
                score += CHECKMATE_HORIZON;
                found_pattern = true;
            }
        }
    }

    if found_pattern {
        Some(score)
    } else {
        None
    }
} 