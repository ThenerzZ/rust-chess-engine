use chess_core::{Board, Move, piece::PieceType};
use crate::evaluation::evaluate_position;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Safe score bounds that won't overflow when negated
const MATE_SCORE: i32 = 30000;
const ALPHA_INIT: i32 = -MATE_SCORE;
const BETA_INIT: i32 = MATE_SCORE;
const QUIESCENCE_DEPTH: u8 = 4;
const R: u8 = 2; // Null move reduction depth

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

pub fn search_best_move(board: &Board, depth: u8) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = ALPHA_INIT;
    let mut tt = TRANSPOSITION_TABLE.lock().unwrap();

    // Clear TT if it's getting too large
    if tt.len() > 900_000 {
        tt.clear();
    }

    let moves = generate_ordered_moves(board);
    
    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax_with_quiescence(&new_board, depth - 1, -BETA_INIT, -best_score, &mut tt);
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
            }
        }
    }

    best_move
}

fn negamax_with_quiescence(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, tt: &mut HashMap<String, TTEntry>) -> i32 {
    // Adjust mate scores by depth to prefer shorter paths to mate
    let mate_score = MATE_SCORE - depth as i32;
    
    if board.is_checkmate() {
        return -mate_score;
    }
    if board.is_stalemate() {
        return 0;
    }

    let alpha_orig = alpha;
    let pos_key = get_position_key(board);

    // Transposition table lookup
    if let Some(entry) = tt.get(&pos_key) {
        if entry.depth >= depth {
            // Adjust stored mate scores
            let score = if entry.score > MATE_SCORE - 1000 {
                entry.score - depth as i32
            } else if entry.score < -MATE_SCORE + 1000 {
                entry.score + depth as i32
            } else {
                entry.score
            };

            match entry.entry_type {
                EntryType::Exact => return score,
                EntryType::LowerBound => {
                    alpha = alpha.max(score);
                    if alpha >= beta {
                        return score;
                    }
                }
                EntryType::UpperBound => {
                    beta = beta.min(score);
                    if alpha >= beta {
                        return score;
                    }
                }
            }
        }
    }

    if depth == 0 {
        return quiescence_search(board, alpha, beta, QUIESCENCE_DEPTH);
    }

    // Null move pruning
    if depth >= 3 && !is_endgame_or_in_check(board) {
        let score = -negamax_with_quiescence(board, depth - R - 1, -beta, -beta + 1, tt);
        if score >= beta {
            return beta;
        }
    }

    let mut best_score = ALPHA_INIT;
    let moves = generate_ordered_moves(board);

    if moves.is_empty() {
        return 0; // Stalemate
    }

    let mut best_move = None;

    for chess_move in moves {
        let mut new_board = board.clone();
        if new_board.make_move(chess_move).is_ok() {
            let score = -negamax_with_quiescence(&new_board, depth - 1, -beta, -alpha, tt);
            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);
                alpha = alpha.max(score);
            }
            if alpha >= beta {
                break; // Beta cutoff
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

    alpha = alpha.max(stand_pat);

    let captures = generate_captures(board);
    
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

fn generate_ordered_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    
    // First, add captures
    let mut captures = generate_captures(board);
    captures.sort_by_cached_key(|m| {
        -get_mvv_lva_score(board, *m)
    });
    moves.extend(captures);
    
    // Then, add non-captures
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = chess_core::Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    let piece_moves = board.get_valid_moves(pos);
                    for mv in piece_moves {
                        if board.get_piece(mv.to).is_none() {
                            moves.push(mv);
                        }
                    }
                }
            }
        }
    }
    
    moves
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
        let victim_value = get_piece_value(victim.piece_type);
        let attacker_value = get_piece_value(attacker.piece_type);
        victim_value * 10 - attacker_value
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