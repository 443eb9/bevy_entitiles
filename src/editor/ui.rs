use bevy::{
    a11y::accesskit::Node,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    math::UVec2,
    render::color::Color,
    text::{Text, Text2dBundle, TextAlignment, TextSection, TextStyle},
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        AlignContent, AlignItems, AlignSelf, JustifyContent, JustifyItems, JustifySelf, Style, Val,
    },
};

use crate::tilemap::{map::Tilemap, tile::TileBuilder};

use super::{
    palatte::{PalatteTilemap, TilePalatte},
    EditorStyle,
};

#[derive(Component, Default)]
pub struct ScrollingList {
    pub position: f32,
}

pub fn setup_ui(
    mut root_cmd: Commands,
    mut commands: Commands,
    presets: Res<TilePalatte>,
    style: Res<EditorStyle>,
    mut palatte_tilemap: Query<&mut Tilemap, With<PalatteTilemap>>,
) {
    root_cmd
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|root| {
            spawn_title(root, Entity::PLACEHOLDER, "".to_string(), &style);
            spawn_left_section(
                &mut commands,
                root,
                &presets.presets,
                &mut palatte_tilemap.single_mut(),
            );
        });
}

pub fn spawn_title(
    root: &mut ChildBuilder,
    map_entity: Entity,
    map_name: String,
    style: &Res<EditorStyle>,
) {
    root.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|title_node| {
        title_node.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!("Editing {}({:?})", map_name, map_entity),
                    style: TextStyle {
                        font_size: style.title_size,
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        });
    });
}

pub fn spawn_left_section(
    commands: &mut Commands,
    root: &mut ChildBuilder,
    presets: &Vec<TileBuilder>,
    palatte_tilemap: &mut Tilemap,
) {
    root.spawn(NodeBundle {
        style: Style {
            width: Val::Px(350.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|left| {
        left.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Palattes".to_string(),
                    style: TextStyle {
                        font_size: 24.,
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        });
        spawn_palattes_selector(commands, left, presets, palatte_tilemap);
    });
}

pub fn spawn_palattes_selector(
    commands: &mut Commands,
    left: &mut ChildBuilder,
    presets: &Vec<TileBuilder>,
    palatte_tilemap: &mut Tilemap,
) {
    left.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            ..Default::default()
        },
        ScrollingList::default(),
    ))
    .with_children(|_| {
        for i in 0..presets.len() {
            palatte_tilemap.set(commands, UVec2 { x: 0, y: i as u32 }, &presets[i]);
        }
    });
}
