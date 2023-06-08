use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Slider},
    EguiContexts,
};
use rand::Rng;

use crate::params::{
    Params, ParamsVector, ANGLE_MINMAX, BASE_RADIUS_MINMAX, CHILDREN_MINMAX, LEAF_RADIUS_MINMAX,
    LEVELS_MINMAX, SCALE_MINMAX, TRANSLATION_FACTOR_MINMAX,
};

use crate::tree::generate;

pub struct ProceduralTreePlugin;

impl Plugin for ProceduralTreePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewParamsEvent>()
            .insert_resource(RedrawTimer(Timer::from_seconds(0.2, TimerMode::Once)))
            .init_resource::<Params>()
            .init_resource::<ParamsVector>()
            .add_system(render_tree)
            .add_system(rotator_system)
            .add_system(random_walk)
            .add_system(ui_system);
    }
}

#[derive(Resource)]
struct RedrawTimer(Timer);

struct NewParamsEvent;

/// This component indicates the root entity for our tree
#[derive(Component)]
struct TreeRoot;

/// Rotates the tree root
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<TreeRoot>>) {
    for mut transform in &mut query {
        transform.rotate_y(0.1 * time.delta_seconds());
    }
}

fn random_walk(
    time: Res<Time>,
    mut timer: ResMut<RedrawTimer>,
    mut params: ResMut<Params>,
    mut params_vel: ResMut<ParamsVector>,
    mut param_changed_event: EventWriter<NewParamsEvent>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        params_vel.nudge();
        let mut params_pos = ParamsVector::from_params(&params);
        params_pos.add(&mut params_vel);
        *params = params_pos.to_params();
        //println!("New params {:?}", params);
        param_changed_event.send(NewParamsEvent);
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

        let t = params.angle_from_parent_branch * 2.0 / std::f32::consts::PI;
        let color_r = (1.0 - t * 2.0).max(0.0);
        let color_g = if t < 0.5 { 2.0 * t } else { 2.0 - 2.0 * t };
        let color_b = (t * 2.0 - 1.0).max(0.0);

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
                        material: materials.add(Color::rgb(color_r, color_g, color_b).into()),
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
    mut timer: ResMut<RedrawTimer>,
    mut param_changed_event: EventWriter<NewParamsEvent>,
    mut contexts: EguiContexts,
) {
    egui::Window::new("Procedural Tree Parameters").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Children: ");
            if ui
                .add(Slider::new(
                    &mut params.children,
                    CHILDREN_MINMAX[0]..=CHILDREN_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Levels: ");
            if ui
                .add(Slider::new(
                    &mut params.levels,
                    LEVELS_MINMAX[0]..=LEVELS_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Child Translation Factor: ");
            if ui
                .add(Slider::new(
                    &mut params.child_translation_factor,
                    TRANSLATION_FACTOR_MINMAX[0]..=TRANSLATION_FACTOR_MINMAX[1],
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
                    ANGLE_MINMAX[0]..=ANGLE_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Child Scale: ");
            if ui
                .add(Slider::new(
                    &mut params.child_scale,
                    SCALE_MINMAX[0]..=SCALE_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Base Radius: ");
            if ui
                .add(Slider::new(
                    &mut params.base_radius,
                    BASE_RADIUS_MINMAX[0]..=BASE_RADIUS_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            ui.label("Leaf Radius: ");
            if ui
                .add(Slider::new(
                    &mut params.leaf_radius,
                    LEAF_RADIUS_MINMAX[0]..=LEAF_RADIUS_MINMAX[1],
                ))
                .changed()
            {
                param_changed_event.send(NewParamsEvent);
            };
        });

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Generate")).clicked() {
                // Randomize params
                let mut rng = rand::thread_rng();
                params.children = rng.gen_range(CHILDREN_MINMAX[0]..=CHILDREN_MINMAX[1]);
                params.levels = rng.gen_range(LEVELS_MINMAX[0]..=LEVELS_MINMAX[1]);
                params.child_translation_factor =
                    rng.gen_range(TRANSLATION_FACTOR_MINMAX[0]..=TRANSLATION_FACTOR_MINMAX[1]);
                params.angle_from_parent_branch = rng.gen_range(ANGLE_MINMAX[0]..=ANGLE_MINMAX[1]);
                params.child_scale = rng.gen_range(SCALE_MINMAX[0]..=SCALE_MINMAX[1]);
                params.base_radius = rng.gen_range(BASE_RADIUS_MINMAX[0]..=BASE_RADIUS_MINMAX[1]);
                params.leaf_radius = rng.gen_range(LEAF_RADIUS_MINMAX[0]..=LEAF_RADIUS_MINMAX[1]);
                param_changed_event.send(NewParamsEvent);
            }
        });

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Random Walk")).clicked() {
                if timer.0.mode() == TimerMode::Once {
                    timer.0.set_mode(TimerMode::Repeating);
                } else {
                    timer.0.set_mode(TimerMode::Once);
                }
            }
        })
    });
}
