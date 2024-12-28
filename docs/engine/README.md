# Chess Engine Architecture Documentation

This document details the architectural design and implementation of the chess engine.

## Table of Contents
1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Data Structures](#data-structures)
4. [Engine Subsystems](#engine-subsystems)
5. [User Interface](#user-interface)
6. [Testing Framework](#testing-framework)

## Overview

The chess engine is organized into three main crates:
```
chess-engine/
├── crates/
│   ├── chess_core/    # Core chess logic and data structures
│   ├── chess_engine/  # Search and evaluation algorithms
│   └── chess_ui/      # Bevy-based user interface
```

### Design Principles
1. **Modularity**: Clear separation of concerns between components
2. **Performance**: Efficient data structures and algorithms
3. **Extensibility**: Easy to add new features and improvements
4. **Testability**: Comprehensive test coverage at all levels

## Core Components

### 1. Board Representation
```rust
pub struct Board {
    pieces: [[Option<Piece>; 8]; 8],
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
    zobrist_hash: u64,
}

impl Board {
    pub fn make_move(&mut self, mv: Move) -> Result<(), Error> {
        // Validate move
        if !self.is_legal_move(mv) {
            return Err(Error::IllegalMove);
        }
        
        // Update board state
        self.update_position(mv);
        self.update_castling_rights(mv);
        self.update_en_passant(mv);
        self.update_clocks(mv);
        self.update_hash(mv);
        
        Ok(())
    }
}
```

### 2. Move Generation
```rust
pub struct MoveGenerator {
    attack_tables: AttackTables,
    move_lists: Vec<Vec<Move>>,
}

impl MoveGenerator {
    pub fn generate_moves(&mut self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        
        // Generate piece moves
        self.generate_pawn_moves(board, &mut moves);
        self.generate_knight_moves(board, &mut moves);
        self.generate_bishop_moves(board, &mut moves);
        self.generate_rook_moves(board, &mut moves);
        self.generate_queen_moves(board, &mut moves);
        self.generate_king_moves(board, &mut moves);
        
        moves
    }
}
```

### 3. Position Evaluation
```rust
pub struct Evaluator {
    piece_square_tables: PieceSquareTables,
    pawn_hash_table: PawnHashTable,
    history_table: HistoryTable,
}

impl Evaluator {
    pub fn evaluate(&self, board: &Board) -> i32 {
        let material = self.evaluate_material(board);
        let position = self.evaluate_position(board);
        let king_safety = self.evaluate_king_safety(board);
        let mobility = self.evaluate_mobility(board);
        
        material + position + king_safety + mobility
    }
}
```

## Data Structures

### 1. Transposition Table
```rust
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let num_entries = (size_mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        TranspositionTable {
            entries: vec![TTEntry::default(); num_entries],
            size: num_entries,
        }
    }
    
    pub fn store(&mut self, key: u64, depth: i32, score: i32, flag: TTFlag, best_move: Move) {
        let index = self.index(key);
        let entry = &mut self.entries[index];
        
        if entry.should_replace(key, depth) {
            *entry = TTEntry::new(key, depth, score, flag, best_move);
        }
    }
}
```

### 2. Move Ordering
```rust
pub struct MoveOrderer {
    history_scores: [[i32; 64]; 64],
    killer_moves: Vec<[Option<Move>; 2]>,
    counter_moves: HashMap<Move, Move>,
}

impl MoveOrderer {
    pub fn score_moves(&mut self, moves: &mut Vec<Move>, board: &Board, tt_move: Option<Move>) {
        for mv in moves.iter_mut() {
            mv.score = self.calculate_move_score(mv, board, tt_move);
        }
        moves.sort_by_key(|mv| -mv.score);
    }
}
```

### 3. Search Stack
```rust
pub struct SearchStack {
    ply: i32,
    pv: Vec<Move>,
    killer_moves: [Option<Move>; 2],
    static_eval: i32,
    null_move_pruning_tried: bool,
}

pub struct SearchInfo {
    nodes: u64,
    depth: i32,
    selective_depth: i32,
    time_elapsed: Duration,
    pv: Vec<Move>,
}
```

## Engine Subsystems

### 1. Time Management
```rust
pub struct TimeManager {
    initial_time: Duration,
    increment: Duration,
    moves_to_go: Option<u32>,
    overhead: Duration,
}

impl TimeManager {
    pub fn allocate_time(&self, position: &Board) -> Duration {
        let base_time = self.calculate_base_time();
        let position_factor = self.position_complexity_factor(position);
        
        base_time * position_factor
    }
}
```

### 2. Search Control
```rust
pub struct SearchController {
    time_manager: TimeManager,
    search_limits: SearchLimits,
    stop_flag: Arc<AtomicBool>,
}

impl SearchController {
    pub fn start_search(&mut self, position: &Board) -> SearchResult {
        let allocated_time = self.time_manager.allocate_time(position);
        let mut search_info = SearchInfo::new();
        
        for depth in 1..=self.search_limits.max_depth {
            let score = self.search_iteration(position, depth, &mut search_info);
            
            if self.should_stop(allocated_time) {
                break;
            }
            
            self.update_best_move(score, search_info.pv[0]);
        }
        
        SearchResult::new(search_info)
    }
}
```

### 3. UCI Protocol
```rust
pub struct UciInterface {
    engine: Engine,
    options: UciOptions,
}

impl UciInterface {
    pub fn handle_command(&mut self, command: &str) {
        match command.split_whitespace().next() {
            Some("uci") => self.handle_uci(),
            Some("isready") => self.handle_isready(),
            Some("position") => self.handle_position(command),
            Some("go") => self.handle_go(command),
            Some("stop") => self.handle_stop(),
            Some("quit") => self.handle_quit(),
            _ => println!("Unknown command: {}", command),
        }
    }
}
```

## User Interface

### 1. Game State
```rust
pub struct GameState {
    board: Board,
    move_history: Vec<Move>,
    selected_square: Option<Square>,
    legal_moves: Vec<Move>,
    ai_thinking: bool,
}

impl GameState {
    pub fn handle_click(&mut self, square: Square) {
        match self.selected_square {
            Some(from) => self.try_move(from, square),
            None => self.select_square(square),
        }
    }
}
```

### 2. Rendering
```rust
pub struct ChessBoard {
    pieces: HashMap<Square, Entity>,
    squares: HashMap<Square, Entity>,
    highlights: HashMap<Square, Entity>,
}

impl ChessBoard {
    pub fn update(&mut self, commands: &mut Commands, board: &Board) {
        self.update_pieces(commands, board);
        self.update_highlights(commands);
        self.update_last_move(commands);
    }
}
```

### 3. Input Handling
```rust
pub fn handle_input(
    mut state: ResMut<GameState>,
    mouse_button: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(square) = get_clicked_square(windows) {
            state.handle_click(square);
        }
    }
}
```

## Testing Framework

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_generation() {
        let board = Board::initial_position();
        let moves = MoveGenerator::new().generate_moves(&board);
        assert_eq!(moves.len(), 20); // Initial position has 20 legal moves
    }
    
    #[test]
    fn test_evaluation() {
        let board = Board::initial_position();
        let eval = Evaluator::new().evaluate(&board);
        assert_eq!(eval, 0); // Initial position should be equal
    }
}
```

### 2. Integration Tests
```rust
#[test]
fn test_complete_game() {
    let mut engine = Engine::new();
    let mut board = Board::initial_position();
    
    // Play a complete game
    while !board.is_game_over() {
        let best_move = engine.search(&board, SearchLimits::default());
        board.make_move(best_move).unwrap();
    }
    
    // Verify final position
    assert!(board.is_checkmate() || board.is_stalemate());
}
```

### 3. Performance Tests
```rust
#[bench]
fn bench_move_generation(b: &mut Bencher) {
    let board = Board::initial_position();
    let mut generator = MoveGenerator::new();
    
    b.iter(|| {
        generator.generate_moves(&board);
    });
}

#[bench]
fn bench_evaluation(b: &mut Bencher) {
    let board = Board::initial_position();
    let evaluator = Evaluator::new();
    
    b.iter(|| {
        evaluator.evaluate(&board);
    });
}
```

## Future Improvements

### 1. Core Engine
- [ ] NNUE evaluation
- [ ] Syzygy tablebase support
- [ ] Better time management
- [ ] Improved search extensions

### 2. User Interface
- [ ] Analysis mode
- [ ] Move explanation
- [ ] Opening explorer
- [ ] Game database integration

### 3. Testing
- [ ] More extensive test positions
- [ ] Automated testing against other engines
- [ ] Performance regression tests
- [ ] Memory usage monitoring 