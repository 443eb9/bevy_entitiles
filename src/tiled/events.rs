use bevy::{asset::AssetId, math::Vec2, prelude::Event, reflect::Reflect};

use crate::tiled::resources::PackedTiledTilemap;

#[derive(Event, Clone)]
pub enum TiledMapEvent {
    Load(TiledMapLoader),
    Unload(TiledMapUnloader),
}

#[derive(Reflect, Clone)]
pub struct TiledMapLoader {
    pub map: AssetId<PackedTiledTilemap>,
    /// Override the original tilemap translation or not.
    pub trans_ovrd: Option<Vec2>,
}

#[derive(Reflect, Clone)]
pub struct TiledMapUnloader {
    pub map: AssetId<PackedTiledTilemap>,
}
