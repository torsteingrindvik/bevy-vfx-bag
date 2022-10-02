#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::camera::vignette::{Vignette, VignettePlugin};

fn main() {
    let mut app = App::new();

    // Set up the base example
    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        // Add required plugin for using vignette
        .add_plugin(VignettePlugin)
        .add_startup_system(vignette_startup)
        .run();
}

fn vignette_startup(mut commands: Commands) {
    // Normal camera spawn
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        // Adds effect to this camera
        .insert(Vignette);
}
