// Core chess game logic modules
pub mod board;
pub mod piece;
pub mod position;
pub mod moves;

// Re-export main types for convenience
pub use board::Board;
pub use piece::{Piece, Color, PieceType};
pub use position::Position;
pub use moves::{Move, MoveType}; 