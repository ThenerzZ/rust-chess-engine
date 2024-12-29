// Standard imports for time management, chess logic, and parallel processing
use std::time::{Instant, Duration};
use chess_core::{Board, Move, Position, piece::PieceType, moves::MoveType};
use crate::evaluation::evaluate_position;
use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use once_cell::sync::Lazy;
use rayon::prelude::*;

// Time management settings - how long the AI can think about moves
const MIN_TIME_PER_MOVE: Duration = Duration::from_millis(100);  // Don't move too quickly
const MAX_TIME_PER_MOVE: Duration = Duration::from_secs(15);     // Don't think forever
const TIME_BUFFER: Duration = Duration::from_millis(50);         // Safety margin for time management
const MOVES_TO_GO: u32 = 40;                                     // Assume this many moves left in the game
const MAX_DEPTH: u8 = 15;                                        // Maximum search depth
const MIN_DEPTH: u8 = 4;                                         // Always search at least this deep

// Flag to stop searching when we run out of time
static SEARCH_TERMINATED: AtomicBool = AtomicBool::new(false);

// Manages how long we can spend thinking about a move
struct TimeManager {
    start_time: Instant,      // When we started thinking
    allocated_time: Duration, // How long we can think
}

impl TimeManager {
    // Creates a new time manager based on total time left and estimated moves to go
    fn new(total_time: Duration, moves_left: Option<u32>) -> Self {
        let moves_to_go = moves_left.unwrap_or(MOVES_TO_GO);
        let base_time = total_time.div_f32(moves_to_go as f32);
        let allocated_time = base_time.min(MAX_TIME_PER_MOVE).max(MIN_TIME_PER_MOVE);
        
        Self {
            start_time: Instant::now(),
            allocated_time,
        }
    }

    // Checks if we still have time to continue searching
    fn should_continue(&self) -> bool {
        let elapsed = self.start_time.elapsed();
        elapsed + TIME_BUFFER < self.allocated_time
    }

    // Returns how long we've been thinking
    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// Core search algorithm parameters
const MATE_SCORE: i32 = 20000;                    // Value representing checkmate
const ALPHA_INIT: i32 = -19000;                   // Initial alpha for search window
const BETA_INIT: i32 = 19000;                     // Initial beta for search window
const QUIESCENCE_DEPTH: u8 = 6;                   // Increased from 4 to search deeper in tactical positions
const MAX_MOVES_TO_CONSIDER: usize = 50;          // Increased from 35 to consider more moves
const MAX_TT_SIZE: usize = 1_000_000;            // Size of transposition table
const WINDOW_SIZE_INIT: i32 = 100;               // Increased from 50 to search wider

// Pruning parameters - reduced to make search more thorough
const FUTILITY_MARGIN: [i32; 4] = [0, 200, 400, 600];  // Increased margins
const DELTA_MARGIN: i32 = 300;                         // Increased from 150
const NULL_MOVE_R: u8 = 2;                            // Reduced from 3 to search more thoroughly
const NULL_MOVE_MATERIAL_THRESHOLD: i32 = 500;        // Reduced from 800 to be more aggressive
const LMR_DEPTH_THRESHOLD: u8 = 2;                    // Reduced from 3
const LMR_MOVE_THRESHOLD: usize = 2;                  // Reduced from 3

// Move generation and history heuristic parameters
const MAX_TACTICAL_MOVES: usize = 8;                  // Maximum number of tactical moves to consider
const HISTORY_MAX: i32 = 8000;                       // Maximum history score before scaling

// Move ordering scores - helps search better moves first
const CAPTURE_SCORE_BASE: i32 = 10000;               // Base score for captures
const PROMOTION_SCORE_BASE: i32 = 9000;              // Base score for pawn promotions
const HISTORY_SCORE_MAX: i32 = 6000;                 // Maximum score for history heuristic

// Types of entries in our transposition table
#[derive(Clone, Copy)]
enum EntryType {
    Exact,      // The stored score is exact
    LowerBound, // The real score might be higher
    UpperBound, // The real score might be lower
}

// Entry in our transposition table - caches results of previous searches
#[derive(Clone)]
struct TTEntry {
    depth: u8,              // How deep we searched
    score: i32,             // Score we found
    entry_type: EntryType,  // How reliable this score is
    best_move: Option<Move>, // Best move found at this position
}

// Global cache of positions we've already analyzed
static TRANSPOSITION_TABLE: Lazy<Mutex<HashMap<String, TTEntry>>> = 
    Lazy::new(|| Mutex::new(HashMap::with_capacity(MAX_TT_SIZE)));

// Table that remembers which moves were good in similar positions
static HISTORY_TABLE: Lazy<Mutex<Vec<Vec<i32>>>> = 
    Lazy::new(|| Mutex::new(vec![vec![0; 64]; 64]));

// Principal Variation (PV) - the best line of play we've found
const MAX_PV_LENGTH: usize = 64;  // Maximum length of the principal variation
static PV_TABLE: Lazy<Mutex<Vec<Move>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(MAX_PV_LENGTH)));

