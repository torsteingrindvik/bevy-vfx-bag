#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{post_processing::blur::Blur, BevyVfxBagPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_system(examples_common::print_on_change::<Blur>)
        .add_plugin(BevyVfxBagPlugin::default())
        .add_startup_system(startup)
        .add_system(update)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        Blur::default(),
    ));
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut blur: Query<&mut Blur>) {
    let mut blur = blur.single_mut();

    if keyboard_input.just_pressed(KeyCode::Left) {
        blur.kernel_radius -= 0.001;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        blur.kernel_radius += 0.001;
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        blur.amount += 0.1;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        blur.amount -= 0.1;
    }
}
