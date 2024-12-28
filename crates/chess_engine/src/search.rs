use chess_core::{Board, Move, Position};
use crate::evaluation::evaluate_position;

// Constants for the minimax algorithm
const MAX_SCORE: i32 = 1_000_000;
const MIN_SCORE: i32 = -1_000_000;

/// Searches for the best move in the current position
pub fn search_best_move(board: &Board, depth: u8) -> Option<Move> {
    let mut alpha = MIN_SCORE;
    let beta = MAX_SCORE;
    let mut best_move = None;
    let mut best_score = MIN_SCORE;

    // Generate all legal moves
    let mut moves = Vec::new();
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

    // Sort moves to improve alpha-beta pruning efficiency
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
            let score = -negamax(&new_board, depth - 1, -beta, -alpha);
            
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
    }

    best_move
}

/// Negamax implementation of minimax with alpha-beta pruning
fn negamax(board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate_position(board);
    }

    if board.is_checkmate() {
        return MIN_SCORE;
    }

    if board.is_stalemate() || board.has_insufficient_material() {
        return 0;
    }

    let mut max_score = MIN_SCORE;
    let mut moves = Vec::new();

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

    moves.sort_by_key(|m| {
        if board.get_piece(m.to).is_some() {
            1
        } else {
            0
        }
    });

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax(&new_board, depth - 1, -beta, -alpha);
            max_score = max_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
    }

    max_score
} 