// Killer moves - good moves that caused beta cutoffs at the same depth
static KILLER_MOVES: Lazy<Mutex<Vec<Option<[Move; 2]>>>> = Lazy::new(|| Mutex::new(vec![None; MAX_PV_LENGTH]));

// Parameters for Principal Variation Search (PVS)
const FULL_DEPTH_MOVES: usize = 4;  // Search this many moves with full window
const REDUCTION_LIMIT: u8 = 3;      // Don't reduce moves until this depth

// Creates a dummy move for initialization purposes
fn create_default_move() -> Move {
    Move {
        from: Position { rank: 0, file: 0 },
        to: Position { rank: 0, file: 0 },
        move_type: MoveType::Normal,
        promotion: None,
    }
}

// Main function that finds the best move in a given position
pub fn search_best_move(board: &Board, total_time: Duration, moves_left: Option<u32>) -> Option<Move> {
    println!("\nStarting new search with time limit: {:?}", total_time);
    let start_time = Instant::now();
    
    SEARCH_TERMINATED.store(false, Ordering::SeqCst);
    let time_manager = TimeManager::new(total_time, moves_left);
    
    // Clear transposition table if it's getting too large
    let mut tt = TRANSPOSITION_TABLE.lock().unwrap();
    let tt_size = tt.len();
    if tt_size > MAX_TT_SIZE {
        println!("Clearing transposition table (size: {})", tt_size);
        tt.clear();
    }
    
    // Try to find an obvious move first
    let mut moves = Vec::new();
    for pos in (1..=8).flat_map(|rank| (1..=8).map(move |file| Position { rank, file })) {
        if let Some(piece) = board.get_piece(pos) {
            if piece.color == board.current_turn() {
                moves.extend(board.get_valid_moves(pos));
            }
        }
    }
    println!("Generated {} possible moves", moves.len());
    
    if let Some(obvious) = find_obvious_move(board, &moves) {
        println!("Found obvious move: {:?}", obvious);
        return Some(obvious);
    }
    
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;
    let mut pv_table = Vec::new();
    let mut history = vec![vec![0; 64]; 64];
    
    // Aspiration windows for better move ordering
    let mut window_size = WINDOW_SIZE_INIT;
    
    for depth in 1..=MAX_DEPTH {
        let elapsed = start_time.elapsed();
        if !time_manager.should_continue() {
            println!("Stopping search at depth {} due to time limit ({:?} elapsed)", depth, elapsed);
            break;
        }
        
        println!("\nSearching at depth {}", depth);
        let depth_start = Instant::now();
        
        // Calculate alpha and beta with overflow protection
        let alpha = best_score.saturating_sub(window_size);
        let beta = best_score.saturating_add(window_size);
        
        let mut score = principal_variation_search(
            board,
            depth,
            alpha,
            beta,
            &mut tt,
            &mut history,
            &mut pv_table,
            true,
        );
        
        // If score is outside our window, research with full window
        if score <= alpha || score >= beta {
            println!("Score {} outside window [{}, {}], researching with full window", score, alpha, beta);
            score = principal_variation_search(
                board,
                depth,
                -MATE_SCORE,
                MATE_SCORE,
                &mut tt,
                &mut history,
                &mut pv_table,
                true,
            );
        }
        
        let depth_time = depth_start.elapsed();
        println!("Depth {} completed in {:?}, score: {}", depth, depth_time, score);
        
        // Update best move if we found one
        if !pv_table.is_empty() {
            best_move = Some(pv_table[0]);
            best_score = score;
            println!("New best move: {:?}, score: {}", best_move, best_score);
        }
        
        // Early exit if we found a forced mate
        if score.abs() > MATE_SCORE - 100 {
            println!("Found forced mate, stopping search");
            break;
        }
        
        // Gradually increase window size with overflow protection
        window_size = window_size.saturating_mul(5).saturating_div(4);
    }
    
    let total_time = start_time.elapsed();
    println!("\nSearch completed in {:?}", total_time);
    if let Some(mv) = best_move {
        println!("Best move found: {:?} with score {}", mv, best_score);
    } else {
        println!("No valid move found!");
    }
    
    best_move
}

