use chess_core::{Board, Move};
use crate::evaluation::evaluate_position;

const ALPHA_INIT: i32 = i32::MIN + 1;
const BETA_INIT: i32 = i32::MAX;

pub fn search_best_move(board: &Board, depth: u8) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;

    // Generate all possible moves
    let mut moves = Vec::new();
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    moves.extend(board.get_valid_moves(pos));
                }
            }
        }
    }

    // Sort moves to improve alpha-beta pruning efficiency
    moves.sort_by_cached_key(|m| {
        if board.get_piece(m.to).is_some() {
            // Prioritize captures based on piece values
            if let Some(captured) = board.get_piece(m.to) {
                match captured.piece_type {
                    chess_core::piece::PieceType::Queen => 0,
                    chess_core::piece::PieceType::Rook => 1,
                    chess_core::piece::PieceType::Bishop |
                    chess_core::piece::PieceType::Knight => 2,
                    chess_core::piece::PieceType::Pawn => 3,
                    _ => 4,
                }
            } else {
                5
            }
        } else {
            6 // Non-captures last
        }
    });

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax(&new_board, depth - 1, -BETA_INIT, -best_score);
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
            }
        }
    }

    best_move
}

fn negamax(board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate_position(board);
    }

    let mut best_score = ALPHA_INIT;

    // Generate all possible moves
    let mut moves = Vec::new();
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    moves.extend(board.get_valid_moves(pos));
                }
            }
        }
    }

    if moves.is_empty() {
        // If no moves are available, it's either checkmate or stalemate
        return if board.is_checkmate() {
            ALPHA_INIT // Checkmate
        } else {
            0 // Stalemate
        };
    }

    // Sort moves to improve alpha-beta pruning efficiency
    moves.sort_by_cached_key(|m| {
        if board.get_piece(m.to).is_some() {
            // Prioritize captures based on piece values
            if let Some(captured) = board.get_piece(m.to) {
                match captured.piece_type {
                    chess_core::piece::PieceType::Queen => 0,
                    chess_core::piece::PieceType::Rook => 1,
                    chess_core::piece::PieceType::Bishop |
                    chess_core::piece::PieceType::Knight => 2,
                    chess_core::piece::PieceType::Pawn => 3,
                    _ => 4,
                }
            } else {
                5
            }
        } else {
            6 // Non-captures last
        }
    });

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax(&new_board, depth - 1, -beta, -alpha);
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break; // Beta cutoff
            }
        }
    }

    best_score
} 