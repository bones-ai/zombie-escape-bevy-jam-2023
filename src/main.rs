use bevy::{
    app::AppExit,
    asset::AssetMetaCheck,
    audio::{PlaybackMode, Volume, VolumeLevel},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::{vec2, vec3},
    prelude::*,
    time::{common_conditions::on_timer, Stopwatch},
    utils::{Duration, HashSet, Instant},
    window::PrimaryWindow,
};
use rand::Rng;

// Sprite
const SPRITE_SHEET_PATH: &str = "jam-assets.png";
const TILE_W: usize = 16;
const TILE_H: usize = 16;
const SPRITE_SHEET_W: usize = 160 / TILE_W;
const SPRITE_SHEET_H: usize = 160 / TILE_H;

// Window
const WW: usize = 1000;
const WH: usize = 800;
const BG_COLOR: (u8, u8, u8) = (23, 23, 38);

// Car
const TURN_SPEED: f32 = 20.0;
const CAR_THRUST: f32 = 20.0;
const MAX_SPEED: f32 = 40.0;
const FRICTION: f32 = 20.0;
const MAX_CAR_HEALTH: f32 = 200.0;
const MIN_SPEED_TO_STEER: f32 = 0.0;
const TURBO_BOOST: f32 = 60.0;
const TURBO_INTERVAL_SEC: f32 = 5.0;

// Guns and Bullets
const BULLET_TIME: f32 = 1.0;
const BULLET_SPEED: f32 = 20.0 * 100.0;
const BULLET_SPAWN_INTERVAL: f32 = 0.3;
const BULLET_HIT_BOX: f32 = 10.0;

// Roads
const ROAD_WIDTH: usize = 5;
const ROAD_HEIGHT: usize = 600;
const ROAD_SCALE: f32 = 5.0;

// Zombies
const ZOMBIE_SPEED: f32 = 2.55 * 100.0;
const ZOMBIE_ATTACK: f32 = 2.0;

// UI
const COLOR_BROWN: Color = Color::rgb(0.5, 0.25, 0.33);
const COLOR_BLACK: Color = Color::rgb(0.09, 0.09, 0.14);
const COLOR_ORANGE: Color = Color::rgb(0.85, 0.61, 0.38);
const COLOR_LIGHT_ORANGE: Color = Color::rgb(1.0, 0.94, 0.85);

// Textures
#[derive(Resource)]
struct GlobalTextureHandle(Option<Handle<TextureAtlas>>);
#[derive(Component)]
struct GameEntity;

// Car
#[derive(Component)]
struct Car;
#[derive(Component, Reflect)]
struct TurnSpeed(f32);
#[derive(Component, Reflect)]
struct Speed(f32);
#[derive(Component)]
struct Turbo(Stopwatch);
#[derive(Component)]
struct Obstacle;
#[derive(Resource)]
struct VehicleObstacleTiles(Vec<VehicleObstacle>);
#[derive(Event)]
struct PlayerDeadEvent;
#[derive(Resource)]
struct PlayerScore(u32);
#[derive(Resource)]
struct PlayerPos(Vec3);

// Bullet
#[derive(Component)]
struct Bullet(Instant);
#[derive(Component)]
struct BulletDirection(Vec3);

// Roads
#[derive(Component)]
struct Road;
#[derive(Resource)]
struct RoadTiles(HashSet<(i32, i32)>);

// Zombies
#[derive(Component)]
struct Zombie;
#[derive(Event)]
struct ZombieHitPlayer;

// Stats
#[derive(Resource)]
struct CarHealth(f32);
#[derive(Resource)]
struct CarProgress(f32);
#[derive(Component)]
struct DebugText;

// Music
#[derive(Component)]
struct BgMusic;
#[derive(Component)]
struct InGameMusic;
#[derive(Component)]
struct EffectGunShotMusic;
#[derive(Component)]
struct EffectZombieKillMusic;
#[derive(Component)]
struct EffectZombieSpawnMusic;
#[derive(Component)]
struct MainMenuZombie(Vec2);

// UI
#[derive(Component)]
struct GameUIHealthBar;
#[derive(Component)]
struct GameUITurbo;
#[derive(Component)]
struct GameUICarProgress;

#[derive(Component)]
struct MainMenuComponent;
#[derive(Component)]
struct PauseMenuComponent;
#[derive(Component)]
struct GameOverMenuComponent;
#[derive(Component)]
struct SettingsMenuComponent;
#[derive(Component)]
enum MainMenuButtonAction {
    Play,
    Settings,
    Quit,
}
#[derive(Component)]
enum PauseMenuButtonAction {
    Resume,
    // Restart,
    ExitToMainMenu,
}
#[derive(Component)]
enum GameOverMenuButtonAction {
    Restart,
    ExitToMainMenu,
}
#[derive(Component)]
enum SettingsMenuButtonAction {
    Difficulty,
    EntityCount,
    Music,
    GodMode,
    DebugInfo,
    ExitToMainMenu,
}

// wasd controls
struct CarControls(bool, bool, bool, bool);

enum VehicleObstacleType {
    Car1,
    Car2,
    Car3,
    // Truck,
}

struct VehicleObstacle {
    pos: (f32, f32),
    vehicle_type: VehicleObstacleType,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    LoadAssets,
    MainMenu,
    SettingsMenu,
    GameInit,
    InGame,
    PauseMenu,
    GameOver,
}

#[derive(Resource)]
struct GameSettings {
    difficulty: Difficulty,
    entity_count: EntityCount,
    music: bool,
    god_mode: bool,
    debug_info: bool,
}

#[derive(Default)]
enum Difficulty {
    #[default]
    Easy,
    Moderate,
    Hard,
}

#[derive(Default)]
enum EntityCount {
    Hundred,
    FiveHundred,
    Thousand,
    #[default]
    FiveThousand,
    TenThousand,
    TwentyThousand,
    FiftyThousand,
}

