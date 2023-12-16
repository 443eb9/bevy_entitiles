#import bevy_entitiles::common::{
    VertexInput, VertexOutput, tilemap, atlas_uvs, anim_seqs
}
#import bevy_sprite::mesh2d_view_bindings::view

#ifdef SQUARE
    #import bevy_entitiles::square::get_mesh_origin
#endif

#ifdef ISOMETRIC
    #import bevy_entitiles::isometric::get_mesh_origin
#endif

#ifdef HEXAGONAL
    #import bevy_entitiles::hexagonal::get_mesh_origin
#endif

@vertex
fn tilemap_vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mesh_origin = get_mesh_origin(input);
    
    var translations = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );
    
    var position_model = (translations[input.v_index % 4u] - tilemap.pivot)
                          * tilemap.tile_render_size + mesh_origin;
    var position_world = vec4<f32>(tilemap.translation + position_model, 0., 1.);

    output.position = view.view_proj * position_world;
    output.color = pow(input.color, vec4<f32>(2.2));

#ifndef PURE_COLOR
    output.is_animated = input.index.z;
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
    output.uv = uvs[input.v_index % 4u];
    output.flip = input.flip;

    if input.index.z == 1u {
        // means that this tile is a animated tile
        var animation = tilemap.anim_seqs[input.texture_indices.x];
        var frame = u32(tilemap.time * animation.fps) % animation.length;
        var texture_index = animation.seq[frame / 4u][frame % 4u];

        output.texture_indices[0] = i32(texture_index);
    } else {
        output.texture_indices = input.texture_indices;
    }
#endif

    return output;
}

@fragment
fn tilemap_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
#ifdef PURE_COLOR
    return input.color;
#else
    var color = vec4<f32>(0., 0., 0., 0.);

    for (var i = 0u; i < 4u; i++) {
        if input.texture_indices[i] < 0 {
            continue;
        }

        var uv = input.uv;
        if (input.flip[i] & 1u) != 0u {
            uv.x = 1. - uv.x;
        }
        if (input.flip[i] & 2u) != 0u {
            uv.y = 1. - uv.y;
        }
        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      uv, input.texture_indices[i]);
        color = mix(color, tex_color, tex_color.a * pow(tilemap.layer_opacities[i], 1. / 2.2));

        if input.is_animated == 1u {
            break;
        }
    }
    return color * input.color;
    // return vec4<f32>(input.uv_a, 0., 1.);
#endif
}
