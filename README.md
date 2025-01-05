# Chess Engine

A high-performance chess engine written in Rust, featuring advanced search algorithms and a modern Bevy-based UI.

## Features

### Core Engine
- Principal Variation Search (PVS) with alpha-beta pruning
- Advanced move ordering:
  - PV moves
  - MVV-LVA (Most Valuable Victim - Least Valuable Attacker) capture sorting
  - Killer moves
  - History heuristic
- Sophisticated evaluation function:
  - Material balance
  - Piece-square tables
  - Pawn structure evaluation (doubled, isolated, passed pawns)
  - Bishop pair bonus
  - Piece mobility
  - King safety evaluation
- Transposition table for position caching
- Smart time management based on position complexity and game phase
- Support for all standard chess rules including:
  - En passant captures
  - Castling with proper validation
  - Pawn promotion
  - Fifty-move rule
  - Threefold repetition
  - Insufficient material detection

### User Interface
- Modern, clean UI built with Bevy
- Drag-and-drop piece movement
- Legal move highlighting
- Game state visualization
- Move history display
- Current evaluation display

### Game Features
- Full implementation of FIDE chess rules
- Support for standard time controls
- PGN export/import capabilities
- Draw detection:
  - Stalemate
  - Insufficient material
  - Threefold repetition
  - Fifty-move rule

## Project Structure
The project is organized into several crates:
- `chess_core`: Core chess logic and rules
- `chess_engine`: Search and evaluation implementation
- `chess_ui`: Bevy-based user interface

## Building and Running

### Prerequisites
- Rust 1.70 or later
- Cargo package manager

### Build Instructions
```bash
# Clone the repository
git clone https://github.com/yourusername/chess-engine.git
cd chess-engine

# Build the project
cargo build --release

# Run the chess engine
cargo run --release
```

## Technical Details

### Search Algorithm
- Principal Variation Search with aspiration windows
- Iterative deepening
- Quiescence search for tactical stability
- Move ordering optimizations
- Transposition table for position caching

### Evaluation Function
- Material evaluation with piece-square tables
- Pawn structure analysis
- King safety assessment
- Mobility evaluation
- Bishop pair bonus
- Endgame-specific evaluations

### Performance Optimizations
- Efficient board representation
- Smart move generation
- Position caching
- Memory-efficient data structures

## License
[MIT License](LICENSE)

## Contributing
Contributions are welcome! Please feel free to submit pull requests.
