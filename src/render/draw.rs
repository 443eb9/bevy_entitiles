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

use super::{
    binding::{TilemapBindGroups, TilemapViewBindGroup},
    buffer::{StandardMaterialUniformBuffer, TilemapUniformBuffer},
    chunk::RenderChunkStorage,
    resources::TilemapInstances,
};

pub type DrawTilemap = (
    SetItemPipeline,
    SetTilemapViewBindGroup<0>,
    SetUniformBuffersBindGroup<1>,
    SetTilemapColorTextureBindGroup<2>,
    SetTilemapStorageBufferBindGroup<3>,
    DrawTileMesh,
);

pub type DrawTilemapWithoutTexture = (
    SetItemPipeline,
    SetTilemapViewBindGroup<0>,
    SetUniformBuffersBindGroup<1>,
    DrawTileMesh,
);

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
pub struct SetUniformBuffersBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetUniformBuffersBindGroup<I> {
    type Param = (
        SRes<TilemapInstances>,
        SRes<TilemapBindGroups>,
        SRes<TilemapUniformBuffer>,
        SRes<StandardMaterialUniformBuffer>,
    );

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (instances, bind_groups, tilemap_uniform_buffer, std_material_buffer): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(instance) = instances.0.get(&item.entity) else {
            error!("Failed to get tilemap uniform bind groups!");
            return RenderCommandResult::Failure;
        };
        
        let material = instance.material;

        if let (Some(tilemap_uniform_bind_group), Some(uniform_offset), Some(material_offset)) = (
            bind_groups.into_inner().tilemap_uniform_buffer.as_ref(),
            tilemap_uniform_buffer.offsets.get(&item.entity),
            std_material_buffer.offsets.get(&material),
        ) {
            pass.set_bind_group(
                I,
                tilemap_uniform_bind_group,
                &[uniform_offset.index(), material_offset.index()],
            );
            RenderCommandResult::Success
        } else {
            error!("Failed to get tilemap uniform bind groups!");
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct SetTilemapStorageBufferBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapStorageBufferBindGroup<I> {
    type Param = SRes<TilemapBindGroups>;

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
        }

        RenderCommandResult::Success
    }
}

#[derive(Default)]
pub struct SetTilemapColorTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTilemapColorTextureBindGroup<I> {
    type Param = (SRes<TilemapBindGroups>, SRes<TilemapInstances>);

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
        let material = &instances.0.get(&item.entity).unwrap().material;

        if let Some(bind_group) = &bind_groups.into_inner().materials.get(material) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            error!("Filed to get color texture bind group!");
            RenderCommandResult::Failure
        }
    }
}

#[derive(Default)]
pub struct DrawTileMesh;
impl RenderCommand<Transparent2d> for DrawTileMesh {
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
