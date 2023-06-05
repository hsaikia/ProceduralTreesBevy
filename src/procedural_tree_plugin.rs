use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};
use rand::Rng;
pub struct ProceduralTreePlugin;

impl Plugin for ProceduralTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewParamsEvent>()
            .init_resource::<Params>()
            .add_system(render_tree)
            .add_system(rotator_system)
            .add_system(ui_system);
    }
}

#[derive(Resource)]
pub struct Params {
    pub children: u8,                        // should be greater than 1
    pub levels: u8,                          // should be greater than 0
    pub first_child_translation_factor: f32, // must be in [0, 1]
    pub angle_from_parent_branch: f32,       // must be in [0, PI / 2]
    pub child_scale: f32,                    // must be in [0.2, 0.8]
    pub base_radius: f32,                    // must be in [0.1, 0.3]
    pub leaf_radius: f32,                    // must be in [0.1, 0.5]
}

impl Default for Params {
    fn default() -> Self {
        Self {
            children: 2,
            levels: 1,
            first_child_translation_factor: 1.0,
            angle_from_parent_branch: 0.5 * std::f32::consts::PI,
            child_scale: 0.7,
            base_radius: 0.15,
            leaf_radius: 0.4,
        }
    }
}

pub struct Branch(pub Transform, pub Option<usize>, pub bool);

fn generate_leaves(parent_idx: usize, all: &mut Vec<Branch>) {
    let mut child_transform = Transform::IDENTITY;
    child_transform = child_transform.with_translation(child_transform.local_y());
    all.push(Branch(child_transform, Some(parent_idx), true));
}

fn generate_branches(params: &Params, level: u8, parent_idx: usize, all: &mut Vec<Branch>) {
    assert!(level >= 1);
    assert!(params.children > 1);
    for i in 0..params.children {
        let angle_from_root_branch = params.angle_from_parent_branch;
        let child_gap_f32 = f32::from(i) / f32::from(params.children);
        let angle_around_root_branch = 2.0 * std::f32::consts::PI * child_gap_f32;
        let child_idx_f32 = f32::from(i) / f32::from(params.children - 1);

        let translation_along_root =
            (1.0 - child_idx_f32) * params.first_child_translation_factor + child_idx_f32;

        let mut child_transform = Transform::IDENTITY;
        child_transform.rotate_local_y(angle_around_root_branch);
        child_transform = child_transform.with_translation(
            child_transform.local_z()
                * (params.base_radius + params.child_scale * 0.5 * angle_from_root_branch.sin())
                + child_transform.local_y()
                    * ((translation_along_root - 0.5)
                        + params.child_scale * 0.5 * angle_from_root_branch.cos()),
        );
        child_transform = child_transform.with_scale(Vec3::splat(params.child_scale));
        child_transform.rotate_local_x(angle_from_root_branch);

        let child_idx = all.len();
        all.push(Branch(child_transform, Some(parent_idx), false));
        if level < params.levels {
            generate_branches(params, level + 1, child_idx, all);
        } else {
            generate_leaves(child_idx, all);
        }
    }
}

struct NewParamsEvent;

pub fn generate(params: &Params) -> Vec<Branch> {
    let base = Transform::default();
    let mut ret: Vec<Branch> = Vec::new();
    ret.push(Branch(base, None, false));
    generate_branches(params, 1, 0, &mut ret);
    ret
}

/// This component indicates the root entity for our tree
#[derive(Component)]
struct TreeRoot;

/// Rotates the tree root
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<TreeRoot>>) {
    for mut transform in &mut query {
        transform.rotate_y(0.1 * time.delta_seconds());
    }
}

fn render_tree(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<TreeRoot>>,
    params: Res<Params>,
    mut param_events: EventReader<NewParamsEvent>,
) {
    for _ in param_events.iter() {
        // Remove the old tree
        // Should only be one - the tree root
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Generate and add the new tree
        let tree = generate(&params);

        let mut entity_parent_indices: Vec<(Entity, Option<usize>)> = Vec::new();
        for branch in &tree {
            if branch.2 {
                // leaves are spheres
                let entity_id = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(
                            Mesh::try_from(shape::Icosphere {
                                radius: params.leaf_radius,
                                subdivisions: 2,
                            })
                            .unwrap(),
                        ),
                        transform: branch.0,
                        material: materials.add(Color::rgb(0.2, 0.7, 0.3).into()),
                        ..default()
                    })
                    .id();
                entity_parent_indices.push((entity_id, branch.1));
            } else {
                // cylinders (tree branches)
                let entity_id = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cylinder {
                            radius: params.base_radius,
                            height: 1.0,
                            resolution: 6,
                            segments: 6,
                        })),
                        transform: branch.0,
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        ..default()
                    })
                    .id();
                entity_parent_indices.push((entity_id, branch.1));
            }

            //println!("{:?} -> {:?} Parent {:?}", entity_id, branch.0, branch.1);
        }

        for (child_id, par_id) in &entity_parent_indices {
            if par_id.is_some() {
                let parent_id = entity_parent_indices[par_id.unwrap()].0;
                commands.entity(parent_id).push_children(&[*child_id]);
                //println!("Child {:?} -> Parent {:?}", child_id, parent_id);
            }
        }

        // Add the TreeRoot component to the root node
        commands.entity(entity_parent_indices[0].0).insert(TreeRoot);
    }
}

fn ui_system(
    mut params: ResMut<Params>,
    mut param_changed_event: EventWriter<NewParamsEvent>,
    mut contexts: EguiContexts,
) {
    egui::Window::new("Procedural Tree Parameters").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Children: ");
            if ui.add(Slider::new(&mut params.children, 2..=6)).changed() {
                param_changed_event.send(NewParamsEvent);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Levels: ");
            if ui.add(Slider::new(&mut params.levels, 1..=5)).changed() {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Child Translation Factor: ");
            if ui
                .add(Slider::new(
                    &mut params.first_child_translation_factor,
                    0.0..=1.0,
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Deviation Angle from Parent Branch: ");
            if ui
                .add(Slider::new(
                    &mut params.angle_from_parent_branch,
                    0.0..=0.5 * std::f32::consts::PI,
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Child Scale: ");
            if ui
                .add(Slider::new(&mut params.child_scale, 0.2..=0.8))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Base Radius: ");
            if ui
                .add(Slider::new(&mut params.base_radius, 0.1..=0.3))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Leaf Radius: ");
            if ui
                .add(Slider::new(&mut params.leaf_radius, 0.1..=0.5))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Generate")).clicked() {
                // Randomize params
                let mut rng = rand::thread_rng();
                params.children = rng.gen_range(2..=6);
                params.levels = rng.gen_range(1..=5);
                params.first_child_translation_factor = rng.gen_range(0.0..=1.0);
                params.angle_from_parent_branch = rng.gen_range(0.0..=0.5 * std::f32::consts::PI);
                params.child_scale = rng.gen_range(0.2..=0.8);
                params.base_radius = rng.gen_range(0.1..=0.3);
                params.leaf_radius = rng.gen_range(0.1..=0.5);
                param_changed_event.send(NewParamsEvent);
            }
        })
    });
}
