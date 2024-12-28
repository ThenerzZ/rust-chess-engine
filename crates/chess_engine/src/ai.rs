use chess_core::{Board, Move};
use crate::search::search_best_move;
use std::time::{Duration, Instant};

const MAX_THINK_TIME: Duration = Duration::from_secs(5);
const MIN_DEPTH: u8 = 4;
const MAX_DEPTH: u8 = 8;

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
        let mut best_move = None;
        
        // Iterative deepening
        for current_depth in 1..=self.depth {
            if start_time.elapsed() > MAX_THINK_TIME {
                break;
            }
            
            if let Some(mv) = search_best_move(board, current_depth) {
                best_move = Some(mv);
            }
        }
        
        best_move
    }
}

impl Default for ChessAI {
    fn default() -> Self {
        ChessAI { depth: MIN_DEPTH }
    }
} 