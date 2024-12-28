use std::time::{Instant, Duration};
use chess_core::{Board, Move, Position, piece::PieceType, moves::MoveType};
use crate::evaluation::evaluate_position;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, atomic::{AtomicBool, Ordering}};
use once_cell::sync::Lazy;
use rayon::prelude::*;

// Time management constants
const MIN_TIME_PER_MOVE: Duration = Duration::from_millis(100);
const MAX_TIME_PER_MOVE: Duration = Duration::from_secs(15);
const TIME_BUFFER: Duration = Duration::from_millis(50);
const MOVES_TO_GO: u32 = 40;
const MAX_DEPTH: u8 = 15;
const MIN_DEPTH: u8 = 4;

// Search termination flag
static SEARCH_TERMINATED: AtomicBool = AtomicBool::new(false);

// Time management structure
struct TimeManager {
    start_time: Instant,
    allocated_time: Duration,
}

impl TimeManager {
    fn new(total_time: Duration, moves_left: Option<u32>) -> Self {
        let moves_to_go = moves_left.unwrap_or(MOVES_TO_GO);
        let base_time = total_time.div_f32(moves_to_go as f32);
        let allocated_time = base_time.min(MAX_TIME_PER_MOVE).max(MIN_TIME_PER_MOVE);
        
        Self {
            start_time: Instant::now(),
            allocated_time,
        }
    }

    fn should_continue(&self) -> bool {
        let elapsed = self.start_time.elapsed();
        elapsed + TIME_BUFFER < self.allocated_time
    }

    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// Search parameters
const MATE_SCORE: i32 = 20000;
const ALPHA_INIT: i32 = -19000;
const BETA_INIT: i32 = 19000;
const QUIESCENCE_DEPTH: u8 = 4;
const MAX_MOVES_TO_CONSIDER: usize = 35;
const MAX_TT_SIZE: usize = 1_000_000;

// More balanced pruning
const FUTILITY_MARGIN: [i32; 4] = [0, 100, 200, 300];
const DELTA_MARGIN: i32 = 150;
const NULL_MOVE_R: u8 = 3;
const NULL_MOVE_MATERIAL_THRESHOLD: i32 = 800;
const LMR_DEPTH_THRESHOLD: u8 = 3;
const LMR_MOVE_THRESHOLD: usize = 3;

// Move generation
const MAX_TACTICAL_MOVES: usize = 8;
const HISTORY_MAX: i32 = 8000;

// Transposition table entry types
#[derive(Clone, Copy)]
enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

// Transposition table entry
#[derive(Clone)]
struct TTEntry {
    depth: u8,
    score: i32,
    entry_type: EntryType,
    best_move: Option<Move>,
}

// Global transposition table
static TRANSPOSITION_TABLE: Lazy<Mutex<HashMap<String, TTEntry>>> = 
    Lazy::new(|| Mutex::new(HashMap::with_capacity(MAX_TT_SIZE)));

// History table for move ordering
static HISTORY_TABLE: Lazy<Mutex<Vec<Vec<i32>>>> = 
    Lazy::new(|| Mutex::new(vec![vec![0; 64]; 64]));

// Add at the top with other constants
const MAX_PV_LENGTH: usize = 64;
static PV_TABLE: Lazy<Mutex<Vec<Move>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(MAX_PV_LENGTH)));
static KILLER_MOVES: Lazy<Mutex<Vec<Option<[Move; 2]>>>> = Lazy::new(|| Mutex::new(vec![None; MAX_PV_LENGTH]));

// Move ordering scores (inspired by Stockfish)
const CAPTURE_SCORE_BASE: i32 = 10000;
const PROMOTION_SCORE_BASE: i32 = 9000;
const KILLER_SCORE: i32 = 8000;
const COUNTER_MOVE_SCORE: i32 = 7000;
const HISTORY_SCORE_MAX: i32 = 6000;

// PVS search parameters
const FULL_DEPTH_MOVES: usize = 4;  // Number of moves to search with full window
const REDUCTION_LIMIT: u8 = 3;      // Minimum depth for reductions

