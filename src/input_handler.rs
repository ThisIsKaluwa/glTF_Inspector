use bevy::prelude::*;

use crate::InspectorState;

// Adjust the scaling factor
pub fn update_explosion_factor(
    mut state: ResMut<InspectorState>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if !keyboard_input.is_changed() {
        return;
    }
    let mut changed = false;
    if keyboard_input.just_pressed(KeyCode::Q) {
        state.explosion_factor += 0.2;
        changed = true;
    } else if keyboard_input.just_pressed(KeyCode::E) && (state.explosion_factor > 0.0) {
        state.explosion_factor -= 0.2;
        changed = true;
    }

    if changed && state.explosion_factor <= 0.0 {
        state.explosion_factor = 0.0;
    }
}
