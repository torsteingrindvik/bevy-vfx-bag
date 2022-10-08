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
        .add_startup_system(startup)
        // Shows how to change the effect at runtime
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands, image_handle: Res<BevyVfxBagImage>) {
    // Normal camera spawn
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        })
        .insert(UiCameraConfig { show_ui: false });
}

fn update(
    mut vignette: ResMut<Vignette>,
    time: Res<Time>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    // (0.0 -> 2.0)
    let mut feathering = time.seconds_since_startup().sin() as f32 + 1.0;
    // (0.0 -> 0.5)
    feathering /= 4.0;

    vignette.feathering = feathering;

    text.0 = format!(
        "Radius: {:.1?}, Feathering: {:.1?}",
        vignette.radius, feathering
    );
}
