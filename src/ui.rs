use crate::{
    utils::{get_current_scene, get_gltf},
    InspectorState,
};
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(explosion_text_setup)
            .add_system(explosion_text)
            .add_system(scene_mesh_count)
            .add_system(scene_info_name)
            .add_system(scene_index);
    }
}

#[derive(Component)]
pub struct UIPanel;
/// Displays the current explosion factor in the top left of the screen
/// Inspired by alien_cake_addict example https://bevyengine.org/examples/game/alien-cake-addict/ last accessed: 2022-02-07
pub fn explosion_text_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            // Root Panel, covers the entire screen
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(UIPanel)
        .with_children(|parent| {
            spawn_left_panel(parent, &asset_server);
            spawn_right_panel(parent, &asset_server);
        });
}

#[derive(Component)]
struct ScalingFactor;

#[derive(Component)]
struct InfoPanel;

#[derive(Component)]
pub struct LeftPanel;

fn spawn_left_panel(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    parent
        .spawn_bundle(NodeBundle {
            // Left Panel, for general information and Component list
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Px(300.0), Val::Percent(100.0)),
                border: Rect {
                    top: Val::Px(20.0),
                    left: Val::Px(10.0),
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                },
                ..Default::default()
            },
            color: Color::rgb(0.10, 0.10, 0.10).into(),
            ..Default::default()
        })
        .insert(LeftPanel)
        .with_children(|parent| {
            let style = TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::rgb(1.0, 1.0, 1.0),
            };
            let mut text = Text::with_section("Name\n", style.clone(), Default::default());
            text.sections.push(TextSection {
                value: "_".repeat(32) + "\n",
                style: style.clone(),
            });
            text.sections.push(TextSection {
                value: "Scaling factor: \n".to_string(),
                style: style.clone(),
            });
            text.sections.push(TextSection {
                value: "MeshCount \n".to_string(),
                style: style.clone(),
            });
            text.sections.push(TextSection {
                value: "_".repeat(32) + "\n",
                style: style.clone(),
            });
            text.sections.push(TextSection {
                value: "SceneIndex\n".to_string(),
                style,
            });
            parent
                .spawn_bundle(TextBundle {
                    text,
                    style: Style {
                        max_size: Size::new(Val::Px(280.0), Val::Undefined),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(InfoPanel);
        });
}
#[derive(Component)]
pub struct RightPanel;

#[derive(Component)]
pub struct InstructionPanel;
fn spawn_right_panel(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    parent
        .spawn_bundle(NodeBundle {
            // Right Panel, for information about the clicked node
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Auto, Val::Undefined),
                border: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            color: Color::rgb(0.00, 0.10, 0.00).into(),
            ..Default::default()
        })
        .insert(RightPanel)
        .with_children(|parent| {
            let mut text = Text::with_section(
                "How to use:\n",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
                Default::default(),
            );
            text.sections.push(TextSection {
                value: "
Q+E: Increase and decrease explosion factor

Left+Right Arrows: Switch between models

Scrolling: Zoom

CTRL+Scrolling: Scroll through List

ScrollWheel pressed: Move Camera

RightClick + Mouse Movement: Tilt Camera

LeftClick: Select Mesh for inspection

Esc: Remove inspection view"
                    .to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 18.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            });
            parent
                .spawn_bundle(TextBundle {
                    text,
                    style: Style {
                        position: Rect {
                            top: Val::Percent(1.0),
                            left: Val::Percent(2.0),
                            ..Default::default()
                        },
                        max_size: Size::new(Val::Px(200.), Val::Undefined),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(InstructionPanel);
        });
}

fn scene_info_name(state: Res<InspectorState>, mut query: Query<&mut Text, With<InfoPanel>>) {
    let mut text = query.single_mut();
    if let Some(file) = state.current_file {
        text.sections[0].value = format!("{} \n", file.name)
    }
}

/// Show the current explosion factor
fn explosion_text(state: Res<InspectorState>, mut query: Query<&mut Text, With<InfoPanel>>) {
    let mut text = query.single_mut();
    text.sections[2].value = format!("Explosion factor: {:.1} \n", state.explosion_factor);
}

/// Show information about the current scene
fn scene_mesh_count(state: Res<InspectorState>, mut query: Query<&mut Text, With<InfoPanel>>) {
    if let Some(gltf) = get_gltf(&state) {
        let scene = get_current_scene(&state, &gltf);
        // Show information about the current scene
        let mut text = query.single_mut();
        text.sections[3].value =
            format!("Mesh count is: {} \n", meshcount_for_nodes(scene.nodes()));
    }
}

/// Show information about the current scene
fn scene_index(state: Res<InspectorState>, mut query: Query<&mut Text, With<InfoPanel>>) {
    if let Some(gltf) = get_gltf(&state) {
        let scene = get_current_scene(&state, &gltf);
        // Show information about the current scene
        let mut text = query.single_mut();
        text.sections[5].value = format!("Scene #{}: \n", scene.index());
    }
}

fn meshcount_for_nodes<'a>(nodes: impl Iterator<Item = gltf::Node<'a>>) -> usize {
    let mut count = 0;
    for node in nodes {
        if node.mesh().is_some() {
            count += 1;
        }
        count += meshcount_for_nodes(node.children())
    }
    count
}
