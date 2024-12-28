use chess_core::{Board, piece::{PieceType, Color}};

// Standard piece values used in chess engines, measured in centipawns (100 = 1 pawn)
// These values are based on traditional chess theory
const PAWN_VALUE: i32 = 100;    // Base unit of measurement
const KNIGHT_VALUE: i32 = 320;   // Slightly more than 3 pawns
const BISHOP_VALUE: i32 = 330;   // Slightly more than a knight
const ROOK_VALUE: i32 = 500;     // Worth 5 pawns
const QUEEN_VALUE: i32 = 900;    // Worth 9 pawns
const KING_VALUE: i32 = 20000;   // Very high value as losing king means losing game

// Piece-square tables define bonuses/penalties for piece positions
// Positive values are good positions, negative values are bad
// These tables are from White's perspective (will be flipped for Black)

// Pawn position table:
// - Encourages pawns to advance (higher values in ranks 4,5)
// - Rewards central pawns (higher values in d,e files)
// - Penalizes backward pawns
const PAWN_TABLE: [[i32; 8]; 8] = [
    [0,  0,  0,  0,  0,  0,  0,  0],    // 8th rank (promotion)
    [50, 50, 50, 50, 50, 50, 50, 50],   // 7th rank (near promotion)
    [10, 10, 20, 30, 30, 20, 10, 10],   // 6th rank
    [5,  5, 10, 25, 25, 10,  5,  5],    // 5th rank (center control)
    [0,  0,  0, 20, 20,  0,  0,  0],    // 4th rank
    [5, -5,-10,  0,  0,-10, -5,  5],    // 3rd rank
    [5, 10, 10,-20,-20, 10, 10,  5],    // 2nd rank (starting position)
    [0,  0,  0,  0,  0,  0,  0,  0]     // 1st rank
];

// Knight position table:
// - Encourages knights to control center
// - Penalizes edge positions (knights are less effective there)
const KNIGHT_TABLE: [[i32; 8]; 8] = [
    [-50,-40,-30,-30,-30,-30,-40,-50],   // Edge penalties
    [-40,-20,  0,  0,  0,  0,-20,-40],
    [-30,  0, 10, 15, 15, 10,  0,-30],
    [-30,  5, 15, 20, 20, 15,  5,-30],   // Center control
    [-30,  0, 15, 20, 20, 15,  0,-30],
    [-30,  5, 10, 15, 15, 10,  5,-30],
    [-40,-20,  0,  5,  5,  0,-20,-40],
    [-50,-40,-30,-30,-30,-30,-40,-50]    // Edge penalties
];

// Bishop position table:
// - Encourages bishops to control long diagonals
// - Rewards central control
// - Penalizes edge positions
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

// Rook position table:
// - Encourages rooks to control 7th rank
// - Slight bonus for central files
const ROOK_TABLE: [[i32; 8]; 8] = [
    [0,  0,  0,  0,  0,  0,  0,  0],
    [5, 10, 10, 10, 10, 10, 10,  5],    // 7th rank bonus
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [0,  0,  0,  5,  5,  0,  0,  0]     // Slight center bonus
];

// Queen position table:
// - Similar to rook but with more emphasis on center control
// - Penalizes early development (should keep queen protected early game)
const QUEEN_TABLE: [[i32; 8]; 8] = [
    [-20,-10,-10, -5, -5,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5,  5,  5,  5,  0,-10],
    [-5,  0,  5,  5,  5,  5,  0, -5],
    [0,  0,  5,  5,  5,  5,  0, -5],
    [-10,  5,  5,  5,  5,  5,  0,-10],
    [-10,  0,  5,  0,  0,  0,  0,-10],
    [-20,-10,-10, -5, -5,-10,-10,-20]
];

// King middle game table:
// - Encourages king safety (castling)
// - Heavy penalties for central positions
// - Different table would be used for endgame
const KING_MIDDLE_GAME_TABLE: [[i32; 8]; 8] = [
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-20,-30,-30,-40,-40,-30,-30,-20],
    [-10,-20,-20,-20,-20,-20,-20,-10],
    [20, 20,  0,  0,  0,  0, 20, 20],   // Castled position bonus
    [20, 30, 10,  0,  0, 10, 30, 20]    // Starting rank, encourages castling
];

/// Evaluates a chess position and returns a score from White's perspective
/// Positive scores favor White, negative scores favor Black
pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;
    
    // First component: Material and position evaluation
    // Loop through all pieces and sum their material value plus position bonus
    for (pos, piece) in board.get_all_pieces() {
        let piece_value = get_piece_value(piece.piece_type);
        let position_bonus = get_position_bonus(piece.piece_type, *pos, piece.color);
        
        let value = piece_value + position_bonus;
        if piece.color == Color::White {
            score += value;
        } else {
            score -= value;
        }
    }

    // Second component: Mobility evaluation
    // More legal moves = better position
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let mobility = board.get_valid_moves(pos).len() as i32;
                if piece.color == Color::White {
                    score += mobility * 5; // Each legal move worth 5 centipawns
                } else {
                    score -= mobility * 5;
                }
            }
        }
    }

    // Third component: Game state evaluation
    // Being in check is bad, giving check is good
    if board.is_in_check(Color::White) {
        score -= 50; // Penalty for being in check
    }
    if board.is_in_check(Color::Black) {
        score += 50; // Bonus for giving check
    }

    // Return score from current player's perspective
    if board.current_turn() == Color::White {
        score
    } else {
        -score
    }
}

/// Returns the base material value of a piece
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

/// Returns the position bonus/penalty for a piece at a given position
/// Takes into account the piece's color to flip the table for Black pieces
fn get_position_bonus(piece_type: PieceType, pos: chess_core::Position, color: Color) -> i32 {
    // Convert board position to table indices
    // For Black pieces, we flip the table vertically and horizontally
    let (rank_idx, file_idx) = if color == Color::White {
        (8 - pos.rank as usize, pos.file as usize - 1)
    } else {
        (pos.rank as usize - 1, 8 - pos.file as usize)
    };

    // Return the position bonus from the appropriate table
    match piece_type {
        PieceType::Pawn => PAWN_TABLE[rank_idx][file_idx],
        PieceType::Knight => KNIGHT_TABLE[rank_idx][file_idx],
        PieceType::Bishop => BISHOP_TABLE[rank_idx][file_idx],
        PieceType::Rook => ROOK_TABLE[rank_idx][file_idx],
        PieceType::Queen => QUEEN_TABLE[rank_idx][file_idx],
        PieceType::King => KING_MIDDLE_GAME_TABLE[rank_idx][file_idx],
    }
} 