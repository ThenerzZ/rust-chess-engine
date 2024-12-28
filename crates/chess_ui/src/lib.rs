use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    tasks::{AsyncComputeTaskPool, Task},
};
use chess_core::{
    Board, Position, Move,
    piece::{Color as ChessColor, PieceType as ChessPieceType},
};
use chess_engine::ChessAI;
use std::{collections::HashMap, time::Duration};

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

#[derive(Resource)]
struct ChessAssets {
    piece_sprites: HashMap<(ChessPieceType, bool), Handle<Image>>,
    valid_move_indicator: Handle<Image>,
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
    // Load assets first
    let mut piece_sprites = HashMap::new();
    
    // Load white pieces
    piece_sprites.insert((ChessPieceType::King, true), 
        asset_server.load("white_king.png"));
    piece_sprites.insert((ChessPieceType::Queen, true),
        asset_server.load("white_queen.png"));
    piece_sprites.insert((ChessPieceType::Rook, true),
        asset_server.load("white_rook.png"));
    piece_sprites.insert((ChessPieceType::Bishop, true),
        asset_server.load("white_bishop.png"));
    piece_sprites.insert((ChessPieceType::Knight, true),
        asset_server.load("white_knight.png"));
    piece_sprites.insert((ChessPieceType::Pawn, true),
        asset_server.load("white_pawn.png"));
    
    // Load black pieces
    piece_sprites.insert((ChessPieceType::King, false),
        asset_server.load("black_king.png"));
    piece_sprites.insert((ChessPieceType::Queen, false),
        asset_server.load("black_queen.png"));
    piece_sprites.insert((ChessPieceType::Rook, false),
        asset_server.load("black_rook.png"));
    piece_sprites.insert((ChessPieceType::Bishop, false),
        asset_server.load("black_bishop.png"));
    piece_sprites.insert((ChessPieceType::Knight, false),
        asset_server.load("black_knight.png"));
    piece_sprites.insert((ChessPieceType::Pawn, false),
        asset_server.load("black_pawn.png"));

    // Load valid move indicator
    let valid_move_indicator = asset_server.load("valid_move.png");

    let piece_sprites_clone = piece_sprites.clone();
    let valid_move_indicator_clone = valid_move_indicator.clone();

    commands.insert_resource(ChessAssets {
        piece_sprites,
        valid_move_indicator,
    });

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
    spawn_initial_pieces(&mut commands, board_offset, square_size, ChessAssets {
        piece_sprites: piece_sprites_clone,
        valid_move_indicator: valid_move_indicator_clone,
    });
    
    // UI
    spawn_ui(commands);
}

