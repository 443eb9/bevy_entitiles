#import bevy_entitiles::common::{
    TilemapVertexInput, TilemapVertexOutput, tilemap, view, atlas_uvs,
    anim_seqs, material, texture_descs
}

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

    output.position = view.clip_from_world * position_world;
    output.tint = input.tint;

#ifndef PURE_COLOR
#ifdef ATLAS
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
#else // ATLAS
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
#endif // ATLAS
    output.uv = uvs[(input.v_index) % 4u];
    output.anim_flag = input.index.z;

    if input.index.z != -1 {
        // Means that this tile is a animated tile
        let start = input.index.z;
        let length = input.index.w;
        // The number before the start index is the fps.
        // See `register` function in TilemapAnimations.
#ifdef WASM
        let fps = f32(anim_seqs[start - 1][0]);
        var frame = i32(tilemap.time * fps) % length;
#ifdef ATLAS
        output.texture_indices[0] = anim_seqs[start + frame * 2][0];
        output.atlas_indices[0] = anim_seqs[start + frame * 2 + 1][0];
#else // ATLAS
        output.atlas_indices[0] = anim_seqs[start + frame][0];
#endif // ATLAS

#else // WASM
        let fps = f32(anim_seqs[start - 1]);
        var frame = i32(tilemap.time * fps) % length;

#ifdef ATLAS
        output.texture_indices[0] = anim_seqs[start + frame * 2];
        output.atlas_indices[0] = anim_seqs[start + frame * 2 + 1];
#else // ATLAS
        output.atlas_indices[0] = anim_seqs[start + frame];
#endif // ATLAS
#endif // WASM
    } else {
        output.atlas_indices = input.atlas_indices;
#ifdef ATLAS
        output.texture_indices = input.texture_indices;
#endif // ATLAS
    }
#endif // PURE_COLOR

    return output;
}

@fragment
fn tilemap_fragment(input: TilemapVertexOutput) -> @location(0) vec4<f32> {
#ifdef PURE_COLOR
    return input.tint;
#else // PURE_COLOR
    var color = vec4<f32>(0., 0., 0., 0.);

    // Sample the 4 layers.
    for (var i = 0u; i < 4u; i++) {
        if input.atlas_indices[i] < 0 {
            // No texture for this layer.
            continue;
        }
        let atlas_index = u32(input.atlas_indices[i] & 0x1FFFFFFF);
        // Shift 29 bits but not 30 because it's a signed integer,
        // and we need to identify if the layer is empty or not according to the sign.
        let flip = input.atlas_indices[i] >> 29;

#ifdef ATLAS
        if input.texture_indices[i] < 0 {
            // No texture for this layer.
            continue;
        }
#endif // ATLAS

        var uv = input.uv;
        // Flip the uv if needed.
        if (flip & 2) != 0 {
            uv.x = 1. - uv.x;
        }
        if (flip & 1) != 0 {
            uv.y = 1. - uv.y;
        }

#ifdef ATLAS
        let texture_index = u32(input.texture_indices[i]);
        let desc = &texture_descs[texture_index];

        // If `atlas` feature is enabled, we need to calculate the uv.
        let tile_index = vec2<f32>(f32(atlas_index % (*desc).tile_count.x),
                                   f32(atlas_index / (*desc).tile_count.x));
        let atlas_uv = (tile_index + uv) * (*desc).tile_uv_size * (*desc).uv_scale;
        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      atlas_uv, texture_index);
#else // ATLAS
        // Otherwise, sample the texture at the right layer using the uv directly.
        let tex_color = textureSample(bevy_entitiles::common::color_texture,
                                      bevy_entitiles::common::color_texture_sampler,
                                      uv, atlas_index);
#endif // ATLAS
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
#endif // PURE_COLOR
}
