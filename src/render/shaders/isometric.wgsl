#define_import_path bevy_entitiles::isometric

#import bevy_entitiles::common::{VertexInput, tilemap}

fn get_mesh_origin(input: VertexInput) -> vec2<f32> {
    let index = vec2<f32>(input.index.xy) * tilemap.ext_dir;
    return vec2<f32>(
        (index.x - index.y - 1.) / 2. * tilemap.tile_slot_size.x,
        (index.x + index.y) / 2. * tilemap.tile_slot_size.y
    );
}
