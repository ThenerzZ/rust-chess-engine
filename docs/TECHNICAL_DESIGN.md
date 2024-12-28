# Chess Engine Technical Design Document

## Table of Contents
1. [System Overview](#system-overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [Data Structures](#data-structures)
5. [Algorithms](#algorithms)
6. [Performance Optimizations](#performance-optimizations)
7. [Threading Model](#threading-model)
8. [Memory Management](#memory-management)
9. [Error Handling](#error-handling)
10. [Testing Strategy](#testing-strategy)

## System Overview

### Purpose
The chess engine is designed to provide a high-performance chess AI capable of playing at a strong level while maintaining efficient resource usage. It implements advanced search techniques and sophisticated evaluation methods.

### Key Features
- Principal Variation Search with iterative deepening
- Advanced pruning techniques
- Parallel search capabilities
- Sophisticated evaluation function
- Time management system
- Transposition table caching

### Design Goals
1. Performance
   - Fast move generation
   - Efficient search algorithms
   - Minimal memory overhead
   - Quick position evaluation

2. Correctness
   - Accurate move generation
   - Valid game state management
   - Proper rule enforcement
   - Reliable search results

3. Maintainability
   - Clean code structure
   - Comprehensive documentation
   - Extensive test coverage
   - Clear separation of concerns

## Architecture

### High-Level Design
```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Chess UI     │     │   Chess Engine  │     │   Chess Core    │
│  (Bevy-based)   │◄───►│    (Search)     │◄───►│  (Game Logic)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Component Interaction
1. UI to Engine
   - Move requests
   - Game state updates
   - Time control info
   - User commands

2. Engine to Core
   - Position analysis
   - Move validation
   - Game state queries
   - Rule enforcement

3. Core to Engine
   - Legal moves
   - Position evaluation
   - Game termination
   - State validation

## Core Components

### 1. Board Representation
```rust
pub struct Board {
    pieces: [[Option<Piece>; 8]; 8],
    current_turn: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Position>,
    halfmove_clock: u16,
    fullmove_number: u16,
}

pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

pub struct Position {
    rank: u8,
    file: u8,
}
```

### 2. Move Generation
```rust
pub struct Move {
    from: Position,
    to: Position,
    move_type: MoveType,
    promotion: Option<PieceType>,
}

pub enum MoveType {
    Normal,
    Capture,
    EnPassant,
    Castle,
    Promotion,
}

impl Board {
    pub fn get_valid_moves(&self, pos: Position) -> Vec<Move> {
        // Implementation details in move generation section
    }
}
```

### 3. Search System
```rust
pub struct SearchConfig {
    max_depth: u8,
    min_time: Duration,
    max_time: Duration,
    nodes_limit: Option<u64>,
}

pub struct SearchInfo {
    depth: u8,
    score: i32,
    nodes: u64,
    time: Duration,
    pv: Vec<Move>,
}

pub trait SearchEngine {
    fn search(&mut self, position: &Board, config: SearchConfig) -> SearchInfo;
    fn stop(&mut self);
    fn is_searching(&self) -> bool;
}
```

### 4. Evaluation System
```rust
pub struct EvaluationComponents {
    material: i32,
    position: i32,
    mobility: i32,
    king_safety: i32,
    pawn_structure: i32,
}

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> i32;
    fn evaluate_detailed(&self, board: &Board) -> EvaluationComponents;
}
```

## Data Structures

### 1. Transposition Table
```rust
pub struct TTEntry {
    key: u64,
    depth: u8,
    score: i32,
    entry_type: EntryType,
    best_move: Option<Move>,
}

pub enum EntryType {
    Exact,
    LowerBound,
    UpperBound,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    size: usize,
}
```

### 2. Move Ordering
```rust
pub struct MoveOrderer {
    history_table: [[i32; 64]; 64],
    killer_moves: Vec<[Option<Move>; 2]>,
    counter_moves: HashMap<Move, Move>,
}

impl MoveOrderer {
    pub fn score_moves(&self, moves: &mut Vec<Move>, board: &Board) {
        // Scoring implementation
    }
}
```

### 3. Piece-Square Tables
```rust
pub struct PieceSquareTables {
    middlegame: [[i32; 64]; 6],
    endgame: [[i32; 64]; 6],
}

impl PieceSquareTables {
    pub fn get_score(&self, piece: Piece, square: Position, phase: f32) -> i32 {
        // Score calculation
    }
}
```

## Algorithms

### 1. Principal Variation Search
```rust
fn principal_variation_search(
    board: &Board,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
) -> i32 {
    if depth == 0 || board.is_terminal() {
        return quiescence_search(board, alpha, beta);
    }

    let mut alpha = alpha;
    let moves = generate_ordered_moves(board);
    
    for (i, mv) in moves.iter().enumerate() {
        let score = if i == 0 {
            -principal_variation_search(board, depth - 1, -beta, -alpha, true)
        } else {
            let score = -principal_variation_search(board, depth - 1, -alpha - 1, -alpha, false);
            if score > alpha && score < beta {
                -principal_variation_search(board, depth - 1, -beta, -score, true)
            } else {
                score
            }
        };
        
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }
    
    alpha
}
```

### 2. Quiescence Search
```rust
fn quiescence_search(board: &Board, mut alpha: i32, beta: i32) -> i32 {
    let stand_pat = evaluate_position(board);
    
    if stand_pat >= beta {
        return beta;
    }
    
    alpha = alpha.max(stand_pat);
    
    let captures = generate_captures(board);
    for capture in captures {
        let score = -quiescence_search(board, -beta, -alpha);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }
    
    alpha
}
```

### 3. Move Generation
```rust
fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    
    // Piece moves
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == board.current_turn() {
                    moves.extend(generate_piece_moves(board, pos));
                }
            }
        }
    }
    
    // Special moves
    moves.extend(generate_castling_moves(board));
    moves.extend(generate_en_passant_moves(board));
    
    moves
}
```

## Performance Optimizations

### 1. Move Generation
- Pre-calculated attack tables
- Bitboard move generation
- Move validation caching
- Legal move filtering

### 2. Search Optimizations
- Transposition table
- Null move pruning
- Futility pruning
- Late move reduction
- History heuristic
- Killer move heuristic

### 3. Evaluation Optimizations
- Incremental evaluation
- Piece-square table caching
- Material balance tracking
- Evaluation pruning
- SIMD operations

## Threading Model

### 1. Main Thread
```rust
pub struct MainThread {
    board: Board,
    time_manager: TimeManager,
    search_info: SearchInfo,
    worker_threads: Vec<WorkerThread>,
}
```

### 2. Worker Threads
```rust
pub struct WorkerThread {
    id: usize,
    board: Board,
    tt: Arc<TranspositionTable>,
    sender: Sender<SearchResult>,
    receiver: Receiver<SearchCommand>,
}
```

### 3. Thread Communication
```rust
pub enum SearchCommand {
    StartSearch(SearchParams),
    StopSearch,
    Quit,
}

pub struct SearchResult {
    thread_id: usize,
    depth: u8,
    score: i32,
    best_move: Move,
    nodes: u64,
}
```

## Memory Management

### 1. Static Allocations
- Piece-square tables
- Attack tables
- Move generation masks
- Evaluation constants

### 2. Dynamic Allocations
- Transposition table
- Move lists
- Search tree nodes
- Game history

### 3. Resource Limits
```rust
pub struct ResourceLimits {
    max_memory: usize,
    max_threads: usize,
    tt_size: usize,
    stack_size: usize,
}
```

## Error Handling

### 1. Error Types
```rust
#[derive(Debug)]
pub enum ChessError {
    InvalidMove(String),
    IllegalPosition(String),
    TimeOut,
    ResourceExhausted,
    InternalError(String),
}
```

### 2. Result Types
```rust
pub type ChessResult<T> = Result<T, ChessError>;

impl Board {
    pub fn make_move(&mut self, mv: Move) -> ChessResult<()> {
        // Implementation
    }
}
```

## Testing Strategy

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_move_generation() {
        // Test implementation
    }
    
    #[test]
    fn test_evaluation() {
        // Test implementation
    }
}
```

### 2. Integration Tests
```rust
#[test]
fn test_complete_game() {
    // Test implementation
}

#[test]
fn test_search_quality() {
    // Test implementation
}
```

### 3. Performance Tests
```rust
#[bench]
fn bench_move_generation(b: &mut Bencher) {
    // Benchmark implementation
}

#[bench]
fn bench_search_speed(b: &mut Bencher) {
    // Benchmark implementation
}
``` 