// Helper function to create a default move
fn create_default_move() -> Move {
    Move {
        from: Position { rank: 0, file: 0 },
        to: Position { rank: 0, file: 0 },
        move_type: MoveType::Normal,
        promotion: None,
    }
}

pub fn search_best_move(board: &Board, total_time: Duration, moves_left: Option<u32>) -> Option<Move> {
    let time_manager = TimeManager::new(total_time, moves_left);
    SEARCH_TERMINATED.store(false, Ordering::SeqCst);

    let mut tt = TRANSPOSITION_TABLE.lock().unwrap();
    if tt.len() > MAX_TT_SIZE {
        tt.clear();
    }
    let mut history = HISTORY_TABLE.lock().unwrap();
    let mut pv_table = PV_TABLE.lock().unwrap();
    
    pv_table.clear();
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;
    let mut current_depth = 1;

    let moves = generate_ordered_moves(
        board, 
        &*history,
        &*pv_table,
        None  // No TT move for initial position
    );

    // Early exit for single legal move
    if moves.len() == 1 {
        return Some(moves[0]);
    }

    // Early move for obvious captures
    if let Some(obvious_move) = find_obvious_move(board, &moves) {
        if is_clearly_winning_capture(board, obvious_move) {
            return Some(obvious_move);
        }
    }

    // Parallel iterative deepening with aspiration windows
    while current_depth <= MAX_DEPTH && (current_depth <= MIN_DEPTH || time_manager.should_continue()) {
        println!("Starting search at depth {}", current_depth);
        let mut alpha = if current_depth == 1 { -MATE_SCORE } else { best_score - 75 };
        let beta = if current_depth == 1 { MATE_SCORE } else { best_score + 75 };
        let mut window = 50;

        // Process moves in parallel
        let chunk_size = (moves.len() + 3) / 4;
        let move_chunks: Vec<Vec<Move>> = moves.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let scores: Vec<(Option<Move>, i32)> = move_chunks.par_iter()
            .map(|chunk_moves| {
                let mut local_best_move = None;
                let mut local_best_score = ALPHA_INIT;

                for &chess_move in chunk_moves {
                    if !time_manager.should_continue() {
                        SEARCH_TERMINATED.store(true, Ordering::SeqCst);
                        break;
                    }

                    let mut new_board = board.clone();
                    if new_board.make_move(chess_move).is_ok() {
                        let mut score = -principal_variation_search(
                            &new_board,
                            current_depth - 1,
                            -beta,
                            -alpha,
                            &mut tt.clone(),
                            &mut history.clone(),
                            &mut pv_table.clone(),
                            true,
                        );

                        if score > local_best_score {
                            local_best_score = score;
                            local_best_move = Some(chess_move);
                        }
                    }
                }
                (local_best_move, local_best_score)
            })
            .collect();

        // Early exit if search was terminated
        if SEARCH_TERMINATED.load(Ordering::SeqCst) {
            break;
        }

        // Update best move
        for (move_option, score) in scores {
            if score > best_score {
                best_score = score;
                best_move = move_option;
            }
        }

        // Adjust window for next iteration
        window = (window * 3) / 2;
        current_depth += 1;

        println!("Depth {} completed in {:?}, best move: {:?}, score: {}", 
            current_depth - 1, 
            time_manager.elapsed(),
            best_move,
            best_score
        );

        // Early exit if we found a winning move
        if best_score > MATE_SCORE - 1000 {
            break;
        }
    }

    best_move
}

