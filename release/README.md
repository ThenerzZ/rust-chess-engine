# Chess Engine v1.0.0 - Release Notes

A high-performance chess engine written in Rust, featuring a sophisticated AI opponent and a modern user interface.

## Installation
1. Extract all files to a directory of your choice
2. Run `start_game.bat` to launch the game
3. Make sure all files and the `assets` folder remain in the same directory

## System Requirements
- Windows 10 or later
- 4GB RAM recommended
- Graphics card with support for modern graphics APIs

## Features

### Game Features
- Complete chess rules implementation including:
  - All standard piece movements
  - Special moves (castling, en passant, pawn promotion)
  - Draw detection (stalemate, insufficient material, threefold repetition)
- Time controls for both players
- Move validation and legal move highlighting
- Game state tracking and display

### User Interface
- Clean, modern interface built with Bevy
- Intuitive drag-and-drop piece movement
- Legal move highlighting
- Clear game state visualization
- Move history display
- Current position evaluation display

### AI Engine
The engine features a sophisticated AI opponent with:
- Strong positional understanding
- Material and positional evaluation
- Advanced search techniques
- Smart time management
- Multiple difficulty levels

## Controls
- Left-click and drag pieces to move them
- Release to place the piece
- Invalid moves will automatically return to their starting position
- Pawn promotion will show a selection dialog
- Game state and current evaluation are shown in the interface

## Known Issues
- The AI might take longer to move in very complex positions
- Some UI elements might need adjustment on different screen resolutions

## Troubleshooting
1. If the game doesn't start:
   - Ensure all files are extracted properly
   - Verify the `assets` folder is present
   - Try running as administrator

2. If graphics appear incorrect:
   - Update your graphics drivers
   - Ensure your system meets the minimum requirements

## Support
For bug reports or suggestions, please create an issue on the project's GitHub repository.

## Credits
- Chess piece designs: [Attribution for assets used]
- Built with Rust and Bevy Engine

## License
This project is licensed under the MIT License - see the LICENSE file for details.
