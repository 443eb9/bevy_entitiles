#define_import_path bevy_entitiles::iso_diamond

#import bevy_entitiles::common::{VertexInput, tilemap}

fn get_mesh_origin(input: VertexInput) -> vec2<f32> {
    return vec2<f32>(
        (input.index.x - input.index.y - 1.) / 2. * tilemap.tile_slot_size.x,
        (input.index.x + input.index.y) / 2. * tilemap.tile_slot_size.y
    );
}
