use serde::{Deserialize, Serialize};

use super::{
    definitions::Definitions,
    level::{EntityRef, Level},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Nullable<T> {
    Data(T),
    Null,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LdtkJson {
    pub bg_color: String,
    pub defs: Definitions,
    pub external_levels: bool,
    pub iid: String,
    pub json_version: String,
    pub levels: Vec<Level>,
    pub toc: Vec<Toc>,
    pub world_grid_height: Nullable<i32>,
    pub world_grid_width: Nullable<i32>,
    pub world_layout: Nullable<WorldLayout>,
    pub worlds: Vec<World>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Toc {
    pub identifier: String,
    pub instances: Vec<EntityRef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WorldLayout {
    Free,
    GridVanilla,
    LinearHorizontal,
    LinearVertical,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub world_grid_width: i32,
    pub iid: String,
    pub world_grid_height: i32,
    pub world_layout: Nullable<WorldLayout>,
    pub default_level_width: i32,
    pub levels: Vec<Level>,
    pub default_level_height: i32,
    pub identifier: String,
}
