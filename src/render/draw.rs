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
        render_phase::{RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass},
        view::ViewUniformOffset,
    },
};

use crate::render::{
    binding::TilemapBindGroups,
    buffer::TilemapBuffers,
    chunk::RenderChunkStorage,
    extract::{TilemapInstances, TilemapMaterialInstances},
    material::TilemapMaterial,
};

pub type DrawTilemapTextured<M> = (
    SetItemPipeline,
    SetTilemapUniformBufferBindGroup<0, M>,
    SetTilemapMaterialBindGroup<1, M>,
    SetTilemapColorTextureBindGroup<2, M>,
    SetTilemapStorageBufferBindGroup<3, M>,
    DrawTileMesh<M>,
);

pub type DrawTilemapNonTextured<M> = (
    SetItemPipeline,
    SetTilemapUniformBufferBindGroup<0, M>,
    SetTilemapMaterialBindGroup<1, M>,
    DrawTileMesh<M>,
);

#[derive(Default)]
pub struct SetTilemapUniformBufferBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapUniformBufferBindGroup<I, M>
{
    type Param = (SRes<TilemapBindGroups<M>>, SRes<TilemapBuffers>);

    type ViewQuery = Read<ViewUniformOffset>;

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        view_uniform_offset: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, tilemap_buffers): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let (Some(tilemap_uniform_bind_group), Some(index)) = (
            bind_groups.into_inner().uniform_buffer.as_ref(),
            tilemap_buffers.shared.indices.get(&item.entity),
        ) {
            pass.set_bind_group(
                I,
                tilemap_uniform_bind_group,
                &[*index, view_uniform_offset.offset],
            );
            RenderCommandResult::Success
        } else {
            error!("Failed to get tilemap uniform bind group! {}", item.entity);
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct SetTilemapMaterialBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapMaterialBindGroup<I, M>
{
    type Param = (
        SRes<TilemapBindGroups<M>>,
        SRes<TilemapMaterialInstances<M>>,
    );

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, materials): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = materials
            .get(&item.entity)
            .and_then(|id| bind_groups.into_inner().materials.get(id))
        {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Failed to get material bind group! {}", item.entity);
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
        if let Some(bind_group) = bind_groups.into_inner().storage_buffers.get(&item.entity) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Failed to get storage bind group! {}", item.entity);
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct SetTilemapColorTextureBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapColorTextureBindGroup<I, M>
{
    type Param = (SRes<TilemapBindGroups<M>>, SRes<TilemapInstances>);

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
        let Some(textures) = instances.get(&item.entity).unwrap().texture.as_ref() else {
            return RenderCommandResult::Success;
        };

        if let Some(bind_group) = &bind_groups.into_inner().textures.get(textures) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Filed to get color texture bind group! {}", item.entity);
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct DrawTileMesh<M: TilemapMaterial>(PhantomData<M>);
impl<M: TilemapMaterial> RenderCommand<Transparent2d> for DrawTileMesh<M> {
    type Param = SRes<RenderChunkStorage>;

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
            for chunk in chunks.value.values() {
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
