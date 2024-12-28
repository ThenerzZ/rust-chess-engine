use chess_core::{Board, Move, Position, piece::PieceType, moves::MoveType};
use crate::evaluation::evaluate_position;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use once_cell::sync::Lazy;

// Safe score bounds that won't overflow when negated
const MATE_SCORE: i32 = 20000;
const ALPHA_INIT: i32 = -19000;
const BETA_INIT: i32 = 19000;
const QUIESCENCE_DEPTH: u8 = 8;
const R: u8 = 2; // Reduced to be less aggressive with null move pruning

// Constants for Late Move Reduction
const LMR_DEPTH_THRESHOLD: u8 = 4;
const LMR_MOVE_THRESHOLD: usize = 4;
const HISTORY_MAX: i32 = 16000;

// Futility pruning margins - more conservative
const FUTILITY_MARGIN: [i32; 4] = [0, 150, 300, 450];

// Delta pruning threshold - more conservative
const DELTA_MARGIN: i32 = 300;

// Minimum game phase for null move pruning
const NULL_MOVE_MATERIAL_THRESHOLD: i32 = 1500;

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
    Lazy::new(|| Mutex::new(HashMap::with_capacity(1_000_000)));

// History table for move ordering
static HISTORY_TABLE: Lazy<Mutex<Vec<Vec<i32>>>> = 
    Lazy::new(|| Mutex::new(vec![vec![0; 64]; 64]));

// Add at the top with other constants
const MAX_PV_LENGTH: usize = 64;
static PV_TABLE: Lazy<Mutex<Vec<Move>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(MAX_PV_LENGTH)));
static KILLER_MOVES: Lazy<Mutex<Vec<Option<[Move; 2]>>>> = Lazy::new(|| Mutex::new(vec![None; MAX_PV_LENGTH]));

// Helper function to create a default move
fn create_default_move() -> Move {
    Move {
        from: Position { rank: 0, file: 0 },
        to: Position { rank: 0, file: 0 },
        move_type: MoveType::Normal,
        promotion: None,
    }
}

pub fn search_best_move(board: &Board, depth: u8) -> Option<Move> {
    let mut tt = TRANSPOSITION_TABLE.lock().unwrap();
    let mut history = HISTORY_TABLE.lock().unwrap();
    let mut pv_table = PV_TABLE.lock().unwrap();
    let killer_moves = KILLER_MOVES.lock().unwrap();
    
    pv_table.clear();
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;

    let default_moves = [create_default_move(); 2];
    let moves = generate_ordered_moves(
        board, 
        &*history,
        &*pv_table,
        killer_moves[depth as usize].as_ref().unwrap_or(&default_moves)
    );

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax_with_quiescence(&new_board, depth - 1, -BETA_INIT, -best_score, &mut tt, &mut *history);
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
                pv_table.push(chess_move);
            }
        }
    }

    best_move
}

