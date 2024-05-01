#import bevy_entitiles::common::{
    TilemapVertexInput, TilemapVertexOutput, tilemap, atlas_uvs, anim_seqs, material
}
#import bevy_sprite::mesh2d_view_bindings::view

// Here the three different imports are for the three different tilemap types.
// They calculates the tile_pivot for each tile.
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
fn tilemap_vertex(input: TilemapVertexInput) -> TilemapVertexOutput {
    var output: TilemapVertexOutput;
    var mesh_origin = get_mesh_origin(input);
    
    var translations = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );

    var position_model = (translations[input.v_index % 4u] - tilemap.pivot)
                          * tilemap.tile_render_size + mesh_origin;
    var position_world = vec4<f32>((tilemap.rot_mat * position_model) + tilemap.translation, 0., 1.);

    output.position = view.view_proj * position_world;
    output.tint = input.tint;

#ifndef PURE_COLOR
#ifdef ATLAS
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
#else
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
#endif
    output.uv = uvs[(input.v_index) % 4u];
    output.flip = input.flip;
    output.anim_flag = input.index.z;

    if input.index.z != -1 {
        // Means that this tile is a animated tile
        let start = input.index.z;
        let length = input.index.w;
        // The number before the start index is the fps.
        // See register function in TilemapAnimations.
        let fps = f32(anim_seqs[start - 1]);
        var frame = i32(tilemap.time * fps) % length;
        output.texture_indices[0] = anim_seqs[start + frame];
    } else {
        output.texture_indices = input.texture_indices;
    }
#endif

    return output;
}

@fragment
fn tilemap_fragment(input: TilemapVertexOutput) -> @location(0) vec4<f32> {
#ifdef PURE_COLOR
    return input.tint;
#else
    var color = vec4<f32>(0., 0., 0., 0.);

    // Sample the 4 layers.
    for (var i = 0u; i < 4u; i++) {
        if input.texture_indices[i] < 0 {
            // No texture for this layer.
            continue;
        }

        var uv = input.uv;
        // Flip the uv if needed.
        if (input.flip[i] & 1u) != 0u {
            uv.x = 1. - uv.x;
        }
        if (input.flip[i] & 2u) != 0u {
            uv.y = 1. - uv.y;
        }
#ifdef ATLAS
        // If `atlas` feature is enabled, we need to calculate the uv.
        let tile_index = vec2<f32>(f32(input.texture_indices[i] % tilemap.texture_tiled_size.x),
                                   f32(input.texture_indices[i] / tilemap.texture_tiled_size.x));
        let atlas_uv = (tile_index + uv) * tilemap.tile_uv_size;
        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      atlas_uv);
#else
        // Otherwise, sample the texture at the right layer using the uv directly.
        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      uv, input.texture_indices[i]);
#endif
        // Mix the color of each layer.
        color = mix(color, tex_color, tex_color.a * tilemap.layer_opacities[i]);

        if input.anim_flag != -1 {
            // Indicates that this tile is a animated tile.
            // We only need to sample the first layer as animated tiles are always single layered.
            break;
        }
    }
    // Apply the tint of the tile and the tilemap.
    return color * input.tint * material.color;
#endif
}
