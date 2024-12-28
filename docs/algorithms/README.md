# Chess Algorithms Documentation

## Overview

This document details the key algorithms used in the chess engine, including move generation, position analysis, and various chess-specific algorithms.

## Move Generation

### 1. Piece Movement Patterns

#### Basic Movement
```rust
const KNIGHT_MOVES: [(i8, i8); 8] = [
    (-2, -1), (-2, 1), (-1, -2), (-1, 2),
    (1, -2), (1, 2), (2, -1), (2, 1)
];

const BISHOP_DIRECTIONS: [(i8, i8); 4] = [
    (-1, -1), (-1, 1), (1, -1), (1, 1)
];

const ROOK_DIRECTIONS: [(i8, i8); 4] = [
    (-1, 0), (1, 0), (0, -1), (0, 1)
];
```

#### Sliding Piece Generation
```rust
fn generate_sliding_moves(pos: Position, directions: &[(i8, i8)], board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    for &(dx, dy) in directions {
        let mut x = pos.file as i8;
        let mut y = pos.rank as i8;
        loop {
            x += dx;
            y += dy;
            if !is_valid_square(x, y) {
                break;
            }
            let new_pos = Position { 
                rank: y as u8, 
                file: x as u8 
            };
            if let Some(piece) = board.get_piece(new_pos) {
                if piece.color != board.current_turn() {
                    moves.push(Move::new(pos, new_pos));
                }
                break;
            }
            moves.push(Move::new(pos, new_pos));
        }
    }
    moves
}
```

### 2. Special Move Generation

#### Castling
```rust
fn generate_castling_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    let rights = board.castling_rights();
    let color = board.current_turn();
    
    if color == Color::White {
        if rights.white_kingside && !is_path_attacked(board, "e1", "g1") {
            moves.push(Move::castle_kingside());
        }
        if rights.white_queenside && !is_path_attacked(board, "e1", "c1") {
            moves.push(Move::castle_queenside());
        }
    } else {
        // Similar logic for black
    }
    moves
}
```

#### En Passant
```rust
fn generate_en_passant_moves(board: &Board, pos: Position) -> Vec<Move> {
    let mut moves = Vec::new();
    if let Some(ep_square) = board.en_passant() {
        if board.get_piece(pos).map_or(false, |p| p.piece_type == PieceType::Pawn) {
            // Check if pawn can capture en passant
            if (pos.file as i8 - ep_square.file as i8).abs() == 1 {
                moves.push(Move::en_passant(pos, ep_square));
            }
        }
    }
    moves
}
```

### 3. Move Validation

#### Check Detection
```rust
fn is_in_check(board: &Board, color: Color) -> bool {
    let king_pos = find_king(board, color);
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color != color {
                    let moves = generate_pseudo_legal_moves(board, pos);
                    if moves.iter().any(|m| m.to == king_pos) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
```

#### Pin Detection
```rust
fn find_pins(board: &Board, color: Color) -> Vec<(Position, Position)> {
    let mut pins = Vec::new();
    let king_pos = find_king(board, color);
    
    // Check all directions for pins
    for &dir in SLIDING_DIRECTIONS.iter() {
        let mut pinned_piece = None;
        let mut x = king_pos.file as i8;
        let mut y = king_pos.rank as i8;
        
        loop {
            x += dir.0;
            y += dir.1;
            if !is_valid_square(x, y) {
                break;
            }
            let pos = Position { 
                rank: y as u8, 
                file: x as u8 
            };
            
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == color {
                    if pinned_piece.is_some() {
                        break;
                    }
                    pinned_piece = Some(pos);
                } else {
                    if is_pinning_piece(piece.piece_type, dir) {
                        if let Some(pinned) = pinned_piece {
                            pins.push((pinned, pos));
                        }
                    }
                    break;
                }
            }
        }
    }
    pins
}
```

## Position Analysis

### 1. Attack Maps

#### Square Control
```rust
fn generate_attack_map(board: &Board, color: Color) -> [[bool; 8]; 8] {
    let mut attack_map = [[false; 8]; 8];
    
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == color {
                    let attacks = generate_attacks(board, pos);
                    for attack in attacks {
                        attack_map[attack.rank as usize - 1][attack.file as usize - 1] = true;
                    }
                }
            }
        }
    }
    attack_map
}
```

