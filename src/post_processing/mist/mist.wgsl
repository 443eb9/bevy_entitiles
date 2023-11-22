#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct FogData {
    min: f32,
    max: f32,
}

@group(0) @binding(0)
var screen_color_texture: texture_2d<f32>;

@group(0) @binding(1)
var screen_color_texture_sampler: sampler;

@group(1) @binding(0)
var screen_height_texture: texture_storage_2d<rgba8unorm, read_write>;

#ifdef FOG
@group(2) @binding(0) 
var<uniform> mist_uniform: FogData;
#endif

fn clouds() {

}

fn fog() {
    
}

@fragment
fn mist(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureLoad(screen_height_texture, vec2<i32>(in.position.xy));
    if color.a < 0.001 {
        return textureSampleLevel(screen_color_texture, screen_color_texture_sampler, in.uv, 0.);
    }
    return color;
}