fn main() {
    App::new()
        // Before anything, meta check never, to be able to run on itch
        .insert_resource(AssetMetaCheck::Never)
        .add_state::<GameState>()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: true,
                        // mode: WindowMode::Fullscreen,
                        focused: true,
                        // present_mode: PresentMode::Immediate,
                        resolution: (WW as f32, WH as f32).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // Events
        .add_event::<ZombieHitPlayer>()
        .add_event::<PlayerDeadEvent>()
        // Resources
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 255,
        )))
        .insert_resource(Msaa::Off)
        .insert_resource(GlobalTextureHandle(None))
        .insert_resource(RoadTiles(HashSet::new()))
        .insert_resource(VehicleObstacleTiles(Vec::new()))
        .insert_resource(CarHealth(MAX_CAR_HEALTH))
        .insert_resource(CarProgress(0.0))
        .insert_resource(GameSettings::default())
        .insert_resource(PlayerScore(0))
        .insert_resource(PlayerPos(Vec3::ZERO))
        // Systems
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_music)
        .add_systems(Update, update_music)
        .add_systems(
            Update,
            menu_buttons_update.run_if(not(in_state(GameState::InGame))),
        )
        // Loading Systems
        .add_systems(OnEnter(GameState::LoadAssets), load_assets)
        // MainMenu Systems
        .add_systems(
            OnEnter(GameState::MainMenu),
            (
                setup_main_menu,
                cleanup_previous_game,
                spawn_main_menu_zombies,
            ),
        )
        .add_systems(
            Update,
            (handle_main_menu_btn_click, update_main_menu_zombies)
                .run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            OnExit(GameState::MainMenu),
            (cleanup_main_menu, cleanup_main_menu_zombies),
        )
        // Settings Systems
        .add_systems(OnEnter(GameState::SettingsMenu), setup_settings_menu)
        .add_systems(
            Update,
            handle_settings_menu_btn_click.run_if(in_state(GameState::SettingsMenu)),
        )
        .add_systems(OnExit(GameState::SettingsMenu), cleanup_settings_menu)
        // PauseMenu Systems
        .add_systems(OnEnter(GameState::PauseMenu), setup_pause_menu)
        .add_systems(
            Update,
            handle_pause_menu_btn_click.run_if(in_state(GameState::PauseMenu)),
        )
        .add_systems(OnExit(GameState::PauseMenu), cleanup_pause_menu)
        // GameOver Systems
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_menu)
        .add_systems(
            Update,
            handle_game_over_menu_btn_click.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            OnExit(GameState::GameOver),
            (cleanup_game_over_menu, cleanup_previous_game),
        )
        // GameInit Systems
        .add_systems(
            OnEnter(GameState::GameInit),
            (setup_game, spawn_road, setup_game_ui),
        )
        .add_systems(
            Update,
            mark_game_setup_done.run_if(in_state(GameState::GameInit)),
        )
        // InGame Systems
        .add_systems(
            Update,
            (
                car_manual_input_system,
                bullet_hit_zombie,
                check_obstacle_collision,
                check_zombie_collision,
                handle_zombie_player_hit,
                update_car_progress,
                handle_turbo_input,
                update_zombies,
                handle_camera_zoom,
                despawn_zombies,
                spawn_zombies,
                despawn_bullets,
                camera_follow_player,
                update_bullet,
                handle_escape_key,
                handle_player_dead_event,
                update_game_ui_health_bar,
                update_game_ui_turbo,
                update_game_ui_car_progress,
                handle_game_complete,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            update_debug_text
                // .run_if(on_timer(Duration::from_secs_f32(1.0)))
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            shoot_gun
                .run_if(on_timer(Duration::from_secs_f32(BULLET_SPAWN_INTERVAL)))
                .run_if(in_state(GameState::InGame)),
        )
        // .add_systems(Update, close_on_esc)
        .run();
}

fn load_assets(
    mut next_state: ResMut<NextState<GameState>>,
    mut global_texture_handle: ResMut<GlobalTextureHandle>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load(SPRITE_SHEET_PATH);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        vec2(TILE_W as f32, TILE_H as f32),
        SPRITE_SHEET_W,
        SPRITE_SHEET_H,
        None,
        None,
    );
    global_texture_handle.0 = Some(texture_atlases.add(texture_atlas));
    next_state.set(GameState::MainMenu);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("menubg.mp3"),
            settings: PlaybackSettings {
                volume: Volume::Absolute(VolumeLevel::new(0.7)),
                mode: PlaybackMode::Loop,
                ..Default::default()
            },
            ..default()
        },
        BgMusic,
    ));
}

fn update_music(music_query: Query<&AudioSink, With<BgMusic>>, settings: Res<GameSettings>) {
    if music_query.is_empty() {
        return;
    }

    let music = music_query.get_single().unwrap();
    if settings.music && music.is_paused() {
        music.play();
        return;
    }
    if !settings.music && !music.is_paused() {
        music.pause();
        return;
    }
}

fn setup_game_ui(mut commands: Commands, handle: Res<GlobalTextureHandle>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::End,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(40.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::End,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::RowReverse,
                                align_items: AlignItems::Center,
                                align_content: AlignContent::Start,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(AtlasImageBundle {
                                style: Style {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    margin: UiRect::px(16.0, 32.0, 8.0, 8.0),
                                    ..default()
                                },
                                texture_atlas: handle.0.clone().unwrap(),
                                texture_atlas_image: UiTextureAtlasImage {
                                    index: 10,
                                    ..default()
                                },
                                ..default()
                            });
                            parent.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(200.0),
                                        height: Val::Px(32.0),
                                        margin: UiRect::px(16.0, 16.0, 8.0, 8.0),
                                        ..default()
                                    },
                                    texture_atlas: handle.0.clone().unwrap(),
                                    texture_atlas_image: UiTextureAtlasImage {
                                        index: 13,
                                        ..default()
                                    },
                                    ..default()
                                },
                                GameUIHealthBar,
                            ));
                        });
                });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::End,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(40.0),
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::End,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::ColumnReverse,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        margin: UiRect::all(Val::Px(40.0)),
                                        ..default()
                                    },
                                    texture_atlas: handle.0.clone().unwrap(),
                                    texture_atlas_image: UiTextureAtlasImage {
                                        index: 12,
                                        ..default()
                                    },
                                    ..default()
                                },
                                GameUITurbo,
                            ));
                        });
                });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Percent(50.0),
                    top: Val::Percent(25.0),
                    margin: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                border_color: COLOR_LIGHT_ORANGE.into(),
                background_color: COLOR_BROWN.into(),
                ..default()
            });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            parent.spawn(AtlasImageBundle {
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    margin: UiRect::all(Val::Px(16.0)),
                    top: Val::Percent(25.0),
                    ..default()
                },
                texture_atlas: handle.0.clone().unwrap(),
                texture_atlas_image: UiTextureAtlasImage {
                    index: 14,
                    ..default()
                },
                ..default()
            });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            GameEntity,
        ))
        .with_children(|parent| {
            parent
                .spawn(AtlasImageBundle {
                    style: Style {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(16.0)),
                        top: Val::Percent(70.0),
                        ..default()
                    },
                    texture_atlas: handle.0.clone().unwrap(),
                    texture_atlas_image: UiTextureAtlasImage {
                        index: 15,
                        ..default()
                    },
                    ..default()
                })
                .insert(GameUICarProgress);
        });
}

