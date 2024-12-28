use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    tasks::{AsyncComputeTaskPool, Task},
    sprite::Anchor,
};
use chess_core::{
    Board, Position, Move,
    piece::{Color as ChessColor, PieceType as ChessPieceType},
};
use chess_engine::ChessAI;
use std::{collections::HashMap, time::Duration};

const SQUARE_SIZE: f32 = 80.0;

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

#[derive(Resource, Clone)]
pub struct ChessAssets {
    white_king: Handle<Image>,
    white_queen: Handle<Image>,
    white_rook: Handle<Image>,
    white_bishop: Handle<Image>,
    white_knight: Handle<Image>,
    white_pawn: Handle<Image>,
    black_king: Handle<Image>,
    black_queen: Handle<Image>,
    black_rook: Handle<Image>,
    black_bishop: Handle<Image>,
    black_knight: Handle<Image>,
    black_pawn: Handle<Image>,
    valid_move: Handle<Image>,
}

#[derive(Component)]
struct ChessBoard;

#[derive(Component, Copy, Clone)]
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

#[derive(Component)]
struct ValidMoveIndicator;

#[derive(Component)]
struct MovingPiece {
    target_position: Vec3,
    speed: f32,
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
        .add_systems(PreStartup, setup)
        .add_systems(Update, (
            handle_resize,
            handle_input,
            update_selected_pieces,
            update_ai,
            update_ui_text,
            show_valid_moves,
            update_piece_movement,
        ));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load assets
    let chess_assets = ChessAssets {
        white_king: asset_server.load("white_king.png"),
        white_queen: asset_server.load("white_queen.png"),
        white_rook: asset_server.load("white_rook.png"),
        white_bishop: asset_server.load("white_bishop.png"),
        white_knight: asset_server.load("white_knight.png"),
        white_pawn: asset_server.load("white_pawn.png"),
        black_king: asset_server.load("black_king.png"),
        black_queen: asset_server.load("black_queen.png"),
        black_rook: asset_server.load("black_rook.png"),
        black_bishop: asset_server.load("black_bishop.png"),
        black_knight: asset_server.load("black_knight.png"),
        black_pawn: asset_server.load("black_pawn.png"),
        valid_move: asset_server.load("valid_move.png"),
    };

    commands.insert_resource(chess_assets.clone());

    // Camera
    commands.spawn(Camera2dBundle::default());

    // Board
    let board_size = 8.0;
    let board_offset = Vec3::new(-board_size * SQUARE_SIZE / 2.0, -board_size * SQUARE_SIZE / 2.0, 0.0);

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.1, 0.1, 0.1),
                    custom_size: Some(Vec2::new(board_size * SQUARE_SIZE + 20.0, board_size * SQUARE_SIZE + 20.0)),
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
                board_offset.x + file as f32 * SQUARE_SIZE + SQUARE_SIZE / 2.0,
                board_offset.y + rank as f32 * SQUARE_SIZE + SQUARE_SIZE / 2.0,
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
                        custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
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
    spawn_initial_pieces(&mut commands, board_offset, &chess_assets);
    
    // UI
    spawn_ui(commands);
}

fn spawn_initial_pieces(
    commands: &mut Commands,
    board_offset: Vec3,
    assets: &ChessAssets,
) {
    // Spawn white pieces
    spawn_piece(commands, ChessPieceType::Rook, true, 1, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Knight, true, 2, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Bishop, true, 3, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Queen, true, 4, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::King, true, 5, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Bishop, true, 6, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Knight, true, 7, 1, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Rook, true, 8, 1, board_offset, assets);
    for file in 1..=8 {
        spawn_piece(commands, ChessPieceType::Pawn, true, file, 2, board_offset, assets);
    }

    // Spawn black pieces
    spawn_piece(commands, ChessPieceType::Rook, false, 1, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Knight, false, 2, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Bishop, false, 3, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Queen, false, 4, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::King, false, 5, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Bishop, false, 6, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Knight, false, 7, 8, board_offset, assets);
    spawn_piece(commands, ChessPieceType::Rook, false, 8, 8, board_offset, assets);
    for file in 1..=8 {
        spawn_piece(commands, ChessPieceType::Pawn, false, file, 7, board_offset, assets);
    }
}

