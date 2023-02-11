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
        .add_system(examples_common::print_on_change::<Raindrops>)
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
    let mut raindrops = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Left) {
        raindrops.speed -= 0.1;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        raindrops.speed += 0.1;
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        raindrops.warping += 0.01;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        raindrops.warping -= 0.01;
    }

    for scroll in mouse_wheel_events.iter() {
        if scroll.y > 0.0 {
            raindrops.zoom += 0.1;
        } else if scroll.y < 0.0 {
            raindrops.zoom -= 0.1;
        }
    }
}
