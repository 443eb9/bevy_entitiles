use bevy::prelude::Vec2;

#[inline]
pub fn world_pos_to_chunk_square(
    tilemap_pos: Vec2,
    tilemap_render_chunk_size: u32,
    tilemap_tile_render_size: Vec2,
    world_pos: Vec2,
) -> Vec2 {
    let rel_pos = world_pos - tilemap_pos;
    let crs = tilemap_tile_render_size * tilemap_render_chunk_size as f32;
    rel_pos.div_euclid(crs)
}

#[inline]
pub fn world_pos_to_chunk_iso_diamond(
    tilemap_pos: Vec2,
    tilemap_render_chunk_size: u32,
    tilemap_tile_render_size: Vec2,
    world_pos: Vec2,
) -> Vec2 {
    let rel_pos = world_pos - tilemap_pos;
    let crs = tilemap_tile_render_size * tilemap_render_chunk_size as f32;
    let h = crs.x * crs.y;

    Vec2::new(
        (rel_pos.x * crs.y + rel_pos.y * crs.x - h / 2.) / h,
        (rel_pos.y * crs.x - h / 2. - rel_pos.x * crs.y) / h,
    )
}
