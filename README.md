# Rust Chess Engine

A high-performance chess engine written in Rust, featuring advanced search algorithms, sophisticated evaluation, and a modern Bevy-based UI.

## Latest Release
🎉 **Version 1.0.0 is now available!** 
- [Download v1.0.0 for Windows](releases/v1.0.0/chess-engine-v1.0.0-windows.zip)
- [Release Notes](releases/v1.0.0/README.md)

⚠️ Note: Linux and MacOS are not officially supported in this release.

## Features

### Core Engine
- Principal Variation Search (PVS) with iterative deepening
- Advanced pruning techniques (null move, futility, late move reduction)
- Sophisticated evaluation function with multiple components
- Efficient move generation using bitboards
- Transposition table for position caching
- Multi-threaded search capabilities

### User Interface
- Modern, responsive Bevy-based UI
- Legal move highlighting
- Move history tracking

### Game Features
- All standard chess rules implemented
- Support for special moves (castling, en passant, promotions)
- Move validation
- Game state persistence

## Getting Started

### Prerequisites
- Rust 1.70 or higher
- Cargo package manager
- CMake (for building dependencies)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/ThenerzZ/chess-engine.git
cd chess-engine
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

## Project Structure

```
chess-engine/
├── crates/
│   ├── chess_core/      # Core chess logic and rules
│   ├── chess_engine/    # AI and search implementation
│   └── chess_ui/        # Bevy-based user interface
├── docs/                # Documentation
│   ├── algorithms/      # Search and evaluation algorithms
│   ├── engine/          # Engine architecture
│   └── assets/          # Asset documentation
├── releases/            # Release packages and notes
├── assets/             # Game assets
└── tests/              # Integration tests
```

## Documentation

Detailed documentation is available in the `/docs` directory:

- [Technical Design Document](docs/TECHNICAL_DESIGN.md)
- [Search Algorithms](docs/algorithms/README.md)
- [Evaluation System](docs/evaluation/README.md)
- [Engine Architecture](docs/engine/README.md)

## Performance

The engine achieves strong playing strength through:

- Efficient move generation using bitboards
- Advanced search techniques (PVS, null move pruning)
- Sophisticated evaluation function
- Multi-threaded search
- Transposition table caching

Typical performance metrics:
- 1-2M nodes/second on modern hardware
- Search depths of 8-12 ply in middlegame
- Sub-second move generation
- Memory usage under 1GB

## Development

### Building from Source

1. Clone the repository
2. Install dependencies:
```bash
rustup update
rustup component add clippy
rustup component add rustfmt
```

3. Build the project:
```bash
cargo build
```

### Running Tests

Run the test suite:
```bash
cargo test
```

Run performance benchmarks:
```bash
cargo bench
```

### Code Style

The project follows Rust standard practices:
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes
- Maintain test coverage
- Document public APIs

## Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and formatting
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Stockfish](https://stockfishchess.org/) - Inspiration for search techniques
- [Bevy](https://bevyengine.org/) - Game engine framework
- [Chess Programming Wiki](https://www.chessprogramming.org/) - Invaluable resource

## Contact

- Project Link: [https://github.com/ThenerzZ/chess-engine](https://github.com/ThenerzZ/chess-engine)
- Documentation: [https://github.com/ThenerzZ/chess-engine](https://github.com/ThenerzZ/chess-engine)

## Roadmap

### Version 1.1
- [ ] UCI protocol support
- [ ] Opening book integration
- [ ] Endgame tablebases
- [ ] Improved time management

### Version 1.2
- [ ] Neural network evaluation
- [ ] NNUE support
- [ ] Distributed search
- [ ] Cloud analysis support

### Version 1.3
- [ ] Chess960 support
- [ ] Analysis features
- [ ] Training mode
- [ ] Position setup