#### Piece Mobility
```rust
fn calculate_mobility(board: &Board, pos: Position) -> i32 {
    let piece = board.get_piece(pos).unwrap();
    let moves = generate_pseudo_legal_moves(board, pos);
    let safe_squares = moves.iter()
        .filter(|&m| !is_square_attacked(board, m.to, !piece.color))
        .count();
    
    safe_squares as i32 * MOBILITY_WEIGHTS[piece.piece_type as usize]
}
```

### 2. Pattern Recognition

#### Pawn Structure Analysis
```rust
fn analyze_pawn_structure(board: &Board, color: Color) -> PawnStructure {
    let mut structure = PawnStructure::default();
    
    // Find pawn chains
    for file in 1..=8 {
        let mut chain_length = 0;
        for rank in 1..=8 {
            let pos = Position { rank, file };
            if is_pawn_at(board, pos, color) {
                chain_length += 1;
                if is_protected_by_pawn(board, pos, color) {
                    structure.protected_pawns += 1;
                }
            }
        }
        structure.chains.push(chain_length);
    }
    
    // Identify isolated and doubled pawns
    for file in 1..=8 {
        let pawns_on_file = count_pawns_on_file(board, file, color);
        if pawns_on_file > 1 {
            structure.doubled_pawns += pawns_on_file - 1;
        }
        if pawns_on_file > 0 && !has_adjacent_pawns(board, file, color) {
            structure.isolated_pawns += 1;
        }
    }
    
    structure
}
```

#### King Safety Analysis
```rust
fn analyze_king_safety(board: &Board, color: Color) -> KingSafety {
    let king_pos = find_king(board, color);
    let mut safety = KingSafety::default();
    
    // Analyze pawn shield
    safety.pawn_shield = count_pawn_shield(board, king_pos, color);
    
    // Count attackers
    let enemy_color = !color;
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                if piece.color == enemy_color {
                    let attacks = generate_attacks(board, pos);
                    if attacks.iter().any(|&a| is_in_king_zone(a, king_pos)) {
                        safety.attackers += 1;
                        safety.attack_value += ATTACK_WEIGHTS[piece.piece_type as usize];
                    }
                }
            }
        }
    }
    
    safety
}
```

## Search Tree Algorithms

### 1. Move Ordering

#### History Heuristic
```rust
fn order_moves(moves: &mut Vec<Move>, history: &[[i32; 64]; 64]) {
    moves.sort_by_key(|&mv| {
        let from_idx = (mv.from.rank - 1) * 8 + (mv.from.file - 1);
        let to_idx = (mv.to.rank - 1) * 8 + (mv.to.file - 1);
        -history[from_idx as usize][to_idx as usize]
    });
}
```

#### MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
```rust
fn mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let victim = board.get_piece(mv.to);
    let attacker = board.get_piece(mv.from).unwrap();
    
    if let Some(victim) = victim {
        PIECE_VALUES[victim.piece_type as usize] * 10 -
        PIECE_VALUES[attacker.piece_type as usize]
    } else {
        0
    }
}
```

### 2. Position Evaluation

#### Material Balance
```rust
fn evaluate_material(board: &Board) -> i32 {
    let mut score = 0;
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let value = PIECE_VALUES[piece.piece_type as usize];
                score += if piece.color == Color::White { value } else { -value };
            }
        }
    }
    score
}
```

#### Positional Scoring
```rust
fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;
    let phase = get_game_phase(board);
    
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { rank, file };
            if let Some(piece) = board.get_piece(pos) {
                let sq_idx = (rank - 1) * 8 + (file - 1);
                let mg_score = PIECE_SQUARE_TABLES_MG[piece.piece_type as usize][sq_idx];
                let eg_score = PIECE_SQUARE_TABLES_EG[piece.piece_type as usize][sq_idx];
                
                let piece_score = interpolate(mg_score, eg_score, phase);
                score += if piece.color == Color::White { 
                    piece_score 
                } else { 
                    -piece_score 
                };
            }
        }
    }
    score
}
```

