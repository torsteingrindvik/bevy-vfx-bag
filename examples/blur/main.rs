#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::blur::{Blur, BlurPlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

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

fn startup(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        })
        .insert(PostProcessingInput);
}

fn update(
    keyboard_input: Res<Input<KeyCode>>,
    mut blur: ResMut<Blur>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    let mut blur_diff = 0.0;
    let mut radius_diff = 0.0;

    if keyboard_input.just_pressed(KeyCode::Left) {
        radius_diff = -0.001;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        radius_diff = 0.001;
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        blur_diff = 0.1;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        blur_diff = -0.1;
    }

    blur.amount += blur_diff;
    blur.kernel_radius += radius_diff;

    // Display blur amount on screen
    text.0 = format!(
        "Blur (↑↓): {:.2?}, radius (←→): {:.3?}",
        blur.amount, blur.kernel_radius
    );
}
