#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::{
    image::lut::{Lut, LutPlugin},
    BevyVfxBagImage, BevyVfxBagPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(LutPlugin)
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands, image_handle: Res<BevyVfxBagImage>) {
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

// Cycle through some preset LUTs.
fn update(mut lut: ResMut<Lut>, mut text: ResMut<examples_common::ExampleText>) {
    let num_luts = 10;

    text.0 = format!("LUT {}/{num_luts}", 5);
}
