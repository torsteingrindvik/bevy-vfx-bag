#[path = "../examples_common.rs"]
mod examples_common;

use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_vfx_bag::post_processing::{raindrops::Raindrops, PostProcessingPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(PostProcessingPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    info!("Press [up|down|left|right|mouse scroll] to change settings");

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(Raindrops::default());
}

fn update(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Raindrops, With<Camera>>,
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

    let mut raindrops = query.single_mut();

    raindrops.time_scaling += time_scaling_diff;
    raindrops.intensity += intensity_diff;
    raindrops.zoom += zoom_diff;

    if time_scaling_diff != 0.0 || intensity_diff != 0.0 || zoom_diff != 0.0 {
        info!(
            "Time scaling: {:.2?}, intensity: {:.2?}, zoom: {:.2?}",
            raindrops.time_scaling, raindrops.intensity, raindrops.zoom
        );
    }
}