fn update_game_ui_health_bar(
    mut ui_bar_query: Query<&mut Style, With<GameUIHealthBar>>,
    car_health: Res<CarHealth>,
) {
    if ui_bar_query.is_empty() {
        return;
    }

    let mut health_bar = ui_bar_query.single_mut();
    health_bar.width = Val::Px(car_health.0);
}

fn update_game_ui_car_progress(
    mut car_progress_ui: Query<&mut Style, With<GameUICarProgress>>,
    car_progress: Res<CarProgress>,
) {
    if car_progress_ui.is_empty() {
        return;
    }

    let mut style = car_progress_ui.single_mut();
    let scaled_progress = ((((car_progress.0 * 100.0) / 100.0) + 1.0) * (70.0 - 27.0)) + 27.0;
    style.top = Val::Percent(70.0 + (70.0 - scaled_progress));
}

fn handle_game_complete(
    mut car_progress: ResMut<CarProgress>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    // Player going the opposite way
    if car_progress.0 < -0.1 {
        game_state.set(GameState::GameOver);
        return;
    }

    if car_progress.0 < 1.0 {
        return;
    }

    car_progress.0 = 1.0;
    game_state.set(GameState::GameOver);
}

fn update_game_ui_turbo(
    mut turbo_ui: Query<&mut Visibility, With<GameUITurbo>>,
    car_query: Query<&Turbo, With<Car>>,
) {
    if turbo_ui.is_empty() {
        return;
    }
    if car_query.is_empty() {
        return;
    }

    let turbo = car_query.single();
    let mut turbo_button = turbo_ui.single_mut();
    let turbo_percentage = (turbo.0.elapsed().as_secs_f32() / TURBO_INTERVAL_SEC).min(1.0) * 100.0;

    if turbo_percentage >= 100.0 {
        *turbo_button = Visibility::Visible;
    } else {
        *turbo_button = Visibility::Hidden;
    }
}

fn setup_game(
    mut commands: Commands, 
    mut player_pos: ResMut<PlayerPos>, 
    global_texture_handle: Res<GlobalTextureHandle>, 
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 20.0,
                font: asset_server.load("font.ttf"),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(16.0)),
            ..default()
        }),
        DebugText,
        GameEntity,
    ));

    // Spawn Car
    let (x, y, z) = (150.0, 50.0, 10.0);
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: global_texture_handle.0.clone().unwrap(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform::from_scale(Vec3::splat(3.0))
                .with_translation(vec3(x, y, z)),
            ..default()
        },
        Car,
        Speed(10.0),
        TurnSpeed(0.0),
        Turbo(Stopwatch::new()),
        GameEntity,
    ));
    player_pos.0 = vec3(x, y, z);
}

fn cleanup_main_menu_zombies(mut commands: Commands, zombies: Query<Entity, With<MainMenuZombie>>) {
    for e in zombies.iter() {
        commands.entity(e).despawn();
    }
}

fn spawn_main_menu_zombies(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    texture_handle: Res<GlobalTextureHandle>,
) {
    let mut rng = rand::thread_rng();
    let camera_transform = camera_query.single().translation;
    for _ in 0..200 {
        let mut tile = rng.gen_range(30..40);
        let mut scale = 2.5;

        if rng.gen_range(0.0..1.0) > 0.8 {
            tile = rng.gen_range(40..44);
            scale = 3.2;
        }

        let x = rng.gen_range(-500.0 + camera_transform.x..500.0 + camera_transform.x + 300.0);
        let y = rng.gen_range(-500.0 + camera_transform.y..500.0 + camera_transform.y + 300.0);
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_handle.0.clone().unwrap(),
                sprite: TextureAtlasSprite::new(tile),
                transform: Transform::from_scale(Vec3::splat(scale))
                    .with_translation(vec3(x, y, 0.0)),
                ..default()
            },
            MainMenuZombie(vec2(
                x + rng.gen_range(-500.0..500.0),
                y + rng.gen_range(-500.0..500.0),
            )),
        ));
    }
}

fn update_main_menu_zombies(
    mut zombies: Query<(&mut Transform, &MainMenuZombie), With<MainMenuZombie>>,
    time: Res<Time>
) {
    for (mut transform, target) in zombies.iter_mut() {
        let dir = vec3(
            target.0.x - transform.translation.x,
            target.0.y - transform.translation.y,
            0.0,
        )
        .normalize();
        transform.translation += dir * ZOMBIE_SPEED * 0.025 * time.delta_seconds();
    }
}

fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: COLOR_BLACK,
        font: asset_server.load("font.ttf"),
        ..Default::default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            MainMenuComponent,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "Paused",
                            TextStyle {
                                font_size: 70.0,
                                font: asset_server.load("font.ttf"),
                                color: COLOR_LIGHT_ORANGE,
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );
                    parent.spawn(
                        TextBundle::from_section(
                            "- WASD to move\n- SpaceBar for Turbo (when available)\n- Hold left click to shoot",
                            TextStyle {
                                font_size: 30.0,
                                font: asset_server.load("font.ttf"),
                                color: COLOR_LIGHT_ORANGE,
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            PauseMenuButtonAction::Resume,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Resume",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            PauseMenuButtonAction::ExitToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Main Menu", button_text_style));
                        });
                });
        });
}

fn setup_game_over_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_score: Res<PlayerScore>,
    car_progress: Res<CarProgress>,
) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        font: asset_server.load("font.ttf"),
        color: COLOR_BLACK,
        ..Default::default()
    };
    let message = if car_progress.0 < 0.0 {
        "Zombies that way ;)\nGo north!"
    } else if car_progress.0 >= 0.98 {
        "You Survived!"
    } else {
        "You got Mauled"
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            GameOverMenuComponent,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            message,
                            TextStyle {
                                font: asset_server.load("font.ttf"),
                                font_size: 70.0,
                                color: COLOR_LIGHT_ORANGE,
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        }),
                    );
                    parent.spawn(
                        TextBundle::from_section(
                            format!("Score: {:?}", player_score.0).to_string(),
                            TextStyle {
                                font: asset_server.load("font.ttf"),
                                font_size: 60.0,
                                color: COLOR_LIGHT_ORANGE,
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::bottom(Val::Px(32.0)),
                            ..default()
                        }),
                    );
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            GameOverMenuButtonAction::Restart,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Restart",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            GameOverMenuButtonAction::ExitToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Main Menu", button_text_style));
                        });
                });
        });
}

