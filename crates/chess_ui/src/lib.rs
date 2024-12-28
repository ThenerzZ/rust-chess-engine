use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    tasks::{AsyncComputeTaskPool, Task},
    sprite::Anchor,
};
use chess_core::{
    Board, Position, Move,
    piece::PieceType as ChessPieceType,
};
use chess_engine::ChessAI;

const SQUARE_SIZE: f32 = 80.0;

pub struct ChessUiPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum Turn {
    #[default]
    Player,
    AI,
}

#[derive(Resource)]
pub struct GameState {
    pub board: Board,
    pub selected_square: Option<Position>,
    pub valid_moves: Vec<Move>,
    pub ai: ChessAI,
    pub ai_thinking: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Board::new(),
            ai: ChessAI::new(4),
            ai_thinking: false,
            selected_square: None,
            valid_moves: Vec::new(),
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

// Components
#[derive(Component)]
struct ChessBoard;

#[derive(Component, Copy, Clone)]
struct Square {
    position: Position,
}

#[derive(Component, Copy, Clone)]
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

#[derive(Component)]
struct GameStatusText;

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct LastMoveText;

#[derive(Component)]
struct EvaluationText;

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
            update_game_status,
            handle_new_game_button,
            update_last_move,
            update_evaluation_text,
        ));
    }
}

// System functions
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
    let mut commands = commands;
    spawn_initial_pieces(&mut commands, board_offset, &chess_assets);
    
    // UI
    spawn_ui(&mut commands);
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
) {
    let texture = match (piece_type, is_white) {
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

    let position = Position { rank, file };
    let world_pos = board_position_to_world(position, 2.0);

    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(world_pos)
                .with_scale(Vec3::splat(1.0)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(SQUARE_SIZE * 0.8, SQUARE_SIZE * 0.8)),
                anchor: Anchor::Center,
                ..default()
            },
            ..default()
        },
        Piece {
            piece_type,
            is_white,
            position,
        },
    ));
}

fn handle_resize(
    mut board_query: Query<(&mut Transform, &mut Sprite), With<ChessBoard>>,
    mut square_query: Query<(&mut Transform, &mut Sprite, &Square), (With<Square>, Without<ChessBoard>)>,
    mut piece_query: Query<(&mut Transform, &mut Sprite, &Piece), (With<Piece>, Without<ChessBoard>, Without<Square>)>,
) {
    let board_size = 8.0 * SQUARE_SIZE;
    
    // Update board
    if let Ok((mut transform, mut sprite)) = board_query.get_single_mut() {
        sprite.custom_size = Some(Vec2::new(board_size + 20.0, board_size + 20.0));
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }

    let board_offset = Vec3::new(-board_size / 2.0, -board_size / 2.0, 0.0);

    // Update squares
    for (mut transform, mut sprite, square) in square_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE));
        transform.translation = Vec3::new(
            board_offset.x + (square.position.file as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
            board_offset.y + (square.position.rank as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
            1.0,
        );
    }

    // Update pieces
    for (mut transform, mut sprite, piece) in piece_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(SQUARE_SIZE * 0.9, SQUARE_SIZE * 0.9));
        transform.translation = Vec3::new(
            board_offset.x + (piece.position.file as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
            board_offset.y + (piece.position.rank as f32 - 1.0) * SQUARE_SIZE + SQUARE_SIZE / 2.0,
            2.0,
        );
    }
}

