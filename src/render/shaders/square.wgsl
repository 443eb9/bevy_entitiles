#define_import_path bevy_entitiles::square

#import bevy_entitiles::common::{TilemapVertexInput, tilemap}

fn get_mesh_origin(input: TilemapVertexInput) -> vec2<f32> {
    let index = vec2<f32>(input.index.xy) * tilemap.axis_dir;
    
    return index.xy * tilemap.slot_size
           - (1. - tilemap.axis_dir) / 2. * tilemap.slot_size;
}