// Looks for simple winning captures that we can make immediately
fn find_obvious_move(board: &Board, moves: &[Move]) -> Option<Move> {
    for &mv in moves {
        if let Some(victim) = board.get_piece(mv.to) {
            let attacker = board.get_piece(mv.from).unwrap();
            // If we can capture a higher value piece with a lower value one
            if get_piece_value(victim.piece_type) > get_piece_value(attacker.piece_type) {
                let mut new_board = board.clone();
                if new_board.make_move(mv).is_ok() {
                    // Make sure it's not a trap where we lose the piece
                    if !is_piece_hanging(&new_board, mv.to) {
                        return Some(mv);
                    }
                }
            }
        }
    }
    None
}

// The main recursive search function that implements Principal Variation Search (PVS)
fn principal_variation_search(
    board: &Board,
    depth: u8,
    alpha: i32,
    beta: i32,
    tt: &mut HashMap<String, TTEntry>,
    history: &mut Vec<Vec<i32>>,
    pv_table: &mut Vec<Move>,
    is_pv_node: bool,
) -> i32 {
    // Early exits
    if SEARCH_TERMINATED.load(Ordering::SeqCst) {
        return evaluate_position(board);
    }

    if depth == 0 || board.is_checkmate() || board.is_stalemate() {
        let score = quiescence_search(board, alpha, beta, QUIESCENCE_DEPTH);
        if depth == 0 {
            println!("Reached depth 0, quiescence score: {}", score);
        }
        return score;
    }

    // Try to use cached result if we have one
    let pos_key = get_position_key(board);
    let original_alpha = alpha;
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;
    let mut current_alpha = alpha;

    // Check transposition table
    if let Some(entry) = tt.get(&pos_key) {
        if entry.depth >= depth && !is_pv_node {
            let score = adjust_mate_score(entry.score, depth);
            match entry.entry_type {
                EntryType::Exact => return score,
                EntryType::LowerBound => current_alpha = current_alpha.max(score),
                EntryType::UpperBound => {
                    if score <= alpha {
                        return score;
                    }
                }
            }
            if current_alpha >= beta {
                return score;
            }
        }
        best_move = entry.best_move;
    }

    // Generate and try moves
    let mut moves = generate_ordered_moves(board, history, pv_table, best_move);
    let mut searched_moves = 0;
    let mut has_legal_moves = false;

    println!("Searching {} moves at depth {}", moves.len(), depth);

    // Try each move
    for mv in moves {
        let mut new_board = board.clone();
        if new_board.make_move(mv).is_ok() {
            has_legal_moves = true;
            searched_moves += 1;

            let score = if searched_moves == 1 {
                // Search first move with full window
                -principal_variation_search(
                    &new_board,
                    depth - 1,
                    -beta,
                    -current_alpha,
                    tt,
                    history,
                    pv_table,
                    is_pv_node,
                )
            } else {
                // Try late move reductions for other moves
                let reduction = if depth >= REDUCTION_LIMIT && searched_moves > FULL_DEPTH_MOVES {
                    ((searched_moves as f32).ln().floor() as u8).min(depth - 1)
                } else {
                    0
                };

                // First try a shallow search
                let mut score = -principal_variation_search(
                    &new_board,
                    depth - 1 - reduction,
                    -(current_alpha + 1),
                    -current_alpha,
                    tt,
                    history,
                    pv_table,
                    false,
                );

                // If the shallow search looks promising, do a full search
                if score > current_alpha && score < beta {
                    score = -principal_variation_search(
                        &new_board,
                        depth - 1,
                        -beta,
                        -current_alpha,
                        tt,
                        history,
                        pv_table,
                        is_pv_node,
                    );
                }
                score
            };

            // Update best move if we found a better one
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                if score > current_alpha {
                    current_alpha = score;
                    if is_pv_node {
                        println!("New best move at depth {}: {:?}, score: {}", depth, mv, score);
                        pv_table.clear();
                        pv_table.push(mv);
                    }
                }
            }

            // Beta cutoff - position is too good, opponent won't allow it
            if current_alpha >= beta {
                if !is_capture(board, mv) {
                    update_history(history, mv, depth);
                }
                break;
            }
        }
    }

    // Handle special cases
    if !has_legal_moves {
        return if is_endgame_or_in_check(board) { -MATE_SCORE + depth as i32 } else { 0 };
    }

    // Save position to transposition table
    let entry_type = if best_score <= original_alpha {
        EntryType::UpperBound
    } else if best_score >= beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };

    tt.insert(pos_key, TTEntry {
        depth,
        score: best_score,
        entry_type,
        best_move,
    });

    best_score
}

