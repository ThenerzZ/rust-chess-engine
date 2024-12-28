pub mod evaluation;
pub mod search;
pub mod ai;

// Re-export only the public interface
pub use ai::ChessAI;

// These are internal implementation details
pub(crate) use evaluation::evaluate_position;
pub(crate) use search::search_best_move; 