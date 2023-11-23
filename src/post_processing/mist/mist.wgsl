#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import noisy_bevy::fbm_simplex_3d
#import bevy_render::{globals::Globals, view::View}

struct FogData {
    min: f32,
    max: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    scale: f32,
    multiplier: f32,
    speed: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var<uniform> view: View;

@group(1) @binding(0)
var screen_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var screen_color_texture_sampler: sampler;

@group(2) @binding(0)
var screen_height_texture: texture_storage_2d<rgba8unorm, read_write>;

#ifdef FOG
@group(3) @binding(0) 
var<uniform> mist_uniform: FogData;
#endif

fn clouds() {

}

fn round_to_int(uv: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(uv + vec2<f32>(0.5, 0.5));
}

@fragment
fn mist(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = round_to_int(in.position.xy);
    let height = textureLoad(screen_height_texture, uv).xy;
    textureStore(screen_height_texture, uv, vec4<f32>(0., 0., 0., 0.));
    var color = textureSampleLevel(screen_color_texture, screen_color_texture_sampler, in.uv, 0.);

#ifdef HEIGHT_FORCE_DISPLAY
    if height.x + height.y < 0.001 {
        return color;
    }
    return vec4<f32>(height * 10., 0., 1.);
#else

#ifdef FOG
    let n_pos = vec3<f32>(in.position.xy * mist_uniform.scale, globals.time * mist_uniform.speed) + view.world_position;
    let noise = saturate(fbm_simplex_3d(n_pos, i32(mist_uniform.octaves), mist_uniform.lacunarity, mist_uniform.gain) * mist_uniform.multiplier);
    // let h = height.x + height.y;
    // if h > mist_uniform.min && h < mist_uniform.max {
    //     let fog = (h - mist_uniform.min) / (mist_uniform.max - mist_uniform.min);
    //     color = mix(color, vec4<f32>(0., 0., 0., 1.), fog * noise);
    // }

    color += vec4<f32>(noise);

    // color += vec4<f32>(fog * noise);
    // return vec4<f32>(noise);
#endif

    return color;
#endif
}
