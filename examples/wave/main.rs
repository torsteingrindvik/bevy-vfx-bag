#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;
use bevy_vfx_bag::{post_processing::wave::Wave, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [1|2|3|4|5] to change which wave preset to use.");

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Wave::default(),
    ));
}

fn update(mut query: Query<&mut Wave, With<Camera>>, keyboard_input: Res<Input<KeyCode>>) {
    let mut wave = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Key1) {
        info!("We're rowing on land.");
        *wave = Wave {
            waves_x: 1.,
            waves_y: 20.,
            speed_x: 1.3,
            speed_y: 20.,
            amplitude_x: 0.25,
            amplitude_y: 0.005,
        };
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        info!("We're being lazy in the x direction.");
        *wave = Wave {
            waves_x: 1.,
            speed_x: 0.5,
            amplitude_x: 0.1,
            ..default()
        };
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        info!("Now we're fighting the wind!");
        *wave = Wave {
            waves_y: 10.,
            speed_y: 14.,
            amplitude_y: 0.002,
            ..default()
        };
    } else if keyboard_input.just_pressed(KeyCode::Key4) {
        info!("These are some rough seas...");
        *wave = Wave {
            waves_x: 1.,
            waves_y: 2.,
            speed_x: 1.,
            speed_y: 1.,
            amplitude_x: 0.03,
            amplitude_y: 0.04,
        };
    } else if keyboard_input.just_pressed(KeyCode::Key5) {
        info!("Oh no, earthquake!");
        *wave = Wave {
            waves_x: 2.0,
            waves_y: 0.1,
            speed_x: 1.,
            speed_y: 13.,
            amplitude_x: 0.02,
            amplitude_y: 0.03,
        };
    };
}
