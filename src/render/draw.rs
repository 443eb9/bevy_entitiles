use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::lifetimeless::{Read, SRes},
    render::{
        mesh::GpuBufferInfo,
        render_phase::{RenderCommand, RenderCommandResult},
        render_resource::PipelineCache,
    },
};

use crate::tilemap::Tilemap;

use super::{BindGroups, RenderChunkStorage};

pub type DrawTilemap = (SetTileTextureBindingGroup<0>, SetPipeline, DrawTileMesh);

pub struct SetTileTextureBindingGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTileTextureBindingGroup<I> {
    type Param = SRes<BindGroups>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<Tilemap>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemaps: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        bind_groups: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            bind_groups
                .into_inner()
                .tile_textures
                .get(&tilemaps.texture)
                .unwrap(),
            &[],
        );

        RenderCommandResult::Success
    }
}

pub struct SetPipeline;
impl RenderCommand<Transparent2d> for SetPipeline {
    type Param = SRes<PipelineCache>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        pipeline_cache: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(item.pipeline)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct DrawTileMesh;
impl RenderCommand<Transparent2d> for DrawTileMesh {
    type Param = SRes<RenderChunkStorage>;

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<Tilemap>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        tilemap: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        render_chunks: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(chunks) = render_chunks.into_inner().value.get(&tilemap.id) {
            for chunk in chunks.iter() {
                if let Some(gpu_mesh) = &chunk.gpu_mesh {
                    pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    match &gpu_mesh.buffer_info {
                        GpuBufferInfo::Indexed {
                            buffer,
                            count,
                            index_format,
                        } => {
                            pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                            pass.draw_indexed(0..*count, 0, 0..1);
                        }
                        GpuBufferInfo::NonIndexed => {
                            pass.draw(0..gpu_mesh.vertex_count, 0..1);
                        }
                    }
                }
            }
        }

        RenderCommandResult::Success
    }
}
