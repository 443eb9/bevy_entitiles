#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_entitiles::value_noise::{fbm_3d, fbm_2d}

struct PostProcessingUniforms {
    time: f32,
    camera_pos: vec2<f32>,
    camera_scale: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: PostProcessingUniforms;

@group(1) @binding(0)
var screen_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var screen_color_texture_sampler: sampler;

@group(2) @binding(0)
var screen_height_texture: texture_storage_2d<rgba8unorm, read_write>;

#ifdef FOG
struct FogLayer {
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    scale: f32,
    multiplier: f32,
    speed: f32,
    offset: vec2<f32>,
}

struct FogData {
    layers: array<FogLayer, 4>,
    layer_count: u32,
    min: f32,
    max: f32,
    intensity: f32,
    color: vec3<f32>,
}

@group(3) @binding(0)
var<uniform> mist_uniform: FogData;
#endif

fn round_to_int(uv: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(uv + vec2<f32>(0.5, 0.5));
}

fn map_height(height: vec2<f32>) -> f32 {
    return height.x + height.y * 255.;
}

@fragment
fn mist(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = round_to_int(in.position.xy);
    let height = textureLoad(screen_height_texture, uv).xy;
    textureStore(screen_height_texture, uv, vec4<f32>(0., 0., 0., 0.));
    var color = textureSampleLevel(screen_color_texture, screen_color_texture_sampler, in.uv, 0.);

#ifdef HEIGHT_FORCE_DISPLAY
    return vec4<f32>(height, 0., 1.);
#else

    // if real_height < 0.000001 {
    //     return color;
    // }

#ifdef FOG
    // let cam_offset = vec2<f32>(-uniforms.camera_pos.x, uniforms.camera_pos.y);
    // let layers = &mist_uniform.layers;
    // var fog = vec4<f32>(0., 0., 0., 0.);
    // let weight = saturate(real_height - mist_uniform.min) / (mist_uniform.max - mist_uniform.min);
    
    // if weight > 0.01 {
    //     for (var i = 0u; i < mist_uniform.layer_count; i++) {
    //         let n_pos = vec3<f32>((in.position.xy - cam_offset / uniforms.camera_scale) * (*layers)[i].scale + (*layers)[i].offset, uniforms.time * (*layers)[i].speed);
    //         fog += saturate(fbm_3d(n_pos, i32((*layers)[i].octaves), (*layers)[i].lacunarity, (*layers)[i].gain) * (*layers)[i].multiplier);
    //     }

    //     var fog_col = vec4<f32>(fog * weight * mist_uniform.intensity);
    //     color += vec4<f32>(fog_col.rgb * mist_uniform.color, fog_col.a);
    // }
#endif

    return color;
#endif
}
