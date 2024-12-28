use chess_core::{Board, Move, Position};
use crate::evaluation::evaluate_position;

// Constants for the minimax algorithm
// These values represent "infinity" in our scoring system
// They're slightly less than i32::MAX to prevent overflow during negation
const MAX_SCORE: i32 = 1_000_000;    // Represents a winning position
const MIN_SCORE: i32 = -1_000_000;   // Represents a losing position

/// Searches for the best move in the current position
/// Uses minimax algorithm with alpha-beta pruning
pub fn search_best_move(board: &Board) -> Option<Move> {
    let depth = 4;  // Search depth (number of plies to look ahead)
    let mut alpha = MIN_SCORE;  // Best score that White can achieve
    let beta = MAX_SCORE;       // Best score that Black can achieve
    let mut best_move = None;   // Best move found so far
    let mut best_score = MIN_SCORE;  // Score of the best move

    // First, generate all legal moves in the current position
    let mut moves = Vec::new();
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                // Only generate moves for pieces of the current player
                if piece.color == board.current_turn() {
                    moves.extend(board.get_valid_moves(pos));
                }
            }
        }
    }

    // Sort moves to improve alpha-beta pruning efficiency
    // Examining captures first often leads to more pruning
    moves.sort_by_key(|m| {
        if board.get_piece(m.to).is_some() {
            1  // Captures first
        } else {
            0  // Non-captures second
        }
    });

    // Try each move and evaluate resulting position
    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            // Recursively evaluate position using negamax
            // Negating opponent's score gives our score
            let score = -negamax(&new_board, depth - 1, -beta, -alpha);
            
            // Update best move if this move scores better
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
            }

            // Update alpha (best score we can achieve)
            alpha = alpha.max(score);
            
            // Beta cutoff: opponent won't allow this position
            if alpha >= beta {
                break;
            }
        }
    }

    best_move
}

/// Negamax implementation of minimax with alpha-beta pruning
/// Returns the best score achievable from the current position
fn negamax(board: &Board, depth: i32, mut alpha: i32, beta: i32) -> i32 {
    // Base case: at maximum depth or terminal position
    if depth == 0 {
        return evaluate_position(board);
    }

    // Check for terminal positions
    if board.is_checkmate() {
        return MIN_SCORE;  // Loss (checkmate)
    }

    if board.is_stalemate() || board.has_insufficient_material() {
        return 0;  // Draw
    }

    // Initialize best score seen so far
    let mut max_score = MIN_SCORE;
    let mut moves = Vec::new();

    // Generate all legal moves
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    moves.extend(board.get_valid_moves(pos));
                }
            }
        }
    }

    // Sort moves (captures first) for better alpha-beta pruning
    moves.sort_by_key(|m| {
        if board.get_piece(m.to).is_some() {
            1
        } else {
            0
        }
    });

    // Try each move recursively
    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            // Recursively evaluate position
            // Negamax: our best move is the negative of opponent's best move
            let score = -negamax(&new_board, depth - 1, -beta, -alpha);
            
            // Update best score
            max_score = max_score.max(score);
            
            // Update alpha (best achievable score)
            alpha = alpha.max(score);
            
            // Beta cutoff (opponent won't allow this position)
            if alpha >= beta {
                break;
            }
        }
    }

    max_score
} 