fn negamax_with_quiescence(
    board: &Board,
    mut depth: u8,
    mut alpha: i32,
    mut beta: i32, 
    tt: &mut HashMap<String, TTEntry>,
    history: &mut Vec<Vec<i32>>
) -> i32 {
    // Adjust mate scores by depth to prefer shorter paths to mate
    let mate_score = MATE_SCORE - depth as i32;
    
    if board.is_checkmate() {
        return -MATE_SCORE + depth as i32;
    }
    if board.is_stalemate() {
        return 0;
    }

    let alpha_orig = alpha;
    let pos_key = get_position_key(board);

    // Transposition table lookup
    if let Some(entry) = tt.get(&pos_key) {
        if entry.depth >= depth {
            let score = adjust_mate_score(entry.score, depth);
            match entry.entry_type {
                EntryType::Exact => return score,
                EntryType::LowerBound => alpha = alpha.max(score),
                EntryType::UpperBound => beta = beta.min(score),
            }
            if alpha >= beta {
                return score;
            }
        }
    }

    if depth == 0 {
        return quiescence_search(board, alpha, beta, QUIESCENCE_DEPTH);
    }

    // Check extension - look deeper if in check
    let in_check = is_endgame_or_in_check(board);
    if in_check {
        depth += 1;
    }

    // More selective futility pruning
    if depth <= 3 && !in_check {
        let eval = evaluate_position(board);
        // Only apply futility pruning if not in a tactical position
        if !is_tactical_position(board) && eval + FUTILITY_MARGIN[depth as usize] <= alpha {
            return eval;
        }
    }

    // More selective null move pruning
    if depth >= 3 && !in_check && !is_endgame_phase(board) {
        let material = get_material_count(board);
        if material >= NULL_MOVE_MATERIAL_THRESHOLD {
            let r = if depth >= 6 { R + 1 } else { R };
            let score = -negamax_with_quiescence(board, depth - r - 1, -beta, -beta + 1, tt, history);
            if score >= beta {
                return beta;
            }
        }
    }

    let mut best_score = ALPHA_INIT;
    let default_moves = [create_default_move(); 2];
    let pv_table = &*PV_TABLE.lock().unwrap();
    let killer_moves = &*KILLER_MOVES.lock().unwrap();
    let moves = generate_ordered_moves(
        board, 
        history,
        pv_table,
        killer_moves[depth as usize].as_ref().unwrap_or(&default_moves)
    );

    if moves.is_empty() {
        return 0; // Stalemate
    }

    let mut best_move = None;
    let mut searched_moves = 0;

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            searched_moves += 1;
            let mut score;

            // Late Move Reduction
            if depth >= LMR_DEPTH_THRESHOLD && searched_moves > LMR_MOVE_THRESHOLD 
                && !is_capture(board, chess_move) && !gives_check(&new_board) {
                // Reduced depth search
                score = -negamax_with_quiescence(&new_board, depth - 2, -beta, -alpha, tt, history);
                // Re-search if the reduced search was promising
                if score > alpha {
                    score = -negamax_with_quiescence(&new_board, depth - 1, -beta, -alpha, tt, history);
                }
            } else {
                score = -negamax_with_quiescence(&new_board, depth - 1, -beta, -alpha, tt, history);
            }

            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
                alpha = alpha.max(score);

                // Update history table for good quiet moves
                if !is_capture(board, chess_move) {
                    update_history(history, chess_move, depth);
                }
            }
            if alpha >= beta {
                // Update killer moves and history for beta cutoffs
                if !is_capture(board, chess_move) {
                    update_history(history, chess_move, depth * 2);
                }
                break;
            }
        }
    }

    // Store position in transposition table
    let entry_type = if best_score <= alpha_orig {
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
    let stand_pat = evaluate_position(board);
    
    if depth == 0 {
        return stand_pat;
    }

    if stand_pat >= beta {
        return beta;
    }

    // Delta pruning
    if stand_pat < alpha - DELTA_MARGIN {
        return alpha;
    }

    alpha = alpha.max(stand_pat);

    let mut captures = generate_captures(board);
    
    // Sort captures by MVV-LVA
    captures.sort_by_cached_key(|m| -get_mvv_lva_score(board, *m));
    
    // SEE pruning - skip bad captures
    captures.retain(|m| static_exchange_evaluation(board, *m) >= 0);

    for capture in captures {
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
    killer_moves: &[Move; 2]
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut move_scores = Vec::new();
    
    // Collect all moves with their scores
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    for mv in board.get_valid_moves(pos) {
                        let score = if pv_table.iter().any(|&m| m.from == mv.from && m.to == mv.to) {
                            1_000_000  // PV moves first
                        } else if board.get_piece(mv.to).is_some() {
                            100_000 + get_mvv_lva_score(board, mv)  // Captures
                        } else if killer_moves.iter().any(|&m| m.from == mv.from && m.to == mv.to) {
                            10_000  // Killer moves
                        } else {
                            get_history_score(history, mv)  // History moves
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
    move_indices.sort_by_key(|&i| -move_scores[i]);
    
    // Return moves in sorted order
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