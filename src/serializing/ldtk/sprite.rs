use bevy::{
    asset::{Asset, Handle},
    math::{IVec2, IVec4, Vec2, Vec4},
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
        texture::Image,
    },
    sprite::Material2d,
};
use serde::Serialize;

use crate::math::extension::DivToCeil;

use super::{json::definitions::TilesetRect, ENTITY_SPRITE_SHADER};

#[derive(ShaderType, Clone, Copy, Debug)]
pub struct AtlasRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl From<TilesetRect> for AtlasRect {
    fn from(value: TilesetRect) -> Self {
        Self {
            min: IVec2::new(value.x_pos, value.y_pos).as_vec2(),
            max: IVec2::new(value.x_pos + value.width, value.y_pos + value.height).as_vec2(),
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct LdtkEntityMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub atlas_rect: AtlasRect,
}

impl Material2d for LdtkEntityMaterial {
    fn fragment_shader() -> ShaderRef {
        ENTITY_SPRITE_SHADER.into()
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct NineSliceBorders {
    pub is_valid: bool,
    pub up: i32,
    pub right: i32,
    pub down: i32,
    pub left: i32,
}

pub struct NineSliceMesh {
    pub vertices: Vec<Vec2>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
}

impl NineSliceBorders {
    pub fn generate_mesh(
        &self,
        render_size: IVec2,
        tile_size: IVec2,
        pivot: Vec2,
    ) -> NineSliceMesh {
        let inner_pxs = IVec2::new(
            render_size.x - self.left - self.right,
            render_size.y - self.up - self.down,
        );
        let sliced_tile_inner_size = IVec2::new(
            tile_size.x - self.left - self.right,
            tile_size.y - self.up - self.down,
        );
        let border_pxs = IVec4::new(self.up, self.down, self.left, self.right).as_vec4();

        let tile_size = tile_size.as_vec2();
        let render_size = render_size.as_vec2();
        let border_uvs = Vec4::new(
            border_pxs.x / tile_size.y,
            border_pxs.y / tile_size.y,
            border_pxs.z / tile_size.x,
            border_pxs.w / tile_size.x,
        );
        let valid_inner_range = [
            Vec2::new(border_pxs.z, border_pxs.x),
            Vec2::new(render_size.x - border_pxs.w, render_size.y - border_pxs.y),
        ];

        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        let mut vertex_indices = Vec::new();
        let base_indices = [0, 3, 1, 1, 3, 2];
        let mut quad_count = 0;
        // corners
        // u_l
        vertices.extend_from_slice(&[
            Vec2::new(0., 0.),
            Vec2::new(border_pxs.z, 0.),
            Vec2::new(border_pxs.z, border_pxs.x),
            Vec2::new(0., border_pxs.x),
        ]);
        uvs.extend_from_slice(&[
            Vec2::new(0., 0.),
            Vec2::new(border_uvs.z, 0.),
            Vec2::new(border_uvs.z, border_uvs.x),
            Vec2::new(0., border_uvs.x),
        ]);
        vertex_indices.extend(base_indices);
        quad_count += 1;
        // u_r
        vertices.extend_from_slice(&[
            Vec2::new(render_size.x - border_pxs.w, 0.),
            Vec2::new(render_size.x, 0.),
            Vec2::new(render_size.x, border_pxs.x),
            Vec2::new(render_size.x - border_pxs.w, border_pxs.x),
        ]);
        uvs.extend_from_slice(&[
            Vec2::new(1. - border_uvs.w, 0.),
            Vec2::new(1., 0.),
            Vec2::new(1., border_uvs.x),
            Vec2::new(1. - border_uvs.w, border_uvs.x),
        ]);
        vertex_indices.extend(base_indices.iter().map(|v| v + quad_count * 4));
        quad_count += 1;
        // d_l
        vertices.extend_from_slice(&[
            Vec2::new(0., render_size.y - border_pxs.y),
            Vec2::new(border_pxs.z, render_size.y - border_pxs.y),
            Vec2::new(border_pxs.z, render_size.y),
            Vec2::new(0., render_size.y),
        ]);
        uvs.extend_from_slice(&[
            Vec2::new(0., 1. - border_uvs.y),
            Vec2::new(border_uvs.z, 1. - border_uvs.y),
            Vec2::new(border_uvs.z, 1.),
            Vec2::new(0., 1.),
        ]);
        vertex_indices.extend(base_indices.iter().map(|v| v + quad_count * 4));
        quad_count += 1;
        // d_r
        vertices.extend_from_slice(&[
            Vec2::new(render_size.x - border_pxs.w, render_size.y - border_pxs.y),
            Vec2::new(render_size.x, render_size.y - border_pxs.y),
            Vec2::new(render_size.x, render_size.y),
            Vec2::new(render_size.x - border_pxs.w, render_size.y),
        ]);
        uvs.extend_from_slice(&[
            Vec2::new(1. - border_uvs.w, 1. - border_uvs.y),
            Vec2::new(1., 1. - border_uvs.y),
            Vec2::new(1., 1.),
            Vec2::new(1. - border_uvs.w, 1.),
        ]);
        vertex_indices.extend(base_indices.iter().map(|v| v + quad_count * 4));
        quad_count += 1;

        // up and down
        let tiled_count = inner_pxs.div_to_ceil(sliced_tile_inner_size);
        let tiled_size = Vec2 {
            x: (tile_size.x - border_pxs.z - border_pxs.w) as f32,
            y: (tile_size.y - border_pxs.x - border_pxs.y) as f32,
        };
        let origins = [
            Vec2::new(border_pxs.z, 0.),
            Vec2::new(border_pxs.z, render_size.y - border_pxs.y),
            Vec2::new(0., border_pxs.x),
            Vec2::new(render_size.x - border_pxs.w, border_pxs.x),
        ];
        let extents = [
            Vec2::new(tiled_size.x, border_pxs.x),
            Vec2::new(tiled_size.x, border_pxs.y),
            Vec2::new(border_pxs.z, tiled_size.y),
            Vec2::new(border_pxs.w, tiled_size.y),
        ];
        let repeat = [
            IVec2::new(tiled_count.x, 1),
            IVec2::new(tiled_count.x, 1),
            IVec2::new(1, tiled_count.y),
            IVec2::new(1, tiled_count.y),
        ];
        let border_slice_uvs = [
            [
                Vec2::new(border_uvs.z, 0.),
                Vec2::new(1. - border_uvs.w, 0.),
                Vec2::new(1. - border_uvs.w, border_uvs.x),
                Vec2::new(border_uvs.z, border_uvs.x),
            ],
            [
                Vec2::new(border_uvs.z, 1. - border_uvs.y),
                Vec2::new(1. - border_uvs.w, 1. - border_uvs.y),
                Vec2::new(1. - border_uvs.w, 1.),
                Vec2::new(border_uvs.z, 1.),
            ],
            [
                Vec2::new(0., border_uvs.x),
                Vec2::new(border_uvs.z, border_uvs.x),
                Vec2::new(border_uvs.z, 1. - border_uvs.y),
                Vec2::new(0., 1. - border_uvs.y),
            ],
            [
                Vec2::new(1. - border_uvs.w, border_uvs.x),
                Vec2::new(1., border_uvs.x),
                Vec2::new(1., 1. - border_uvs.y),
                Vec2::new(1. - border_uvs.w, 1. - border_uvs.y),
            ],
        ];
        for i in 0..4 {
            for dx in 0..repeat[i].x {
                for dy in 0..repeat[i].y {
                    let (dx, dy) = (dx as f32, dy as f32);
                    vertices.extend_from_slice(&[
                        Vec2 {
                            x: origins[i].x + dx * extents[i].x,
                            y: origins[i].y + dy * extents[i].y,
                        },
                        Vec2 {
                            x: origins[i].x + (dx + 1.) * extents[i].x,
                            y: origins[i].y + dy * extents[i].y,
                        },
                        Vec2 {
                            x: origins[i].x + (dx + 1.) * extents[i].x,
                            y: origins[i].y + (dy + 1.) * extents[i].y,
                        },
                        Vec2 {
                            x: origins[i].x + dx * extents[i].x,
                            y: origins[i].y + (dy + 1.) * extents[i].y,
                        },
                    ]);
                    uvs.extend_from_slice(&border_slice_uvs[i]);
                    vertex_indices.extend(base_indices.iter().map(|v| v + quad_count * 4));
                    quad_count += 1;
                }
            }
        }

        // inner
        let origin = Vec2::new(border_pxs.z, border_pxs.x);
        let inner_slice_uvs = [
            Vec2::new(border_uvs.x, border_uvs.z),
            Vec2::new(1. - border_uvs.y, border_uvs.z),
            Vec2::new(1. - border_uvs.y, 1. - border_uvs.w),
            Vec2::new(border_uvs.x, 1. - border_uvs.w),
        ];
        for dx in 0..tiled_count.x {
            for dy in 0..tiled_count.y {
                let (dx, dy) = (dx as f32, dy as f32);
                vertices.extend([
                    origin + tiled_size * Vec2::new(dx, dy),
                    origin + tiled_size * Vec2::new(dx + 1., dy),
                    origin + tiled_size * Vec2::new(dx + 1., dy + 1.),
                    origin + tiled_size * Vec2::new(dx, dy + 1.),
                ]);
                uvs.extend_from_slice(&inner_slice_uvs);
                vertex_indices.extend(base_indices.iter().map(|v| v + quad_count * 4));
                quad_count += 1;
            }
        }

        NineSliceMesh {
            vertices: vertices
                .into_iter()
                .map(|v| Vec2::new(v.x, -v.y) - pivot * render_size)
                .collect(),
            uvs,
            indices: vertex_indices,
        }
    }
}
