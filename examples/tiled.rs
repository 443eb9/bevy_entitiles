use avian2d::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_entitiles::{
    prelude::*,
    render::chunk::RenderChunkSort,
    tiled::{
        app_ext::TiledApp,
        events::{TiledMapEvent, TiledMapLoader},
        resources::{PackedTiledTilemap, TiledLoadedMaps},
    },
};
use bevy_entitiles_derive::{TiledClass, TiledCustomTile, TiledEnum, TiledObject};
use helpers::EntiTilesHelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EntiTilesPlugin,
            EntiTilesHelpersPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, switching)
        .insert_resource(TiledLoadConfig {
            ignore_unregisterd_objects: true,
            ignore_unregisterd_custom_tiles: true,
            z_index: 0.,
        })
        .register_tiled_object::<BlockBundle>("BlockBundle")
        .register_tiled_object::<PlainBlockBundle>("PlainBlockBundle")
        .register_tiled_object::<PlayerBundle>("PlayerBundle")
        .register_tiled_object::<DetectAreaBundle>("DetectAreaBundle")
        .register_tiled_object::<PointMarker>("PointMarker")
        .register_tiled_custom_tile::<TileBundle>("TileBundle")
        .register_type::<Block>()
        .register_type::<TileInfos>()
        .insert_resource(RenderChunkSort::XThenY)
        .run();
}

#[derive(Resource, Deref)]
pub struct TiledMaps(HashMap<&'static str, Handle<PackedTiledTilemap>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(TiledMaps(
        [
            (
                "hexagonal",
                asset_server.load("tiled/tilemaps/hexagonal.tmx"),
            ),
            ("infinite", asset_server.load("tiled/tilemaps/infinite.tmx")),
            (
                "orthogonal",
                asset_server.load("tiled/tilemaps/orthogonal.tmx"),
            ),
            (
                "isometric",
                asset_server.load("tiled/tilemaps/isometric.tmx"),
            ),
            (
                "isometricCube",
                asset_server.load("tiled/tilemaps/isometricCube.tmx"),
            ),
        ]
        .into(),
    ));

    commands.spawn(TextBundle::from_section(
        "Press digit 1-5 to switch between maps.",
        Default::default(),
    ));
}

macro_rules! map_switching {
    ($key:ident, $map:expr, $input:expr, $loaded_maps:expr, $event:expr, $tiled_maps: expr) => {
        if $input.just_pressed(KeyCode::$key) {
            $loaded_maps.unload_all(&mut $event);
            $event.send(TiledMapEvent::Load(TiledMapLoader {
                map: $tiled_maps[$map].id(),
                trans_ovrd: None,
            }));
        }
    };
}

fn switching(
    input: Res<ButtonInput<KeyCode>>,
    mut event: EventWriter<TiledMapEvent>,
    loaded_maps: Res<TiledLoadedMaps>,
    tiled_maps: Res<TiledMaps>,
) {
    map_switching!(Digit1, "hexagonal", input, loaded_maps, event, tiled_maps);
    map_switching!(Digit2, "infinite", input, loaded_maps, event, tiled_maps);
    map_switching!(Digit3, "orthogonal", input, loaded_maps, event, tiled_maps);
    map_switching!(Digit4, "isometric", input, loaded_maps, event, tiled_maps);
    #[rustfmt::skip] // Looks awful :/
    map_switching!(Digit5, "isometricCube", input, loaded_maps, event, tiled_maps);
}

/*
 * Here many macro attributes are the same as LDtk's.
 * So if you want to know what they do, you can go to examples/ldtk.rs.
 */

#[derive(TiledObject, Bundle, Default)]
#[spawn_sprite]
pub struct PlainBlockBundle {
    // You have to use `TiledClass`es for objects.
    // Primitive properties are not allowed and will cause panic.
    pub block: PlainBlock,
}

#[derive(TiledClass, Component, Default)]
pub struct PlainBlock;

#[derive(TiledObject, Bundle, Default)]
#[spawn_sprite]
pub struct BlockBundle {
    pub block: Block,
}

#[derive(TiledClass, Component, Reflect, Default)]
pub struct Block {
    #[tiled_name = "Collision"]
    pub collision: bool,
    #[tiled_name = "Hardness"]
    pub hardness: f32,
    #[tiled_name = "Name"]
    pub name: String,
    #[tiled_name = "Tint"]
    pub tint: Color,
    #[tiled_name = "Shape"]
    pub shape: ShapeType,
}

#[derive(TiledEnum, Reflect, Default)]
pub enum ShapeType {
    #[default]
    Square,
    Isometry,
    Hexagon,
    Polygon,
    Eclipse,
}

#[derive(TiledObject, Component)]
// `instantiate_shape` means to spawn the shape as a certain component.
// - For points, this object will be added a `TiledPointObject` component.
// - For others, a collider will be added. (If you've enabled `physics` feature)
#[instantiate_shape]
pub struct PointMarker;

#[derive(TiledObject, Bundle, Default)]
#[spawn_sprite]
#[global_object]
#[instantiate_shape]
pub struct PlayerBundle {
    pub player: Player,
    pub moveable: MoveableObject,
}

#[derive(TiledClass, Component, Reflect, Default)]
pub struct Player {
    #[tiled_name = "Hp"]
    pub hp: f32,
    // This property is not assigned an explicit value in Tiled:
    // it does not even appear in the .tmx file,
    // it will be initialized with a default value
    #[tiled_name = "Level"]
    pub level: i32,
    // This property does not exist at all in Tiled:
    // it does not even appear in the .tmx file,
    // it will be initialized with a default value
    pub mp: i32,
}

#[derive(TiledClass, Component, Default)]
pub struct MoveableObject {
    #[tiled_name = "Speed"]
    pub speed: f32,
}

#[derive(TiledObject, Bundle, Default)]
#[instantiate_shape]
pub struct DetectAreaBundle {
    pub detect_area: DetectArea,
}

#[derive(TiledClass, Component, Default)]
pub struct DetectArea;

#[derive(TiledClass, Component, Reflect, Default)]
pub struct TileInfos {
    #[tiled_name = "AllowLineOfSight"]
    pub allow_line_of_sight: bool,
    #[tiled_name = "DamagePerSecond"]
    pub damage_per_second: i32,
}

#[derive(TiledCustomTile, Bundle, Default)]
pub struct TileBundle {
    pub infos: TileInfos,
    pub detect_area: DetectArea,
}
