use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::LogDiagnosticsPlugin,
    prelude::*,
    text::FontSmoothing,
};

mod params;
mod procedural_tree_plugin;
mod tree;

use procedural_tree_plugin::ProceduralTreePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Procedural Tree Generator".into(),
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::LinearRgba(LinearRgba::GREEN),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    ..default()
                },
            },
            ProceduralTreePlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(20.0, 20.0)))),
        MeshMaterial3d::from(materials.add(Color::linear_rgba(0.3, 0.5, 0.3, 1.0))),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // light
    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(7.0, 3.5, 0.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7.0, 3.5, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
