# Rust Chess Engine v1.0.0

Welcome to the first major release of our high-performance chess engine written in Rust! This release marks a significant milestone in our project, delivering a fully-functional chess engine with a modern UI and strong playing capabilities.

## ⚠️ Platform Support
- **Windows**: Fully supported (Windows 10/11)
- **Linux/MacOS**: No official support in this release
- **System Requirements**:
  - 2GB RAM minimum (4GB recommended)
  - OpenGL 3.3 capable graphics
  - Multi-core processor recommended for best performance

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
   - Make sure your system meets the minimum requirements

## 🎮 Game Features

### Core Engine Features
- Complete chess rules implementation with all special moves
- Strong AI opponent using advanced search techniques
- Multi-threaded performance optimization
- Sophisticated position evaluation

### User Interface
- Modern, responsive UI built with Bevy
- Intuitive drag-and-drop piece movement
- Legal move highlighting
- Complete move history
- Game state visualization
- Time control display

## 📊 Technical Details
- Written in pure Rust
- Average search speed: 1-2M nodes/second on modern hardware
- Typical search depth: 8-12 ply in middlegame
- Memory usage: Under 1GB
- Multi-threaded architecture

## ⚠️ Known Issues
- The game must be run from the provided launcher or executable
- Moving or deleting files from the installation folder will break the game
- No support for UCI chess protocols in this release
- Game settings can only be changed through the UI (no config files)

## 📝 License

This release is distributed under the MIT License. See LICENSE file for more details.

## 🤝 Support

For bug reports and feature requests, please use our GitHub issues page:
- GitHub Issues: [https://github.com/ThenerzZ/chess-engine/issues](https://github.com/ThenerzZ/chess-engine/issues)
- Documentation: [https://github.com/ThenerzZ/chess-engine/docs](https://github.com/ThenerzZ/chess-engine/docs)

---

Thank you for using our chess engine! We hope you enjoy this release and look forward to your feedback. 