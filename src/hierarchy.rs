use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::{
    file_picker::FileChangedEvent,
    ui::LeftPanel,
    utils::{get_current_scene, get_gltf},
    InspectorState,
};

#[derive(Component, Default)]
struct ScrollingList {
    pub position: f32,
}

#[derive(Component)]
struct ScrollingListPanel;

/// This plugin prints the structure of the opened glTF file into a scrollable
/// list, showing the tree of nodes and the meshes attached to the individual nodes
pub struct HierarchyPlugin;

impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(remove_existing_list)
            .add_system(print_structure)
            .add_system(mouse_scroll);
    }
}

/// Finds and removes the existing List UI element if the file has changed
fn remove_existing_list(
    mut reader: EventReader<FileChangedEvent>,
    mut commands: Commands,
    query: Query<Entity, With<ScrollingListPanel>>,
) {
    if reader.iter().next().is_none() {
        return;
    }
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive()
    }
}

// Prints the structure of the loaded GLTF file as
fn print_structure(
    mut reader: EventReader<FileChangedEvent>,
    state: Res<InspectorState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut left_panel_query: Query<Entity, With<LeftPanel>>,
) {
    if reader.iter().next().is_none() {
        return;
    }

    let gltf = match get_gltf(&state) {
        Some(it) => it,
        _ => return,
    };

    // List based on https://bevyengine.org/examples/ui/ui/ last accessed: 2022-02-06
    // List with hidden overflow
    let entity = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                overflow: Overflow::Hidden,
                position: Rect {
                    top: Val::Percent(1.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: Color::rgb(0.10, 0.10, 0.10).into(),
            ..Default::default()
        })
        .insert(ScrollingListPanel)
        .with_children(|parent| {
            // Moving panel
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        flex_grow: 1.0,
                        max_size: Size::new(Val::Undefined, Val::Undefined),
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .insert(ScrollingList::default())
                .with_children(|parent| {
                    for root_node in get_current_scene(&state, &gltf)
                        .nodes()
                        .filter(|node| node.camera().is_none())
                    {
                        let style = Style {
                            flex_shrink: 0.,
                            size: Size::new(Val::Undefined, Val::Px(20.)),
                            margin: Rect {
                                left: Val::Percent(0.0),
                                right: Val::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        let text_style = TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.,
                            color: Color::WHITE,
                        };
                        let text_alignment = TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Left,
                        };

                        parent
                            .spawn_bundle(TextBundle {
                                style: style.clone(),
                                text: Text::with_section(
                                    format!(
                                        "Scene {} references node {}",
                                        state.current_file.unwrap().scene_index,
                                        root_node.index(),
                                    ),
                                    text_style.clone(),
                                    text_alignment,
                                ),
                                ..Default::default()
                            })
                            .insert(NodeIndex(root_node.index()));

                        // List items
                        traverse_gltf(parent, root_node, &asset_server)
                    }
                });
        })
        .id();
    let left_panel = left_panel_query.single_mut();
    commands.entity(left_panel).add_child(entity);
}

/// Since every node is only part of the scene tree in exactly one place,
/// knowing this place allows us to locate it within in a scene
#[derive(Clone, Copy, Debug, Component, PartialEq, Eq)]
pub struct NodeIndex(pub usize);

/// Meshes can be referenced from multiple nodes
#[derive(Clone, Copy, Debug, Component, PartialEq, Eq)]
pub struct MeshIndex(pub usize);

/// As a mesh can be made up of multiple primitives
/// this struct captures both the MeshIndex and the
/// index of the primitive
#[derive(Clone, Copy, Debug, Component, PartialEq, Eq)]
pub struct PrimitiveIdentifier {
    pub mesh_index: MeshIndex,
    pub primitive_index: usize,
}

/// Tranverses a [gtlf::Gltf] file and generates a list of elements
fn traverse_gltf(
    parent: &mut ChildBuilder,
    parent_node: gltf::Node,
    asset_server: &Res<AssetServer>,
) {
    let style = Style {
        flex_shrink: 0.,
        size: Size::new(Val::Undefined, Val::Px(20.)),
        margin: Rect {
            left: Val::Percent(0.0),
            right: Val::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 20.,
        color: Color::WHITE,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Left,
    };
    if let Some(mesh) = parent_node.mesh() {
        parent
            .spawn_bundle(TextBundle {
                style: style.clone(),
                text: Text::with_section(
                    format!(
                        "Node {} references mesh {}",
                        parent_node.index(),
                        mesh.index(),
                    ),
                    text_style.clone(),
                    text_alignment,
                ),
                ..Default::default()
            })
            .insert(MeshIndex(mesh.index()));
    }

    for node in parent_node
        .children()
        .filter(|node| node.camera().is_none())
    {
        parent
            .spawn_bundle(TextBundle {
                style: style.clone(),
                text: Text::with_section(
                    format!(
                        "Node {} references node {}",
                        parent_node.index(),
                        node.index(),
                    ),
                    text_style.clone(),
                    text_alignment,
                ),
                ..Default::default()
            })
            .insert(NodeIndex(node.index()));
        traverse_gltf(parent, node, asset_server);
    }
}

/// Copied from https://bevyengine.org/examples/ui/ui/ last accessed: 2022-02-06
/// Moves the list if LCtrl is held
fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Children, &Node)>,
    query_item: Query<&Node>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if !keyboard_input.pressed(KeyCode::LControl) {
        return;
    }
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, children, uinode) in query_list.iter_mut() {
            let items_height: f32 = children
                .iter()
                .map(|entity| query_item.get(*entity).unwrap().size.y)
                .sum();
            let panel_height = uinode.size.y;
            let max_scroll = (items_height - panel_height).max(0.);
            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.position.top = Val::Px(scrolling_list.position);
        }
    }
}
