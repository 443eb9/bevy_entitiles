#define_import_path bevy_entitiles::hexagonal

#import bevy_entitiles::common::{VertexInput, tilemap}

fn get_mesh_origin(input: VertexInput) -> vec2<f32> {
    /*
     * ANOTHER MATHEMATICAL MAGIC!!!!!!!
     */
    let index = vec2<f32>(input.index.xy);

    return vec2<f32>(
        tilemap.slot_size.x * (index.x - 0.5 * index.y),
        (tilemap.slot_size.y + tilemap.hex_legs) / 2. * index.y,
    );
}
