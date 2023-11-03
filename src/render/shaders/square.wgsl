#define_import_path bevy_entitiles::square

#import bevy_entitiles::common VertexInput, tilemap

fn get_mesh_center(input: VertexInput) -> vec2<f32> {
    return input.grid_index * tilemap.tile_render_size;
}
