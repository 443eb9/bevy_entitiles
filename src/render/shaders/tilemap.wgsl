#import bevy_entitiles::common::{VertexInput, VertexOutput, color_texture, color_texture_sampler, tilemap}
#import bevy_sprite::mesh2d_view_bindings::view

#ifdef SQUARE
    #import bevy_entitiles::square::get_mesh_center
#endif

#ifdef ISO_DIAMOND
    #import bevy_entitiles::iso_diamond::get_mesh_center
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mesh_center = get_mesh_center(input);

    var translations = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );

    var position_model = vec2<f32>(mesh_center + translations[input.v_index % 4u] * tilemap.tile_render_size);
    var position_world = vec4<f32>(tilemap.translation + position_model, 0., 1.);

    output.position = view.view_proj * position_world;
    output.color = input.color;

    output.uv = input.uv;
#ifdef FLIP_H
    output.uv.x = 1. - output.uv.x;
#endif
#ifdef FLIP_V
    output.uv.y = 1. - output.uv.y;
#endif
    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
#ifdef PURE_COLOR
    let color = input.color;
#else
    let color = textureSample(color_texture, color_texture_sampler, input.uv);
#endif

#ifdef POST_PROCESSING
    bevy_entitiles::height_texture::sample_height(input, view.viewport.zw);
#endif

    return color * input.color;
}
