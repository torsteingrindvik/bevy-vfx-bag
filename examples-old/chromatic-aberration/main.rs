#[path = "../examples_common.rs"]
mod examples_common;

use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::chromatic_aberration::{ChromaticAberration, ChromaticAberrationPlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(ChromaticAberrationPlugin)
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
    time: Res<Time>,
    mut chromatic_aberration: ResMut<ChromaticAberration>,
    mut text: ResMut<examples_common::ExampleText>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut magnitude = chromatic_aberration.magnitude_r;

    if keyboard_input.just_pressed(KeyCode::Up) {
        magnitude += 0.001;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        magnitude -= 0.001;
    }

    chromatic_aberration.magnitude_r = magnitude;
    chromatic_aberration.magnitude_g = magnitude;
    chromatic_aberration.magnitude_b = magnitude;

    let t = time.elapsed_seconds();

    chromatic_aberration.dir_r = Vec2::from_angle(t);
    chromatic_aberration.dir_g = Vec2::from_angle(2. * t);
    chromatic_aberration.dir_b = Vec2::from_angle(3. * t);

    let base_angle = Vec2::new(1., 0.);
    let angle = |color_dir| base_angle.angle_between(color_dir) * 180. / PI + 180.;

    text.0 = format!(
        "(Press ↑↓) Magnitude all: {magnitude:.3?}, R: {:4.0?}° G: {:4.0?}° B: {:4.0?}°",
        angle(chromatic_aberration.dir_r),
        angle(chromatic_aberration.dir_g),
        angle(chromatic_aberration.dir_b)
    );
}
