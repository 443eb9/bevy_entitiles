#define_import_path bevy_entitiles::isometric

#import bevy_entitiles::common::{TilemapVertexInput, tilemap}

fn get_mesh_origin(input: TilemapVertexInput) -> vec2<f32> {
    let index = vec2<f32>(input.index.xy) * tilemap.axis_dir;
    let flipped = (1. - tilemap.axis_dir) / 4.;
    
    return vec2<f32>(
        (index.x - index.y),
        (index.x + index.y)
    ) / 2. * tilemap.slot_size
    - (flipped.x + flipped.y) * vec2<f32>(0., tilemap.slot_size.y);
}