// Creates a unique string key for a board position
fn get_position_key(board: &Board) -> String {
    let mut key = String::with_capacity(100);
    // Add each piece's position and type to the key
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                key.push_str(&format!("{}{}:{:?}{:?},", 
                    pos.rank, pos.file, piece.piece_type, piece.color));
            }
        }
    }
    // Add whose turn it is
    key.push_str(&format!("turn:{:?}", board.current_turn()));
    key
}

// Search captures to make sure we don't miss any tactical opportunities
fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    // Check if we need to stop searching
    if SEARCH_TERMINATED.load(Ordering::SeqCst) {
        return evaluate_position(board);
    }

    // Get a quick evaluation of the current position
    let stand_pat = evaluate_position(board);
    
    // Stop searching if we're too deep or the game is over
    if depth == 0 || board.is_checkmate() || board.is_stalemate() {
        return stand_pat;
    }

    // Position is already too good - opponent won't allow it
    if stand_pat >= beta {
        return beta;
    }

    // Don't search further if even the best capture can't improve our position
    if stand_pat < alpha - DELTA_MARGIN {
        return alpha;
    }

    // Current position is better than what we've found so far
    alpha = alpha.max(stand_pat);

    // Look at all possible captures
    let mut captures = generate_captures(board);
    if captures.is_empty() {
        return stand_pat;
    }
    
    // Sort captures by how good they look
    captures.sort_by_cached_key(|m| {
        let see_score = static_exchange_evaluation(board, *m);
        let mvv_lva = get_mvv_lva_score(board, *m);
        -(see_score * 1000 + mvv_lva)
    });
    
    // Only look at captures that don't lose too much material
    captures.retain(|m| {
        let see_score = static_exchange_evaluation(board, *m);
        see_score >= -50 // Only slightly losing captures might be worth checking
    });

    // Try each capture
    for capture in captures {
        // Stop if we're out of time
        if SEARCH_TERMINATED.load(Ordering::SeqCst) {
            return alpha;
        }

        // Make the capture and evaluate the resulting position
        let mut new_board = board.clone();
        if new_board.make_move(capture).is_ok() {
            let score = -quiescence_search(&new_board, -beta, -alpha, depth - 1);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
    }

    alpha
}

// Generates a list of moves sorted by how good they're likely to be
fn generate_ordered_moves(
    board: &Board,
    history: &Vec<Vec<i32>>,
    pv_table: &Vec<Move>,
    tt_move: Option<Move>,
) -> Vec<Move> {
    let mut moves = Vec::new();
    for pos in (1..=8).flat_map(|rank| (1..=8).map(move |file| Position { rank, file })) {
        if let Some(piece) = board.get_piece(pos) {
            if piece.color == board.current_turn() {
                moves.extend(board.get_valid_moves(pos));
            }
        }
    }
    
    if moves.is_empty() {
        return moves;
    }
    
    // Score moves
    let mut scored_moves: Vec<(Move, i32)> = moves.into_iter()
        .map(|mv| {
            let mut score = 0;
            
            // PV move gets highest priority
            if !pv_table.is_empty() && pv_table[0] == mv {
                score += 20000;
            }
            
            // TT move gets high priority
            if let Some(tt_mv) = tt_move {
                if tt_mv == mv {
                    score += 19000;
                }
            }
            
            // Captures and promotions
            if mv.move_type == MoveType::Capture {
                score += get_mvv_lva_score(board, mv);
                if is_clearly_winning_capture(board, mv) {
                    score += 1000;
                }
            }
            
            if let Some(promotion) = mv.promotion {
                score += PROMOTION_SCORE_BASE;
            }
            
            // History heuristic
            score += get_history_score(history, mv);
            
            (mv, score)
        })
        .collect();
    
    // Sort moves by score
    scored_moves.sort_by_key(|(_, score)| -score);
    scored_moves.into_iter().map(|(mv, _)| mv).collect()
}