fn spawn_initial_pieces(
    commands: &mut Commands,
    board_offset: Vec3,
    square_size: f32,
    assets: ChessAssets,
) {
    let board = Board::new();
    
    for rank in 0..8 {
        for file in 0..8 {
            if let Some(piece) = board.get_piece(Position {
                file: (file + 1) as u8,
                rank: (8 - rank) as u8,
            }) {
                let is_white = piece.color == ChessColor::White;
                let position = Vec3::new(
                    board_offset.x + file as f32 * square_size + square_size / 2.0,
                    board_offset.y + rank as f32 * square_size + square_size / 2.0,
                    2.0,
                );

                commands.spawn((
                    SpriteBundle {
                        texture: assets.piece_sprites.get(&(piece.piece_type, is_white))
                            .expect("Sprite should exist for piece").clone(),
                        transform: Transform::from_translation(position),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(square_size * 0.8, square_size * 0.8)),
                            ..default()
                        },
                        ..default()
                    },
                    Piece {
                        piece_type: piece.piece_type,
                        is_white,
                        position: Position {
                            file: (file + 1) as u8,
                            rank: (8 - rank) as u8,
                        },
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
    mut param_set: ParamSet<(
        Query<(Entity, &mut Piece, &mut Transform, Option<&SelectedPiece>)>,
        Query<(&Transform, &Square)>,
        Query<Entity, With<SelectedPiece>>,
    )>,
    turn: Res<State<Turn>>,
) {
    if *turn.get() != Turn::Player || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_q.get_single().expect("Camera not found");

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let world_pos = ray.origin.truncate();

            // Find clicked square
            let mut clicked_square = None;
            {
                let squares = param_set.p1();
                for (transform, square) in squares.iter() {
                    let square_pos = transform.translation.truncate();
                    let square_size = transform.scale.truncate() * 80.0;
                    let min = square_pos - square_size / 2.0;
                    let max = square_pos + square_size / 2.0;

                    if world_pos.x >= min.x && world_pos.x <= max.x &&
                       world_pos.y >= min.y && world_pos.y <= max.y {
                        clicked_square = Some(*square);
                        break;
                    }
                }
            }

            if let Some(square) = clicked_square {
                // Find selected piece
                let mut selected_piece = None;
                {
                    let pieces = param_set.p0();
                    for (entity, piece, _, _) in pieces.iter().filter(|(_, _, _, selected)| selected.is_some()) {
                        selected_piece = Some((entity, piece.position));
                        break;
                    }
                }

                let clicked_piece = game_state.board.get_piece(square.position);

                match (selected_piece, clicked_piece) {
                    (Some((selected_entity, from_pos)), maybe_piece) => {
                        if maybe_piece.map_or(true, |p| p.color != ChessColor::White) {
                            let chess_move = Move::new(from_pos, square.position);
                            if game_state.board.make_move(chess_move).is_ok() {
                                let mut pieces = param_set.p0();
                                if let Ok((_, mut piece, mut transform, _)) = pieces.get_mut(selected_entity) {
                                    move_piece(&mut commands, selected_entity, &mut piece, &mut transform, from_pos, square.position, window);
                                }
                                commands.entity(selected_entity).remove::<SelectedPiece>();
                                turn_state.set(Turn::AI);
                            } else {
                                commands.entity(selected_entity).remove::<SelectedPiece>();
                            }
                        } else {
                            commands.entity(selected_entity).remove::<SelectedPiece>();
                            let mut pieces = param_set.p0();
                            for (entity, piece, _, _) in pieces.iter() {
                                if piece.position == square.position {
                                    commands.entity(entity).insert(SelectedPiece);
                                    break;
                                }
                            }
                        }
                    },
                    (None, Some(piece)) if piece.color == ChessColor::White => {
                        {
                            let selected = param_set.p2();
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                        }
                        let pieces = param_set.p0();
                        for (entity, piece, _, _) in pieces.iter() {
                            if piece.position == square.position {
                                commands.entity(entity).insert(SelectedPiece);
                                break;
                            }
                        }
                    },
                    _ => {
                        let selected = param_set.p2();
                        for entity in selected.iter() {
                            commands.entity(entity).remove::<SelectedPiece>();
                        }
                    }
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
    chess_assets: Res<ChessAssets>,
    selected_pieces: Query<&Piece, With<SelectedPiece>>,
    indicators: Query<Entity, With<ValidMoveIndicator>>,
) {
    // Remove old indicators
    for entity in indicators.iter() {
        commands.entity(entity).despawn();
    }

    // Show new indicators for selected piece
    if let Some(piece) = selected_pieces.iter().next() {
        let valid_moves = game_state.board.get_valid_moves(piece.position);
        
        for valid_move in valid_moves {
            let window_size = 800.0;
            let square_size = window_size / 10.0;
            let board_size = 8.0 * square_size;
            let board_offset = Vec3::new(-board_size / 2.0, -board_size / 2.0, 0.0);

            let position = Vec3::new(
                board_offset.x + (valid_move.to.file as f32 - 1.0) * square_size + square_size / 2.0,
                board_offset.y + (8.0 - valid_move.to.rank as f32) * square_size + square_size / 2.0,
                1.5,
            );

            commands.spawn((
                SpriteBundle {
                    texture: chess_assets.valid_move_indicator.clone(),
                    transform: Transform::from_translation(position)
                        .with_scale(Vec3::splat(0.3)),
                    sprite: Sprite {
                        color: Color::rgba(0.0, 1.0, 0.0, 0.3),
                        ..default()
                    },
                    ..default()
                },
                ValidMoveIndicator,
            ));
        }
    }
}

fn update_piece_movement(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &MovingPiece)>,
    mut commands: Commands,
) {
    for (entity, mut transform, moving) in query.iter_mut() {
        let direction = moving.target_position - transform.translation;
        if direction.length() < 1.0 {
            transform.translation = moving.target_position;
            commands.entity(entity).remove::<MovingPiece>();
        } else {
            transform.translation += direction.normalize() * moving.speed * time.delta_seconds();
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