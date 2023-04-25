use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_mod_picking::{
    DebugCursorPickingPlugin,
    DebugEventsPickingPlugin,
    DefaultPickingPlugins,
    PickableBundle,
    PickingCameraBundle,
    PickingEvent,
};

#[derive(Component)]
pub struct PickableGltf;

fn set_pickible_recursive(
    commands: &mut Commands,
    entity: &Entity,
    mesh_query: &Query<(Entity, &Parent), With<Handle<Mesh>>>,
    children_query: &Query<&Children>,
) {
    for (mesh_entity, mesh_parent) in mesh_query.iter(){
        if mesh_parent.get() == *entity {
            commands.entity(mesh_entity).insert(PickableBundle::default());
        }
    }

    if let Ok(children) = children_query.get(*entity) {
        for child in children.iter() {
            set_pickible_recursive(commands, child, mesh_query, children_query);
        }
    }
}

pub fn make_gltf_scene_pickable(
    mut commands: Commands,
    mut unpickable_query: Query<(Entity, &Name, &Children), With<PickableGltf>>,
    mesh_query: Query<(Entity, &Parent), With<Handle<Mesh>>>,
    children_query: Query<&Children>
){
    for (entity, name, _children) in unpickable_query.iter_mut(){
        println!("[MODELS] Setting Pickable on {name}");
        set_pickible_recursive(&mut commands, &entity, &mesh_query, &children_query);
        commands.entity(entity).remove::<PickableGltf>();
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(DebugEventsPickingPlugin)
        .add_state::<GameState>()
        .add_systems((
            setup_cameras.on_startup(),
            setup.in_schedule(OnEnter(GameState::Playing)),
        ))
        .add_system(bevy::window::close_on_esc)
        // TODO: investigate if this can be a startup system
        .add_system(make_gltf_scene_pickable)
        .add_system(print_events)
        .run();
}

#[derive(Default)]
struct Board {
    tiles: [Option<Piece>; BOARD_SIZE_I * BOARD_SIZE_J],
}

#[derive(Debug)]
struct Piece {
    entity: Entity,
}

#[derive(Resource, Default)]
struct Game {
    board: Board,
}

const NUM_PIECES: u8 = 10;

// I is horizontal, J is vertical
const BOARD_ORIGIN_I: f32 = -1.72;
const BOARD_ORIGIN_J: f32 = 0.13;
const BOARD_HEIGHT: f32 = 1.45;

const BOARD_SLOT_FACTOR: f32 = 0.2;

const BOARD_SIZE_I: usize = 10;
const BOARD_SIZE_J: usize = 3;

const PIECE_1_SCALE: f32 = 0.09;

const RESET_FOCUS: [f32; 3] = [
    -0.95,
    0.8,
    -0.5,
];

fn print_events(mut events: EventReader<PickingEvent>, game: Res<Game>) {
    for event in events.iter() {
        if let PickingEvent::Clicked(e) = event {
            info!("A click event happened: {:?}", e);
            for tile in &game.board.tiles {
                if tile.is_some() {
                    let piece = tile.as_ref().unwrap();
                    println!("1: {:#?}, 2: {:#?}", piece.entity, *e);
                    if piece.entity == *e {
                        println!("{:#?}", piece);
                    }
                }
            }
        }
    }
}

fn setup_cameras(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(
                -0.8,
                2.5,
                1.5,
            )
            .looking_at(Vec3::from(RESET_FOCUS), Vec3::Y),
            ..default()
        },
        PickingCameraBundle::default(),
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    let piece_1_scene = asset_server.load("piece.glb#Scene0");

    for i in 0..NUM_PIECES as usize {
        let scene = piece_1_scene.clone();
        let scale = Vec3::splat(PIECE_1_SCALE);

        let piece_i = BOARD_ORIGIN_I + i as f32 * BOARD_SLOT_FACTOR;
        let piece_j = BOARD_ORIGIN_J + 0 as f32 * BOARD_SLOT_FACTOR;

        game.board.tiles[i] = Some(Piece {
            entity: commands
                .spawn(SceneBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            piece_i,
                            BOARD_HEIGHT,
                            piece_j,
                        ),
                        rotation: Quat::from_rotation_y(-PI / 2.),
                        scale,
                    },
                    scene,
                    ..default()
                })
                .insert(Name::new(format!("Piece {}", i)))
                .insert(PickableGltf)
                .id(),
        })
    }

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        point_light: PointLight {
            intensity: 3500.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        ..default()
    });
}
