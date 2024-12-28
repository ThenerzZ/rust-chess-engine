use bevy::prelude::*;
use chess_ui::ChessUiPlugin;

fn main() {
    App::new()
        .add_plugins(ChessUiPlugin)
        .run();
} 