## Future Improvements

### 1. Move Generation
- [ ] Bitboard-based move generation
- [ ] Magic bitboard sliding piece moves
- [ ] More efficient pin detection
- [ ] Better move ordering heuristics

### 2. Position Analysis
- [ ] Improved pawn structure evaluation
- [ ] Better king safety assessment
- [ ] More sophisticated mobility calculation
- [ ] Pattern-based tactical detection

### 3. Search Algorithms
- [ ] Neural network evaluation
- [ ] Better pruning techniques
- [ ] Improved time management
- [ ] Selective search extensions 

# Search Algorithms Documentation

This document details the search algorithms and optimizations used in the chess engine.

## Table of Contents
1. [Principal Variation Search](#principal-variation-search)
2. [Search Optimizations](#search-optimizations)
3. [Move Ordering](#move-ordering)
4. [Time Management](#time-management)
5. [Parallel Search](#parallel-search)

## Principal Variation Search

### Overview
Principal Variation Search (PVS) is an enhancement of the alpha-beta algorithm that assumes the first move at each node is the best move. This assumption allows for faster pruning of the search tree.

### Implementation
```rust
fn principal_variation_search(
    board: &Board,
    depth: i32,
    alpha: i32,
    beta: i32,
    pv_node: bool,
) -> i32 {
    if depth <= 0 {
        return quiescence_search(board, alpha, beta);
    }

    let mut alpha = alpha;
    let moves = generate_ordered_moves(board);
    
    for (i, mv) in moves.iter().enumerate() {
        let score = if i == 0 {
            // Full window search for first move
            -principal_variation_search(board, depth - 1, -beta, -alpha, true)
        } else {
            // Null window search for remaining moves
            let score = -principal_variation_search(board, depth - 1, -alpha - 1, -alpha, false);
            if score > alpha && score < beta {
                // Re-search with full window if the move might be better
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

### Key Components
1. **Iterative Deepening**
   - Searches progressively deeper
   - Improves move ordering
   - Provides anytime behavior

2. **Aspiration Windows**
   - Uses narrow windows around previous score
   - Falls back to full window if needed
   - Reduces search space

3. **Quiescence Search**
   - Evaluates tactical sequences
   - Prevents horizon effect
   - Considers captures and checks

## Search Optimizations

### 1. Transposition Table
```rust
pub struct TTEntry {
    key: u64,
    depth: i32,
    score: i32,
    flag: TTFlag,
    best_move: Option<Move>,
}

impl TranspositionTable {
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = self.index(key);
        let entry = &self.entries[index];
        
        if entry.key == key {
            Some(entry.clone())
        } else {
            None
        }
    }
}
```

### 2. Null Move Pruning
```rust
fn null_move_pruning(board: &Board, depth: i32, beta: i32) -> Option<i32> {
    if depth >= 3 && !board.in_check() {
        let score = -principal_variation_search(board, depth - 3, -beta, -beta + 1, false);
        if score >= beta {
            return Some(beta);
        }
    }
    None
}
```

### 3. Futility Pruning
```rust
fn futility_pruning(board: &Board, depth: i32, alpha: i32) -> bool {
    if depth < 3 && !board.in_check() {
        let eval = evaluate_position(board);
        let margin = FUTILITY_MARGINS[depth as usize];
        if eval + margin <= alpha {
            return true;
        }
    }
    false
}
```

### 4. Late Move Reduction
```rust
fn late_move_reduction(depth: i32, move_number: usize) -> i32 {
    if depth >= 3 && move_number >= 4 {
        1 + (depth / 3).min(move_number as i32 / 6)
    } else {
        0
    }
}
```

## Move Ordering

### Priority System
1. Hash move from transposition table
2. Winning captures (MVV-LVA)
3. Equal captures
4. Killer moves
5. History moves
6. Quiet moves

### Implementation
```rust
pub struct MoveOrderer {
    history_table: [[i32; 64]; 64],
    killer_moves: Vec<[Option<Move>; 2]>,
    counter_moves: HashMap<Move, Move>,
}

impl MoveOrderer {
    pub fn score_moves(&mut self, moves: &mut Vec<Move>, board: &Board, tt_move: Option<Move>) {
        for mv in moves.iter_mut() {
            let score = self.calculate_move_score(mv, board, tt_move);
            mv.score = score;
        }
        moves.sort_by_key(|mv| -mv.score);
    }

    fn calculate_move_score(&self, mv: &Move, board: &Board, tt_move: Option<Move>) -> i32 {
        if Some(mv) == tt_move {
            return 2_000_000;
        }

        match mv.move_type {
            MoveType::Capture => {
                let victim = board.piece_at(mv.to).unwrap();
                let attacker = board.piece_at(mv.from).unwrap();
                1_000_000 + mvv_lva_score(victim, attacker)
            }
            MoveType::Quiet => {
                let mut score = 0;
                if self.is_killer_move(mv) {
                    score += 500_000;
                }
                score += self.history_table[mv.from.to_index()][mv.to.to_index()];
                score
            }
            _ => 0,
        }
    }
}
```

## Time Management

### Strategy
1. **Initial Allocation**
   - Based on remaining time
   - Considers game phase
   - Reserves time buffer

2. **Dynamic Adjustment**
   - Extends time in critical positions
   - Reduces time in forced moves
   - Handles time pressure

### Implementation
```rust
pub struct TimeManager {
    initial_time: Duration,
    increment: Duration,
    moves_to_go: Option<u32>,
    buffer_time: Duration,
}

impl TimeManager {
    pub fn allocate_time(&self, position: &Board) -> Duration {
        let base_time = self.calculate_base_time();
        let position_factor = self.position_complexity_factor(position);
        let phase_factor = self.game_phase_factor(position);
        
        base_time * position_factor * phase_factor
    }

    fn calculate_base_time(&self) -> Duration {
        match self.moves_to_go {
            Some(moves) => self.initial_time / moves as u32,
            None => self.initial_time / 30, // Estimate 30 more moves
        }
    }
}
```

## Parallel Search

### Architecture
1. **Main Thread**
   - Manages search parameters
   - Coordinates worker threads
   - Handles time management

2. **Worker Threads**
   - Search different subtrees
   - Share transposition table
   - Report results to main thread

### Implementation
```rust
pub struct SearchManager {
    threads: Vec<SearchThread>,
    tt: Arc<TranspositionTable>,
    time_manager: TimeManager,
}

impl SearchManager {
    pub fn start_search(&mut self, position: &Board, limits: SearchLimits) {
        let (tx, rx) = mpsc::channel();
        
        for thread in &mut self.threads {
            let tx = tx.clone();
            thread.start_search(position.clone(), limits.clone(), tx);
        }
        
        self.collect_results(rx);
    }

    fn collect_results(&mut self, rx: Receiver<SearchResult>) {
        while let Ok(result) = rx.recv() {
            self.update_best_move(result);
        }
    }
}
```

### Thread Synchronization
1. **Shared Resources**
   - Transposition table
   - History tables
   - Best move information

2. **Split Points**
   - Dynamic load balancing
   - Work stealing
   - Lazy SMP implementation

## Performance Considerations

### 1. Node Count Targets
- Early game: 1-2M nodes
- Middle game: 2-4M nodes
- Endgame: 3-5M nodes

### 2. Depth Targets
- Early game: 6-8 ply
- Middle game: 8-12 ply
- Endgame: 10-14 ply

### 3. Memory Usage
- Transposition table: 256MB - 1GB
- Per-thread stack: 8MB
- Total limit: 2GB

## Future Improvements

### 1. Search Enhancements
- [ ] Multi-PV search
- [ ] Internal iterative deepening
- [ ] Singular extensions
- [ ] Countermove history

### 2. Parallel Search
- [ ] NUMA awareness
- [ ] Better load balancing
- [ ] Young Brothers Wait
- [ ] Dynamic thread count

### 3. Time Management
- [ ] Position complexity analysis
- [ ] Pattern-based time allocation
- [ ] Better endgame time usage
- [ ] Tournament time management 