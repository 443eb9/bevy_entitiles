#define_import_path bevy_entitiles::square

#import bevy_entitiles::common::{VertexInput, tilemap}

fn get_mesh_origin(input: VertexInput) -> vec2<f32> {
    let index = vec2<f32>(input.index.xy) * tilemap.ext_dir;
    return index.xy * tilemap.tile_slot_size;
}