fn setup_settings_menu(
    mut commands: Commands,
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
) {
    let button_style = Style {
        width: Val::Px(500.0),
        height: Val::Px(85.0),
        margin: UiRect::axes(Val::Px(40.0), Val::Px(1.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: COLOR_BLACK,
        font: asset_server.load("font.ttf"),
        ..Default::default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            SettingsMenuComponent,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "Settings",
                            TextStyle {
                                font_size: 70.0,
                                font: asset_server.load("font.ttf"),
                                color: COLOR_LIGHT_ORANGE,
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::Difficulty,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                settings.difficulty_as_str().to_string(),
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::EntityCount,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                settings.entity_count_as_str().to_string(),
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::Music,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                settings.music_as_str().to_string(),
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::GodMode,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                settings.god_mode_as_str().to_string(),
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::DebugInfo,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                settings.debug_info_as_str().to_string(),
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            SettingsMenuButtonAction::ExitToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Back", button_text_style));
                        });
                });
        });
}

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: COLOR_BLACK,
        font: asset_server.load("font.ttf"),
        ..Default::default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            MainMenuComponent,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "That's a LOT of Zombies",
                            TextStyle {
                                font_size: 70.0,
                                color: COLOR_LIGHT_ORANGE,
                                font: asset_server.load("font.ttf"),
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::axes(Val::Px(30.0), Val::Px(90.0)),
                            ..default()
                        }),
                    );
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            MainMenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "New Game",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            MainMenuButtonAction::Settings,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Settings",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: COLOR_ORANGE.into(),
                                ..default()
                            },
                            MainMenuButtonAction::Quit,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Quit", button_text_style));
                        });
                });
        });
}

fn menu_buttons_update(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => COLOR_LIGHT_ORANGE.into(),
            Interaction::Hovered => COLOR_LIGHT_ORANGE.into(),
            _ => COLOR_ORANGE.into(),
        }
    }
}

fn handle_main_menu_btn_click(
    interaction_query: Query<
        (&Interaction, &MainMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_app_exit: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MainMenuButtonAction::Quit => ev_app_exit.send(AppExit),
                MainMenuButtonAction::Settings => {
                    game_state.set(GameState::SettingsMenu);
                }
                MainMenuButtonAction::Play => {
                    game_state.set(GameState::GameInit);
                }
            }
        }
    }
}

fn handle_settings_menu_btn_click(
    interaction_query: Query<
        (&Interaction, &SettingsMenuButtonAction, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<&mut Text>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action, children) in &interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                SettingsMenuButtonAction::Difficulty => {
                    settings.update_difficulty();
                    text.sections[0].value = settings.difficulty_as_str().to_string();
                }
                SettingsMenuButtonAction::EntityCount => {
                    settings.update_entity_count();
                    text.sections[0].value = settings.entity_count_as_str().to_string();
                }
                SettingsMenuButtonAction::Music => {
                    settings.music = !settings.music;
                    text.sections[0].value = settings.music_as_str().to_string();
                }
                SettingsMenuButtonAction::DebugInfo => {
                    settings.debug_info = !settings.debug_info;
                    text.sections[0].value = settings.debug_info_as_str().to_string();
                }
                SettingsMenuButtonAction::GodMode => {
                    settings.god_mode = !settings.god_mode;
                    text.sections[0].value = settings.god_mode_as_str().to_string();
                }
                SettingsMenuButtonAction::ExitToMainMenu => {
                    game_state.set(GameState::MainMenu);
                }
            }
        }
    }
}

fn handle_pause_menu_btn_click(
    interaction_query: Query<
        (&Interaction, &PauseMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                PauseMenuButtonAction::Resume => {
                    game_state.set(GameState::InGame);
                }
                PauseMenuButtonAction::ExitToMainMenu => {
                    game_state.set(GameState::MainMenu);
                }
            }
        }
    }
}

fn handle_game_over_menu_btn_click(
    interaction_query: Query<
        (&Interaction, &GameOverMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                GameOverMenuButtonAction::ExitToMainMenu => {
                    game_state.set(GameState::MainMenu);
                }
                GameOverMenuButtonAction::Restart => {
                    game_state.set(GameState::GameInit);
                }
            }
        }
    }
}

fn cleanup_main_menu(
    mut commands: Commands,
    main_menu_query: Query<Entity, With<MainMenuComponent>>,
) {
    if main_menu_query.is_empty() {
        return;
    }

    let main_menu = main_menu_query.single();
    commands.entity(main_menu).despawn_recursive();
}

fn cleanup_settings_menu(
    mut commands: Commands,
    settings_menu_query: Query<Entity, With<SettingsMenuComponent>>,
) {
    if settings_menu_query.is_empty() {
        return;
    }

    let settings_menu = settings_menu_query.single();
    commands.entity(settings_menu).despawn_recursive();
}

fn cleanup_pause_menu(
    mut commands: Commands,
    pause_menu_query: Query<Entity, With<MainMenuComponent>>,
) {
    if pause_menu_query.is_empty() {
        return;
    }

    let pause_menu = pause_menu_query.single();
    commands.entity(pause_menu).despawn_recursive();
}

fn cleanup_game_over_menu(
    mut commands: Commands,
    game_over_menu_query: Query<Entity, With<GameOverMenuComponent>>,
) {
    if game_over_menu_query.is_empty() {
        return;
    }

    let game_over_menu = game_over_menu_query.single();
    commands.entity(game_over_menu).despawn_recursive();
}

fn handle_camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    car_progress: Res<CarProgress>,
    time: Res<Time>,
) {
    for mut projection in query.iter_mut() {
        if car_progress.0 > 0.90 {
            if projection.scale <= 1.8 {
                return;
            }

            projection.scale -= 0.2 * time.delta_seconds();
            return;
        }

        if projection.scale >= 2.0 {
            return;
        }
        projection.scale += 0.1 * time.delta_seconds();
    }
}

fn handle_turbo_input(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut car_query: Query<&mut Turbo, With<Car>>,
    keyboard_input: Res<Input<KeyCode>>,
    settings: Res<GameSettings>,
) {
    if car_query.is_empty() {
        return;
    }

    let mut turbo = car_query.single_mut();
    turbo.0.tick(time.delta());

    if turbo.0.elapsed().as_secs_f32() <= TURBO_INTERVAL_SEC {
        return;
    }
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    turbo.0.reset();
    if settings.music {
        commands.spawn(AudioBundle {
            source: asset_server.load("turbo.mp3"),
            settings: PlaybackSettings {
                volume: Volume::Absolute(VolumeLevel::new(0.5)),
                ..Default::default()
            },
            ..default()
        });
    }
}

