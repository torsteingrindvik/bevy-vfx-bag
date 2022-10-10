#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{input::mouse::MouseWheel, prelude::*};

use bevy::render::camera::RenderTarget;
use bevy_vfx_bag::image::rainrops::{Raindrops, RaindropsPlugin};
use bevy_vfx_bag::{BevyVfxBagImage, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(RaindropsPlugin)
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

fn update(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut raindrops: ResMut<Raindrops>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    let mut time_scaling_diff = 0.0;
    let mut intensity_diff = 0.0;
    let mut zoom_diff = 0.0;

    if keyboard_input.just_pressed(KeyCode::Left) {
        time_scaling_diff = -0.1;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        time_scaling_diff = 0.1;
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        intensity_diff = 0.01;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        intensity_diff = -0.01;
    }

    for scroll in mouse_wheel_events.iter() {
        if scroll.y > 0.0 {
            zoom_diff += 0.1;
        } else if scroll.y < 0.0 {
            zoom_diff -= 0.1;
        }
    }

    raindrops.time_scaling += time_scaling_diff;
    raindrops.intensity += intensity_diff;
    raindrops.zoom += zoom_diff;

    // Display parameters on screen
    text.0 = format!(
        "(Press ←↑→↓, MouseWheel) Time scaling: {:.2?}, intensity: {:.2?}, zoom: {:.2?}",
        raindrops.time_scaling, raindrops.intensity, raindrops.zoom
    );
}
