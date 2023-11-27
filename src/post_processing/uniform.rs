use bevy::{
    ecs::{
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{Vec2, Vec3Swizzles},
    render::{
        camera::OrthographicProjection,
        render_resource::{BindGroupEntry, DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        Extract,
    },
    time::Time,
    transform::components::Transform,
};

use super::{mist::pipeline::MistPipeline, PostProcessingBindGroups, PostProcessingView};

#[derive(Default, Resource, ShaderType, Clone, Copy)]
pub struct PostProcessingUniform {
    pub time: f32,
    pub camera_pos: Vec2,
    pub camera_scale: f32,
}

#[derive(Resource, Default)]
pub struct PostProcessingUniforms {
    pub buffer: DynamicUniformBuffer<PostProcessingUniform>,
}

pub fn extract_uniforms(
    mut commands: Commands,
    time: Extract<Res<Time>>,
    view_target: Extract<Query<(&Transform, &OrthographicProjection), With<PostProcessingView>>>,
) {
    let camera = view_target.single();
    let uniform = PostProcessingUniform {
        time: time.elapsed_seconds(),
        camera_pos: camera.0.translation.xy(),
        camera_scale: camera.1.scale,
    };
    commands.insert_resource(uniform);
}

pub fn prepare_uniforms(
    uniforms: Res<PostProcessingUniform>,
    mut uniforms_buffer: ResMut<PostProcessingUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    uniforms_buffer.buffer.clear();
    uniforms_buffer.buffer.push(*uniforms);
    uniforms_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}

pub fn bind_uniforms(
    mut bind_groups: ResMut<PostProcessingBindGroups>,
    uniforms_buffer: Res<PostProcessingUniforms>,
    render_device: Res<RenderDevice>,
    pipeline: Res<MistPipeline>,
) {
    if bind_groups.uniforms_bind_group.is_none() {
        if let Some(binding) = uniforms_buffer.buffer.binding() {
            bind_groups.uniforms_bind_group = Some(render_device.create_bind_group(
                "uniforms_bind_group",
                &pipeline.uniforms_layout,
                &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
            ));
        }
    }
}
