#[path = "../examples_common.rs"]
mod examples_common;

use bevy::prelude::*;

use bevy_vfx_bag::{
    image::pixelate::{Pixelate, PixelatePlugin},
    BevyVfxBagPlugin, PostProcessingInput,
};

fn main() {
    let mut app = App::new();

    app.add_plugin(examples_common::SaneDefaultsPlugin)
        .add_plugin(examples_common::ShapesExamplePlugin::without_3d_camera())
        .add_plugin(BevyVfxBagPlugin)
        .add_plugin(PixelatePlugin)
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
    mut pixelate: ResMut<Pixelate>,
    mut text: ResMut<examples_common::ExampleText>,
) {
    let mut pixelate_diff = 0.0;

    if keyboard_input.just_pressed(KeyCode::Up) {
        pixelate_diff = 1.0;
    } else if keyboard_input.just_pressed(KeyCode::Down) {
        pixelate_diff = -1.0;
    }

    pixelate.block_size += pixelate_diff;
    pixelate.block_size = 1.0_f32.max(pixelate.block_size);

    text.0 = format!("Pixelate block size (↑↓): {:.2?}", pixelate.block_size);
}