fn update_debug_text(
    time: Res<Time>,
    mut text_query: Query<&mut Text, With<DebugText>>,
    zom_query: Query<(With<Zombie>, Without<DebugText>)>,
    diagnostics: Res<DiagnosticsStore>,
    car_progress: Res<CarProgress>,
    car_health: Res<CarHealth>,
    player_score: Res<PlayerScore>,
    settings: Res<GameSettings>,
) {
    if text_query.is_empty() || !settings.debug_info {
        return;
    }

    let mut text = text_query.single_mut();
    let mut fps = 0.0;
    if let Some(d) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = d.smoothed() {
            fps = value;
        }
    }
    text.sections[0].value = format!(
        "Fps: {:.2?}\nTime: {:.2?}\nProgress: {:.2?}\nHealth: {:?}\nZoms: {:?}\nScore: {:?}",
        fps,
        time.delta_seconds(),
        car_progress.0,
        car_health.0,
        zom_query.iter().len(),
        player_score.0
    );
}

fn update_car_progress(
    car_query: Query<&Transform, With<Car>>,
    mut car_progress: ResMut<CarProgress>,
) {
    if car_query.is_empty() {
        return;
    }

    let car_transform = car_query.single();
    let (_, cy) = (car_transform.translation.x, car_transform.translation.y);

    let total_h = TILE_H as f32 * ROAD_SCALE * ROAD_HEIGHT as f32;
    car_progress.0 = cy / total_h;
}

fn handle_zombie_player_hit(
    mut car_health: ResMut<CarHealth>,
    mut reader_zombie_player_hit: EventReader<ZombieHitPlayer>,
    mut writer_player_dead: EventWriter<PlayerDeadEvent>,
    settings: Res<GameSettings>,
) {
    if reader_zombie_player_hit.is_empty() || settings.god_mode {
        return;
    }

    car_health.0 -= ZOMBIE_ATTACK * reader_zombie_player_hit.len() as f32;
    reader_zombie_player_hit.clear();

    // Player dead
    if car_health.0 <= 0.0 {
        writer_player_dead.send(PlayerDeadEvent);
    }
}

fn check_obstacle_collision(
    obstacles: Res<VehicleObstacleTiles>,
    mut car_query: Query<(&Transform, &mut Speed), With<Car>>,
) {
    if car_query.is_empty() {
        return;
    }

    let (car_transform, mut speed) = car_query.single_mut();
    for obstacle in obstacles.0.iter() {
        if (obstacle.pos.0 - car_transform.translation.x).abs() <= 25.0
            && (obstacle.pos.1 - car_transform.translation.y).abs() <= 25.0
        {
            speed.0 = -6.0;
        }

        // speed = 10.0 or some +ve value slows the vehicle. given it was > 10.0 to begin with
    }
}

fn spawn_zombies(
    mut commands: Commands,
    texture_handle: Res<GlobalTextureHandle>,
    car_query: Query<&Transform, With<Car>>,
    zombie_query: Query<With<Zombie>>,
    car_progress: Res<CarProgress>,
    settings: Res<GameSettings>,
) {
    if car_query.is_empty() {
        return;
    }

    let max_zombies = settings.get_num_max_zombies();
    let num_zombies = (max_zombies as f32 * car_progress.0 + 5.0).min(max_zombies as f32);
    if zombie_query.iter().len() >= num_zombies as usize {
        return;
    }

    let mut rng = rand::thread_rng();
    let car_transform = car_query.single();
    let (cx, cy) = (car_transform.translation.x, car_transform.translation.y);

    let normal_zombie_probability = match settings.difficulty {
        Difficulty::Easy => 0.99,
        Difficulty::Moderate => 0.98,
        Difficulty::Hard => 0.96,
    };
    let is_enable_road_zombies = match settings.difficulty {
        Difficulty::Easy => car_progress.0 >= 0.7,
        Difficulty::Moderate => car_progress.0 >= 0.6,
        Difficulty::Hard => true,
    };
    let is_enable_half_road_zombies = match settings.difficulty {
        Difficulty::Easy => car_progress.0 >= 0.4,
        Difficulty::Moderate => true,
        Difficulty::Hard => true,
    };

    for _ in 0..50 {
        let (mut x, mut y) = (rng.gen_range(0.0..400.0), rng.gen_range(0.0..400.0));
        if rng.gen_range(0.0..1.0) < normal_zombie_probability {
            (x, y) = match rng.gen_range(1..=8) {
                1 => (cx + WW as f32 + x, cy + y),
                2 => (cx + WW as f32 + x, cy + y + rng.gen_range(1000.0..1200.0)),
                3 => (cx + WW as f32 + x, cy + y + rng.gen_range(1500.0..1800.0)),
                7 => (cx + WW as f32 + x, cy + y + rng.gen_range(1800.0..2500.0)),
                4 => (cx - WW as f32 - x, cy - y),
                5 => (cx - WW as f32 - x, cy - y + rng.gen_range(1000.0..1200.0)),
                6 => (cx - WW as f32 - x, cy - y + rng.gen_range(1500.0..1800.0)),
                _ => (cx - WW as f32 - x, cy - y + rng.gen_range(1800.0..2500.0)),
            };
        } else if is_enable_road_zombies {
            if rng.gen_range(0.0..1.0) <= 0.5 {
                (x, y) = (
                    cx + x + (WW as f32) / 2.0 + rng.gen_range(0.0..100.0),
                    cy + y + rng.gen_range(1000.0..1500.0),
                );
            } else {
                (x, y) = (
                    cx - x - (WW as f32) / 2.0 - rng.gen_range(0.0..100.0),
                    cy + y + rng.gen_range(1000.0..1500.0),
                );
            }
        } else if is_enable_half_road_zombies {
            if rng.gen_range(0.0..1.0) <= 0.5 {
                (x, y) = (
                    cx + x + (WW as f32) / 2.0 + rng.gen_range(100.0..200.0),
                    cy + y + rng.gen_range(1000.0..1500.0),
                );
            } else {
                (x, y) = (
                    cx - x - (WW as f32) / 2.0 - rng.gen_range(100.0..200.0),
                    cy + y + rng.gen_range(1000.0..1500.0),
                );
            }
        } else {
            // Don't spawn the zoms on top of player
            (x, y) = (10000.0, 10000.0);
        }

        let mut tile = rng.gen_range(30..40);
        let mut scale = 2.5;
        if rng.gen_range(0.0..1.0) > 0.9 && car_progress.0 >= 0.3 {
            tile = rng.gen_range(40..44);
            scale = 3.2;
        }
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_handle.0.clone().unwrap(),
                sprite: TextureAtlasSprite::new(tile),
                transform: Transform::from_scale(Vec3::splat(scale))
                    .with_translation(vec3(x, y, 1.0)),
                ..default()
            },
            Zombie,
            GameEntity,
        ));
    }
}

