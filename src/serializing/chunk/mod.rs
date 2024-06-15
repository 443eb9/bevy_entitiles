use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        entity::Entity,
        query::With,
        system::{ParallelCommands, Query, Res},
    },
};

use crate::serializing::chunk::{
    load::{ChunkLoadCache, ChunkLoadConfig, ScheduledLoadChunks},
    save::{ChunkSaveCache, ChunkSaveConfig, ScheduledSaveChunks},
};

pub mod load;
pub mod save;

pub const TILE_CHUNKS_FOLDER: &str = "tile_chunks";
pub const PATH_TILE_CHUNKS_FOLDER: &str = "path_tile_chunks";
pub const PHYSICS_TILE_CHUNKS_FOLDER: &str = "physics_tile_chunks";

pub struct EntiTilesChunkSerializingPlugin;

impl Plugin for EntiTilesChunkSerializingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                save::save_color_layer,
                #[cfg(feature = "algorithm")]
                save::save_path_layer,
                #[cfg(feature = "physics")]
                save::save_physics_layer,
                save::render_chunk_remover,
                load::load_color_layer,
                #[cfg(feature = "algorithm")]
                load::load_path_layer,
                #[cfg(feature = "physics")]
                load::load_physics_layer,
                chunk_tag_remover,
            ),
        );

        app.register_type::<ChunkSaveConfig>()
            .register_type::<ChunkLoadConfig>();

        app.init_resource::<ChunkLoadCache>()
            .init_resource::<ChunkLoadConfig>()
            .init_resource::<ChunkSaveCache>()
            .init_resource::<ChunkSaveConfig>();
    }
}

fn chunk_tag_remover(
    commands: ParallelCommands,
    saves_query: Query<Entity, With<ScheduledSaveChunks>>,
    save_cache: Res<ChunkSaveCache>,
    loads_query: Query<Entity, With<ScheduledLoadChunks>>,
    load_cache: Res<ChunkLoadCache>,
) {
    saves_query.par_iter().for_each(|entity| {
        if save_cache
            .0
            .get(&entity)
            .is_some_and(|chunks| chunks.is_empty())
        {
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<ScheduledSaveChunks>();
            });
        }
    });

    loads_query.par_iter().for_each(|entity| {
        if load_cache
            .0
            .get(&entity)
            .is_some_and(|chunks| chunks.is_empty())
        {
            commands.command_scope(|mut c| {
                c.entity(entity).remove::<ScheduledLoadChunks>();
            });
        }
    });
}
