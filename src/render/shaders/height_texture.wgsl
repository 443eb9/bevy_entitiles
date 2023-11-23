#define_import_path bevy_entitiles::height_texture

#import bevy_entitiles::common::VertexOutput

@group(3) @binding(0)
var height_texture: texture_2d<f32>;

@group(3) @binding(1)
var height_texture_sampler: sampler;

@group(4) @binding(0)
var screen_height_texture: texture_storage_2d<rgba8unorm, read_write>;

fn round_to_int(uv: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(uv + vec2<f32>(0.5, 0.5));
}

fn sample_height(input: VertexOutput, screen_size: vec2<f32>) {
    let color = textureSample(height_texture, height_texture_sampler, input.uv).r;
    textureStore(screen_height_texture, round_to_int(input.position.xy), vec4<f32>(color, input.height, 0., 1.));
}