fn cleanup_previous_game(
    mut commands: Commands,
    mut road_tiles: ResMut<RoadTiles>,
    mut obstacles: ResMut<VehicleObstacleTiles>,
    mut car_health: ResMut<CarHealth>,
    mut car_progress: ResMut<CarProgress>,
    mut player_score: ResMut<PlayerScore>,
    mut player_position: ResMut<PlayerPos>,
    mut cam_query: Query<&mut OrthographicProjection, With<Camera>>,
    entities: Query<Entity, With<GameEntity>>,
) {
    for e in entities.iter() {
        commands.entity(e).despawn_recursive();
    }

    road_tiles.0.clear();
    obstacles.0.clear();
    car_health.0 = MAX_CAR_HEALTH;
    car_progress.0 = 0.0;
    player_score.0 = 0;
    player_position.0 = Vec3::ZERO;

    for mut projection in cam_query.iter_mut() {
        projection.scale = 1.0;
    }
}

fn despawn_zombies(
    mut commands: Commands,
    car_query: Query<&Transform, (With<Car>, Without<Zombie>)>,
    zombie_query: Query<(Entity, &Transform), With<Zombie>>,
) {
    if car_query.is_empty() {
        return;
    }

    let car_transform = car_query.single();
    for (e, t) in zombie_query.iter() {
        if car_transform.translation.y - t.translation.y <= 700.0 {
            continue;
        }

        commands.entity(e).despawn();
    }
}

fn bullet_hit_zombie(
    mut commands: Commands,
    mut player_score: ResMut<PlayerScore>,
    bullets_query: Query<&Transform, With<Bullet>>,
    zombie_query: Query<(Entity, &Transform), (With<Zombie>, Without<Bullet>)>,
) {
    for (e, t) in zombie_query.iter() {
        for b in bullets_query.iter() {
            if (b.translation.x - t.translation.x).abs() <= BULLET_HIT_BOX
                && (b.translation.y - t.translation.y).abs() <= BULLET_HIT_BOX
            {
                player_score.0 += 1;
                commands.entity(e).despawn();
            }
        }
    }
}

fn check_zombie_collision(
    zombie_query: Query<&Transform, With<Zombie>>,
    car_query: Query<&Transform, (With<Car>, Without<Zombie>)>,
    mut writer_player_hit: EventWriter<ZombieHitPlayer>,
) {
    if car_query.is_empty() {
        return;
    }

    let car_transform = car_query.single();
    let (car_x, car_y) = (car_transform.translation.x, car_transform.translation.y);
    for t in zombie_query.iter() {
        let y_dist = (t.translation.y - car_y).abs();
        let x_dist = (t.translation.x - car_x).abs();
        if x_dist <= 20.0 && y_dist <= 20.0 {
            writer_player_hit.send(ZombieHitPlayer);
        }
    }
}

fn update_zombies(
    time: Res<Time>,
    mut zombie_query: Query<&mut Transform, With<Zombie>>,
    car_query: Query<&Transform, (With<Car>, Without<Zombie>)>,
) {
    if car_query.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let car_transform = car_query.single();
    let (car_x, car_y) = (car_transform.translation.x, car_transform.translation.y);
    let target_x = car_x;

    for mut z in zombie_query.iter_mut() {
        let mut target_y = car_y;
        if z.translation.y - target_y > 500.0 && rng.gen_range(0.0..1.0) > 0.5 {
            target_y += rng.gen_range(500.0..1500.0);
        }

        let dir = vec3(target_x - z.translation.x, target_y - z.translation.y, 0.0).normalize();
        let rand_dir = vec3(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5), 0.0);

        z.translation += (dir + rand_dir) * ZOMBIE_SPEED * time.delta_seconds();
    }
}

