use bevy::prelude::*;
mod input_handler;
use hierarchy::HierarchyPlugin;
use input_handler::update_explosion_factor;
use selection::SelectionPlugin;
mod ui;
use crate::ui::UIPlugin;
mod camera;
use crate::camera::*;
mod explosion;
use crate::explosion::*;
mod file_picker;
use crate::file_picker::{File, FilePickerPlugin};
mod hierarchy;
use bevy_mod_picking::{InteractablePickingPlugin, PickingPlugin};
mod selection;
mod utils;

#[derive(Default)]
pub struct InspectorState {
    explosion_factor: f32,
    current_file: Option<&'static File>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            title: "glTF Inspector".to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<InspectorState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(UIPlugin)
        .add_system(create_light)
        .add_system(update_explosion_factor)
        .add_plugin(HierarchyPlugin)
        .add_plugin(ExplosionPlugin)
        .add_plugin(FilePickerPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(SelectionPlugin)
        .run();
}

//creates a light like the sun for all the scenes
fn create_light(mut commands: Commands) {
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0, //10.000 lux = indirect sun/daylight
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
}
