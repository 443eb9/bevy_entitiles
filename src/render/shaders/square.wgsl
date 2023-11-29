#define_import_path bevy_entitiles::square

#import bevy_entitiles::common::{VertexInput, tilemap}

fn get_mesh_origin(input: VertexInput) -> vec2<f32> {
    return input.index * tilemap.tile_slot_size;
}
