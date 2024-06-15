use bevy::math::{IVec2, UVec2, Vec2};

use crate::tilemap::map::{TilemapAxisFlip, TilemapTransform, TilemapType};

/// Get the world position of the pivot of a slot.
pub fn index_to_world(
    index: IVec2,
    ty: TilemapType,
    transform: &TilemapTransform,
    pivot: Vec2,
    slot_size: Vec2,
) -> Vec2 {
    let index = index.as_vec2();
    transform.transform_point({
        match ty {
            TilemapType::Square => (index - pivot) * slot_size,
            TilemapType::Isometric => {
                (Vec2 {
                    x: (index.x - index.y),
                    y: (index.x + index.y),
                } / 2.
                    - pivot)
                    * slot_size
            }
            TilemapType::Hexagonal(legs) => Vec2 {
                x: slot_size.x * (index.x - 0.5 * index.y - pivot.x),
                y: (slot_size.y + legs as f32) / 2. * (index.y - pivot.y),
            },
        }
    })
}

/// Get the relative position of the pivot of a slot to the tilemap.
pub fn index_to_rel(
    index: IVec2,
    ty: TilemapType,
    transform: &TilemapTransform,
    pivot: Vec2,
    slot_size: Vec2,
) -> Vec2 {
    index_to_world(index, ty, transform, pivot, slot_size) - transform.translation
}

/// Get the tile collider in local space.
pub fn get_tile_collider(
    ty: TilemapType,
    slot_size: Vec2,
    size: UVec2,
    transform: &TilemapTransform,
    pivot: Vec2,
) -> Vec<Vec2> {
    let size = size.as_ivec2();
    match ty {
        TilemapType::Square => {
            let left_down = index_to_world(IVec2::ZERO, ty, transform, pivot, slot_size);
            let up_right = index_to_world(size, ty, transform, pivot, slot_size);

            vec![
                left_down,
                Vec2::new(up_right.x, left_down.y),
                up_right,
                Vec2::new(left_down.x, up_right.y),
            ]
        }
        TilemapType::Isometric => {
            let down = index_to_world(IVec2::ZERO, ty, transform, pivot, slot_size);
            let up = index_to_world(size, ty, transform, pivot, slot_size);
            let left = index_to_world(IVec2::new(0, size.y), ty, transform, pivot, slot_size);
            let right = index_to_world(IVec2::new(size.x, 0), ty, transform, pivot, slot_size);
            let offset = transform.apply_rotation(Vec2::new(slot_size.x / 2., 0.));

            vec![down + offset, up + offset, right + offset, left + offset]
        }
        TilemapType::Hexagonal(leg) => {
            let mut vertices = Vec::new();
            let Vec2 {
                x: slot_x,
                y: slot_y,
            } = slot_size;
            let leg_gap = slot_size.y / 2. - leg as f32 / 2.;

            /*
             * /3\
             * 4 2
             * | |
             * 5 1
             * \0/
             */

            // 0, 1
            (0..size.x).into_iter().for_each(|x| {
                let pivot = index_to_world(IVec2 { x, y: 0 }, ty, transform, pivot, slot_size);
                vertices.extend_from_slice(&[
                    pivot + Vec2::new(slot_x / 2., 0.),
                    pivot + Vec2::new(slot_x, leg_gap),
                ]);
            });

            // 1, 2
            (0..size.y).into_iter().for_each(|y| {
                let pivot =
                    index_to_world(IVec2::new(size.x - 1, y), ty, transform, pivot, slot_size);
                vertices.extend_from_slice(&[
                    pivot + Vec2::new(slot_x, leg_gap),
                    pivot + Vec2::new(slot_x, slot_y - leg_gap),
                ]);
            });

            // 3, 4
            (0..size.x).rev().into_iter().for_each(|x| {
                let pivot =
                    index_to_world(IVec2::new(x, size.y - 1), ty, transform, pivot, slot_size);
                vertices.extend_from_slice(&[
                    pivot + Vec2::new(slot_x / 2., slot_y),
                    pivot + Vec2::new(0., slot_y - leg_gap),
                ]);
            });

            // 4, 5
            (0..size.y).rev().into_iter().for_each(|y| {
                let pivot = index_to_world(IVec2::new(0, y), ty, transform, pivot, slot_size);
                vertices.extend_from_slice(&[
                    pivot + Vec2::new(0., slot_y - leg_gap),
                    pivot + Vec2::new(0., leg_gap),
                ]);
            });

            vertices.push(vertices[0]);
            vertices
        }
    }
}