// Find obvious moves like capturing a piece with no counter-play
fn find_obvious_move(board: &Board, moves: &[Move]) -> Option<Move> {
    for &mv in moves {
        if let Some(victim) = board.get_piece(mv.to) {
            let attacker = board.get_piece(mv.from).unwrap();
            // If we can capture a higher value piece with a lower value one
            if get_piece_value(victim.piece_type) > get_piece_value(attacker.piece_type) {
                let mut new_board = board.clone();
                if new_board.make_move(mv).is_ok() {
                    // Check if the capture is safe
                    if !is_piece_hanging(&new_board, mv.to) {
                        return Some(mv);
                    }
                }
            }
        }
    }
    None
}

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
    if SEARCH_TERMINATED.load(Ordering::SeqCst) {
        return evaluate_position(board);
    }

    if depth == 0 || board.is_checkmate() || board.is_stalemate() {
        return quiescence_search(board, alpha, beta, QUIESCENCE_DEPTH);
    }

    let pos_key = get_position_key(board);
    let original_alpha = alpha;
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;
    let mut current_alpha = alpha;

    // TT lookup
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

    // Static evaluation and pruning
    let static_eval = evaluate_position(board);
    let in_check = is_endgame_or_in_check(board);

    // Reverse futility pruning
    if !is_pv_node && !in_check && depth <= 3 {
        let margin = FUTILITY_MARGIN[depth as usize];
        if static_eval - margin >= beta {
            return static_eval;
        }
    }

    // Null move pruning
    if !is_pv_node && depth >= 3 && !in_check && static_eval >= beta {
        let r = if depth >= 6 { NULL_MOVE_R + 1 } else { NULL_MOVE_R };
        let score = -principal_variation_search(
            board,
            depth - r - 1,
            -beta,
            -beta + 1,
            tt,
            history,
            pv_table,
            false,
        );
        if score >= beta && score < MATE_SCORE - MAX_DEPTH as i32 {
            return score;
        }
    }

    let mut moves = generate_ordered_moves(board, history, pv_table, best_move);
    let mut searched_moves = 0;
    let mut has_legal_moves = false;

    // PV search
    for mv in moves {
        let mut new_board = board.clone();
        if new_board.make_move(mv).is_ok() {
            has_legal_moves = true;
            searched_moves += 1;

            let score = if searched_moves == 1 {
                // First move with full window
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
                // Late Move Reductions
                let reduction = if depth >= REDUCTION_LIMIT && searched_moves > FULL_DEPTH_MOVES {
                    ((searched_moves as f32).ln().floor() as u8).min(depth - 1)
                } else {
                    0
                };

                // Search with null window
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

                // Re-search if score is between alpha and beta
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

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                if score > current_alpha {
                    current_alpha = score;
                    
                    // Update PV table
                    if is_pv_node {
                        pv_table.clear();
                        pv_table.push(mv);
                    }
                }
            }

            // Beta cutoff
            if current_alpha >= beta {
                if !is_capture(board, mv) {
                    update_history(history, mv, depth);
                }
                break;
            }
        }
    }

    // Handle checkmate and stalemate
    if !has_legal_moves {
        return if in_check { -MATE_SCORE + depth as i32 } else { 0 };
    }

    // Store position in transposition table
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

fn get_position_key(board: &Board) -> String {
    let mut key = String::with_capacity(100);
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                key.push_str(&format!("{}{}:{:?}{:?},", 
                    pos.rank, pos.file, piece.piece_type, piece.color));
            }
        }
    }
    key.push_str(&format!("turn:{:?}", board.current_turn()));
    key
}

fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    // Check if search should be terminated
    if SEARCH_TERMINATED.load(Ordering::SeqCst) {
        return evaluate_position(board);
    }

    // Quick evaluation of the current position
    let stand_pat = evaluate_position(board);
    
    // Early exit conditions
    if depth == 0 || board.is_checkmate() || board.is_stalemate() {
        return stand_pat;
    }

    // Stand pat pruning
    if stand_pat >= beta {
        return beta;
    }

    // Delta pruning - if even the best possible capture can't improve alpha
    if stand_pat < alpha - DELTA_MARGIN {
        return alpha;
    }

    alpha = alpha.max(stand_pat);

    // Generate and sort captures
    let mut captures = generate_captures(board);
    if captures.is_empty() {
        return stand_pat;
    }
    
    // Sort captures by MVV-LVA and SEE
    captures.sort_by_cached_key(|m| {
        let see_score = static_exchange_evaluation(board, *m);
        let mvv_lva = get_mvv_lva_score(board, *m);
        -(see_score * 1000 + mvv_lva)
    });
    
    // Only look at promising captures
    captures.retain(|m| {
        let see_score = static_exchange_evaluation(board, *m);
        see_score >= -50 // Only slightly losing captures might be worth checking
    });

    let mut searched_moves = 0;
    for capture in captures {
        // Periodically check if search should be terminated
        if searched_moves % 8 == 0 && SEARCH_TERMINATED.load(Ordering::SeqCst) {
            return alpha;
        }

        searched_moves += 1;
        let mut new_board = board.clone();
        if new_board.make_move(capture).is_ok() {
            let score = -quiescence_search(&new_board, -beta, -alpha, depth - 1);
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
    }

    alpha
}

