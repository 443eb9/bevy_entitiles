#import bevy_ui::ui_vertex_output::UiVertexOutput

struct UiTile {
    @location(0) uv_min: vec2<f32>,
    @location(1) uv_max: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> tile: UiTile;

@fragment
fn fragment(in: UiVertexOutput) -> location(0) vec4<f32> {
    let area = in.uv_max - in.uv_min;
    let uv = in.uv * area + in.uv_min;
}