/// Get the tile collider in world space.
pub fn get_tile_collider_world(
    origin: IVec2,
    ty: TilemapType,
    size: UVec2,
    transform: &TilemapTransform,
    pivot: Vec2,
    slot_size: Vec2,
) -> Vec<Vec2> {
    let offset = index_to_rel(origin, ty, transform, pivot, slot_size) + pivot * slot_size;
    get_tile_collider(ty, slot_size, size, transform, pivot)
        .into_iter()
        .map(|v| v + offset)
        .collect()
}

/// Calculate the size of the tilemap in world space.
pub fn calculate_map_size(size: UVec2, slot_size: Vec2, ty: TilemapType) -> Vec2 {
    let sizef = size.as_vec2();
    match ty {
        TilemapType::Square => Vec2::new(sizef.x * slot_size.x, sizef.y * slot_size.y),
        TilemapType::Isometric => (sizef.x + sizef.y) / 2. * slot_size,
        TilemapType::Hexagonal(leg) => {
            if size.y == 0 {
                Vec2::ZERO
            } else {
                let leg = leg as f32;
                let a = (leg + slot_size.y) / 2.;
                let b = (slot_size.y - leg) / 2.;
                Vec2::new((sizef.x + sizef.y) * slot_size.x, b + sizef.y * a)
            }
        }
    }
}

/// Calculate the size of the staggered tilemap in world space.
pub fn calculate_map_size_staggered(size: UVec2, slot_size: Vec2, leg: u32) -> Vec2 {
    let leg = leg as f32;
    let sizef = size.as_vec2();
    if size.y == 0 {
        return Vec2::ZERO;
    }
    if size.y == 1 {
        return slot_size * sizef;
    }

    let a = (leg + slot_size.y) / 2.;
    let b = (slot_size.y - leg) / 2.;
    Vec2::new((sizef.x + 0.5) * slot_size.x, b + sizef.y * a)
}

/// Get the tilemap axis vectors.
pub fn get_tilemap_axis(ty: TilemapType, slot_size: Vec2, flip: TilemapAxisFlip) -> (Vec2, Vec2) {
    let (x, y) = match ty {
        TilemapType::Square => (Vec2::X, Vec2::Y),
        TilemapType::Isometric => (
            slot_size.normalize(),
            Vec2::new(slot_size.x, -slot_size.y).normalize(),
        ),
        TilemapType::Hexagonal(leg) => (
            Vec2::X,
            Vec2::new(-slot_size.x, (slot_size.y + leg as f32) / 2.).normalize(),
        ),
    };
    let flip = flip.as_vec2();
    (x * flip.x, y * flip.y)
}

/// Stagger mode for staggered tilemaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaggerMode {
    Odd,
    Even,
}

/// Convert the diamond index to the staggered index.
pub fn staggerize_index(index: IVec2, staggered_mode: StaggerMode) -> IVec2 {
    match staggered_mode {
        StaggerMode::Odd => IVec2::new(index.x + (index.y + 1) / 2, index.y),
        StaggerMode::Even => IVec2::new(index.x + index.y / 2, index.y),
    }
}

/// Convert the staggered index to the diamond index.
pub fn destaggerize_index(index: IVec2, staggered_mode: StaggerMode) -> IVec2 {
    match staggered_mode {
        StaggerMode::Odd => IVec2::new(index.x - index.y / 2, index.y),
        StaggerMode::Even => IVec2::new(index.x - (index.y + 1) / 2, index.y),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calc_staggered_size() {
        let size = UVec2::new(3, 3);
        let slot_size = Vec2::new(32., 32.);
        let leg = 8;
        let size = calculate_map_size_staggered(size, slot_size, leg);
        assert_eq!(size, Vec2::new(112., 66.));
    }
}