// Finds all possible captures in the current position
fn generate_captures(board: &Board) -> Vec<Move> {
    let mut captures = Vec::new();
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    let moves = board.get_valid_moves(pos);
                    for mv in moves {
                        if board.get_piece(mv.to).is_some() {
                            captures.push(mv);
                        }
                    }
                }
            }
        }
    }
    captures
}

// Scores captures based on Most Valuable Victim - Least Valuable Attacker principle
fn get_mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let victim = board.get_piece(mv.to);
    let attacker = board.get_piece(mv.from);
    
    if let (Some(victim), Some(attacker)) = (victim, attacker) {
        let victim_value = get_piece_static_value(victim.piece_type);
        let attacker_value = get_piece_static_value(attacker.piece_type);
        
        // Add bonus for moves that improve piece mobility
        let mobility_bonus = board.get_valid_moves(mv.to).len() as i32 * 5;
        
        // Prefer capturing high value pieces with low value pieces
        victim_value * 100 - attacker_value * 10 + mobility_bonus
    } else {
        0
    }
}

// Basic piece values for simple evaluations
fn get_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 1,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 0,  // King's value doesn't matter for captures
    }
}

// Checks if we're in endgame or if the king is under attack
fn is_endgame_or_in_check(board: &Board) -> bool {
    let mut queens = 0;
    let mut pieces = 0;
    let current_color = board.current_turn();
    let mut king_attacked = false;

    // Count material and look for king attacks
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                match piece.piece_type {
                    PieceType::Queen => queens += 1,
                    PieceType::Rook | PieceType::Bishop | PieceType::Knight => pieces += 1,
                    PieceType::King if piece.color == current_color => {
                        // Look for any enemy pieces that can attack our king
                        for r in 1..=8 {
                            for f in 1..=8 {
                                let attack_pos = chess_core::Position { rank: r, file: f };
                                if let Some(attacker) = board.get_piece(attack_pos) {
                                    if attacker.color != current_color {
                                        let moves = board.get_valid_moves(attack_pos);
                                        if moves.iter().any(|m| m.to == pos) {
                                            king_attacked = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if king_attacked {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // We're in endgame if there are few pieces left
    let is_endgame = queens == 0 || (queens == 2 && pieces <= 2);
    is_endgame || king_attacked
} 

// Updates the history table when a move causes a beta cutoff
fn update_history(history: &mut Vec<Vec<i32>>, mv: Move, bonus: u8) {
    let from_idx = ((mv.from.rank - 1) * 8 + (mv.from.file - 1)) as usize;
    let to_idx = ((mv.to.rank - 1) * 8 + (mv.to.file - 1)) as usize;
    
    history[from_idx][to_idx] += bonus as i32;
    
    // Scale down all history scores if they get too large
    if history[from_idx][to_idx] > HISTORY_MAX {
        for row in history.iter_mut() {
            for cell in row.iter_mut() {
                *cell /= 2;
            }
        }
    }
}

// Gets the history score for a move
fn get_history_score(history: &Vec<Vec<i32>>, mv: Move) -> i32 {
    let from_idx = ((mv.from.rank - 1) * 8 + (mv.from.file - 1)) as usize;
    let to_idx = ((mv.to.rank - 1) * 8 + (mv.to.file - 1)) as usize;
    history[from_idx][to_idx]
}

// Checks if a move is a capture
fn is_capture(board: &Board, mv: Move) -> bool {
    board.get_piece(mv.to).is_some()
}

// Checks if a move gives check to the opponent
fn gives_check(board: &Board) -> bool {
    let current_color = board.current_turn();
    
    // Find the opponent's king
    let mut king_pos = None;
    'outer: for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::King && piece.color != current_color {
                    king_pos = Some(pos);
                    break 'outer;
                }
            }
        }
    }

    // See if any of our pieces can attack the king
    if let Some(king_pos) = king_pos {
        for rank in 1..=8 {
            for file in 1..=8 {
                let pos = chess_core::Position { rank, file };
                if let Some(piece) = board.get_piece(pos) {
                    if piece.color == current_color {
                        let moves = board.get_valid_moves(pos);
                        if moves.iter().any(|m| m.to == king_pos) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    
    false
} 

// Evaluates a capture sequence to see if it's good for us
fn static_exchange_evaluation(board: &Board, mv: Move) -> i32 {
    let victim = board.get_piece(mv.to);
    let attacker = board.get_piece(mv.from);
    
    if let (Some(victim), Some(attacker)) = (victim, attacker) {
        let victim_value = get_piece_static_value(victim.piece_type);
        let attacker_value = get_piece_static_value(attacker.piece_type);
        
        // Simple evaluation - just look at material difference
        victim_value - attacker_value
    } else {
        0
    }
}

// More precise piece values for static evaluation
fn get_piece_static_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,    // Base pawn value
        PieceType::Knight => 325,  // Slightly higher than bishop in closed positions
        PieceType::Bishop => 325,  // Equal to knight but better in open positions
        PieceType::Rook => 500,    // Worth about 5 pawns
        PieceType::Queen => 900,   // Most valuable piece after king
        PieceType::King => 20000,  // Effectively infinite value
    }
} 

// Checks if a position requires careful tactical play
fn is_tactical_position(board: &Board) -> bool {
    // Position is tactical if:
    // 1. There are hanging pieces
    // 2. There are multiple possible captures
    // 3. Queens are on the board
    let mut capture_count = 0;
    let mut has_queen = false;

    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.piece_type == PieceType::Queen {
                    has_queen = true;
                }
                // Check if piece is hanging
                if is_piece_hanging(board, pos) {
                    return true;
                }
                // Count possible captures
                let moves = board.get_valid_moves(pos);
                for mv in moves {
                    if board.get_piece(mv.to).is_some() {
                        capture_count += 1;
                        if capture_count > 2 {
                            return true;
                        }
                    }
                }
            }
        }
    }
    
    has_queen && capture_count > 0
}

// Checks if a piece can be captured without losing material
fn is_piece_hanging(board: &Board, pos: chess_core::Position) -> bool {
    if let Some(piece) = board.get_piece(pos) {
        let piece_value = get_piece_value(piece.piece_type);
        
        // Find the lowest value attacker
        let mut min_attacker_value = i32::MAX;
        for rank in 1..=8 {
            for file in 1..=8 {
                let attack_pos = chess_core::Position { rank, file };
                if let Some(attacker) = board.get_piece(attack_pos) {
                    if attacker.color != piece.color {
                        let moves = board.get_valid_moves(attack_pos);
                        if moves.iter().any(|m| m.to == pos) {
                            min_attacker_value = min_attacker_value.min(get_piece_value(attacker.piece_type));
                        }
                    }
                }
            }
        }
        
        // Piece is hanging if it can be captured by a lower value piece
        if min_attacker_value < piece_value {
            return true;
        }
    }
    false
}

// Calculates total material value on the board
fn get_material_count(board: &Board) -> i32 {
    let mut total = 0;
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                total += get_piece_static_value(piece.piece_type);
            }
        }
    }
    total
}

// Adjusts mate scores based on distance to mate
fn adjust_mate_score(score: i32, depth: u8) -> i32 {
    if score > MATE_SCORE - 1000 {
        // We found a mate - prefer shorter mates
        score - depth as i32
    } else if score < -MATE_SCORE + 1000 {
        // We're getting mated - prefer longer mates
        score + depth as i32
    } else {
        score
    }
} 

// Updates the killer move table after a good quiet move
fn update_killer_moves(killer_moves: &mut Option<[Move; 2]>, mv: Move) {
    let moves = killer_moves.get_or_insert([create_default_move(); 2]);
    
    // Keep track of the two most recent killer moves
    if moves[0].from != mv.from || moves[0].to != mv.to {
        moves[1] = moves[0];
        moves[0] = mv;
    }
} 

// Checks if a capture is clearly winning material
fn is_clearly_winning_capture(board: &Board, mv: Move) -> bool {
    if let Some(victim) = board.get_piece(mv.to) {
        if let Some(attacker) = board.get_piece(mv.from) {
            let victim_value = get_piece_value(victim.piece_type);
            let attacker_value = get_piece_value(attacker.piece_type);
            
            // Only return true if we're winning significant material
            if victim_value > attacker_value + 2 {
                let mut new_board = board.clone();
                if new_board.make_move(mv).is_ok() {
                    // Make sure the piece isn't immediately recaptured
                    return !is_piece_hanging(&new_board, mv.to);
                }
            }
        }
    }
    false
} 