use chess_core::{Board, Move};
use crate::search::search_best_move;

#[derive(Clone)]
pub struct ChessAI {
    search_depth: u8,
}

impl ChessAI {
    pub fn new(search_depth: u8) -> Self {
        Self { search_depth }
    }

    pub fn get_best_move(&self, board: &Board) -> Option<Move> {
        search_best_move(board)
    }
} 