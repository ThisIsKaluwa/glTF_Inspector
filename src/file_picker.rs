/// This module implements the systems for switching between files
use crate::InspectorState;
use bevy::prelude::*;
use lazy_static::lazy_static;

pub struct File {
    pub path: &'static str,
    pub camera_transform: Transform,
    pub scene_index: usize,
    pub explosion_scale: Vec3,
    pub name: &'static str,
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for File {}

lazy_static! {
    pub static ref FILES: [File; 6] = [
        File {
            path: "models/FlightHelmet/FlightHelmet.gltf",
            name: "Flight Helmet",
            camera_transform: Transform::from_translation(Vec3::new(-0.07, 1.02, 2.40))
                .looking_at(Vec3::ZERO, Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(1.0, 1.0, 1.0),
        },
        File {
            path: "models/ammo_collection/scene.gltf",
            name: "Ammo",
            camera_transform: Transform::from_translation(Vec3::new(1.04, 0.59, 0.12,))
                .looking_at(Vec3::ZERO, Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(-10.0, 10.0, -10.0),
        },
        File {
            path: "models/steampunk_underwater_explorer/scene.gltf",
            name: "Steampunk Underwater Explorer",
            camera_transform: Transform::from_translation(Vec3::new(-12.91, 6.06, -9.04))
                .looking_at(Vec3::ZERO, Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(-10.0, -10.0, 10.0),
        },
        File {
            path: "models/Wraith/wraith.gltf",
            name: "Wraith",
            camera_transform: Transform::from_translation(Vec3::new(-1.84, 70.99, 86.87))
                .looking_at(Vec3::new(0.0, 40.0, 0.0), Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(2.0, 2.0, 2.0),
        },
        File {
            path: "models/StarWars/scene.gltf",
            name: "ATM6 Walker",
            camera_transform: Transform::from_translation(Vec3::new(33.58, 39.04, 63.81))
                .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(20.0, 20.0, 20.0),
        },
        File {
            path: "models/ToyCar/glTF/ToyCar.gltf",
            name: "Toy Car",
            camera_transform: Transform::from_translation(Vec3::new(0.09, 0.07, 0.12))
                .looking_at(Vec3::ZERO, Vec3::Y),
            scene_index: 0,
            explosion_scale: Vec3::new(0.2, 0.2, -0.2),
        }
    ];
}

pub struct FilePickerPlugin;

impl Plugin for FilePickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FileChangedEvent>()
            .add_startup_system(intial_file)
            .add_system(switch_between_files);
    }
}

pub struct FileChangedEvent;

fn intial_file(mut state: ResMut<InspectorState>, mut writer: EventWriter<FileChangedEvent>) {
    state.current_file = Some(&FILES[0]);
    writer.send(FileChangedEvent);
}

fn next_file(current: &'static File) -> &'static File {
    FILES
        .iter()
        .position(|f| f == current)
        .and_then(|i| i.checked_add(1))
        .and_then(|index| FILES.get(index))
        .unwrap_or(&FILES[0])
}
fn prev_file(current: &'static File) -> &'static File {
    FILES
        .iter()
        .position(|f| f == current)
        .and_then(|i| i.checked_sub(1))
        .and_then(|index| FILES.get(index))
        .unwrap_or_else(|| FILES.last().unwrap())
}
fn switch_between_files(
    mut state: ResMut<InspectorState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut writer: EventWriter<FileChangedEvent>,
) {
    if !keyboard_input.is_changed() {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Left) {
        state.current_file = state.current_file.map(prev_file);
        writer.send(FileChangedEvent);
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        state.current_file = state.current_file.map(next_file);
        writer.send(FileChangedEvent);
    }
}