fn spawn_piece(
    commands: &mut Commands,
    piece_type: ChessPieceType,
    is_white: bool,
    file: u8,
    rank: u8,
    board_offset: Vec3,
    assets: &ChessAssets,
) -> Entity {
    let position = Vec3::new(
        board_offset.x + (file as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
        board_offset.y + (rank as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
        2.0,
    );

    let sprite = match (piece_type, is_white) {
        (ChessPieceType::King, true) => assets.white_king.clone(),
        (ChessPieceType::Queen, true) => assets.white_queen.clone(),
        (ChessPieceType::Rook, true) => assets.white_rook.clone(),
        (ChessPieceType::Bishop, true) => assets.white_bishop.clone(),
        (ChessPieceType::Knight, true) => assets.white_knight.clone(),
        (ChessPieceType::Pawn, true) => assets.white_pawn.clone(),
        (ChessPieceType::King, false) => assets.black_king.clone(),
        (ChessPieceType::Queen, false) => assets.black_queen.clone(),
        (ChessPieceType::Rook, false) => assets.black_rook.clone(),
        (ChessPieceType::Bishop, false) => assets.black_bishop.clone(),
        (ChessPieceType::Knight, false) => assets.black_knight.clone(),
        (ChessPieceType::Pawn, false) => assets.black_pawn.clone(),
    };

    commands.spawn((
        SpriteBundle {
            texture: sprite,
            transform: Transform::from_translation(position),
            sprite: Sprite {
                custom_size: Some(Vec2::new(SQUARE_SIZE * 0.9, SQUARE_SIZE * 0.9)),
                anchor: Anchor::Center,
                ..default()
            },
            ..default()
        },
        Piece {
            piece_type,
            is_white,
            position: Position { file, rank },
        },
    )).id()
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
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut game_state: ResMut<GameState>,
    pieces: Query<(Entity, &Piece, &Transform)>,
    selected_pieces: Query<Entity, With<SelectedPiece>>,
    mut next_turn: ResMut<NextState<Turn>>,
    mouse_button: Res<Input<MouseButton>>,
    turn: Res<State<Turn>>,
) {
    if *turn.get() != Turn::Player {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if mouse_button.just_pressed(MouseButton::Left) {
            // Calculate clicked board position
            let clicked_pos = Position {
                file: ((world_position.x + 4.0 * SQUARE_SIZE) / SQUARE_SIZE).floor() as u8 + 1,
                rank: (8.0 - ((world_position.y + 4.0 * SQUARE_SIZE) / SQUARE_SIZE).floor()) as u8,
            };

            // Check if there's already a selected piece
            if let Ok(selected_entity) = selected_pieces.get_single() {
                // Get the selected piece's details
                if let Some((_, selected_piece, _)) = pieces.iter().find(|(entity, _, _)| *entity == selected_entity) {
                    // Check if the clicked position is a valid move
                    let valid_moves = game_state.board.get_valid_moves(selected_piece.position);
                    if let Some(valid_move) = valid_moves.iter().find(|m| m.to == clicked_pos) {
                        // Make the move
                        if game_state.board.make_move(*valid_move).is_ok() {
                            // Update piece positions
                            for (entity, piece, _) in pieces.iter() {
                                if piece.position == valid_move.from {
                                    commands.entity(entity).insert(MovingPiece {
                                        target_position: board_position_to_world(valid_move.to, 2.0),
                                        speed: 500.0,
                                    });
                                    commands.entity(entity).insert(Piece {
                                        piece_type: piece.piece_type,
                                        is_white: piece.is_white,
                                        position: valid_move.to,
                                    });
                                }
                            }

                            // Remove selection
                            commands.entity(selected_entity).remove::<SelectedPiece>();
                            
                            // Switch to AI's turn
                            next_turn.set(Turn::AI);
                            return;
                        }
                    }
                }
                
                // If clicked on a different friendly piece, select it instead
                if let Some((entity, piece, _)) = pieces.iter().find(|(_, p, _)| {
                    p.position == clicked_pos && p.is_white
                }) {
                    commands.entity(selected_entity).remove::<SelectedPiece>();
                    commands.entity(entity).insert(SelectedPiece);
                    return;
                }

                // If clicked elsewhere, deselect the piece
                commands.entity(selected_entity).remove::<SelectedPiece>();
            } else {
                // No piece selected - try to select a friendly piece
                if let Some((entity, piece, _)) = pieces.iter().find(|(_, p, _)| {
                    p.position == clicked_pos && p.is_white
                }) {
                    commands.entity(entity).insert(SelectedPiece);
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
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut turn_state: ResMut<NextState<Turn>>,
    mut pieces: Query<(Entity, &mut Piece, &mut Transform)>,
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

    if game_state.ai_task.is_none() && game_state.ai_thinking_timer.finished() {
        let board = game_state.board.clone();
        let ai = game_state.ai.clone();
        let thread_pool = AsyncComputeTaskPool::get();
        
        game_state.ai_task = Some(thread_pool.spawn(async move {
            ai.get_best_move(&board)
        }));
    }

    if let Some(mut task) = game_state.ai_task.take() {
        if let Some(ai_move) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task)) {
            if let Some(chess_move) = ai_move {
                if game_state.board.make_move(chess_move).is_ok() {
                    for (entity, mut piece, mut transform) in pieces.iter_mut() {
                        if piece.position == chess_move.from {
                            move_piece(&mut commands, entity, &mut piece, &mut transform, chess_move.from, chess_move.to, windows.single());
                            break;
                        }
                    }
                    turn_state.set(Turn::Player);
                }
            }
            game_state.ai_thinking_timer.reset();
        } else {
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

fn show_valid_moves(
    mut commands: Commands,
    game_state: Res<GameState>,
    selected_pieces: Query<(&Piece, &Transform), With<SelectedPiece>>,
    chess_assets: Res<ChessAssets>,
    indicators: Query<Entity, With<ValidMoveIndicator>>,
) {
    // Remove existing indicators
    for entity in indicators.iter() {
        commands.entity(entity).despawn();
    }

    // Show valid moves for selected piece
    if let Ok((piece, transform)) = selected_pieces.get_single() {
        if piece.is_white {  // Only show moves for white pieces during player's turn
            let valid_moves = game_state.board.get_valid_moves(piece.position);
            for valid_move in valid_moves {
                let target_pos = board_position_to_world(valid_move.to, transform.translation.z);
                commands.spawn((
                    SpriteBundle {
                        texture: chess_assets.valid_move.clone(),
                        transform: Transform::from_translation(target_pos)
                            .with_scale(Vec3::splat(1.0)),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                            anchor: Anchor::Center,
                            ..default()
                        },
                        ..default()
                    },
                    ValidMoveIndicator,
                ));
            }
        }
    }
}

fn board_position_to_world(pos: Position, z: f32) -> Vec3 {
    Vec3::new(
        ((pos.file as f32 - 1.0) - 3.5) * SQUARE_SIZE,
        ((8.0 - pos.rank as f32) - 3.5) * SQUARE_SIZE,
        z,
    )
}

fn update_piece_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &MovingPiece)>,
) {
    for (entity, mut transform, moving) in query.iter_mut() {
        let direction = (moving.target_position - transform.translation).normalize();
        let distance = (moving.target_position - transform.translation).length();
        
        if distance < 1.0 {
            transform.translation = moving.target_position;
            commands.entity(entity).remove::<MovingPiece>();
        } else {
            transform.translation += direction * moving.speed * time.delta_seconds();
        }
    }
}

fn move_piece(
    commands: &mut Commands,
    piece_entity: Entity,
    piece: &mut Piece,
    transform: &mut Transform,
    from: Position,
    to: Position,
    window: &Window,
) {
    piece.position = to;
    
    let min_dimension = window.width().min(window.height());
    let square_size = min_dimension / 10.0;
    let board_size = 8.0 * square_size;
    let board_offset = Vec3::new(-board_size / 2.0, -board_size / 2.0, 0.0);

    let target_pos = Vec3::new(
        board_offset.x + (to.file as f32 - 1.0) * square_size + square_size / 2.0,
        board_offset.y + (8.0 - to.rank as f32) * square_size + square_size / 2.0,
        2.0,
    );

    commands.entity(piece_entity).insert(MovingPiece {
        target_position: target_pos,
        speed: 500.0, // Adjust this value to change movement speed
    });
}