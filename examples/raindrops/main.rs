#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::image::rainrops::RaindropsPlugin;
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(RaindropsPlugin)
        .add_startup_system(startup)
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