fn handle_input(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    mut game_state: ResMut<GameState>,
    mut pieces: Query<(Entity, &mut Piece, &mut Transform)>,
    selected_pieces: Query<Entity, With<SelectedPiece>>,
    mut turn_state: ResMut<NextState<Turn>>,
    turn: Res<State<Turn>>,
) {
    // Only handle input during player's turn
    if *turn.get() != Turn::Player {
        return;
    }

    if game_state.board.is_checkmate() || game_state.board.is_stalemate() {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.single();
        if let Some(position) = get_board_position(window.cursor_position(), window) {
            // If a piece is already selected
            if let Ok(selected_entity) = selected_pieces.get_single() {
                let selected_piece = pieces.iter().find(|(entity, piece, _)| {
                    *entity == selected_entity
                }).map(|(_, piece, _)| *piece);

                if let Some(piece) = selected_piece {
                    // Try to make a move
                    let valid_moves = game_state.board.get_valid_moves(piece.position);
                    if let Some(valid_move) = valid_moves.iter().find(|m| m.to == position) {
                        // First check if there's a piece to capture at the destination
                        let captured_entity = pieces.iter()
                            .find(|(_, p, _)| p.position == valid_move.to)
                            .map(|(e, _, _)| e);

                        if game_state.board.make_move(*valid_move).is_ok() {
                            // Remove captured piece if any
                            if let Some(entity) = captured_entity {
                                commands.entity(entity).despawn();
                            }

                            // Move the piece
                            if let Some((entity, mut piece, _transform)) = pieces.iter_mut().find(|(e, _, _)| *e == selected_entity) {
                                move_piece(
                                    &mut commands,
                                    entity,
                                    &mut piece,
                                    valid_move.to,
                                );
                            }

                            // Deselect the piece
                            commands.entity(selected_entity).remove::<SelectedPiece>();

                            // Switch to AI's turn
                            turn_state.set(Turn::AI);
                            return;
                        }
                    }
                }

                // If clicked on another friendly piece, select it instead
                if let Some((entity, _piece, _)) = pieces.iter().find(|(_, p, _)| {
                    p.position == position && p.is_white
                }) {
                    commands.entity(selected_entity).remove::<SelectedPiece>();
                    commands.entity(entity).insert(SelectedPiece);
                    return;
                }

                // If clicked elsewhere, deselect the piece
                commands.entity(selected_entity).remove::<SelectedPiece>();
            } else {
                // No piece selected - try to select a friendly piece
                if let Some((entity, _piece, _)) = pieces.iter().find(|(_, p, _)| {
                    p.position == position && p.is_white
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
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut pieces: Query<(Entity, &mut Piece, &mut Transform)>,
    mut turn_state: ResMut<NextState<Turn>>,
    turn: Res<State<Turn>>,
) {
    // Only process during AI's turn
    if *turn.get() != Turn::AI {
        return;
    }

    // Set thinking state if not already set
    if !game_state.ai_thinking {
        game_state.ai_thinking = true;
        return;
    }

    // Clone the board to avoid borrow issues
    let board_clone = game_state.board.clone();
    
    // Get AI's move
    if let Some(ai_move) = game_state.ai.get_move(&board_clone) {
        // Try to make the move
        if game_state.board.make_move(ai_move).is_ok() {
            println!("AI attempting move: {:?}", ai_move);
            println!("Move successful on board");
            println!("Moving piece from {:?} to {:?}", ai_move.from, ai_move.to);
            
            // Check if there's a piece to capture at the destination
            let captured_entity = pieces.iter()
                .find(|(_, p, _)| p.position == ai_move.to)
                .map(|(e, _, _)| e);

            // Remove captured piece if any
            if let Some(entity) = captured_entity {
                commands.entity(entity).despawn();
            }
            
            // Find and move the AI piece
            for (entity, mut piece, transform) in pieces.iter_mut() {
                if piece.position == ai_move.from {
                    // Update piece position
                    piece.position = ai_move.to;
                    
                    // Calculate target position in world coordinates
                    let target_pos = board_position_to_world(ai_move.to, transform.translation.z);
                    
                    // Add movement component
                    commands.entity(entity).insert(MovingPiece {
                        target_position: target_pos,
                        speed: 500.0,
                    });
                    break;
                }
            }
        } else {
            println!("Invalid AI move attempted: {:?}", ai_move);
        }
    }
    
    game_state.ai_thinking = false;
    turn_state.set(Turn::Player);
}

fn spawn_ui(commands: &mut Commands) {
    // Main UI container
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // Top bar
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.2, 0.2, 0.2).into(),
            ..default()
        }).with_children(|parent| {
            // Left section with game status and evaluation
            parent.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                // Game status text
                parent.spawn((
                    TextBundle::from_section(
                        "White's Turn",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    GameStatusText,
                ));

                // Evaluation text
                parent.spawn((
                    TextBundle::from_section(
                        "Eval: 0.0",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::left(Val::Px(20.0)),
                        ..default()
                    }),
                    EvaluationText,
                ));

                // AI thinking text
                parent.spawn((
                    TextBundle::from_section(
                        "AI is thinking...",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::YELLOW,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::left(Val::Px(20.0)),
                        ..default()
                    }),
                    AiThinkingText,
                ));
            });

            // New Game button
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.4, 0.4, 0.4).into(),
                    ..default()
                },
                MenuButton,
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "New Game",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
        });

        // Bottom bar
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.2, 0.2, 0.2).into(),
            ..default()
        }).with_children(|parent| {
            // Last move text
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                LastMoveText,
            ));
        });
    });
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
    selected_pieces: Query<&Piece, With<SelectedPiece>>,
    chess_assets: Res<ChessAssets>,
    indicators: Query<Entity, With<ValidMoveIndicator>>,
) {
    // Remove existing indicators
    for entity in indicators.iter() {
        commands.entity(entity).despawn();
    }

    // Show valid moves for selected piece
    if let Ok(piece) = selected_pieces.get_single() {
        if piece.is_white {  // Only show moves for white pieces during player's turn
            let valid_moves = game_state.board.get_valid_moves(piece.position);
            for valid_move in valid_moves {
                let target_pos = board_position_to_world(valid_move.to, 2.0);
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

fn get_board_position(cursor_position: Option<Vec2>, window: &Window) -> Option<Position> {
    cursor_position.map(|cursor| {
        let window_size = Vec2::new(window.width(), window.height());
        let board_size = 8.0 * SQUARE_SIZE;
        
        // Center the board in the window
        let board_start = (window_size - Vec2::splat(board_size)) / 2.0;
        
        // Calculate relative position on board
        let relative_pos = cursor - board_start;
        
        // Convert to file and rank (1-based)
        let file = (relative_pos.x / SQUARE_SIZE).floor() as u8 + 1;
        // Calculate rank from bottom (rank 1) to top (rank 8)
        let rank = (8.0 - (relative_pos.y / SQUARE_SIZE).floor()) as u8;
        
        // Clamp values to valid range
        let file = file.clamp(1, 8);
        let rank = rank.clamp(1, 8);
        
        Position { file, rank }
    })
}

fn board_position_to_world(pos: Position, z: f32) -> Vec3 {
    Vec3::new(
        ((pos.file as f32 - 1.0) - 3.5) * SQUARE_SIZE,
        ((pos.rank as f32 - 1.0) - 3.5) * SQUARE_SIZE,
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
            // Snap to final position when close enough
            transform.translation = moving.target_position;
            commands.entity(entity).remove::<MovingPiece>();
        } else {
            // Smooth movement
            let movement = direction * moving.speed * time.delta_seconds();
            // Prevent overshooting
            if movement.length() > distance {
                transform.translation = moving.target_position;
                commands.entity(entity).remove::<MovingPiece>();
            } else {
                transform.translation += movement;
            }
        }
    }
}

fn move_piece(
    commands: &mut Commands,
    piece_entity: Entity,
    piece: &mut Piece,
    to: Position,
) {
    // Update the piece's position immediately
    piece.position = to;
    
    // Calculate the target position in world coordinates
    let target_pos = board_position_to_world(to, 2.0);

    // Add the MovingPiece component to handle smooth movement
    commands.entity(piece_entity).insert(MovingPiece {
        target_position: target_pos,
        speed: 500.0,
    });
}

fn update_game_status(
    game_state: Res<GameState>,
    turn: Res<State<Turn>>,
    mut query: Query<&mut Text, With<GameStatusText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let status = if game_state.board.is_checkmate() {
            if *turn.get() == Turn::Player {
                "Checkmate - Black wins!"
            } else {
                "Checkmate - White wins!"
            }
        } else if game_state.board.is_stalemate() {
            "Stalemate - Draw!"
        } else {
            match *turn.get() {
                Turn::Player => "White's Turn",
                Turn::AI => "Black's Turn",
            }
        };
        text.sections[0].value = status.to_string();
    }
}

fn handle_new_game_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
    mut game_state: ResMut<GameState>,
    mut turn_state: ResMut<NextState<Turn>>,
    pieces: Query<Entity, With<Piece>>,
    mut commands: Commands,
    chess_assets: Res<ChessAssets>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Reset game state
                game_state.board = Board::new();
                game_state.ai_thinking = false;
                
                // Remove all pieces
                for entity in pieces.iter() {
                    commands.entity(entity).despawn();
                }
                
                // Spawn new pieces
                let board_size = 8.0;
                let board_offset = Vec3::new(
                    -board_size * SQUARE_SIZE / 2.0,
                    -board_size * SQUARE_SIZE / 2.0,
                    0.0
                );
                spawn_initial_pieces(&mut commands, board_offset, &chess_assets);
                
                // Reset turn to player
                turn_state.set(Turn::Player);
                
                // Update button color
                *color = Color::rgb(0.3, 0.3, 0.3).into();
            }
            Interaction::Hovered => {
                *color = Color::rgb(0.5, 0.5, 0.5).into();
            }
            Interaction::None => {
                *color = Color::rgb(0.4, 0.4, 0.4).into();
            }
        }
    }
}

