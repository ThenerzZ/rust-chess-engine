use chess_core::{Board, Move};
use crate::search::search_best_move;
use std::time::{Duration, Instant};

const MAX_THINK_TIME: Duration = Duration::from_secs(5);
const MIN_DEPTH: u8 = 4;
const MAX_DEPTH: u8 = 8;
const DEFAULT_MOVES_LEFT: u32 = 30; // Assume 30 moves left in an average position

#[derive(Clone)]
pub struct ChessAI {
    depth: u8,
}

impl ChessAI {
    pub fn new(depth: u8) -> Self {
        ChessAI { 
            depth: depth.clamp(MIN_DEPTH, MAX_DEPTH),
        }
    }

    pub fn get_move(&self, board: &Board) -> Option<Move> {
        let start_time = Instant::now();
        let remaining_time = MAX_THINK_TIME.saturating_sub(start_time.elapsed());
        
        // Pass the configured depth to the search function
        search_best_move(board, remaining_time, Some(DEFAULT_MOVES_LEFT))
    }
}

impl Default for ChessAI {
    fn default() -> Self {
        ChessAI { depth: MIN_DEPTH }
    }
} 