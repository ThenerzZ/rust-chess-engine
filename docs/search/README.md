# Search Algorithms Documentation

## Overview

The chess engine employs sophisticated search algorithms to find the best moves in any given position. The main search algorithm is Principal Variation Search (PVS), enhanced with various optimizations for improved performance.

## Principal Variation Search (PVS)

### Basic Concept
PVS is an enhancement of the alpha-beta algorithm that assumes the first move at each node is likely to be the best move. This assumption is strengthened by our move ordering system.

### Implementation Details
```rust
fn principal_variation_search(
    board: &Board,
    depth: u8,
    alpha: i32,
    beta: i32,
    tt: &mut HashMap<String, TTEntry>,
    history: &mut Vec<Vec<i32>>,
    pv_table: &mut Vec<Move>,
    is_pv_node: bool,
) -> i32
```

### Key Components
1. **Iterative Deepening**
   - Starts with shallow searches and progressively goes deeper
   - Uses results from previous iterations to improve move ordering
   - Allows for time management and early termination

2. **Aspiration Windows**
   - Initial window: `[alpha = -MATE_SCORE, beta = MATE_SCORE]`
   - Subsequent windows: `[best_score - 75, best_score + 75]`
   - Re-searches with wider windows if score falls outside

3. **Parallel Search**
   - Divides root moves among available CPU cores
   - Uses Rayon for parallel processing
   - Maintains thread-safe data structures

## Search Optimizations

### 1. Transposition Table
- Caches evaluated positions
- Stores:
  - Depth of search
  - Evaluation score
  - Best move found
  - Type of score (exact, upper bound, lower bound)
- Implementation:
  ```rust
  struct TTEntry {
      depth: u8,
      score: i32,
      entry_type: EntryType,
      best_move: Option<Move>,
  }
  ```

### 2. Move Ordering
Priority sequence:
1. Transposition table moves
2. Winning captures (MVV-LVA)
3. Equal captures
4. Killer moves
5. History moves
6. Quiet moves

### 3. Pruning Techniques

#### Null Move Pruning
- Gives opponent an extra move
- If position is still good, skips detailed search
- Conditions:
  ```rust
  if !is_pv_node && depth >= 3 && !in_check && static_eval >= beta {
      // Try null move pruning
  }
  ```

#### Futility Pruning
- Skips moves unlikely to improve position
- Applied at shallow depths
- Margins increase with depth:
  ```rust
  const FUTILITY_MARGIN: [i32; 4] = [0, 100, 200, 300];
  ```

#### Late Move Reduction (LMR)
- Reduces search depth for later moves
- Applied after searching `FULL_DEPTH_MOVES`
- Reduction increases logarithmically
- Implementation:
  ```rust
  let reduction = if depth >= REDUCTION_LIMIT && searched_moves > FULL_DEPTH_MOVES {
      ((searched_moves as f32).ln().floor() as u8).min(depth - 1)
  } else {
      0
  };
  ```

### 4. Quiescence Search
- Continues searching captures beyond regular depth
- Prevents horizon effect
- Uses delta pruning for efficiency
- Implementation:
  ```rust
  fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, depth: u8) -> i32
  ```

## Time Management

### Allocation Strategy
- Base time per move: `total_time / moves_to_go`
- Minimum time: 100ms
- Maximum time: 15 seconds
- Safety buffer: 50ms

### Dynamic Adjustment
- Extends search for:
  - Complex positions
  - Critical positions
  - When finding improvements
- Terminates search when:
  - Time is running low
  - Clear best move found
  - Checkmate found

## Performance Considerations

### Memory Management
- Transposition table size: 1 million entries
- History table: 64x64 matrix
- Principal variation: Maximum 64 moves
- Killer moves: 2 per ply

### Search Efficiency
- Average branching factor: ~35 moves
- Typical search depth: 12-15 ply
- Quiescence search: Up to 4 additional ply
- Node count: 1-10 million per second

## Future Improvements

1. **Search Enhancements**
   - [ ] Internal iterative deepening
   - [ ] Multi-PV search
   - [ ] Selective extensions
   - [ ] Better singular move detection

2. **Performance Optimizations**
   - [ ] NNUE evaluation
   - [ ] Bitboard move generation
   - [ ] Better parallel search scaling
   - [ ] Memory pool for node allocation

3. **Time Management**
   - [ ] Better sudden death handling
   - [ ] Position complexity assessment
   - [ ] Opening/endgame adjustments
   - [ ] Time recovery mechanism 