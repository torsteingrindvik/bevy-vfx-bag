#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::camera::blur::{Blur, BlurPlugin};
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(BlurPlugin)
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

fn update(time: Res<Time>, mut blur: ResMut<Blur>, mut text: ResMut<examples_common::ExampleText>) {
    blur.amount = ((time.seconds_since_startup().sin() + 1.0) / 2.) as f32;

    // Display blur amount on screen
    text.0 = format!(
        "Blur: {:.2?}, radius: {:.2?}",
        blur.amount, blur.kernel_radius
    );
}