fn spawn_road(
    mut commands: Commands,
    mut road_tiles: ResMut<RoadTiles>,
    mut obstacle_tiles: ResMut<VehicleObstacleTiles>,
    texture_handle: Res<GlobalTextureHandle>,
) {
    let mut rng = rand::thread_rng();
    let top_y = ROAD_HEIGHT as i32;
    let bottom_y = -10;
    let left_x = 0;
    let right_x = ROAD_WIDTH as i32;
    let mut offset = 0;
    let mut n_offset = 0;
    let mut p_offset = 0;

    for j in bottom_y..=top_y {
        let is_top_y = j == top_y || j == top_y - 1;

        if is_top_y {
            for a in 0..=ROAD_WIDTH as i32 {
                let (x, y) = (
                    (offset + a) as f32 * TILE_W as f32 * ROAD_SCALE,
                    j as f32 * TILE_H as f32 * ROAD_SCALE,
                );
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_handle.0.clone().unwrap(),
                        sprite: TextureAtlasSprite::new(17),
                        transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                            .with_translation(vec3(x, y, 1.0)),
                        ..default()
                    },
                    Road,
                    GameEntity,
                ));
            }
        }

        if j % 5 == 0 && !is_top_y {
            p_offset = offset;
            offset = n_offset;
            n_offset += rng.gen_range(-1..=1);
        }

        // if next tile is curve
        if (j + 1) % 5 == 0 && !is_top_y {
            if (offset - n_offset) == 1 {
                let (x, y) = (
                    (0 + offset - 1) as f32 * TILE_W as f32 * ROAD_SCALE,
                    j as f32 * TILE_H as f32 * ROAD_SCALE,
                );
                let (x, y) = (x + 10.0, y);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_handle.0.clone().unwrap(),
                        sprite: TextureAtlasSprite::new(85),
                        transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                            .with_translation(vec3(x, y, 0.0)),
                        ..default()
                    },
                    Road,
                    GameEntity,
                ));
            } else if (offset - n_offset) == -1 {
                let (x, y) = (
                    (0 + n_offset + 1 + ROAD_WIDTH as i32 - 1) as f32 * TILE_W as f32 * ROAD_SCALE,
                    j as f32 * TILE_H as f32 * ROAD_SCALE,
                );
                let (x, y) = (x - 10.0, y);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_handle.0.clone().unwrap(),
                        sprite: TextureAtlasSprite::new(86),
                        transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                            .with_translation(vec3(x, y, 0.0)),
                        ..default()
                    },
                    Road,
                    GameEntity,
                ));
            }
        }

        // OBSTACLE
        if rng.gen_range(0.0..1.0) > 0.9 && j > 30 {
            let (mut x, y) = (
                (rng.gen_range((offset + 1)..(offset + 5))) as f32 * TILE_W as f32 * ROAD_SCALE,
                j as f32 * TILE_H as f32 * ROAD_SCALE,
            );
            x += rng.gen_range(-1.0..=-1.0) * TILE_W as f32 * ROAD_SCALE;

            let obstacle = VehicleObstacle::new((x, y));
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_handle.0.clone().unwrap(),
                    sprite: TextureAtlasSprite::new(obstacle.vehicle_type.sprite_idx()),
                    transform: Transform::from_scale(Vec3::splat(3.0))
                        .with_translation(vec3(x, y, 1.0)),
                    ..default()
                },
                Obstacle,
                GameEntity,
            ));
            obstacle_tiles.0.push(obstacle);
        }

        // road decorations
        if rng.gen_range(0.0..1.0) > 0.6 {
            let (mut x, y) = (
                (offset - 2 - rng.gen_range(1..=2)) as f32 * TILE_W as f32 * ROAD_SCALE,
                j as f32 * TILE_H as f32 * ROAD_SCALE,
            );
            if rng.gen_range(0.0..1.0) > 0.5 {
                x += 2.0 * (ROAD_WIDTH) as f32 * TILE_W as f32 * ROAD_SCALE;
                x += rng.gen_range(1.0..3.0) * TILE_W as f32 * ROAD_SCALE;
            }

            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_handle.0.clone().unwrap(),
                    sprite: TextureAtlasSprite::new(50),
                    transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                        .with_translation(vec3(x, y, 0.0)),
                    ..default()
                },
                Road,
                GameEntity,
            ));
        }

        if j % 5 == 0 && !is_top_y {
            if (offset - p_offset) == 1 {
                let (x, y) = (
                    (0 + offset - 1) as f32 * TILE_W as f32 * ROAD_SCALE,
                    j as f32 * TILE_H as f32 * ROAD_SCALE,
                );
                let (x, y) = (x + 10.0, y);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_handle.0.clone().unwrap(),
                        sprite: TextureAtlasSprite::new(83),
                        transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                            .with_translation(vec3(x, y, 0.0)),
                        ..default()
                    },
                    Road,
                    GameEntity,
                ));
            } else if (offset - p_offset) == -1 {
                let (x, y) = (
                    (0 + offset + 1 + ROAD_WIDTH as i32) as f32 * TILE_W as f32 * ROAD_SCALE,
                    j as f32 * TILE_H as f32 * ROAD_SCALE,
                );
                let (x, y) = (x - 10.0, y);
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture_handle.0.clone().unwrap(),
                        sprite: TextureAtlasSprite::new(84),
                        transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                            .with_translation(vec3(x, y, 0.0)),
                        ..default()
                    },
                    Road,
                    GameEntity,
                ));
            }
        }

        for i in left_x..=right_x {
            let tile = if i == 0 {
                80
            } else if i == right_x {
                82
            } else {
                81
            };
            let (x, y) = (
                (i + offset) as f32 * TILE_W as f32 * ROAD_SCALE,
                j as f32 * TILE_H as f32 * ROAD_SCALE,
            );
            road_tiles.0.insert((i + offset, j));
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: texture_handle.0.clone().unwrap(),
                    sprite: TextureAtlasSprite::new(tile),
                    transform: Transform::from_scale(Vec3::splat(ROAD_SCALE))
                        .with_translation(vec3(x, y, 0.0)),
                    ..default()
                },
                Road,
                GameEntity,
            ));
        }
    }
}

fn mark_game_setup_done(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::InGame);
}

fn shoot_gun(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    buttons: Res<Input<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
    texture_handle: Res<GlobalTextureHandle>,
    car_query: Query<&Transform, With<Car>>,
    settings: Res<GameSettings>,
) {
    if car_query.is_empty() {
        return;
    }
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    let mut cursor_pos = None;
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_pos = Some(vec3(world_position.x, world_position.y, 0.0));
    }

    let car_transform = car_query.single();
    let (x, y) = (car_transform.translation.x, car_transform.translation.y);
    let direction = if cursor_pos.is_none() {
        car_transform.local_y()
    } else {
        cursor_pos.unwrap() - car_transform.translation
    };

    if settings.music {
        commands.spawn(AudioBundle {
            source: asset_server.load("bulletfire.mp3"),
            settings: PlaybackSettings {
                volume: Volume::Absolute(VolumeLevel::new(0.05)),
                ..Default::default()
            },
            ..default()
        });
    }
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_handle.0.clone().unwrap(),
            sprite: TextureAtlasSprite::new(1),
            transform: Transform::from_scale(Vec3::splat(2.0)).with_translation(vec3(x, y, 15.0)),
            ..default()
        },
        Bullet(Instant::now()),
        BulletDirection(direction),
        GameEntity,
    ));
}

fn update_bullet(
    time: Res<Time>,
    mut bullets_query: Query<(&mut Transform, &BulletDirection), With<Bullet>>
) {
    for (mut transform, bullet_direction) in bullets_query.iter_mut() {
        transform.translation += Vec3::splat(BULLET_SPEED * time.delta_seconds()) * (bullet_direction.0.normalize());
        transform.translation.z = 15.0;
    }
}

