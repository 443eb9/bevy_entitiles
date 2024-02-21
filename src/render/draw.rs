use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    log::error,
    render::{
        mesh::GpuBufferInfo,
        render_phase::{RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::PipelineCache,
        view::ViewUniformOffset,
    },
};

use super::{
    binding::{TilemapBindGroups, TilemapViewBindGroup},
    buffer::{DynamicOffsetComponent, TilemapUniform},
    chunk::RenderChunkStorage,
    material::TilemapMaterial,
    resources::TilemapInstances,
};

pub type DrawTilemap<M> = (
    SetPipeline,
    SetTilemapViewBindGroup<0>,
    SetTilemapUniformBufferBindGroup<1, M>,
    SetTilemapMaterialBindGroup<2, M>,
    SetTilemapColorTextureBindGroup<3, M>,
    SetTilemapStorageBufferBindGroup<4, M>,
    DrawTileMesh<M>,
);

pub struct SetPipeline;
impl RenderCommand<Transparent2d> for SetPipeline {
    type Param = SRes<PipelineCache>;

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        pipeline_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let pipeline_cache = pipeline_cache.into_inner();
        if let Some(pipeline) = pipeline_cache.get_render_pipeline(item.pipeline) {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            error!(
                "Failed to get render pipeline!\n{:?}",
                pipeline_cache.get_render_pipeline_state(item.pipeline)
            );
            RenderCommandResult::Failure
        }
    }
}

pub struct SetTilemapViewBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapViewBindGroup<I> {
    type Param = ();

    type ViewQuery = (Read<ViewUniformOffset>, Read<TilemapViewBindGroup>);

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        (view_uniform_offset, view_bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &view_bind_group.value, &[view_uniform_offset.offset]);

        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct SetTilemapUniformBufferBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapUniformBufferBindGroup<I, M>
{
    type Param = SRes<TilemapBindGroups<M>>;

    type ViewQuery = ();

    type ItemQuery = Read<DynamicOffsetComponent<TilemapUniform>>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        uniform_data: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let (Some(tilemap_uniform_bind_group), Some(uniform_data)) = (
            bind_groups.into_inner().tilemap_uniform_buffer.as_ref(),
            uniform_data,
        ) {
            pass.set_bind_group(I, tilemap_uniform_bind_group, &[uniform_data.index()]);
            RenderCommandResult::Success
        } else {
            error!("Failed to get tilemap uniform bind group!");
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct SetTilemapMaterialBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapMaterialBindGroup<I, M>
{
    type Param = (SRes<TilemapBindGroups<M>>, SRes<TilemapInstances<M>>);

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(inst) = instances.0.get(&item.entity) else {
            error!("Failed to get tilemap instance!");
            return RenderCommandResult::Failure;
        };

        let id = inst.material.id();
        if let Some(bind_group) = bind_groups.into_inner().material_bind_groups.get(&id) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Failed to get material bind group!");
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct SetTilemapStorageBufferBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapStorageBufferBindGroup<I, M>
{
    type Param = SRes<TilemapBindGroups<M>>;

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = bind_groups
            .into_inner()
            .tilemap_storage_buffers
            .get(&item.entity)
        {
            pass.set_bind_group(I, bind_group, &[]);
        }

        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct SetTilemapColorTextureBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapColorTextureBindGroup<I, M>
{
    type Param = (SRes<TilemapBindGroups<M>>, SRes<TilemapInstances<M>>);

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(texture) = instances.0.get(&item.entity).unwrap().texture.as_ref() else {
            return RenderCommandResult::Success;
        };

        if let Some(bind_group) = &bind_groups
            .into_inner()
            .colored_textures
            .get(texture.handle())
        {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Filed to get color texture bind group!");
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct DrawTileMesh<M: TilemapMaterial>(PhantomData<M>);
impl<M: TilemapMaterial> RenderCommand<Transparent2d> for DrawTileMesh<M> {
    type Param = SRes<RenderChunkStorage<M>>;

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        render_chunks: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(chunks) = render_chunks.into_inner().get_chunks(item.entity) {
            for chunk in chunks.values() {
                if !chunk.visible {
                    continue;
                }

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