fn generate_ordered_moves(
    board: &Board,
    history: &Vec<Vec<i32>>,
    pv_table: &Vec<Move>,
    tt_move: Option<Move>,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut move_scores = Vec::new();
    
    let current_color = board.current_turn();
    
    // First try TT move if available
    if let Some(tt_mv) = tt_move {
        if board.get_piece(tt_mv.from).map_or(false, |p| p.color == current_color) {
            moves.push(tt_mv);
            move_scores.push(20000); // Highest priority
        }
    }

    // Generate all legal moves
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == current_color {
                    for mv in board.get_valid_moves(pos) {
                        // Skip TT move as it's already added
                        if tt_move.map_or(false, |tt_mv| tt_mv == mv) {
                            continue;
                        }

                        let score = if let Some(victim) = board.get_piece(mv.to) {
                            // MVV-LVA scoring for captures
                            CAPTURE_SCORE_BASE + get_mvv_lva_score(board, mv)
                        } else if let Some(promotion) = mv.promotion {
                            // Scoring for promotions
                            PROMOTION_SCORE_BASE + match promotion {
                                PieceType::Queen => 500,
                                PieceType::Rook => 400,
                                PieceType::Bishop | PieceType::Knight => 300,
                                _ => 0,
                            }
                        } else {
                            // History heuristic for quiet moves
                            let from_idx = ((mv.from.rank - 1) * 8 + (mv.from.file - 1)) as usize;
                            let to_idx = ((mv.to.rank - 1) * 8 + (mv.to.file - 1)) as usize;
                            history[from_idx][to_idx].min(HISTORY_SCORE_MAX)
                        };

                        moves.push(mv);
                        move_scores.push(score);
                    }
                }
            }
        }
    }

    // Sort moves by score
    let mut move_indices: Vec<usize> = (0..moves.len()).collect();
    move_indices.sort_unstable_by_key(|&i| -move_scores[i]);
    
    // Return sorted moves
    move_indices.into_iter().map(|i| moves[i]).collect()
}

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

// Most Valuable Victim - Least Valuable Attacker (MVV-LVA) scoring
fn get_mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let victim = board.get_piece(mv.to);
    let attacker = board.get_piece(mv.from);
    
    if let (Some(victim), Some(attacker)) = (victim, attacker) {
        let victim_value = get_piece_static_value(victim.piece_type);
        let attacker_value = get_piece_static_value(attacker.piece_type);
        
        // Consider piece mobility in the scoring
        let mobility_bonus = board.get_valid_moves(mv.to).len() as i32 * 5;
        
        // Prioritize captures that improve piece mobility
        victim_value * 100 - attacker_value * 10 + mobility_bonus
    } else {
        0
    }
}

fn get_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 1,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 0,
    }
}

