use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::lifetimeless::{Read, SRes},
    render::{
        render_phase::{RenderCommand, RenderCommandResult},
        render_resource::PipelineCache,
        view::ViewUniformOffset,
    },
};

use super::BindGroups;

pub struct SetTileDataBindingGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTileDataBindingGroup<I> {
    type Param = ();

    type ViewWorldQuery = Read<ViewUniformOffset>;

    type ItemWorldQuery = Read<BindGroups>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        view_uniform_offset: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        bind_groups: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        pass.set_bind_group(I, &bind_groups.tile_data, &[view_uniform_offset.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetTileTextureBindingGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent2d> for SetTileTextureBindingGroup<I> {
    type Param = ();

    type ViewWorldQuery = ();

    type ItemWorldQuery = Read<BindGroups>;

    #[inline]
    fn render<'w>(
        _item: &Transparent2d,
        _view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        bind_groups: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        for (_, bind_group) in bind_groups.tile_textures.iter() {
            pass.set_bind_group(I, bind_group, &[]);
        }

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

pub struct DrawTiles;
impl RenderCommand<Transparent2d> for DrawTiles {
    type Param = ();

    type ViewWorldQuery=();

    type ItemWorldQuery=();

    #[inline]
    fn render<'w>(
        item: &Transparent2d,
        view: bevy::ecs::query::ROQueryItem<'w, Self::ViewWorldQuery>,
        entity: bevy::ecs::query::ROQueryItem<'w, Self::ItemWorldQuery>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.draw(vertices, instances)
    }
}
