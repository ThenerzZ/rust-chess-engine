# Chess Engine

A modern chess game built with Rust and Bevy, featuring a responsive UI and AI opponent.

## Features

- Clean and modern UI with resizable window
- Responsive chess board that adapts to window size
- Intuitive piece selection and movement
- AI opponent with visual feedback during thinking
- Turn-based gameplay with state management
- Visual indicators for selected pieces
- Support for all standard chess moves
- Checkmate and stalemate detection

## Project Structure

The project is organized into three main crates:

- `chess_core`: Core chess logic and game rules
- `chess_engine`: AI implementation and move evaluation
- `chess_ui`: Bevy-based user interface and game flow

## Controls

- Left-click to select a piece
- Left-click on a valid square to move the selected piece
- Selected pieces are shown with reduced opacity
- "AI is thinking..." indicator appears during AI turns

## Technical Details

- Built with Rust and Bevy game engine
- Uses Bevy's state management for turn handling
- Non-blocking AI computation to maintain UI responsiveness
- Efficient piece movement validation
- Component-based architecture for clean separation of concerns

## Building and Running

1. Make sure you have Rust installed
2. Clone the repository
3. Run with cargo:
   ```bash
   cargo run --release
   ```

## Dependencies

- Bevy 0.12.0: Game engine and UI framework
- Other dependencies are managed through workspace Cargo.toml

## Future Improvements

- [ ] Move history and notation
- [ ] Game save/load functionality
- [ ] Difficulty levels for AI
- [ ] Opening book integration
- [ ] Multiplayer support
- [ ] Sound effects and animations
