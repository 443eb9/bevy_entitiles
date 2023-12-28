use bevy::{
    ecs::system::EntityCommands, sprite::MaterialMesh2dBundle, transform::components::Transform, reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::ldtk::resources::LdtkLevelManager;

use super::{
    definitions::{LayerType, TilesetRect},
    field::FieldInstance,
    LdtkColor,
};

/*
 * Level
 */

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    /// Background color of the level (same as `bgColor`, except
    /// the default value is automatically used here if its value is `null`)
    #[serde(rename = "__bgColor")]
    pub bg_color: LdtkColor,

    /// Position informations of the background image, if there is one.
    #[serde(rename = "__bgPos")]
    pub bg_pos: Option<ImagePosition>,

    /// An array listing all other levels touching this one on the world map.
    /// Since 1.4.0, this includes levels that overlap in the same world layer,
    /// or in nearby world layers.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free). For Horizontal and Vertical layouts,
    /// this array is always empty.
    #[serde(rename = "__neighbours")]
    pub neighbours: Vec<Neighbour>,

    /// The optional relative path to the level background image.
    pub bg_rel_path: Option<String>,

    /// This value is not null if the project option
    /// "Save levels separately" is enabled. In this case,
    /// this relative path points to the level Json file.
    pub external_rel_path: Option<String>,

    /// An array containing this level custom field values.
    pub field_instances: Vec<FieldInstance>,

    /// User defined unique identifier
    pub identifier: String,

    /// Unique instance identifier
    pub iid: String,

    /// An array containing all Layer instances.
    /// ## IMPORTANT:
    /// if the project option "Save levels separately" is enabled,
    /// this field will be null.
    ///
    /// This array is **sorted in display order**: the 1st layer is
    /// the top-most and the last is behind.
    pub layer_instances: Vec<LayerInstance>,

    /// Height of the level in pixels
    pub px_hei: i32,

    /// Width of the level in pixels
    pub px_wid: i32,

    /// Unique Int identifier
    pub uid: i32,

    /// Index that represents the "depth" of the level in the world.
    /// Default is 0, greater means "above", lower means "below".
    ///
    /// This value is mostly used for display only and is intended to
    /// make stacking of levels easier to manage.
    pub world_depth: i32,

    /// World X coordinate in pixels.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free).
    /// For Horizontal and Vertical layouts, the value is always -1 here.
    pub world_x: i32,

    /// World Y coordinate in pixels.
    ///
    /// Only relevant for world layouts where level spatial positioning is manual
    /// (ie. GridVania, Free).
    /// For Horizontal and Vertical layouts, the value is always -1 here.
    pub world_y: i32,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct ImagePosition {
    /// An array of 4 float values describing the cropped sub-rectangle
    /// of the displayed background image. This cropping happens when
    /// original is larger than the level bounds
    ///
    /// Array format: `[ cropX, cropY, cropWidth, cropHeight ]`
    pub crop_rect: [f32; 4],

    /// An array containing the `[scaleX,scaleY]` values of the cropped
    /// background image, depending on `bgPos` option.
    pub scale: [f32; 2],

    /// An array containing the `[x,y]` pixel coordinates of the top-left
    /// corner of the cropped background image, depending on `bgPos` option.
    pub top_left_px: [i32; 2],
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct Neighbour {
    /// A single lowercase character tipping on the level location
    /// (`n`orth, `s`outh, `w`est, `e`ast).
    ///
    /// Since 1.4.0, this character value can also be
    /// `<` (neighbour depth is lower),
    /// `>` (neighbour depth is greater)
    /// or `o` (levels overlap and share the same world depth).
    pub dir: String,

    /// Neighbour Instance Identifier
    pub level_iid: String,
}

/*
 * Layer Instance
 */

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LayerInstance {
    /// Grid-based height
    #[serde(rename = "__cHei")]
    pub c_hei: i32,

    /// Grid-based width
    #[serde(rename = "__cWid")]
    pub c_wid: i32,

    /// Grid size
    #[serde(rename = "__gridSize")]
    pub grid_size: i32,

    /// Layer definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Layer opacity as Float [0-1]
    #[serde(rename = "__opacity")]
    pub opacity: f32,

    ///	Total layer X pixel offset, including both instance and definition offsets.
    #[serde(rename = "__pxTotalOffsetX")]
    pub px_total_offset_x: i32,

    /// Total layer Y pixel offset, including both instance and definition offsets.
    #[serde(rename = "__pxTotalOffsetY")]
    pub px_total_offset_y: i32,

    /// The definition UID of corresponding Tileset, if any.
    #[serde(rename = "__tilesetDefUid")]
    pub tileset_def_uid: Option<i32>,

    /// The relative path to corresponding Tileset, if any.
    #[serde(rename = "__tilesetRelPath")]
    pub tileset_rel_path: Option<String>,

    /// Layer type (possible values: IntGrid, Entities, Tiles or AutoLayer)
    #[serde(rename = "__type")]
    pub ty: LayerType,

    /// An array containing all tiles generated by Auto-layer rules.
    /// The array is already sorted in display order
    /// (ie. 1st tile is beneath 2nd, which is beneath 3rd etc.).
    ///
    /// Note: if multiple tiles are stacked in the same cell as the result of different rules,
    /// all tiles behind opaque ones will be discarded.
    pub auto_layer_tiles: Vec<TileInstance>,
    pub entity_instances: Vec<EntityInstance>,
    pub grid_tiles: Vec<TileInstance>,

    /// Unique layer instance identifier
    pub iid: String,

    /// A list of all values in the IntGrid layer, stored in CSV format (Comma Separated Values).
    ///
    /// Order is from left to right, and top to bottom (ie. first row from left to right, followed by second row, etc).
    ///
    /// `0` means "empty cell" and IntGrid values start at 1.
    ///
    /// The array size is `__cWid` x `__cHei` cells.
    pub int_grid_csv: Vec<i32>,

    /// Reference the Layer definition UID
    pub layer_def_uid: i32,

    /// Reference to the UID of the level containing this layer instance
    pub level_id: i32,

    /// This layer can use another tileset by overriding the tileset UID here.
    pub override_tileset_uid: Option<i32>,

    /// X offset in pixels to render this layer, usually 0
    /// ## IMPORTANT:
    /// this should be added to the LayerDef optional offset,
    /// so you should probably prefer using `__pxTotalOffsetX`
    /// which contains the total offset value)
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct TileInstance {
    ///	Alpha/opacity of the tile (0-1, defaults to 1)
    #[serde(rename = "a")]
    pub alpha: f32,

    /// "Flip bits", a 2-bits integer to represent the mirror transformations of the tile.
    /// - Bit 0 = X flip
    /// - Bit 1 = Y flip
    ///
    /// Examples: f=0 (no flip), f=1 (X flip only), f=2 (Y flip only), f=3 (both flips)
    ///
    /// (This is the same as the `TileFlip`)
    #[serde(rename = "f")]
    pub flip: i32,

    /// Pixel coordinates of the tile in the layer (`[x,y]` format).
    /// Don't forget optional layer offsets, if they exist!
    pub px: [i32; 2],

    /// Pixel coordinates of the tile in the tileset ([x,y] format)
    pub src: [i32; 2],

    /// The Tile ID in the corresponding tileset.
    #[serde(rename = "t")]
    pub tile_id: i32,
}

/*
 * Entity Instance
 */

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct EntityInstance {
    /// Grid-based coordinates (`[x,y]` format)
    #[serde(rename = "__grid")]
    pub grid: [i32; 2],

    /// Entity definition identifier
    #[serde(rename = "__identifier")]
    pub identifier: String,

    /// Pivot coordinates (`[x,y]` format, values are from 0 to 1) of the Entity
    #[serde(rename = "__pivot")]
    pub pivot: [f32; 2],

    /// The entity "smart" color, guessed from either Entity definition,
    /// or one its field instances.
    #[serde(rename = "__smartColor")]
    pub smart_color: LdtkColor,

    /// Array of tags defined in this Entity definition.
    #[serde(rename = "__tags")]
    pub tags: Vec<String>,

    /// Optional TilesetRect used to display this entity
    /// (it could either be the default Entity tile,
    /// or some tile provided by a field value, like an Enum).
    #[serde(rename = "__tile")]
    pub tile: Option<TilesetRect>,

    /// X world coordinate in pixels
    #[serde(rename = "__worldX")]
    pub world_x: i32,

    /// Y world coordinate in pixels
    #[serde(rename = "__worldY")]
    pub world_y: i32,

    /// Reference of the Entity definition UID
    pub def_uid: i32,

    /// An array of all custom fields and their values.
    pub field_instances: Vec<FieldInstance>,

    /// Unique instance identifier
    pub iid: String,

    /// Pixel coordinates (`[x,y]` format) in current level coordinate space.
    /// Don't forget optional layer offsets, if they exist!
    #[serde(rename = "px")]
    pub local_pos: [i32; 2],

    /// Entity width in pixels.
    /// For non-resizable entities, it will be the same as Entity definition.
    pub width: i32,

    /// Entity height in pixels.
    /// For non-resizable entities, it will be the same as Entity definition.
    pub height: i32,
}

impl EntityInstance {
    pub fn generate_sprite(&self, commands: &mut EntityCommands, ldtk_manager: &LdtkLevelManager) {
        if self.tile.is_none() {
            return;
        }

        let ldtk_assets = ldtk_manager.get_ldtk_assets();
        commands.insert(MaterialMesh2dBundle {
            mesh: ldtk_assets.clone_mesh_handle(&self.iid),
            material: ldtk_assets.clone_material_handle(&self.iid),
            transform: Transform::from_xyz(self.world_x as f32, -self.world_y as f32, 1.),
            ..Default::default()
        });
    }
}
