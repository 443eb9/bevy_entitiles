use std::marker::PhantomData;

// TODO: Replace with proper logging when render system is updated
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("WARN: {}", format!($($arg)*))
    };
}

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    render::{
        render_phase::{RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass},
        view::ViewUniformOffset,
    },
};

use crate::render::{
    binding::TilemapBindGroups,
    buffer::TilemapBuffers,
    chunk::RenderChunkStorage,
    extract::{TilemapInstances, TilemapMaterialIds},
    material::TilemapMaterial,
};

pub type DrawTilemapTextured<M> = (
    SetItemPipeline,
    SetTilemapUniformBufferBindGroup<0, M>,
    SetTilemapMaterialBindGroup<1, M>,
    SetTilemapColorTextureBindGroup<2, M>,
    SetTilemapArrayBufferBindGroup<3, M>,
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
            tilemap_buffers.shared.indices.get(&item.entity.0),
        ) {
            pass.set_bind_group(
                I,
                tilemap_uniform_bind_group,
                &[*index, view_uniform_offset.offset],
            );
            RenderCommandResult::Success
        } else {
            warn!(
                "Failed to draw tilemap {:?}: Failed to get tilemap uniform bind group! \
                Skipping rendering this frame.",
                item.entity
            );
            RenderCommandResult::Skip
        }
    }
}

#[derive(Default)]
pub struct SetTilemapMaterialBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapMaterialBindGroup<I, M>
{
    type Param = (SRes<TilemapBindGroups<M>>, SRes<TilemapMaterialIds<M>>);

    type ViewQuery = ();

    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        (bind_groups, material_ids): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bind_group) = material_ids
            .get(&item.entity.1)
            .and_then(|id| bind_groups.into_inner().materials.get(&id.asset_id))
        {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            warn!(
                "Failed to draw tilemap {:?}: Failed to get material bind group! \
                Skipping rendering this frame.",
                item.entity
            );
            RenderCommandResult::Skip
        }
    }
}

#[derive(Default)]
pub struct SetTilemapArrayBufferBindGroup<const I: usize, M: TilemapMaterial>(PhantomData<M>);
impl<const I: usize, M: TilemapMaterial> RenderCommand<Transparent2d>
    for SetTilemapArrayBufferBindGroup<I, M>
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
        if let Some(bind_group) = bind_groups.into_inner().array_buffers.get(&item.entity.0) {
            #[cfg(target_arch = "wasm32")]
            pass.set_bind_group(I, bind_group, &[0, 0]);
            #[cfg(not(target_arch = "wasm32"))]
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            warn!(
                "Failed to draw tilemap {:?}: Failed to get storage bind group! \
                Skipping rendering this frame.",
                item.entity
            );
            RenderCommandResult::Skip
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
        let Some(textures) = instances
            .get(&item.entity.1)
            .and_then(|inst| inst.texture.as_ref())
        else {
            warn!(
                "Failed to draw tilemap {:?}: Failed to get tilemap instance. \
                Skipping rendering this frame.",
                item.entity
            );
            return RenderCommandResult::Skip;
        };

        if let Some(bind_group) = &bind_groups.into_inner().textures.get(textures) {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            warn!(
                "Failed to draw tilemap {:?}: Failed to get color texture bind group! \
                Skipping rendering this frame.",
                item.entity
            );
            RenderCommandResult::Skip
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
        if let Some(chunks) = render_chunks.into_inner().get_chunks(item.entity.0) {
            for chunk in chunks.value.values() {
                if !chunk.visible {
                    continue;
                }

                // TODO: Update drawing logic for Bevy 0.16 RenderMesh/MeshAllocator API
                // if let Some(gpu_mesh) = &chunk.gpu_mesh {
                //     pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                //     // Drawing logic needs to be updated for new buffer management
                //     pass.draw(0..gpu_mesh.vertex_count, 0..1);
                // }
            }
        }

        RenderCommandResult::Success
    }
}