// Helper function to determine if position is in endgame or in check
fn is_endgame_or_in_check(board: &Board) -> bool {
    let mut queens = 0;
    let mut pieces = 0;
    let current_color = board.current_turn();
    let mut king_attacked = false;

    // Count material and check if king is attacked
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                match piece.piece_type {
                    PieceType::Queen => queens += 1,
                    PieceType::Rook | PieceType::Bishop | PieceType::Knight => pieces += 1,
                    PieceType::King if piece.color == current_color => {
                        // Check if any opponent piece can attack the king
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

    // Consider it endgame if no queens or few pieces remain
    let is_endgame = queens == 0 || (queens == 2 && pieces <= 2);
    is_endgame || king_attacked
} 

fn update_history(history: &mut Vec<Vec<i32>>, mv: Move, bonus: u8) {
    let from_idx = ((mv.from.rank - 1) * 8 + (mv.from.file - 1)) as usize;
    let to_idx = ((mv.to.rank - 1) * 8 + (mv.to.file - 1)) as usize;
    
    history[from_idx][to_idx] += bonus as i32;
    
    // Prevent overflow by scaling down if necessary
    if history[from_idx][to_idx] > HISTORY_MAX {
        for row in history.iter_mut() {
            for cell in row.iter_mut() {
                *cell /= 2;
            }
        }
    }
}

fn get_history_score(history: &Vec<Vec<i32>>, mv: Move) -> i32 {
    let from_idx = ((mv.from.rank - 1) * 8 + (mv.from.file - 1)) as usize;
    let to_idx = ((mv.to.rank - 1) * 8 + (mv.to.file - 1)) as usize;
    history[from_idx][to_idx]
}

fn is_capture(board: &Board, mv: Move) -> bool {
    board.get_piece(mv.to).is_some()
}

fn gives_check(board: &Board) -> bool {
    let current_color = board.current_turn();
    
    // Find opponent's king
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

    if let Some(king_pos) = king_pos {
        // Check if any piece can attack the king
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

// Static Exchange Evaluation (SEE)
fn static_exchange_evaluation(board: &Board, mv: Move) -> i32 {
    let victim = board.get_piece(mv.to);
    let attacker = board.get_piece(mv.from);
    
    if let (Some(victim), Some(attacker)) = (victim, attacker) {
        let victim_value = get_piece_static_value(victim.piece_type);
        let attacker_value = get_piece_static_value(attacker.piece_type);
        
        // Simple SEE - just consider the immediate capture
        victim_value - attacker_value
    } else {
        0
    }
}

fn get_piece_static_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,
        PieceType::Knight => 325,
        PieceType::Bishop => 325,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
} 

// Helper function to detect tactical positions
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

// Helper function to check if a piece is hanging (can be captured without retaliation)
fn is_piece_hanging(board: &Board, pos: chess_core::Position) -> bool {
    if let Some(piece) = board.get_piece(pos) {
        let piece_value = get_piece_value(piece.piece_type);
        
        // Find lowest value attacker
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

// Helper function to get total material count
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

// Helper function to check if the game is in endgame phase
fn is_endgame_phase(board: &Board) -> bool {
    // Position is in endgame phase if:
    // 1. There are no queens or few pieces remain
    // 2. There are no pieces that can attack the king
    let mut queens = 0;
    let mut pieces = 0;
    let current_color = board.current_turn();
    let mut king_attacked = false;

    // Count material and check if king is attacked
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                match piece.piece_type {
                    PieceType::Queen => queens += 1,
                    PieceType::Rook | PieceType::Bishop | PieceType::Knight => pieces += 1,
                    PieceType::King if piece.color == current_color => {
                        // Check if any opponent piece can attack the king
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

    // Consider it endgame if no queens or few pieces remain
    let is_endgame = queens == 0 || (queens == 2 && pieces <= 2);
    is_endgame || king_attacked
} 

// Adjust mate scores by depth to prefer shorter paths to mate
fn adjust_mate_score(score: i32, depth: u8) -> i32 {
    if score > MATE_SCORE - 1000 {
        score - depth as i32
    } else if score < -MATE_SCORE + 1000 {
        score + depth as i32
    } else {
        score
    }
} 

// Update killer moves after a beta cutoff
fn update_killer_moves(killer_moves: &mut Option<[Move; 2]>, mv: Move) {
    let moves = killer_moves.get_or_insert([create_default_move(); 2]);
    
    if moves[0].from != mv.from || moves[0].to != mv.to {
        moves[1] = moves[0];
        moves[0] = mv;
    }
} 

// Helper function to determine if a capture is clearly winning
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