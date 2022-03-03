use bevy::{
    gltf::{GltfMesh, GltfNode, GltfPrimitive},
    prelude::*,
};
use bevy_mod_picking::PickableBundle;
use gltf::{Node, Primitive};

use crate::{
    hierarchy::{MeshIndex, NodeIndex, PrimitiveIdentifier},
    utils::{get_current_scene, get_gltf},
    InspectorState,
};

#[derive(Debug, PartialEq, Eq)]
enum DrawingStatus {
    Changed, // Some drawing parameter has changes
    Drawing, // Started redrawing
    Drawn,   // No changes needed
}

/// When the inspector state is changed, this module updates the drawn model by
/// 1. Detecting a change has happenend
/// 2. Removing the existing entity tree representing the GLTF model
/// 3. Re-drawing the model with the new changes
///
/// For this to work consistently, I needed to order the stages.
/// This can be done by labeling the systems and establishing an
/// order relationship between them.
/// See https://bevy-cheatbook.github.io/programming/system-order.html
///
#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum DrawingOrder {
    DetectChange,
    RemoveExisitingScene,
    DrawNewScene,
}

struct DrawingState {
    status: DrawingStatus,
}

impl Default for DrawingState {
    fn default() -> Self {
        Self {
            status: DrawingStatus::Changed,
        }
    }
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawingState>()
            .add_system(detect_changes.label(DrawingOrder::DetectChange))
            .add_system(
                remove_existing_scene
                    .label(DrawingOrder::RemoveExisitingScene)
                    .after(DrawingOrder::DetectChange),
            )
            .add_system(
                spawn_gltf_objects
                    .label(DrawingOrder::DrawNewScene)
                    .after(DrawingOrder::RemoveExisitingScene),
            );
    }
}

fn detect_changes(state: Res<InspectorState>, mut drawing_state: ResMut<DrawingState>) {
    if state.is_changed() {
        drawing_state.status = DrawingStatus::Changed;
    }
}

/// A tag struct, that allows me to filter for the
/// Top-Level entity representing the currently shown
/// GLTF-Model
#[derive(Component)]
struct GltfObject;
fn remove_existing_scene(
    drawing_state: Res<DrawingState>,
    mut commands: Commands,
    query: Query<Entity, With<GltfObject>>,
) {
    if drawing_state.status != DrawingStatus::Changed {
        return;
    }
    // Despawning the Top-Level entity and all children
    // https://bevy-cheatbook.github.io/programming/commands.html
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive()
    }
}

fn spawn_gltf_objects(
    mut commands: Commands,
    state: Res<InspectorState>,
    mut drawing_state: ResMut<DrawingState>,
    assets: Res<AssetServer>,
    assets_gltfnode: Res<Assets<GltfNode>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
) {
    if drawing_state.status == DrawingStatus::Drawn {
        return;
    }

    let file = match &state.current_file {
        Some(it) => it,
        _ => return,
    };
    drawing_state.status = DrawingStatus::Drawing;
    // if the GLTF has loaded, we can navigate its contents
    let mut node_handles = vec![];
    let gltf = get_gltf(&state);
    let scene = match gltf.as_ref().map(|gltf| get_current_scene(&state, gltf)) {
        Some(it) => it,
        _ => return,
    };
    for gltf_node in scene.nodes() {
        let path = format!("{}#Node{}", file.path, gltf_node.index());
        let node_handle: Handle<GltfNode> = assets.load(path.as_str());
        node_handles.push((node_handle, gltf_node));
    }
    if node_handles
        .first()
        .and_then(|node| assets_gltfnode.get(&node.0))
        .is_none()
    {
        // No nodes to draw or gltf not yet loaded
        return;
    }
    commands
        .spawn()
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(GltfObject)
        .with_children(|parent| {
            for (index, node) in node_handles.into_iter().enumerate() {
                if let Some(bevy_node) = assets_gltfnode.get(&node.0) {
                    draw_node(parent, (bevy_node, node.1), &assets_gltfmesh, &state, index);
                }
            }
        });
    drawing_state.status = DrawingStatus::Drawn;
}

// This code was originally inspired by https://github.com/bevyengine/bevy/blob/3f6068da3db8038ab69706a40ba714cfd836238d/crates/bevy_gltf/src/loader.rs#L456-L616
// But the modificiations made to tag the nodes and meshes hide the similarities in structure
fn draw_node(
    commands: &mut ChildBuilder,
    node: (&GltfNode, Node),
    assets_gltfmesh: &Res<Assets<GltfMesh>>,
    state: &Res<InspectorState>,
    part_index: usize,
) -> Entity {
    let new_translation = node.0.transform.translation
        + state.explosion_factor * part_index as f32 * state.current_file.unwrap().explosion_scale;
    commands
        .spawn()
        .insert(node.0.transform.with_translation(new_translation))
        .insert(GlobalTransform::default())
        .insert(NodeIndex(node.1.index()))
        .with_children(|parent| {
            node.0
                .mesh
                .as_ref()
                .zip(node.1.mesh())
                .and_then(|handle| {
                    assets_gltfmesh
                        .get(handle.0)
                        .map(|bevy_mesh_ref| (bevy_mesh_ref, handle.1))
                })
                .map(|mesh| draw_mesh(parent, mesh));

            node.0
                .children
                .iter()
                .zip(node.1.children())
                .enumerate()
                .for_each(|(index, node)| {
                    draw_node(parent, node, assets_gltfmesh, state, index);
                });
        })
        .id()
}

fn draw_mesh(parent: &mut ChildBuilder, mesh: (&GltfMesh, gltf::Mesh)) -> Vec<Entity> {
    mesh.0
        .primitives
        .iter()
        .zip(mesh.1.primitives())
        .map(|prim| draw_primitive(parent, prim, MeshIndex(mesh.1.index())))
        .collect()
}
fn draw_primitive(
    parent: &mut ChildBuilder,
    primitive: (&GltfPrimitive, Primitive),
    mesh_index: MeshIndex,
) -> Entity {
    // Spawn a PBR entity with the mesh and material of the first GLTF Primitive
    parent
        .spawn_bundle(PbrBundle {
            mesh: primitive.0.mesh.clone(),
            // (unwrap: material is optional, we assume this primitive has one)
            material: primitive.0.material.clone().unwrap(),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .insert(PrimitiveIdentifier {
            mesh_index,
            primitive_index: primitive.1.index(),
        })
        .id()
}