fn despawn_bullets(mut commands: Commands, bullets_query: Query<(Entity, &Bullet), With<Bullet>>) {
    for (entity, bullet) in bullets_query.iter() {
        if bullet.0.elapsed().as_secs_f32() > BULLET_TIME {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_escape_key(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if !keyboard_input.pressed(KeyCode::Escape) {
        return;
    }

    // game_state.
    game_state.set(GameState::PauseMenu);
}

fn handle_player_dead_event(
    mut player_dead_event: EventReader<PlayerDeadEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if player_dead_event.is_empty() {
        return;
    }

    player_dead_event.clear();
    game_state.set(GameState::GameOver);
}

fn camera_follow_player(
    car_query: Query<&Transform, With<Car>>,
    mut cam_query: Query<(&Camera, &mut Transform), Without<Car>>,
) {
    if car_query.is_empty() {
        return;
    }
    if cam_query.is_empty() {
        return;
    }

    let car_transform = car_query.single();
    let (x, y) = (car_transform.translation.x, car_transform.translation.y);
    let (_, mut transform) = cam_query.single_mut();
    transform.translation = transform.translation.lerp(vec3(x, y + 200.0, 0.0), 0.05);
}

fn car_manual_input_system(
    time: Res<Time>,
    road_tiles: Res<RoadTiles>,
    keyboard_input: Res<Input<KeyCode>>,
    mut car_query: Query<(&mut Speed, &mut TurnSpeed, &mut Transform, &Turbo), With<Car>>,
) {
    if car_query.is_empty() {
        return;
    }

    let (mut speed, mut turn_speed, mut transform, turbo) = car_query.single_mut();
    let w_key = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let a_key = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let s_key = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let d_key = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    update_car_input(
        CarControls(w_key, a_key, s_key, d_key),
        &mut turn_speed,
        &mut speed,
        &time,
    );

    let (x, y) = (transform.translation.x, transform.translation.y);
    let (x, y) = (
        x / (TILE_W as f32 * ROAD_SCALE),
        y / (TILE_H as f32 * ROAD_SCALE),
    );
    let (x1, y1) = (x.ceil() as i32, y.ceil() as i32);
    let (x2, y2) = (x.floor() as i32, y.floor() as i32);
    let is_on_road = road_tiles.0.contains(&(x1, y1)) || road_tiles.0.contains(&(x2, y2));

    if turbo.0.elapsed().as_secs_f32() < 0.2 {
        speed.0 += TURBO_BOOST;
    }

    let time_step = 1.0 / 60.0;
    let rotation_factor = turn_speed.0;
    let movement_factor = if is_on_road {
        speed.0 * 0.1
    } else {
        speed.0 * 0.05
    };

    if speed.0.abs() > MIN_SPEED_TO_STEER {
        transform.rotate_z(rotation_factor * 0.1 * time_step);
    }
    let movement_direction = transform.rotation * Vec3::Y;
    let movement_distance = movement_factor;
    let translation_delta = movement_direction * movement_distance;
    transform.translation += translation_delta * time.delta_seconds() * 100.0;
}

fn update_car_input(
    controls: CarControls,
    turn_speed: &mut TurnSpeed,
    speed: &mut Speed,
    time: &Time,
) {
    let w_key = controls.0;
    let a_key = controls.1;
    let s_key = controls.2;
    let d_key = controls.3;

    turn_speed.0 = if a_key {
        TURN_SPEED
    } else if d_key {
        -TURN_SPEED
    } else {
        0.0
    };

    // Friction code from: https://github.com/Rust-Ninja-Sabi/bevyastro
    speed.0 = if s_key {
        if speed.0.abs() <= 10.0 {
            0.0
        } else {
            speed.0 - FRICTION * time.delta_seconds() * 1.2
        }
    } else if w_key {
        speed.0 + CAR_THRUST * time.delta_seconds()
    } else {
        if speed.0.abs() <= 5.0 {
            // Avoid speed from over shooting
            // and be non zero all the time
            0.0
        } else if speed.0 > 0.0 {
            speed.0 - FRICTION * time.delta_seconds()
        } else if speed.0 < 0.0 {
            speed.0 + FRICTION * time.delta_seconds()
        } else {
            0.0
        }
    };

    speed.0 = speed.0.clamp(-MAX_SPEED + MAX_SPEED / 2.0, MAX_SPEED);
}

impl VehicleObstacle {
    fn new(pos: (f32, f32)) -> Self {
        Self {
            pos,
            vehicle_type: VehicleObstacleType::random(),
        }
    }
}

impl VehicleObstacleType {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < 0.7 {
            match rng.gen_range(0..3) {
                0 => VehicleObstacleType::Car1,
                1 => VehicleObstacleType::Car2,
                _ => VehicleObstacleType::Car3,
            }
        } else {
            VehicleObstacleType::Car3
        }
    }

    fn sprite_idx(&self) -> usize {
        match self {
            Self::Car1 => 60,
            Self::Car2 => 61,
            Self::Car3 => 62,
            // Self::Truck => 63,
        }
    }
}

impl GameSettings {
    fn update_difficulty(&mut self) {
        self.difficulty = match self.difficulty {
            Difficulty::Easy => Difficulty::Moderate,
            Difficulty::Moderate => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }

    fn update_entity_count(&mut self) {
        self.entity_count = match self.entity_count {
            EntityCount::Hundred => EntityCount::FiveHundred,
            EntityCount::FiveHundred => EntityCount::Thousand,
            EntityCount::Thousand => EntityCount::FiveThousand,
            EntityCount::FiveThousand => EntityCount::TenThousand,
            EntityCount::TenThousand => EntityCount::TwentyThousand,
            EntityCount::TwentyThousand => EntityCount::FiftyThousand,
            EntityCount::FiftyThousand => EntityCount::Hundred,
        }
    }

    fn get_num_max_zombies(&self) -> usize {
        match &self.entity_count {
            EntityCount::Hundred => 100,
            EntityCount::FiveHundred => 500,
            EntityCount::Thousand => 1000,
            EntityCount::FiveThousand => 5000,
            EntityCount::TenThousand => 10000,
            EntityCount::TwentyThousand => 20000,
            EntityCount::FiftyThousand => 50000,
        }
    }

    fn difficulty_as_str(&self) -> &str {
        match self.difficulty {
            Difficulty::Easy => "Difficulty - Easy",
            Difficulty::Moderate => "Difficulty - Moderate",
            Difficulty::Hard => "Difficulty - Hard",
        }
    }

    fn god_mode_as_str(&self) -> &str {
        if self.god_mode {
            return "God Mode - On";
        }

        "God Mode - Off"
    }

    fn debug_info_as_str(&self) -> &str {
        if self.debug_info {
            return "Debug Info - On";
        }

        "Debug Info - Off"
    }

    fn music_as_str(&self) -> &str {
        if self.music {
            return "Music - On";
        }

        "Music - Off"
    }

    fn entity_count_as_str(&self) -> &str {
        match self.entity_count {
            EntityCount::Hundred => "Entity Count - 100",
            EntityCount::FiveHundred => "Entity Count - 500",
            EntityCount::Thousand => "Entity Count - 1000",
            EntityCount::FiveThousand => "Entity Count - 5000",
            EntityCount::TenThousand => "Entity Count - 10000",
            EntityCount::TwentyThousand => "Entity Count - 20000",
            EntityCount::FiftyThousand => "Entity Count - 50000",
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::default(),
            entity_count: EntityCount::default(),
            music: true,
            god_mode: false,
            debug_info: false,
        }
    }
}
