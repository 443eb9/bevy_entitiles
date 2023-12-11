#import bevy_entitiles::common::{VertexInput, VertexOutput, tilemap}
#import bevy_sprite::mesh2d_view_bindings::view

#ifdef SQUARE
    #import bevy_entitiles::square::get_mesh_origin
#endif

#ifdef ISO_DIAMOND
    #import bevy_entitiles::iso_diamond::get_mesh_origin
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mesh_center = get_mesh_origin(input);

#ifdef NON_UNIFORM
    var uv_set = array<vec4<f32>, 1>(
        tilemap.atlas_uvs[input.atlas_indices[0]],
    );
#else ifdef PURE_COLOR
    var uv_set = array<vec4<f32>, 4>(
        vec4<f32>(0., 0., tilemap.tile_render_size),
        vec4<f32>(0., 0., tilemap.tile_render_size),
        vec4<f32>(0., 0., tilemap.tile_render_size),
        vec4<f32>(0., 0., tilemap.tile_render_size),
    );
#else
    var uv_set = array<vec4<f32>, 4>(
        vec4<f32>(0., 0., 0., 0.),
        vec4<f32>(0., 0., 0., 0.),
        vec4<f32>(0., 0., 0., 0.),
        vec4<f32>(0., 0., 0., 0.),
    );
    for (var i = 0u; i < 4u; i++) {
        if input.atlas_indices[i] >= 0 {
            uv_set[i] = tilemap.atlas_uvs[input.atlas_indices[i]];
        }
    }
#endif
    let tile_render_size = (uv_set[0].zw - uv_set[0].xy) * tilemap.tile_render_scale;

    var translations = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., tile_render_size.y),
        vec2<f32>(tile_render_size.x, tile_render_size.y),
        vec2<f32>(tile_render_size.x, 0.),
    );

    let translation = translations[input.v_index % 4u] - tilemap.pivot * tile_render_size;
    var position_model = vec2<f32>(mesh_center + translation);
    var position_world = vec4<f32>(tilemap.translation + position_model, 0., 1.);

    output.position = view.view_proj * position_world;
    output.color = pow(input.color, vec4<f32>(2.2));

#ifndef PURE_COLOR
#ifdef NON_UNIFORM
    for (var i = 0u; i < 1u; i++) {
#else // NON_UNIFORM
    for (var i = 0u; i < 4u; i++) {
#endif // NON_UNIFORM
        let uvs = uv_set[i];
        var corner_uvs = array<vec2<f32>, 4>(
            vec2<f32>(uvs.x, uvs.w),
            vec2<f32>(uvs.x, uvs.y),
            vec2<f32>(uvs.z, uvs.y),
            vec2<f32>(uvs.z, uvs.w),
        );
        let uv = corner_uvs[input.v_index % 4u] / tilemap.texture_size;
        if i == 0u {
            output.uv_a = uv;
        } else if i == 1u {
            output.uv_b = uv;
        } else if i == 2u {
            output.uv_c = uv;
        } else if i == 3u {
            output.uv_d = uv;
        }
    }
#endif // PURE_COLOR

    return output;
}

fn blend(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return mix(a, b, b.a);
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
#ifdef PURE_COLOR
    return input.color;
#else ifdef NON_UNIFORM
    return textureSample(bevy_entitiles::common::color_texture,
                         bevy_entitiles::common::color_texture_sampler,
                         input.uv_a) * input.color;
#else
    var color = vec4<f32>(0., 0., 0., 0.);

    for (var i = 0u; i < 4u; i++) {
        var uv = vec2<f32>(0., 0.);
        if i == 0u {
            uv = input.uv_a;
        } else if i == 1u {
            uv = input.uv_b;
        } else if i == 2u {
            uv = input.uv_c;
        } else if i == 3u {
            uv = input.uv_d;
        }

        if uv.x + uv.y < 0.001 {
            continue;
        }

        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      uv);
        color = mix(color, tex_color, tex_color.a);
    }
    return color * input.color;
#endif
}
