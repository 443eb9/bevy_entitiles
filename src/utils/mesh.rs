use bevy::math::Vec2;

use crate::math::aabb::Aabb2d;

// TODO adapt this to all kinds of quads
/// **WARNING** This method is not suitable for all cases. It should only be used for quads in LDtk.
pub fn clip_quad_mesh(vertices: &mut Vec<Vec2>, uvs: &mut Vec<Vec2>, valid_rect: Aabb2d) {
    let size = vertices[2] - vertices[0];
    vertices
        .iter_mut()
        .for_each(|v| *v = v.clamp(valid_rect.min, valid_rect.max));
    let clipped_size = vertices[2] - vertices[0];
    let clipped_ratio = clipped_size / size;
    let uv_size_clipped = (uvs[2] - uvs[0]) * clipped_ratio;
    *uvs = vec![
        uvs[0],
        uvs[0] + Vec2::new(uv_size_clipped.x, 0.),
        uvs[0] + uv_size_clipped,
        uvs[0] + Vec2::new(0., uv_size_clipped.y),
    ];
}
