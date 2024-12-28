use chess_core::{Board, Move};
use crate::search::search_best_move;
use std::time::{Duration, Instant};
use std::collections::HashSet;

const MAX_THINK_TIME: Duration = Duration::from_secs(3);
const MIN_DEPTH: u8 = 1;  // Start from depth 1 for iterative deepening
const MAX_DEPTH: u8 = 6;  // Reduced from 12 to 6 for faster moves
const DEFAULT_MOVES_LEFT: u32 = 30;
const MAX_RETRIES: usize = 3;

#[derive(Clone)]
pub struct ChessAI {
    max_depth: u8,
    max_time: Duration,
    invalid_moves: HashSet<String>, // Track moves by their string representation
}

impl ChessAI {
    pub fn new(depth: u8) -> Self {
        ChessAI { 
            max_depth: depth.clamp(MIN_DEPTH, MAX_DEPTH),
            max_time: MAX_THINK_TIME,
            invalid_moves: HashSet::new(),
        }
    }

    fn move_to_string(mv: &Move) -> String {
        format!("{}{}-{}{}", 
            mv.from.file, mv.from.rank,
            mv.to.file, mv.to.rank)
    }

    pub fn get_move(&mut self, board: &Board) -> Option<Move> {
        let start_time = Instant::now();
        let mut retries = 0;
        
        while retries < MAX_RETRIES {
            let remaining_time = self.max_time.saturating_sub(start_time.elapsed());
            if remaining_time < Duration::from_millis(100) {
                break;
            }

            if let Some(mv) = search_best_move(board, remaining_time, Some(DEFAULT_MOVES_LEFT)) {
                // Skip moves we know are invalid
                let move_str = Self::move_to_string(&mv);
                if self.invalid_moves.contains(&move_str) {
                    retries += 1;
                    continue;
                }

                // Try the move on a clone of the board first
                let mut test_board = board.clone();
                if test_board.make_move(mv).is_ok() {
                    return Some(mv);
                } else {
                    // Move was invalid, remember it and try again
                    self.invalid_moves.insert(move_str);
                    retries += 1;
                }
            } else {
                break;
            }
        }

        // If we've exhausted retries, try to find any valid move
        for pos in (1..=8).flat_map(|rank| (1..=8).map(move |file| chess_core::Position { rank, file })) {
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    for mv in board.get_valid_moves(pos) {
                        let move_str = Self::move_to_string(&mv);
                        if !self.invalid_moves.contains(&move_str) {
                            let mut test_board = board.clone();
                            if test_board.make_move(mv).is_ok() {
                                return Some(mv);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn set_max_time(&mut self, duration: Duration) {
        self.max_time = duration;
    }

    pub fn clear_invalid_moves(&mut self) {
        self.invalid_moves.clear();
    }
}

impl Default for ChessAI {
    fn default() -> Self {
        ChessAI { 
            max_depth: MIN_DEPTH + 3,
            max_time: MAX_THINK_TIME,
            invalid_moves: HashSet::new(),
        }
    }
} 