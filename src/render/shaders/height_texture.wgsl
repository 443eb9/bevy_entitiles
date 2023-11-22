#define_import_path bevy_entitiles::height_texture

#import bevy_entitiles::common::VertexOutput

@group(3) @binding(0)
var height_texture: texture_2d<f32>;

@group(3) @binding(1)
var height_texture_sampler: sampler;

@group(4) @binding(0)
var screen_height_texture: texture_storage_2d<rgba8unorm, read_write>;

fn sample_height(input: VertexOutput, screen_size: vec2<f32>) {
    let color = textureSample(height_texture, height_texture_sampler, input.uv);
    textureStore(screen_height_texture, vec2<i32>(input.position.xy), color);
}
