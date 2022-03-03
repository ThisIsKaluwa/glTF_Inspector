use bevy::prelude::*;
use gltf::Gltf;

use crate::InspectorState;

/// Reads the gltf_file from disk should one be selected
pub fn get_gltf(state: &Res<InspectorState>) -> Option<Gltf> {
    state
        .current_file
        .and_then(|file| gltf::Gltf::open(format!("assets/{}", file.path)).ok())
}

/// Returns the currently active scene
pub fn get_current_scene<'gltf>(
    state: &Res<InspectorState>,
    gltf: &'gltf Gltf,
) -> gltf::Scene<'gltf> {
    gltf.scenes()
        .nth(state.current_file.unwrap().scene_index)
        .unwrap()
}
