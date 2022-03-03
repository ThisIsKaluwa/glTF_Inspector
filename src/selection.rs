use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;

use crate::hierarchy::{MeshIndex, NodeIndex, PrimitiveIdentifier};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(highlight_path).add_system(clear_on_escape);
    }
}
/// Colors all Nodes and Meshes in the list that are on the path of a
/// clicked on primitive on screen
fn highlight_path(
    mut events: EventReader<PickingEvent>,
    mesh_query: Query<(Entity, &Parent, &PrimitiveIdentifier)>,
    node_query: Query<(Entity, &Parent, &NodeIndex)>,
    mut ui_nodes_query: Query<(&mut Text, &NodeIndex), Without<MeshIndex>>,
    mut ui_mesh_query: Query<(&mut Text, &MeshIndex), Without<NodeIndex>>,
) {
    for event in events.iter().filter_map(|e| match e {
        PickingEvent::Clicked(e) => Some(e), // Only clicks are relevant
        _ => None,
    }) {
        clear_nodes(&mut ui_nodes_query);
        clear_mesh(&mut ui_mesh_query);
        if let Ok((_, parent, primitive_identifier)) = mesh_query.get(*event) {
            let node_indices = chain_node_indices(parent.0, &node_query);
            color_nodes(&node_indices, &mut ui_nodes_query);
            color_mesh(primitive_identifier.mesh_index, &mut ui_mesh_query);
        }
    }
}

fn clear_on_escape(
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_nodes_query: Query<(&mut Text, &NodeIndex), Without<MeshIndex>>,
    mut ui_mesh_query: Query<(&mut Text, &MeshIndex), Without<NodeIndex>>,
) {
    if !keyboard_input.is_changed() || !keyboard_input.just_pressed(KeyCode::Escape) {
        return;
    }
    clear_nodes(&mut ui_nodes_query);
    clear_mesh(&mut ui_mesh_query);
}

fn chain_node_indices(
    child_node: Entity,
    node_query: &Query<(Entity, &Parent, &NodeIndex)>,
) -> Vec<NodeIndex> {
    let mut ret_val = vec![];
    if let Ok((_, parent, index)) = node_query.get(child_node) {
        ret_val.append(&mut chain_node_indices(parent.0, node_query));
        ret_val.push(*index);
    }
    ret_val
}

fn color_nodes(
    nodes: &[NodeIndex],
    ui_nodes_query: &mut Query<(&mut Text, &NodeIndex), Without<MeshIndex>>,
) {
    for (mut text, _) in ui_nodes_query
        .iter_mut()
        .filter(|(_, index)| nodes.contains(index))
    {
        text.sections[0].style.color = Color::GREEN;
    }
}

fn color_mesh(
    mesh_index: MeshIndex,
    ui_mesh_query: &mut Query<(&mut Text, &MeshIndex), Without<NodeIndex>>,
) {
    for (mut text, _) in ui_mesh_query
        .iter_mut()
        .filter(|(_, index)| mesh_index == **index)
    {
        text.sections[0].style.color = Color::GREEN;
    }
}

fn clear_nodes(ui_nodes_query: &mut Query<(&mut Text, &NodeIndex), Without<MeshIndex>>) {
    ui_nodes_query.for_each_mut(|(mut text, _)| text.sections[0].style.color = Color::WHITE)
}

fn clear_mesh(ui_mesh_query: &mut Query<(&mut Text, &MeshIndex), Without<NodeIndex>>) {
    ui_mesh_query.for_each_mut(|(mut text, _)| text.sections[0].style.color = Color::WHITE)
}
