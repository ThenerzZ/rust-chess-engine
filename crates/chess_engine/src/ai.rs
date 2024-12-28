use chess_core::{Board, Move};
use crate::{search::search_best_move, evaluation::evaluate_position};
use rayon::prelude::*;
use std::time::{Instant, Duration};

const MAX_THINK_TIME_MS: u64 = 1000; // Maximum thinking time in milliseconds

#[derive(Clone)]
pub struct ChessAI {
    search_depth: u8,
    use_parallel: bool,
}

impl ChessAI {
    pub fn new(search_depth: u8) -> Self {
        Self { 
            search_depth,
            use_parallel: true,
        }
    }

    pub fn get_best_move(&self, board: &Board) -> Option<Move> {
        let start_time = Instant::now();
        let max_time = Duration::from_millis(MAX_THINK_TIME_MS);
        
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
            return None;
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

        // Quick evaluation of all moves in parallel
        let mut move_scores: Vec<(Move, i32)> = if self.use_parallel {
            moves.par_iter()
                .map(|&chess_move| {
                    let mut new_board = board.clone();
                    if new_board.make_move(chess_move).is_ok() {
                        (chess_move, evaluate_position(&new_board))
                    } else {
                        (chess_move, i32::MIN)
                    }
                })
                .collect()
        } else {
            moves.iter()
                .map(|&chess_move| {
                    let mut new_board = board.clone();
                    if new_board.make_move(chess_move).is_ok() {
                        (chess_move, evaluate_position(&new_board))
                    } else {
                        (chess_move, i32::MIN)
                    }
                })
                .collect()
        };

        // Sort moves by initial evaluation
        move_scores.sort_by_key(|&(_, score)| -score);
        let best_move_initial = move_scores[0].0;

        // Iterative deepening with time management
        let mut best_move = best_move_initial;
        let mut current_depth = 2; // Start with depth 2

        while current_depth <= self.search_depth && start_time.elapsed() < max_time {
            if let Some(better_move) = search_best_move(board, current_depth) {
                best_move = better_move;
            }
            current_depth += 1;
        }

        Some(best_move)
    }
} 