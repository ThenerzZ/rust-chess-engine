# Rust Chess Engine v1.0.0

Welcome to the first release of our chess engine written in Rust! This release provides a basic chess game with AI opponent.

## ⚠️ Platform Support
- **Windows**: Fully supported (Windows 10/11)
- **Linux/MacOS**: No official support in this release
- **System Requirements**:
  - OpenGL 3.3 capable graphics
  - 1GB RAM minimum

## 🚀 Quick Start Guide

1. **Installation**:
   - Extract the ZIP archive to any location on your computer
   - Do NOT move or delete any files from the extracted folder

2. **Starting the Game**:
   - Double-click `launch.bat` in the extracted folder
   - Alternatively, you can run `chess-engine.exe` directly

3. **If the Game Doesn't Start**:
   - Ensure your graphics drivers are up to date
   - Verify that all files from the ZIP archive were extracted
   - Check that the `assets` and `resources` folders are present

## 🎮 Game Features

### Core Engine Features
- Basic chess rules implementation
- Simple AI opponent with depth between 4-8 moves
- Basic evaluation including:
  - Standard piece values
  - Piece position tables
  - Pawn structure (doubled pawns, connected pawns)
  - Basic king safety
  - Piece mobility bonuses

### User Interface
- Basic UI built with Bevy
- Drag-and-drop piece movement
- Legal move highlighting
- Turn indicator

## 📊 Technical Details
- Written in Rust
- Uses Bevy game engine for graphics
- Single-player mode (human vs AI)
- Fixed 5-second maximum think time for AI

## ⚠️ Limitations
- Windows-only support
- No game saving/loading
- No time controls
- No move history
- No opening books
- No configuration options
- Must be run from installation folder

## 📝 License

This release is distributed under the MIT License. See LICENSE file for more details.

## 🤝 Support

For bug reports and feature requests, please use our GitHub issues page:
- GitHub Issues: [https://github.com/ThenerzZ/chess-engine/issues](https://github.com/ThenerzZ/chess-engine/issues)

---

Thank you for trying our chess engine!