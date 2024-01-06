use bevy::math::{IVec2, Vec2};

use super::map::{TilePivot, TilemapSlotSize, TilemapTransform, TilemapType};

/// Get the world position of the center of a slot.
pub fn index_to_world(
    index: IVec2,
    ty: &TilemapType,
    transform: &TilemapTransform,
    pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) -> Vec2 {
    let pivot = pivot.0;
    let slot_size = slot_size.0;
    let index = index.as_vec2();
    transform.transform_point({
        match ty {
            TilemapType::Square => (index - pivot) * slot_size,
            TilemapType::Isometric => {
                (Vec2 {
                    x: (index.x - index.y - 1.),
                    y: (index.x + index.y),
                } / 2.
                    - pivot)
                    * slot_size
            }
            TilemapType::Hexagonal(legs) => Vec2 {
                x: slot_size.x * (index.x - 0.5 * index.y - pivot.x),
                y: (slot_size.y + *legs as f32) / 2. * (index.y - pivot.y),
            },
        }
    })
}

pub fn index_to_rel(
    index: IVec2,
    ty: &TilemapType,
    transform: &TilemapTransform,
    pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) -> Vec2 {
    index_to_world(index, ty, transform, pivot, slot_size) - transform.translation
}

pub fn get_tile_convex_hull(ty: &TilemapType, slot_size: &TilemapSlotSize) -> Vec<Vec2> {
    let (x, y) = (slot_size.0.x, slot_size.0.y);
    match ty {
        TilemapType::Square => vec![
            Vec2 { x: 0., y: 0. },
            Vec2 { x: 0., y },
            Vec2 { x, y },
            Vec2 { x, y: 0. },
        ],
        TilemapType::Isometric => vec![
            Vec2 { x: 0., y: y / 2. },
            Vec2 { x: x / 2., y },
            Vec2 { x, y: y / 2. },
            Vec2 { x: x / 2., y: 0. },
        ],
        TilemapType::Hexagonal(c) => {
            let c = *c as f32;
            let half = (y - c) / 2.;

            vec![
                Vec2 { x: 0., y: half },
                Vec2 { x: 0., y: half + c },
                Vec2 { x: x / 2., y: y },
                Vec2 { x: x, y: half + c },
                Vec2 { x: x, y: half },
                Vec2 { x: x / 2., y: 0. },
            ]
        }
    }
}

pub fn get_tile_convex_hull_rel(
    index: IVec2,
    ty: &TilemapType,
    transform: &TilemapTransform,
    pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) -> Vec<Vec2> {
    let offset = index_to_rel(index, ty, transform, pivot, slot_size);
    get_tile_convex_hull(ty, slot_size)
        .into_iter()
        .map(|p| transform.apply_rotation(p) + offset)
        .collect()
}

pub fn get_tile_convex_hull_world(
    index: IVec2,
    ty: &TilemapType,
    transform: &TilemapTransform,
    pivot: &TilePivot,
    slot_size: &TilemapSlotSize,
) -> Vec<Vec2> {
    let offset = index_to_world(index, ty, transform, pivot, slot_size);
    get_tile_convex_hull(ty, slot_size)
        .into_iter()
        .map(|p| transform.apply_rotation(p) + offset)
        .collect()
}
