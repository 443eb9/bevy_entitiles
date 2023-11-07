use std::time::Duration;

use bevy::{
    prelude::{
        default, Color, Commands, Gizmos, Input, IntoSystemConfigs, KeyCode, Plugin, Query, Res,
        Startup, TextBundle, Transform, UVec2, Update, Vec2,
    },
    text::{TextSection, TextStyle},
    time::common_conditions::on_fixed_timer,
};

use crate::{
    math::aabb::AabbBox2d,
    render::{chunk::RenderChunkStorage, extract::ExtractedTilemap},
    tilemap::Tilemap,
};

use self::common::{debug_info_display, DebugFpsText};

pub mod camera_movement;
pub mod common;

/// A bunch of systems for debugging. Since they're not optimized, don't use them unless you're debugging.
pub struct EntiTilesDebugPlugin;

impl Plugin for EntiTilesDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        println!("==============================");
        println!("Debug Enabled");
        println!("==============================");

        app.add_systems(Startup, debug_startup).add_systems(
            Update,
            (
                // draw_tilemap_aabb,
                // draw_chunk_aabb,
                debug_info_display.run_if(on_fixed_timer(Duration::from_millis(100))),
            ),
        );

        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    }
}

pub fn debug_startup(mut commands: Commands) {
    commands.spawn((
        DebugFpsText,
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 32.,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 32.,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ]),
    ));
}

pub fn draw_tilemap_aabb(mut gizmos: Gizmos, tilemaps: Query<(&Tilemap, &Transform)>) {
    for (tilemap, transform) in tilemaps.iter() {
        gizmos.rect_2d(
            tilemap.aabb.center(),
            0.,
            Vec2::new(tilemap.aabb.width(), tilemap.aabb.height()),
            Color::RED,
        )
    }
}

pub fn draw_chunk_aabb(mut gizmos: Gizmos, tilemaps: Query<(&Tilemap, &Transform)>) {
    for (tilemap, tilemap_transform) in tilemaps.iter() {
        let tilemap = ExtractedTilemap {
            id: tilemap.id,
            tile_type: tilemap.tile_type,
            size: tilemap.size,
            tile_size: tilemap.tile_size,
            tile_render_size: tilemap.tile_render_size,
            render_chunk_size: tilemap.render_chunk_size,
            filter_mode: tilemap.filter_mode,
            texture: tilemap.texture.clone(),
            transfrom: *tilemap_transform,
            transform_matrix: tilemap_transform.compute_matrix(),
            flip: tilemap.flip,
            aabb: tilemap.aabb.clone(),
        };
        let count = RenderChunkStorage::calculate_render_chunk_count(
            tilemap.size,
            tilemap.render_chunk_size,
        );

        for y in 0..count.y {
            for x in 0..count.x {
                let aabb = AabbBox2d::from_chunk(UVec2::new(x, y), &tilemap);
                gizmos.rect_2d(
                    aabb.center(),
                    0.,
                    Vec2::new(aabb.width(), aabb.height()),
                    Color::GREEN,
                );
            }
        }
    }
}
