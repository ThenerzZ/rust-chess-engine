#[allow(clippy::module_inception)]
pub mod evaluation;
pub mod search;
pub mod ai;

// Re-export only the public interface
pub use ai::ChessAI;

// These are now internal implementation details
use evaluation::evaluate_position;
use search::search_best_move; 