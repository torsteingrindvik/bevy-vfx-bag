#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::camera::vignette::{Vignette, VignettePlugin};
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    // Set up the base example
    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        // Add required plugin for using any effect at all
        .add_plugin(BevyVfxBagPlugin)
        // Add required plugin for using vignette
        .add_plugin(VignettePlugin)
        // In order to mark a camera with the effect
        .add_startup_system(vignette_startup)
        // Shows how to change the effect at runtime
        // .add_system(vignette_config)
        .run();
}

fn vignette_startup(mut commands: Commands, image_handle: Res<BevyVfxBagImage>) {
    // Normal camera spawn
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        camera: Camera {
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        ..default()
    });
    // Adds effect to this camera
    // .insert(Vignette::default());
}

// fn vignette_config(mut vignette: Query<&mut Vignette>, time: Res<Time>) {
//     for mut vignette in &mut vignette {
//         // (0.0 -> 2.0)
//         let mut feathering = time.seconds_since_startup().sin() as f32 + 1.0;
//         // (0.0 -> 0.5)
//         feathering /= 4.0;

//         vignette.feathering = feathering;
//     }
// }
