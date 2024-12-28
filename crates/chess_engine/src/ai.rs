use chess_core::{Board, Move};
use crate::{search::search_best_move, evaluation::evaluate_position, opening_book::OpeningBook};
use rayon::prelude::*;
use std::time::{Instant, Duration};

const MAX_THINK_TIME_MS: u64 = 1000; // 1 second max think time
const MIN_DEPTH: u8 = 3;  // Minimum search depth
const MAX_DEPTH: u8 = 5;  // Maximum search depth
const EARLY_GAME_DEPTH: u8 = 4;
const MIDDLE_GAME_DEPTH: u8 = 5;
const END_GAME_DEPTH: u8 = 5;

// Time allocation percentages for different game phases
const EARLY_GAME_TIME_PERCENT: f32 = 0.7;  // Use 70% of max time in opening
const MIDDLE_GAME_TIME_PERCENT: f32 = 1.0;  // Use full time in middle game
const END_GAME_TIME_PERCENT: f32 = 0.8;  // Use 80% of max time in endgame

#[derive(Clone)]
pub struct ChessAI {
    search_depth: u8,
    use_parallel: bool,
    opening_book: OpeningBook,
}

impl ChessAI {
    pub fn new(search_depth: u8) -> Self {
        Self { 
            search_depth: search_depth.clamp(MIN_DEPTH, MAX_DEPTH),
            use_parallel: true,
            opening_book: OpeningBook::new(),
        }
    }

    pub fn get_best_move(&self, board: &Board) -> Option<Move> {
        // First, try to get a move from the opening book
        if let Some(book_move) = self.opening_book.get_book_move(board) {
            return Some(book_move);
        }

        let start_time = Instant::now();
        let phase_depth = self.get_phase_depth(board);
        let max_time = self.get_phase_time(board);
        
        // Generate and pre-sort moves
        let moves = self.generate_moves(board);
        if moves.is_empty() {
            return None;
        }

        // Quick evaluation of all moves in parallel
        let mut move_scores = self.evaluate_moves(board, &moves);
        move_scores.sort_by_key(|&(_, score)| -score);
        
        let mut best_move = move_scores[0].0;
        let mut current_depth = MIN_DEPTH;

        // Iterative deepening with time management
        while current_depth <= phase_depth && start_time.elapsed() < max_time {
            if let Some(better_move) = search_best_move(board, current_depth) {
                best_move = better_move;
            }
            
            // Break early if we're running out of time
            if start_time.elapsed() > max_time * 3 / 4 {
                break;
            }
            
            current_depth += 1;
        }

        Some(best_move)
    }

    fn get_phase_time(&self, board: &Board) -> Duration {
        let base_time = Duration::from_millis(MAX_THINK_TIME_MS);
        let time_multiplier = match self.get_game_phase(board) {
            GamePhase::Early => EARLY_GAME_TIME_PERCENT,
            GamePhase::Middle => MIDDLE_GAME_TIME_PERCENT,
            GamePhase::End => END_GAME_TIME_PERCENT,
        };
        
        Duration::from_millis((MAX_THINK_TIME_MS as f32 * time_multiplier) as u64)
    }

    fn get_phase_depth(&self, board: &Board) -> u8 {
        match self.get_game_phase(board) {
            GamePhase::Early => EARLY_GAME_DEPTH.min(self.search_depth),
            GamePhase::Middle => MIDDLE_GAME_DEPTH.min(self.search_depth),
            GamePhase::End => END_GAME_DEPTH.min(self.search_depth),
        }
    }

    fn get_game_phase(&self, board: &Board) -> GamePhase {
        let mut total_pieces = 0;
        let mut total_value = 0;

        for rank in 1..=8 {
            for file in 1..=8 {
                let pos = chess_core::Position { rank, file };
                if let Some(piece) = board.get_piece(pos) {
                    total_pieces += 1;
                    total_value += match piece.piece_type {
                        chess_core::piece::PieceType::Pawn => 1,
                        chess_core::piece::PieceType::Knight | chess_core::piece::PieceType::Bishop => 3,
                        chess_core::piece::PieceType::Rook => 5,
                        chess_core::piece::PieceType::Queen => 9,
                        chess_core::piece::PieceType::King => 0,
                    };
                }
            }
        }

        if total_value >= 30 {
            GamePhase::Early
        } else if total_value >= 15 {
            GamePhase::Middle
        } else {
            GamePhase::End
        }
    }

    fn generate_moves(&self, board: &Board) -> Vec<Move> {
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

        moves
    }

    fn evaluate_moves(&self, board: &Board, moves: &[Move]) -> Vec<(Move, i32)> {
        if self.use_parallel {
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
        }
    }
}

#[derive(Clone, Copy)]
enum GamePhase {
    Early,
    Middle,
    End,
} 