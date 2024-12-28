use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    tasks::{AsyncComputeTaskPool, Task},
};
use chess_core::{Board, Position, Move, piece::{Color as ChessColor, PieceType as ChessPieceType}};
use chess_engine::ChessAI;
use std::time::Duration;

pub struct ChessUiPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum Turn {
    #[default]
    Player,
    AI,
}

#[derive(Resource)]
struct GameState {
    board: Board,
    ai: ChessAI,
    ai_thinking_timer: Timer,
    ai_task: Option<Task<Option<Move>>>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Board::new(),
            ai: ChessAI::new(5),
            ai_thinking_timer: Timer::from_seconds(0.5, TimerMode::Once),
            ai_task: None,
        }
    }
}

impl Plugin for ChessUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chess Engine".into(),
                resolution: WindowResolution::new(800.0, 800.0),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_state::<Turn>()
        .init_resource::<GameState>()
        .add_systems(Startup, (setup, spawn_ui))
        .add_systems(Update, (
            handle_resize,
            handle_input,
            update_selected_pieces,
            update_ai,
            update_ui_text
        ).chain());
    }
}

#[derive(Component)]
struct ChessBoard;

#[derive(Component)]
struct Square {
    position: Position,
}

#[derive(Component)]
struct Piece {
    piece_type: ChessPieceType,
    is_white: bool,
    position: Position,
}

#[derive(Component)]
struct SelectedPiece;

#[derive(Component)]
struct AiThinkingText;

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Board
    let board_size = 8.0;
    let square_size = 80.0;
    let board_offset = Vec3::new(-board_size * square_size / 2.0, -board_size * square_size / 2.0, 0.0);

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.1, 0.1, 0.1),
                    custom_size: Some(Vec2::new(board_size * square_size + 20.0, board_size * square_size + 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            ChessBoard,
        ));

    // Squares
    for rank in 0..8 {
        for file in 0..8 {
            let is_white = (rank + file) % 2 == 0;
            let position = Vec3::new(
                board_offset.x + file as f32 * square_size + square_size / 2.0,
                board_offset.y + rank as f32 * square_size + square_size / 2.0,
                1.0,
            );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if is_white {
                            Color::rgb(0.9, 0.9, 0.9)
                        } else {
                            Color::rgb(0.3, 0.3, 0.3)
                        },
                        custom_size: Some(Vec2::new(square_size, square_size)),
                        ..default()
                    },
                    transform: Transform::from_translation(position),
                    ..default()
                },
                Square {
                    position: Position {
                        file: (file + 1) as u8,
                        rank: (8 - rank) as u8,
                    },
                },
            ));
        }
    }

    // Initial pieces
    spawn_initial_pieces(&mut commands, board_offset, square_size);
}

fn spawn_initial_pieces(commands: &mut Commands, board_offset: Vec3, square_size: f32) {
    let board = Board::new();
    for rank in 1..=8 {
        for file in 1..=8 {
            let pos = Position { file, rank };
            if let Some(piece) = board.get_piece(pos) {
                let is_white = piece.color == ChessColor::White;
                let position = Vec3::new(
                    board_offset.x + (file as f32 - 1.0) * square_size + square_size / 2.0,
                    board_offset.y + (8.0 - rank as f32) * square_size + square_size / 2.0,
                    2.0,
                );

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: if is_white {
                                Color::rgb(1.0, 1.0, 1.0)
                            } else {
                                Color::rgb(0.0, 0.0, 0.0)
                            },
                            custom_size: Some(Vec2::new(square_size * 0.8, square_size * 0.8)),
                            ..default()
                        },
                        transform: Transform::from_translation(position),
                        ..default()
                    },
                    Piece {
                        piece_type: piece.piece_type,
                        is_white,
                        position: pos,
                    },
                ));
            }
        }
    }
}

fn handle_resize(
    windows: Query<&Window>,
    mut board_query: Query<(&mut Transform, &mut Sprite), With<ChessBoard>>,
    mut square_query: Query<(&mut Transform, &mut Sprite, &Square), (With<Square>, Without<ChessBoard>)>,
    mut piece_query: Query<(&mut Transform, &mut Sprite, &Piece), (With<Piece>, Without<ChessBoard>, Without<Square>)>,
) {
    let window = windows.single();
    let min_dimension = window.width().min(window.height());
    let square_size = min_dimension / 10.0;
    let board_size = 8.0 * square_size;
    
    // Update board
    if let Ok((mut transform, mut sprite)) = board_query.get_single_mut() {
        sprite.custom_size = Some(Vec2::new(board_size + 20.0, board_size + 20.0));
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }

    let board_offset = Vec3::new(-board_size / 2.0, -board_size / 2.0, 0.0);

    // Update squares
    for (mut transform, mut sprite, square) in square_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(square_size, square_size));
        transform.translation = Vec3::new(
            board_offset.x + (square.position.file as f32 - 1.0) * square_size + square_size / 2.0,
            board_offset.y + (8.0 - square.position.rank as f32) * square_size + square_size / 2.0,
            1.0,
        );
    }

    // Update pieces
    for (mut transform, mut sprite, piece) in piece_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(square_size * 0.8, square_size * 0.8));
        transform.translation = Vec3::new(
            board_offset.x + (piece.position.file as f32 - 1.0) * square_size + square_size / 2.0,
            board_offset.y + (8.0 - piece.position.rank as f32) * square_size + square_size / 2.0,
            2.0,
        );
    }
}