fn update_last_move(
    mut last_move_query: Query<&mut Text, With<LastMoveText>>,
    game_state: Res<GameState>,
) {
    if let Ok(mut text) = last_move_query.get_single_mut() {
        if let Some(last_move) = game_state.board.last_move() {
            let from_square = format!("{}{}", 
                (b'a' + (last_move.from.file - 1)) as char,
                last_move.from.rank
            );
            let to_square = format!("{}{}", 
                (b'a' + (last_move.to.file - 1)) as char,
                last_move.to.rank
            );
            text.sections[0].value = format!("Last move: {} → {}", from_square, to_square);
        }
    }
}

fn update_evaluation_text(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<EvaluationText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let evaluation = chess_engine::evaluation::evaluate_position(&game_state.board);
        
        // Convert centipawns to pawns for readability
        let eval_in_pawns = evaluation as f32 / 100.0;
        
        // Format the evaluation string
        let eval_text = if eval_in_pawns > 0.0 {
            format!("+{:.1}", eval_in_pawns)
        } else {
            format!("{:.1}", eval_in_pawns)
        };

        // Set color based on who's winning
        let color = if evaluation > 0 {
            Color::rgb(0.2, 0.8, 0.2) // Green for white advantage
        } else if evaluation < 0 {
            Color::rgb(0.8, 0.2, 0.2) // Red for black advantage
        } else {
            Color::WHITE // White for equal position
        };

        text.sections[0].value = format!("Eval: {}", eval_text);
        text.sections[0].style.color = color;
    }
}