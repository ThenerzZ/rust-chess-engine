# Position Evaluation Documentation

This document details the evaluation system used in the chess engine to assess chess positions.

## Table of Contents
1. [Overview](#overview)
2. [Material Evaluation](#material-evaluation)
3. [Piece Positioning](#piece-positioning)
4. [King Safety](#king-safety)
5. [Pawn Structure](#pawn-structure)
6. [Mobility](#mobility)
7. [Game Phase](#game-phase)
8. [Evaluation Tuning](#evaluation-tuning)

## Overview

The evaluation function combines multiple components to produce a single score that represents the position's value from White's perspective. A positive score indicates White is better, while a negative score favors Black.

### Basic Structure
```rust
pub struct Evaluation {
    material: i32,
    position: i32,
    mobility: i32,
    king_safety: i32,
    pawn_structure: i32,
    tempo: i32,
}

impl Evaluation {
    pub fn evaluate(&self, board: &Board) -> i32 {
        let phase = self.calculate_game_phase(board);
        let mg_score = self.evaluate_middlegame(board);
        let eg_score = self.evaluate_endgame(board);
        
        interpolate(mg_score, eg_score, phase)
    }
}
```

## Material Evaluation

### Piece Values
```rust
pub const PIECE_VALUES: [i32; 6] = [
    100,   // Pawn
    320,   // Knight
    330,   // Bishop
    500,   // Rook
    900,   // Queen
    20000, // King
];

pub const PIECE_VALUES_ENDGAME: [i32; 6] = [
    120,   // Pawn (more valuable in endgame)
    310,   // Knight (slightly less valuable)
    330,   // Bishop
    520,   // Rook (more valuable)
    900,   // Queen
    20000, // King
];
```

### Implementation
```rust
impl Evaluation {
    fn evaluate_material(&self, board: &Board) -> i32 {
        let mut score = 0;
        
        for piece in board.pieces() {
            let value = if piece.color == Color::White {
                PIECE_VALUES[piece.piece_type as usize]
            } else {
                -PIECE_VALUES[piece.piece_type as usize]
            };
            score += value;
        }
        
        score
    }
}
```

## Piece Positioning

### Piece-Square Tables
```rust
pub struct PieceSquareTables {
    middlegame: [[i32; 64]; 6],
    endgame: [[i32; 64]; 6],
}

// Example table for knights in middlegame
const KNIGHT_TABLE_MG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];
```

### Implementation
```rust
impl Evaluation {
    fn evaluate_piece_positioning(&self, board: &Board, phase: f32) -> i32 {
        let mut score = 0;
        
        for (square, piece) in board.piece_locations() {
            let mg_score = self.tables.middlegame[piece.type_idx()][square.to_index()];
            let eg_score = self.tables.endgame[piece.type_idx()][square.to_index()];
            
            let interpolated = interpolate(mg_score, eg_score, phase);
            score += if piece.color == Color::White { interpolated } else { -interpolated };
        }
        
        score
    }
}
```

## King Safety

### Evaluation Factors
1. Pawn shield
2. Open files near king
3. Attack patterns
4. Piece tropism

### Implementation
```rust
impl Evaluation {
    fn evaluate_king_safety(&self, board: &Board) -> i32 {
        let mut white_safety = self.evaluate_king_zone(board, Color::White);
        let mut black_safety = self.evaluate_king_zone(board, Color::Black);
        
        // Adjust based on attackers
        white_safety -= self.count_king_attackers(board, Color::White) * 10;
        black_safety -= self.count_king_attackers(board, Color::Black) * 10;
        
        // Pawn shield bonus
        white_safety += self.evaluate_pawn_shield(board, Color::White);
        black_safety += self.evaluate_pawn_shield(board, Color::Black);
        
        white_safety - black_safety
    }
    
    fn evaluate_pawn_shield(&self, board: &Board, color: Color) -> i32 {
        let king_pos = board.king_position(color);
        let mut score = 0;
        
        // Check pawns in front of king
        for file in -1..=1 {
            let shield_pos = king_pos.offset_file(file);
            if let Some(pos) = shield_pos {
                if board.has_pawn_at(pos, color) {
                    score += 10;
                }
            }
        }
        
        score
    }
}
```

## Pawn Structure

### Evaluation Factors
1. Passed pawns
2. Isolated pawns
3. Doubled pawns
4. Pawn chains
5. Backward pawns

### Implementation
```rust
impl Evaluation {
    fn evaluate_pawn_structure(&self, board: &Board) -> i32 {
        let mut score = 0;
        
        // Evaluate white pawns
        for pawn in board.white_pawns() {
            score += self.evaluate_single_pawn(board, pawn, Color::White);
        }
        
        // Evaluate black pawns
        for pawn in board.black_pawns() {
            score -= self.evaluate_single_pawn(board, pawn, Color::Black);
        }
        
        score
    }
    
    fn evaluate_single_pawn(&self, board: &Board, pos: Square, color: Color) -> i32 {
        let mut score = 0;
        
        // Passed pawn bonus
        if self.is_passed_pawn(board, pos, color) {
            score += 30;
        }
        
        // Isolated pawn penalty
        if self.is_isolated_pawn(board, pos, color) {
            score -= 20;
        }
        
        // Doubled pawn penalty
        if self.is_doubled_pawn(board, pos, color) {
            score -= 15;
        }
        
        score
    }
}
```

## Mobility

### Evaluation Factors
1. Number of legal moves
2. Control of center
3. Piece development
4. Piece coordination

### Implementation
```rust
impl Evaluation {
    fn evaluate_mobility(&self, board: &Board) -> i32 {
        let white_mobility = self.calculate_piece_mobility(board, Color::White);
        let black_mobility = self.calculate_piece_mobility(board, Color::Black);
        
        white_mobility - black_mobility
    }
    
    fn calculate_piece_mobility(&self, board: &Board, color: Color) -> i32 {
        let mut mobility = 0;
        
        for piece in board.pieces_of_color(color) {
            let moves = board.generate_moves_for_piece(piece);
            mobility += MOBILITY_BONUS[piece.type_idx()] * moves.len() as i32;
        }
        
        mobility
    }
}

const MOBILITY_BONUS: [i32; 6] = [
    0,   // Pawn
    4,   // Knight
    3,   // Bishop
    2,   // Rook
    1,   // Queen
    0,   // King
];
```

## Game Phase

### Phase Calculation
```rust
impl Evaluation {
    fn calculate_game_phase(&self, board: &Board) -> f32 {
        let total_phase = 24; // 4 minor pieces + 4 rooks + 2 queens
        let mut phase = total_phase;
        
        phase -= board.piece_count(PieceType::Knight) as i32;
        phase -= board.piece_count(PieceType::Bishop) as i32;
        phase -= board.piece_count(PieceType::Rook) as i32 * 2;
        phase -= board.piece_count(PieceType::Queen) as i32 * 4;
        
        (phase as f32) / (total_phase as f32)
    }
    
    fn interpolate(&self, mg_score: i32, eg_score: i32, phase: f32) -> i32 {
        ((mg_score as f32) * phase + (eg_score as f32) * (1.0 - phase)) as i32
    }
}
```

## Evaluation Tuning

### Automated Tuning
```rust
pub struct TuningParameters {
    piece_values: [i32; 6],
    piece_square_tables: PieceSquareTables,
    mobility_weights: [i32; 6],
    king_safety_weights: [i32; 4],
    pawn_structure_weights: [i32; 5],
}

impl TuningParameters {
    pub fn tune(&mut self, positions: &[Position], target_scores: &[i32]) {
        let mut optimizer = Optimizer::new();
        optimizer.optimize(self, positions, target_scores);
    }
}
```

### Performance Optimizations
1. Incremental updates
2. Cached evaluations
3. SIMD operations
4. Evaluation pruning

## Future Improvements

### 1. Evaluation Features
- [ ] Pattern recognition
- [ ] Piece coordination scoring
- [ ] Advanced king safety
- [ ] Trapped pieces detection

### 2. Technical Improvements
- [ ] NNUE integration
- [ ] Automated tuning
- [ ] Evaluation pruning
- [ ] SIMD optimizations

### 3. Analysis Tools
- [ ] Position analysis
- [ ] Feature contribution analysis
- [ ] Evaluation accuracy metrics
- [ ] Performance profiling 