fn handle_input(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut turn_state: ResMut<NextState<Turn>>,
    mut pieces: Query<(Entity, &mut Piece, Option<&SelectedPiece>)>,
    squares: Query<(&Transform, &Square)>,
    selected: Query<Entity, With<SelectedPiece>>,
    turn: Res<State<Turn>>,
) {
    if *turn.get() != Turn::Player || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(cursor_pos) = window.cursor_position() {
        // Convert cursor position to world coordinates
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let world_pos = ray.origin.truncate();

            // Find clicked square
            for (transform, square) in squares.iter() {
                let square_pos = transform.translation.truncate();
                let square_size = transform.scale.truncate() * 80.0; // Base square size
                let min = square_pos - square_size / 2.0;
                let max = square_pos + square_size / 2.0;

                if world_pos.x >= min.x && world_pos.x <= max.x &&
                   world_pos.y >= min.y && world_pos.y <= max.y {
                    
                    // Check if we have a selected piece
                    let mut selected_piece = None;
                    for (entity, piece, _) in pieces.iter().filter(|(_, _, selected)| selected.is_some()) {
                        selected_piece = Some((entity, piece.position));
                        break;
                    }

                    // Check if clicked on a piece
                    let clicked_piece = game_state.board.get_piece(square.position);

                    match (selected_piece, clicked_piece) {
                        // Case 1: We have a selected piece and clicked on a valid destination or enemy piece
                        (Some((selected_entity, from_pos)), maybe_piece) => {
                            if maybe_piece.map_or(true, |p| p.color != ChessColor::White) {
                                // Try to make the move
                                let chess_move = Move::new(from_pos, square.position);
                                if game_state.board.make_move(chess_move).is_ok() {
                                    // Update piece position
                                    if let Ok((_, mut piece, _)) = pieces.get_mut(selected_entity) {
                                        piece.position = square.position;
                                    }
                                    commands.entity(selected_entity).remove::<SelectedPiece>();
                                    turn_state.set(Turn::AI);
                                } else {
                                    // Invalid move, deselect the piece
                                    commands.entity(selected_entity).remove::<SelectedPiece>();
                                }
                            } else {
                                // Clicked on a friendly piece, switch selection
                                commands.entity(selected_entity).remove::<SelectedPiece>();
                                // Select the new piece
                                for (entity, piece, _) in pieces.iter() {
                                    if piece.position == square.position {
                                        commands.entity(entity).insert(SelectedPiece);
                                        break;
                                    }
                                }
                            }
                        },
                        // Case 2: No piece selected and clicked on a friendly piece
                        (None, Some(piece)) if piece.color == ChessColor::White => {
                            // Remove any existing selection (shouldn't be any, but just in case)
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                            // Select the new piece
                            for (entity, piece, _) in pieces.iter() {
                                if piece.position == square.position {
                                    commands.entity(entity).insert(SelectedPiece);
                                    break;
                                }
                            }
                        },
                        // Case 3: No piece selected and clicked on empty square or enemy piece
                        _ => {
                            // Deselect any selected pieces
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn update_selected_pieces(
    mut pieces: Query<(&mut Sprite, Option<&SelectedPiece>), With<Piece>>,
) {
    for (mut sprite, selected) in pieces.iter_mut() {
        if selected.is_some() {
            sprite.color = sprite.color.with_a(0.7);
        } else {
            sprite.color = sprite.color.with_a(1.0);
        }
    }
}

fn update_ai(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut turn_state: ResMut<NextState<Turn>>,
    mut pieces: Query<(&mut Piece, &mut Transform)>,
    windows: Query<&Window>,
    turn: Res<State<Turn>>,
) {
    if *turn.get() != Turn::AI {
        return;
    }

    if game_state.board.is_checkmate() || game_state.board.is_stalemate() {
        return;
    }

    game_state.ai_thinking_timer.tick(time.delta());

    // Start AI computation if not already started
    if game_state.ai_task.is_none() && game_state.ai_thinking_timer.finished() {
        let board = game_state.board.clone();
        let ai = game_state.ai.clone();
        let thread_pool = AsyncComputeTaskPool::get();
        
        game_state.ai_task = Some(thread_pool.spawn(async move {
            ai.get_best_move(&board)
        }));
    }

    // Check if AI has finished computing
    if let Some(mut task) = game_state.ai_task.take() {
        if let Some(ai_move) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task)) {
            // AI computation finished
            if let Some(chess_move) = ai_move {
                if game_state.board.make_move(chess_move).is_ok() {
                    // Update piece position in UI
                    for (mut piece, mut transform) in pieces.iter_mut() {
                        if piece.position == chess_move.from {
                            piece.position = chess_move.to;
                            
                            // Update piece transform
                            let window = windows.single();
                            let min_dimension = window.width().min(window.height());
                            let square_size = min_dimension / 10.0;
                            let board_size = 8.0 * square_size;
                            let board_offset = Vec3::new(-board_size / 2.0, -board_size / 2.0, 0.0);

                            transform.translation = Vec3::new(
                                board_offset.x + (piece.position.file as f32 - 1.0) * square_size + square_size / 2.0,
                                board_offset.y + (8.0 - piece.position.rank as f32) * square_size + square_size / 2.0,
                                2.0,
                            );
                            break;
                        }
                    }
                    turn_state.set(Turn::Player);
                }
            }
            game_state.ai_thinking_timer.reset();
        } else {
            // AI computation still running
            game_state.ai_task = Some(task);
        }
    }
}

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "AI is thinking...",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        AiThinkingText,
    ));
}

fn update_ui_text(
    turn: Res<State<Turn>>,
    mut text_query: Query<&mut Visibility, With<AiThinkingText>>,
) {
    if let Ok(mut visibility) = text_query.get_single_mut() {
        *visibility = if *turn.get() == Turn::AI {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}